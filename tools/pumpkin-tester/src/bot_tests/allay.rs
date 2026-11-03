use std::{sync::Arc, time::Duration};

use pumpkin_core::{
    entity::{Entity, EntityBase, passive::allay::AllayEntity, player::Player},
    server::Server,
};
use pumpkin_data::{
    entity::EntityType, item::Item, item_stack::ItemStack, statistic::StatisticCategory,
};
use pumpkin_protocol::{codec::var_int::VarInt, java::server::play::SInteract};
use pumpkin_util::{GameMode, Hand, math::vector3::Vector3};

use crate::bot::{HeadlessBot, MAX_POLL_TICKS};

const MAX_INTERACT_TICKS: usize = MAX_POLL_TICKS;
// A longer wait would let the death animation finish and remove the Allay.
const DYING_INTERACT_TICKS: usize = 10;

fn ticks_elapsed(limit: usize) -> impl FnMut() -> bool {
    let mut ticks = 0;
    move || {
        ticks += 1;
        ticks >= limit
    }
}

async fn interact(
    bot: &mut HeadlessBot,
    server: &Arc<Server>,
    allay: &AllayEntity,
    hand: i32,
    mut done: impl FnMut() -> bool,
) -> Result<(), String> {
    bot.send_packet(&SInteract {
        entity_id: VarInt(allay.get_entity().entity_id),
        r#type: VarInt(2),
        target_position: Some(Vector3::new(0.0, 0.3, 0.0)),
        hand: Some(VarInt(hand)),
        sneaking: false,
    })
    .await?;
    for _ in 0..MAX_INTERACT_TICKS {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
        if done() {
            break;
        }
    }
    Ok(())
}

pub async fn run_allay_interaction_test(
    server: &Arc<Server>,
    bot: &mut HeadlessBot,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let player = server.get_player_by_name(&bot.name).ok_or("missing bot")?;
    let world = player.world();
    let allay = AllayEntity::new(Entity::new(
        world.clone(),
        player.position() + Vector3::new(1.0, 0.0, 0.0),
        &EntityType::ALLAY,
    ));
    allay.mob_entity.set_no_ai(true);
    world.spawn_entity(allay.clone());
    let living = &allay.mob_entity.living_entity;
    let original_mode = player.gamemode.load();
    player.gamemode.store(GameMode::Survival);
    let inventory = player.inventory();
    let mut sword = ItemStack::new(1, &Item::DIAMOND_SWORD);
    sword.set_custom_name("Allay sample".to_string());
    inventory.set_stack_in_hand(Hand::Right, ItemStack::new(2, &Item::DIAMOND));
    inventory.set_stack_in_hand(Hand::Left, sword.clone());
    let has_sample = || !living.held_item(allay.as_ref()).is_empty();
    interact(bot, server, &allay, 1, has_sample).await?;
    if !living.held_item(allay.as_ref()).are_equal(&sword)
        || !inventory.get_stack_in_hand(Hand::Left).is_empty()
        || inventory.get_stack_in_hand(Hand::Right).item_count != 2
    {
        return Err("Allay did not transfer the offhand sample with its components".to_string());
    }
    {
        let stats = player
            .stats
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if stats.get(StatisticCategory::Broken, i32::from(Item::DIAMOND_SWORD.id)) != 0
            || stats.get(StatisticCategory::Used, i32::from(Item::DIAMOND_SWORD.id)) != 0
        {
            return Err("Giving an Allay a sword changed its used/broken statistics".to_string());
        }
    }

    interact(bot, server, &allay, 1, ticks_elapsed(20)).await?;
    if living.held_item(allay.as_ref()).is_empty() {
        return Err("An empty offhand took the Allay sample".to_string());
    }
    inventory.set_stack_in_hand(Hand::Right, ItemStack::EMPTY.clone());
    let first_free_slot = inventory
        .main_inventory
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
        .position(ItemStack::is_empty)
        .ok_or("bot inventory is full")?;
    interact(bot, server, &allay, 0, || !has_sample()).await?;
    let returned = std::mem::replace(
        &mut inventory
            .main_inventory
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)[first_free_slot],
        ItemStack::EMPTY.clone(),
    );
    if !returned.are_equal(&sword) || !living.held_item(allay.as_ref()).is_empty() {
        return Err("Taking the sample did not return it to the first free slot".to_string());
    }

    check_name_tag_lead_and_death(bot, server, &allay, &player, &sword).await?;
    allay.get_entity().remove();
    player.gamemode.store(original_mode);
    inventory.set_stack_in_hand(Hand::Right, ItemStack::EMPTY.clone());
    inventory.set_stack_in_hand(Hand::Left, ItemStack::EMPTY.clone());
    summary_lines.push("[PASS] bot:allay_hand_transfer_and_interaction_order".to_string());
    Ok(())
}

async fn check_name_tag_lead_and_death(
    bot: &mut HeadlessBot,
    server: &Arc<Server>,
    allay: &AllayEntity,
    player: &Player,
    sword: &ItemStack,
) -> Result<(), String> {
    let living = &allay.mob_entity.living_entity;
    let inventory = player.inventory();
    let mut name_tag = ItemStack::new(1, &Item::NAME_TAG);
    name_tag.set_custom_name("Named Allay".to_string());
    inventory.set_stack_in_hand(Hand::Right, name_tag);
    interact(bot, server, allay, 0, || {
        allay.get_entity().custom_name.load().is_some()
    })
    .await?;
    if allay
        .get_entity()
        .custom_name
        .load()
        .as_ref()
        .as_ref()
        .map(|name| name.clone().get_text())
        .as_deref()
        != Some("Named Allay")
        || !living.held_item(allay).is_empty()
    {
        return Err("Allay took a renamed name tag instead of being named".to_string());
    }
    inventory.set_stack_in_hand(Hand::Right, ItemStack::new(1, &Item::LEAD));
    interact(bot, server, allay, 0, || allay.get_entity().is_leashed()).await?;
    if !allay.get_entity().is_leashed() || !living.held_item(allay).is_empty() {
        return Err("Allay took a lead instead of being leashed".to_string());
    }
    interact(bot, server, allay, 0, || !allay.get_entity().is_leashed()).await?;
    if allay.get_entity().is_leashed() {
        return Err("Empty-hand interaction did not unleash the Allay".to_string());
    }
    living.health.store(0.0);
    inventory.set_stack_in_hand(Hand::Right, sword.clone());
    interact(bot, server, allay, 0, ticks_elapsed(DYING_INTERACT_TICKS)).await?;
    if !inventory.get_stack_in_hand(Hand::Right).are_equal(sword)
        || !living.held_item(allay).is_empty()
    {
        return Err("The dying Allay accepted an item after its death drops".to_string());
    }
    Ok(())
}
