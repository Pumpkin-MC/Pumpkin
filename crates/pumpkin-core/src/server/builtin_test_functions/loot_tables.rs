use pumpkin_data::Block;
use std::sync::Arc;

use pumpkin_data::{dye_color::DyeColor, entity::EntityType, item::Item};
use pumpkin_gametest::{GameTestContext, GameTestError, GameTestResult};
use pumpkin_util::{
    GameMode, PermissionLvl, loot_table::parser::parse_loot_table, math::vector3::Vector3,
};

use crate::{
    entity::{Entity, EntityBase, passive::sheep::SheepEntity},
    server::{Server, server_test_manager::server_world},
    world::loot::LootContextParameters,
};

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

pub(super) async fn test_loot_entity_context_and_nested_tables(
    test: GameTestContext,
) -> GameTestResult<()> {
    test.set_dimensions(5, 5, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;
    let player = test
        .spawn_simulated_player(
            "LootTester",
            Vector3::new(2.5, 1.0, 2.5),
            GameMode::Survival,
        )
        .await?;
    let world = server_world(&test)?;
    let server = world
        .server
        .upgrade()
        .ok_or_else(|| GameTestError::World("Server stopped".to_string()))?;
    player.set_permission_level(PermissionLvl::Four)?;

    let sheep = SheepEntity::new(Entity::new(
        world.clone(),
        test.absolute_vec(Vector3::new(3.5, 1.0, 2.5)),
        &EntityType::SHEEP,
    ));
    sheep.set_color(DyeColor::Red.id());
    world.spawn_entity(sheep.clone());
    player.run_command(&format!(
        "loot give @s kill {}",
        sheep.get_entity().entity_uuid
    ))?;
    test.assert_true(
        player.inventory_count(&Item::RED_WOOL) == 1,
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
    let before = player.inventory_count(&Item::AMETHYST_SHARD);
    player.run_command("loot give @s loot pumpkin_gametest:loot_outer")?;
    test.assert_true(
        player.inventory_count(&Item::AMETHYST_SHARD) == before + 15,
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
