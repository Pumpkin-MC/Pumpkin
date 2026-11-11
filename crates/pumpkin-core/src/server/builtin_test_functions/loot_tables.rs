use std::sync::Arc;

use pumpkin_data::{dye_color::DyeColor, entity::EntityType, item::Item};
use pumpkin_gametest::{GameTestContext, GameTestError, GameTestResult};
use pumpkin_util::{
    GameMode, PermissionLvl, loot_table::parser::parse_loot_table, math::vector3::Vector3,
};

use crate::{
    entity::{Entity, EntityBase, passive::sheep::SheepEntity, player::Player},
    server::{Server, server_test_manager::ServerSimulatedPlayer},
    world::loot::LootContextParameters,
};

use super::build_stone_floor;

const INNER_TABLE: &str = "pumpkin_gametest:loot_inner";
const OUTER_TABLE: &str = "pumpkin_gametest:loot_outer";
const RECURSIVE_TABLE: &str = "pumpkin_gametest:loot_recursive";

struct RegisteredLootTables(Arc<Server>);

impl Drop for RegisteredLootTables {
    fn drop(&mut self) {
        for key in [INNER_TABLE, OUTER_TABLE, RECURSIVE_TABLE] {
            self.0.datapack_manager.remove_loot_table(key);
        }
    }
}

fn count_item(player: &Player, item: &Item) -> u32 {
    player
        .inventory()
        .main_inventory
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
        .filter(|stack| stack.item == item)
        .map(|stack| u32::from(stack.item_count))
        .sum()
}

pub(super) async fn test_loot_entity_context_and_nested_tables(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let simulated = test
        .spawn_simulated_player(
            "LootTester",
            Vector3::new(2.5, 1.0, 2.5),
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
    let dispatcher = server.command_dispatcher.load();
    player.set_permission_lvl(&server, PermissionLvl::Four, &dispatcher);
    let source = player.get_command_source(&server).with_silent();

    let sheep = SheepEntity::new(Entity::new(
        world.clone(),
        test.absolute_vec(Vector3::new(3.5, 1.0, 2.5)),
        &EntityType::SHEEP,
    ));
    sheep.set_color(DyeColor::Red.id());
    world.spawn_entity(sheep.clone());
    dispatcher
        .execute_input(
            &format!("loot give @s kill {}", sheep.get_entity().entity_uuid),
            &source,
        )
        .map_err(|error| GameTestError::World(error.message.get_text()))?;
    test.assert_true(
        count_item(player, &Item::RED_WOOL) == 1,
        "/loot kill lost the sheep's red wool component",
    )?;

    let _registered = RegisteredLootTables(server.clone());
    let inner = parse_loot_table(r#"{"pools":[{"rolls":3,"entries":[{"type":"minecraft:item","name":"minecraft:amethyst_shard","modifier":{"type":"minecraft:set_count","count":2}}]}]}"#)
        .ok_or_else(|| GameTestError::World("Invalid inner loot fixture".to_string()))?;
    let outer = parse_loot_table(r#"{"pools":[{"entries":[{"type":"minecraft:loot_table","value":"pumpkin_gametest:loot_inner","modifier":{"type":"minecraft:set_count","count":3,"add":true}}]}]}"#)
        .ok_or_else(|| GameTestError::World("Invalid outer loot fixture".to_string()))?;
    server
        .datapack_manager
        .insert_loot_table(INNER_TABLE.into(), Arc::new(inner));
    server
        .datapack_manager
        .insert_loot_table(OUTER_TABLE.into(), Arc::new(outer));
    let before = count_item(player, &Item::AMETHYST_SHARD);
    dispatcher
        .execute_input("loot give @s loot pumpkin_gametest:loot_outer", &source)
        .map_err(|error| GameTestError::World(error.message.get_text()))?;
    test.assert_true(
        count_item(player, &Item::AMETHYST_SHARD) == before + 15,
        "Nested datapack loot did not produce fifteen amethyst shards",
    )?;

    let recursive = parse_loot_table(r#"{"pools":[{"entries":[{"type":"minecraft:loot_table","value":"pumpkin_gametest:loot_recursive"}]}]}"#)
        .ok_or_else(|| GameTestError::World("Invalid recursive loot fixture".to_string()))?;
    server
        .datapack_manager
        .insert_loot_table(RECURSIVE_TABLE.into(), Arc::new(recursive));
    let handle = world
        .get_loot_table(RECURSIVE_TABLE)
        .ok_or_else(|| GameTestError::World("Missing recursive loot table".to_string()))?;
    test.assert_true(
        handle
            .generate_loot_with_context(
                0,
                &LootContextParameters {
                    world: Some(&world),
                    ..Default::default()
                },
            )
            .is_empty(),
        "Recursive loot table emitted items",
    )?;
    sheep.get_entity().remove();
    test.succeed();
    Ok(())
}
