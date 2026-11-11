use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use pumpkin_data::{dye_color::DyeColor, entity::EntityType, item::Item, item_stack::ItemStack};
use pumpkin_gametest::{GameTestContext, GameTestError, GameTestResult};
use pumpkin_util::{
    GameMode, Hand,
    loot_table::parser::parse_loot_table,
    math::{boundingbox::BoundingBox, vector3::Vector3},
};

use crate::{
    entity::{Entity, EntityBase, item::ItemEntity, passive::sheep::SheepEntity},
    plugin::{
        BoxFuture, EventHandler,
        api::events::{
            EventPriority,
            player::{
                player_item_break::PlayerItemBreakEvent, player_item_damage::PlayerItemDamageEvent,
            },
        },
    },
    server::{Server, server_test_manager::ServerSimulatedPlayer},
    world::loot::{DynamicLootTable, LootTableHandle},
};

use super::build_stone_floor;

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

impl EventHandler<PlayerItemDamageEvent> for ItemEvents {
    fn handle_blocking<'a>(
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

impl EventHandler<PlayerItemBreakEvent> for ItemEvents {
    fn handle_blocking<'a>(
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

#[expect(
    clippy::too_many_lines,
    reason = "covers the shearing event and loot sequence"
)]
pub(super) async fn test_shearing_tool_loot_and_item_events(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 7, 7).await?;
    let simulated = test
        .spawn_simulated_player(
            "ShearTester",
            Vector3::new(2.5, 1.0, 3.5),
            GameMode::Survival,
        )
        .await?;
    let player = simulated
        .as_any()
        .downcast_ref::<ServerSimulatedPlayer>()
        .ok_or_else(|| GameTestError::World("Expected server simulated player".to_string()))?
        .player();
    let world = player.world();
    let server = world
        .server
        .upgrade()
        .ok_or_else(|| GameTestError::World("Server stopped".to_string()))?;

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
    let _damage_handler = server
        .plugin_manager
        .register_scoped::<PlayerItemDamageEvent, _>(hooks.clone(), EventPriority::Normal, true);
    let _break_handler = server
        .plugin_manager
        .register_scoped::<PlayerItemBreakEvent, _>(hooks.clone(), EventPriority::Normal, true);

    let sheep = SheepEntity::new(Entity::new(
        world.clone(),
        test.absolute_vec(Vector3::new(4.5, 1.0, 3.5)),
        &EntityType::SHEEP,
    ));
    sheep.set_color(DyeColor::Red.id());
    test.assert_true(
        world.spawn_entity(sheep.clone()),
        "Could not spawn test sheep",
    )?;
    let target: Arc<dyn EntityBase> = sheep.clone();
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
        simulated.set_held_item(Hand::Right, tool)?;
        sheep.set_sheared(false);
        hooks.cancel.store(cancel, Ordering::Relaxed);
        let mut stack = simulated.held_item(Hand::Right);
        let handled = target.interact_with_hand(player, &mut stack, Hand::Right);
        let interacted = handled.unwrap_or_else(|| target.interact(player, &mut stack));
        if !interacted {
            server
                .item_registry
                .use_on_entity(&mut stack, player, target.clone());
        }
        simulated.set_held_item(Hand::Right, stack)?;

        let held = simulated.held_item(Hand::Right);
        let actual_damage = (!held.is_empty()).then(|| held.get_damage());
        let damage_events = hooks.damage.load(Ordering::Relaxed);
        let break_events = hooks.broken.load(Ordering::Relaxed);
        let diamonds: usize = world
            .get_entities_at_box(
                &BoundingBox::from_block(&sheep.get_entity().block_pos.load()).expand_all(3.0),
            )
            .iter()
            .filter_map(|entity| entity.cast_any().downcast_ref::<ItemEntity>())
            .map(|entity| {
                let stack = entity
                    .get_item_stack()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if stack.item == &Item::DIAMOND {
                    usize::from(stack.item_count)
                } else {
                    0
                }
            })
            .sum();
        test.assert_true(
            sheep.is_sheared()
                && actual_damage == expected_damage
                && damage_events == expected_events
                && break_events == expected_breaks
                && diamonds == expected_events,
            format!(
                "Shearing: damage={actual_damage:?} (expected {expected_damage:?}), damage events={damage_events} (expected {expected_events}), break events={break_events} (expected {expected_breaks}), match_tool drops={diamonds} (expected {expected_events})"
            ),
        )?;
    }

    test.succeed();
    Ok(())
}
