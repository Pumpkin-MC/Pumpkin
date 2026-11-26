use pumpkin_data::Block;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use pumpkin_data::{dye_color::DyeColor, item::Item, item_stack::ItemStack};
use pumpkin_gametest::{GameTestContext, GameTestError, GameTestResult};
use pumpkin_util::{
    GameMode, Hand,
    loot_table::parser::parse_loot_table,
    math::{position::BlockPos, vector3::Vector3},
};

use super::TestEventHandlers;

use crate::{
    plugin::{
        BoxFuture, EventRwHandler,
        api::events::player::{
            player_item_break::PlayerItemBreakEvent, player_item_damage::PlayerItemDamageEvent,
        },
    },
    server::{Server, server_test_manager::server_world},
    world::loot::{DynamicLootTable, LootTableHandle},
};

const SHEARING_LOOT_TABLE: &str = "minecraft:shearing/sheep/red";

struct RegisteredShearingLootTable {
    server: Arc<Server>,
    previous: Option<Arc<DynamicLootTable>>,
}

impl Drop for RegisteredShearingLootTable {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.take() {
            self.server
                .datapack_manager
                .insert_loot_table(SHEARING_LOOT_TABLE.into(), previous);
        } else {
            self.server
                .datapack_manager
                .remove_loot_table(SHEARING_LOOT_TABLE);
        }
    }
}

struct ItemEvents {
    player_id: i32,
    damage: AtomicUsize,
    broken: AtomicUsize,
    cancel: AtomicBool,
}

impl EventRwHandler<PlayerItemDamageEvent> for ItemEvents {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerItemDamageEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.player.entity_id() == self.player_id
                && event.item_name == Item::SHEARS.registry_key
            {
                self.damage.fetch_add(1, Ordering::Relaxed);
                event.cancelled = self.cancel.load(Ordering::Relaxed);
            }
        })
    }
}

impl EventRwHandler<PlayerItemBreakEvent> for ItemEvents {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerItemBreakEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.player.entity_id() == self.player_id
                && event.item_name == Item::SHEARS.registry_key
            {
                self.broken.fetch_add(1, Ordering::Relaxed);
            }
        })
    }
}

pub(super) async fn test_shearing_tool_loot_and_item_events(
    test: GameTestContext,
) -> GameTestResult<()> {
    test.set_dimensions(7, 5, 7).await?;
    test.build_floor(7, 7, &Block::STONE).await?;
    let player = test
        .spawn_simulated_player(
            "ShearTester",
            Vector3::new(2.5, 1.0, 3.5),
            GameMode::Survival,
        )
        .await?;
    let world = server_world(&test)?;
    let server = world
        .server
        .upgrade()
        .ok_or_else(|| GameTestError::World("Server stopped".to_string()))?;

    let sheep = test
        .spawn_entity("minecraft:sheep", Vector3::new(4.5, 1.0, 3.5))
        .await?;
    sheep.set_color(DyeColor::Red)?;

    let table = parse_loot_table(
        r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:diamond","conditions":[{"condition":"minecraft:match_tool","predicate":{"items":"minecraft:shears"}}]}]}]}"#,
    )
    .ok_or_else(|| GameTestError::World("Invalid shearing loot fixture".to_string()))?;
    let previous = match server.datapack_manager.get_loot_table(SHEARING_LOOT_TABLE) {
        Some(LootTableHandle::Dynamic(table)) => Some(table),
        _ => None,
    };
    server
        .datapack_manager
        .insert_loot_table(SHEARING_LOOT_TABLE.into(), Arc::new(table));
    let _registered_table = RegisteredShearingLootTable {
        server: server.clone(),
        previous,
    };

    let hooks = Arc::new(ItemEvents {
        player_id: player.entity_id(),
        damage: AtomicUsize::new(0),
        broken: AtomicUsize::new(0),
        cancel: AtomicBool::new(false),
    });
    let handlers = TestEventHandlers::new(&server.plugin_manager, player.entity_id());
    handlers.register::<PlayerItemDamageEvent>(hooks.clone());
    handlers.register::<PlayerItemBreakEvent>(hooks.clone());

    let max_damage = ItemStack::new(1, &Item::SHEARS)
        .get_max_damage()
        .ok_or_else(|| GameTestError::World("Shears have no maximum durability".to_string()))?;

    for (damage, cancel, expected_damage, expected_events, expected_breaks) in [
        (0, false, Some(1), 1, 0),
        (max_damage - 1, false, None, 2, 1),
        (0, true, Some(0), 3, 1),
    ] {
        let mut tool = ItemStack::new(1, &Item::SHEARS);
        tool.set_damage(damage);
        player.set_held_item(Hand::Right, tool)?;
        sheep.set_sheared(false)?;
        hooks.cancel.store(cancel, Ordering::Relaxed);
        player.interact_with(&sheep, Hand::Right)?;

        let held = player.held_item(Hand::Right);
        let actual_damage = (!held.is_empty()).then(|| held.get_damage());
        let damage_events = hooks.damage.load(Ordering::Relaxed);
        let break_events = hooks.broken.load(Ordering::Relaxed);
        test.assert_true(
            sheep.is_sheared()?
                && actual_damage == expected_damage
                && damage_events == expected_events
                && break_events == expected_breaks,
            format!(
                "Shearing: damage={actual_damage:?} (expected {expected_damage:?}), damage events={damage_events} (expected {expected_events}), break events={break_events} (expected {expected_breaks})"
            ),
        )?;
        test.assert_item_entity_count(
            "minecraft:diamond",
            &BlockPos::new(4, 1, 3),
            3.0,
            expected_events as u32,
        )
        .await?;
    }

    test.succeed();
    Ok(())
}
