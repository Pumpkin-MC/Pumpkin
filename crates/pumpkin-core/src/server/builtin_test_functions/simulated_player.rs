use pumpkin_data::{item::Item, item_stack::ItemStack};
use pumpkin_gametest::{GameTestContext, GameTestError, GameTestResult};
use pumpkin_util::{GameMode, Hand, math::vector3::Vector3};

use crate::{entity::EntityBase, server::server_test_manager::ServerSimulatedPlayer};

use super::build_stone_floor;

pub(super) async fn test_simulated_player_lifecycle(test: GameTestContext) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;

    let name = "GameTestPlayer";
    let start = test.absolute_vec(Vector3::new(1.5, 1.0, 1.5));
    let destination = test.absolute_vec(Vector3::new(2.5, 1.0, 1.5));
    let second_position = test.absolute_vec(Vector3::new(3.5, 1.0, 1.5));
    let player = test
        .spawn_simulated_player(name, Vector3::new(1.5, 1.0, 1.5), GameMode::Creative)
        .await?;

    test.assert_true(
        test.simulated_player_position(name).await == Some(start),
        "Simulated player was not admitted to the world",
    )?;

    let second = test
        .spawn_simulated_player(name, Vector3::new(3.5, 1.0, 1.5), GameMode::Creative)
        .await?;
    test.assert_true(
        player.name() != second.name()
            && test.simulated_player_position(name).await == Some(second_position),
        "Players with the same requested name were not admitted independently",
    )?;

    player.move_to(destination).await?;
    test.assert_true(
        player.position() == destination
            && test.simulated_player_position(name).await == Some(second_position),
        "Simulated player movement did not update the world roster",
    )?;

    second.disconnect().await?;
    test.assert_true(
        test.simulated_player_position(name).await == Some(destination),
        "Disconnecting one simulated player removed the other",
    )?;
    player.disconnect().await?;
    test.assert_true(
        test.simulated_player_position(name).await.is_none(),
        "Simulated player remained in the world after disconnect",
    )?;
    test.succeed();
    Ok(())
}

pub(super) async fn test_simulated_player_actions(test: GameTestContext) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let simulated = test
        .spawn_simulated_player(
            "ActionTester",
            Vector3::new(2.5, 1.0, 2.5),
            GameMode::Survival,
        )
        .await?;

    let main_hand = ItemStack::new(2, &Item::DIAMOND);
    let off_hand = ItemStack::new(1, &Item::SHIELD);
    simulated.set_selected_hotbar_slot(4)?;
    simulated.set_held_item(Hand::Right, main_hand.clone())?;
    simulated.set_held_item(Hand::Left, off_hand.clone())?;
    test.assert_true(
        simulated.selected_hotbar_slot() == 4
            && simulated.held_item(Hand::Right).are_equal(&main_hand)
            && simulated.held_item(Hand::Left).are_equal(&off_hand),
        "Simulated player did not equip the requested items",
    )?;
    simulated.set_selected_hotbar_slot(0)?;
    test.assert_true(
        simulated.held_item(Hand::Right).is_empty()
            && simulated.held_item(Hand::Left).are_equal(&off_hand),
        "Changing hotbar slot did not change the held item",
    )?;
    simulated.set_selected_hotbar_slot(4)?;
    test.assert_true(
        simulated.held_item(Hand::Right).are_equal(&main_hand),
        "Returning to the hotbar slot did not restore its item",
    )?;

    let player = simulated
        .as_any()
        .downcast_ref::<ServerSimulatedPlayer>()
        .ok_or_else(|| GameTestError::World("Expected server simulated player".to_string()))?
        .player();
    let server = player
        .world()
        .server
        .upgrade()
        .ok_or_else(|| GameTestError::World("Server stopped".to_string()))?;

    player.apply_sprint_input(&server, true);
    test.assert_true(
        player.get_entity().is_sprinting(),
        "Starting sprint did not update player state",
    )?;
    player.apply_sprint_input(&server, false);
    test.assert_true(
        !player.get_entity().is_sprinting(),
        "Stopping sprint did not update player state",
    )?;
    test.succeed();
    Ok(())
}
