use pumpkin_data::Block;
use pumpkin_data::{item::Item, item_stack::ItemStack};
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::{GameMode, Hand, math::vector3::Vector3};

pub(super) async fn test_simulated_player_lifecycle(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(5, 5, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;

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
    test.set_dimensions(5, 5, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;
    let player = test
        .spawn_simulated_player(
            "ActionTester",
            Vector3::new(2.5, 1.0, 2.5),
            GameMode::Survival,
        )
        .await?;

    let main_hand = ItemStack::new(2, &Item::DIAMOND);
    let off_hand = ItemStack::new(1, &Item::SHIELD);
    player.set_selected_hotbar_slot(4)?;
    player.set_held_item(Hand::Right, main_hand.clone())?;
    player.set_held_item(Hand::Left, off_hand.clone())?;
    test.assert_true(
        player.selected_hotbar_slot() == 4
            && player.held_item(Hand::Right).are_equal(&main_hand)
            && player.held_item(Hand::Left).are_equal(&off_hand),
        "Simulated player did not equip the requested items",
    )?;
    player.set_selected_hotbar_slot(0)?;
    test.assert_true(
        player.held_item(Hand::Right).is_empty()
            && player.held_item(Hand::Left).are_equal(&off_hand),
        "Changing hotbar slot did not change the held item",
    )?;
    player.set_selected_hotbar_slot(4)?;
    test.assert_true(
        player.held_item(Hand::Right).are_equal(&main_hand),
        "Returning to the hotbar slot did not restore its item",
    )?;

    player.set_sprinting(true)?;
    test.assert_true(
        player.is_sprinting(),
        "Starting sprint did not update player state",
    )?;
    player.set_sprinting(false)?;
    test.assert_true(
        !player.is_sprinting(),
        "Stopping sprint did not update player state",
    )?;
    test.succeed();
    Ok(())
}
