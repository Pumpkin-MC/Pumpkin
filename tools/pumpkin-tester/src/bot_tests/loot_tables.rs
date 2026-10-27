use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use pumpkin_core::{
    entity::{Entity, EntityBase, passive::sheep::SheepEntity},
    server::Server,
};
use pumpkin_data::{dimension::Dimension, entity::EntityType, item::Item};
use pumpkin_util::loot_table::parser::parse_loot_table;

use crate::bot::HeadlessBot;

pub async fn run_loot_table_tests(
    server: &Arc<Server>,
    bot: &mut HeadlessBot,
    summaries: &mut Vec<String>,
) -> Result<(), String> {
    let start = Instant::now();
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let player = world
        .get_player_by_name("TestBot1")
        .ok_or("Missing TestBot1")?;
    let sheep = SheepEntity::new(Entity::new(world.clone(), bot.position, &EntityType::SHEEP));
    sheep.set_color(pumpkin_data::dye_color::DyeColor::Red.id());
    world.spawn_entity(sheep.clone());
    bot.send_command(&format!(
        "loot give @s kill {}",
        sheep.get_entity().entity_uuid
    ))
    .await?;

    let inner = parse_loot_table(r#"{"pools":[{"rolls":3,"entries":[{"type":"minecraft:item","name":"minecraft:amethyst_shard","modifier":{"type":"minecraft:set_count","count":2}}]}]}"#).ok_or("Invalid inner loot fixture")?;
    let outer = parse_loot_table(r#"{"pools":[{"entries":[{"type":"minecraft:loot_table","value":"loot_test:inner","modifier":{"type":"minecraft:set_count","count":3,"add":true}}]}]}"#).ok_or("Invalid outer loot fixture")?;
    server
        .datapack_manager
        .insert_loot_table("loot_test:inner".into(), Arc::new(inner));
    server
        .datapack_manager
        .insert_loot_table("minecraft:loot_test_outer".into(), Arc::new(outer));
    let before = count_item(&player, &Item::AMETHYST_SHARD);
    bot.send_command("loot give @s loot loot_test_outer")
        .await?;
    for _ in 0..40 {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
        if count_item(&player, &Item::RED_WOOL) == 1
            && count_item(&player, &Item::AMETHYST_SHARD) == before + 15
        {
            break;
        }
    }
    sheep.get_entity().remove();
    if count_item(&player, &Item::RED_WOOL) != 1 {
        return Err("/loot kill lost the entity's component context".into());
    }
    if count_item(&player, &Item::AMETHYST_SHARD) != before + 15 {
        return Err(format!(
            "Nested datapack loot produced {} shards, expected {}",
            count_item(&player, &Item::AMETHYST_SHARD),
            before + 15
        ));
    }

    let recursive = parse_loot_table(r#"{"pools":[{"entries":[{"type":"minecraft:loot_table","value":"loot_test:recursive"}]}]}"#).ok_or("Invalid recursive loot fixture")?;
    server
        .datapack_manager
        .insert_loot_table("loot_test:recursive".into(), Arc::new(recursive));
    let handle = world
        .get_loot_table("loot_test:recursive")
        .ok_or("Missing recursive table")?;
    if !handle
        .generate_loot_with_context(
            0,
            &pumpkin_core::world::loot::LootContextParameters {
                world: Some(&world),
                ..Default::default()
            },
        )
        .is_empty()
    {
        return Err("Recursive loot should stop without emitting items".into());
    }
    summaries.push(format!(
        "[PASS] bot:loot_entity_context_and_nested_datapack_tables ({}ms)",
        start.elapsed().as_millis()
    ));
    Ok(())
}

fn count_item(player: &pumpkin_core::entity::player::Player, item: &Item) -> u32 {
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
