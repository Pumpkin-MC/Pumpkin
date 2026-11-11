use std::sync::Arc;

use pumpkin_data::{entity::EntityType, item::Item, item_stack::ItemStack};
use pumpkin_gametest::{GameTestContext, GameTestError, GameTestResult};
use pumpkin_util::{GameMode, Hand, math::vector3::Vector3};

use crate::{
    entity::{Entity, EntityBase, passive::allay::AllayEntity, player::Player},
    server::server_test_manager::ServerSimulatedPlayer,
};

use super::build_stone_floor;

fn interact(player: &Arc<Player>, allay: &Arc<AllayEntity>, hand: Hand) -> GameTestResult<()> {
    let inventory = player.inventory();
    let mut stack = inventory.get_stack_in_hand(hand);
    let target: Arc<dyn EntityBase> = allay.clone();
    let handled = target.interact_with_hand(player, &mut stack, hand);
    let interacted = handled.unwrap_or_else(|| target.interact(player, &mut stack));
    if !interacted {
        let server = player
            .world()
            .server
            .upgrade()
            .ok_or_else(|| GameTestError::World("Server stopped".to_string()))?;
        server
            .item_registry
            .use_on_entity(&mut stack, player, target);
    }
    inventory.set_stack_in_hand(hand, stack);
    Ok(())
}

#[expect(
    clippy::too_many_lines,
    reason = "covers the Allay interaction sequence"
)]
pub(super) async fn test_allay_interaction(test: GameTestContext) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let simulated = test
        .spawn_simulated_player(
            "AllayTester",
            Vector3::new(1.5, 1.0, 1.5),
            GameMode::Survival,
        )
        .await?;
    let player = simulated
        .as_any()
        .downcast_ref::<ServerSimulatedPlayer>()
        .ok_or_else(|| GameTestError::World("Expected server simulated player".to_string()))?
        .player()
        .clone();
    let world = player.world();
    let allay = AllayEntity::new(Entity::new(
        world.clone(),
        test.absolute_vec(Vector3::new(2.5, 1.0, 1.5)),
        &EntityType::ALLAY,
    ));
    allay.mob_entity.set_no_ai(true);
    world.spawn_entity(allay.clone());

    let inventory = player.inventory();
    let mut sword = ItemStack::new(1, &Item::DIAMOND_SWORD);
    sword.set_custom_name("Allay sample".to_string());
    inventory.set_stack_in_hand(Hand::Right, ItemStack::new(2, &Item::DIAMOND));
    inventory.set_stack_in_hand(Hand::Left, sword.clone());
    interact(&player, &allay, Hand::Left)?;
    test.assert_true(
        allay
            .mob_entity
            .living_entity
            .held_item(allay.as_ref())
            .are_equal(&sword)
            && inventory.get_stack_in_hand(Hand::Left).is_empty()
            && inventory.get_stack_in_hand(Hand::Right).item_count == 2,
        "Allay did not take the offhand sample with its components",
    )?;

    interact(&player, &allay, Hand::Left)?;
    test.assert_true(
        allay
            .mob_entity
            .living_entity
            .held_item(allay.as_ref())
            .are_equal(&sword),
        "An empty offhand took the Allay sample",
    )?;

    inventory.set_stack_in_hand(Hand::Right, ItemStack::EMPTY.clone());
    let first_free_slot = inventory
        .main_inventory
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
        .position(ItemStack::is_empty)
        .ok_or_else(|| GameTestError::World("Simulated player inventory is full".to_string()))?;
    interact(&player, &allay, Hand::Right)?;
    test.assert_true(
        inventory
            .main_inventory
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)[first_free_slot]
            .are_equal(&sword)
            && allay
                .mob_entity
                .living_entity
                .held_item(allay.as_ref())
                .is_empty(),
        "Taking the sample did not return it to the first free slot",
    )?;

    let mut name_tag = ItemStack::new(1, &Item::NAME_TAG);
    name_tag.set_custom_name("Named Allay".to_string());
    inventory.set_stack_in_hand(Hand::Right, name_tag);
    interact(&player, &allay, Hand::Right)?;
    test.assert_true(
        allay
            .get_entity()
            .custom_name
            .load()
            .as_ref()
            .as_ref()
            .map(|name| name.clone().get_text())
            .as_deref()
            == Some("Named Allay")
            && allay
                .mob_entity
                .living_entity
                .held_item(allay.as_ref())
                .is_empty(),
        "Allay took the name tag instead of being named",
    )?;

    inventory.set_stack_in_hand(Hand::Right, ItemStack::new(1, &Item::LEAD));
    interact(&player, &allay, Hand::Right)?;
    test.assert_true(
        allay.get_entity().is_leashed()
            && allay
                .mob_entity
                .living_entity
                .held_item(allay.as_ref())
                .is_empty(),
        "Allay took the lead instead of being leashed",
    )?;
    inventory.set_stack_in_hand(Hand::Right, ItemStack::EMPTY.clone());
    interact(&player, &allay, Hand::Right)?;
    test.assert_true(
        !allay.get_entity().is_leashed()
            && allay
                .mob_entity
                .living_entity
                .held_item(allay.as_ref())
                .is_empty(),
        "Empty hand did not unleash the Allay",
    )?;

    allay.mob_entity.living_entity.health.store(0.0);
    inventory.set_stack_in_hand(Hand::Right, sword.clone());
    interact(&player, &allay, Hand::Right)?;
    test.assert_true(
        inventory.get_stack_in_hand(Hand::Right).are_equal(&sword),
        "Dying Allay accepted an item",
    )?;
    test.start_sequence()
        .then_idle(10)
        .then_execute(move |test| {
            let allay = allay.clone();
            let player = player.clone();
            let sword = sword.clone();
            async move {
                interact(&player, &allay, Hand::Right)?;
                test.assert_true(
                    allay
                        .mob_entity
                        .living_entity
                        .held_item(allay.as_ref())
                        .is_empty()
                        && player
                            .inventory()
                            .get_stack_in_hand(Hand::Right)
                            .are_equal(&sword),
                    "Dying Allay kept its sample or changed the player's item",
                )
            }
        })
        .then_succeed();
    Ok(())
}
