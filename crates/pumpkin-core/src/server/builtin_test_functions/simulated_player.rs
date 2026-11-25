use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_data::{item::Item, item_stack::ItemStack};
use pumpkin_gametest::{GameTestContext, GameTestError, GameTestResult};
use pumpkin_util::{GameMode, Hand, math::vector3::Vector3};

use crate::{entity::EntityBase, server::server_test_manager::server_world};

pub(super) async fn test_simulated_player_is_ephemeral(
    test: GameTestContext,
) -> GameTestResult<()> {
    test.set_dimensions(3, 3, 3).await?;
    test.build_floor(3, 3, &Block::STONE).await?;

    let world = server_world(&test)?;
    let server = world
        .server
        .upgrade()
        .ok_or_else(|| GameTestError::World("Server stopped".to_string()))?;
    let simulated_player = test
        .spawn_simulated_player(
            "EphemeralPlayer",
            Vector3::new(1.5, 1.0, 1.5),
            GameMode::Survival,
        )
        .await?;
    let player = world
        .get_player_by_id(simulated_player.entity_id())
        .ok_or_else(|| GameTestError::World("Simulated player is missing".to_string()))?;
    let uuid = player.gameprofile.id;

    let save_server = server.clone();
    let save_player = player.clone();
    tokio::task::spawn_blocking(move || {
        save_server
            .player_data_storage
            .extract_data_and_save_player(&save_player)
    })
    .await
    .map_err(|error| GameTestError::World(error.to_string()))?
    .map_err(|error| GameTestError::World(error.to_string()))?;
    server
        .advancement_manager
        .save_player(&player)
        .await
        .map_err(|error| GameTestError::World(error.to_string()))?;
    simulated_player.disconnect().await?;

    let cached = server
        .data
        .user_cache
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_by_uuid(uuid)
        .is_some();
    test.assert_true(!cached, "A simulated player was added to the user cache")?;

    let data = tokio::task::spawn_blocking(move || server.player_data_storage.load_data(&uuid))
        .await
        .map_err(|error| GameTestError::World(error.to_string()))?
        .map_err(|error| GameTestError::World(error.to_string()))?;
    test.assert_true(data.is_none(), "A simulated player's data was saved")?;

    let advancement_path = player
        .advancements
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .path
        .clone();
    let advancement_file = tokio::fs::try_exists(advancement_path)
        .await
        .map_err(|error| GameTestError::World(error.to_string()))?;
    test.assert_true(
        !advancement_file,
        "A simulated player's advancements were saved",
    )?;
    test.succeed();
    Ok(())
}

pub(super) async fn test_simulated_player_lifecycle(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(5, 5, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;

    let name = "GameTestPlayer";
    let start = test.absolute_vec(Vector3::new(1.5, 1.0, 1.5));
    let destination = test.absolute_vec(Vector3::new(2.5, 1.0, 1.5));
    let second_position = test.absolute_vec(Vector3::new(3.5, 1.0, 1.5));
    let simulated_player = test
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
        simulated_player.name() != second.name()
            && test.simulated_player_position(name).await == Some(second_position),
        "Simulated players with the same requested name were not admitted independently",
    )?;

    simulated_player.move_to(destination).await?;
    test.assert_true(
        simulated_player.position() == destination
            && test.simulated_player_position(name).await == Some(second_position),
        "Simulated player movement did not update the world roster",
    )?;

    second.disconnect().await?;
    test.assert_true(
        test.simulated_player_position(name).await == Some(destination),
        "Disconnecting one simulated player removed the other",
    )?;
    let world = server_world(&test)?;
    let server = world
        .server
        .upgrade()
        .ok_or_else(|| GameTestError::World("Server stopped".to_string()))?;
    let other_world = server
        .worlds
        .load()
        .iter()
        .find(|candidate| !Arc::ptr_eq(candidate, &world))
        .cloned()
        .ok_or_else(|| GameTestError::World("Cross-world teleport needs two worlds".to_string()))?;
    let player_id = simulated_player.entity_id();
    let player = world
        .get_player_by_id(player_id)
        .ok_or_else(|| GameTestError::World("Simulated player is missing".to_string()))?;
    player.teleport(start, None, None, other_world.clone());

    let destination_world = other_world.clone();
    test.start_sequence()
        .then_wait_until(move |test| {
            let player = player.clone();
            let world = world.clone();
            let destination_world = destination_world.clone();
            async move {
                test.assert_true(
                    Arc::ptr_eq(&player.world(), &destination_world)
                        && player.position() == start
                        && world.get_player_by_id(player_id).is_none()
                        && destination_world.get_player_by_id(player_id).is_some(),
                    "Simulated player did not teleport to the destination world",
                )
            }
        })
        .then_execute(move |test| {
            let simulated_player = simulated_player.clone();
            let other_world = other_world.clone();
            async move {
                simulated_player.disconnect().await?;
                test.assert_true(
                    other_world.get_player_by_id(player_id).is_none(),
                    "Simulated player remained in the destination world after disconnect",
                )
            }
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_simulated_player_actions(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(5, 5, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;
    let simulated_player = test
        .spawn_simulated_player(
            "ActionTester",
            Vector3::new(2.5, 1.0, 2.5),
            GameMode::Survival,
        )
        .await?;

    let main_hand = ItemStack::new(2, &Item::DIAMOND);
    let off_hand = ItemStack::new(1, &Item::SHIELD);
    simulated_player.set_selected_hotbar_slot(4)?;
    simulated_player.set_held_item(Hand::Right, main_hand.clone())?;
    simulated_player.set_held_item(Hand::Left, off_hand.clone())?;
    test.assert_true(
        simulated_player.selected_hotbar_slot() == 4
            && simulated_player
                .held_item(Hand::Right)
                .are_equal(&main_hand)
            && simulated_player.held_item(Hand::Left).are_equal(&off_hand),
        "Simulated player did not equip the requested items",
    )?;
    simulated_player.set_selected_hotbar_slot(0)?;
    test.assert_true(
        simulated_player.held_item(Hand::Right).is_empty()
            && simulated_player.held_item(Hand::Left).are_equal(&off_hand),
        "Changing hotbar slot did not change the held item",
    )?;
    simulated_player.set_selected_hotbar_slot(4)?;
    test.assert_true(
        simulated_player
            .held_item(Hand::Right)
            .are_equal(&main_hand),
        "Returning to the hotbar slot did not restore its item",
    )?;

    simulated_player.set_sprinting(true)?;
    test.assert_true(
        simulated_player.is_sprinting(),
        "Starting sprint did not update player state",
    )?;
    simulated_player.set_sprinting(false)?;
    test.assert_true(
        !simulated_player.is_sprinting(),
        "Stopping sprint did not update player state",
    )?;
    test.succeed();
    Ok(())
}
