use pumpkin_data::Block;
use pumpkin_data::{item::Item, item_stack::ItemStack};
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::{GameMode, Hand, math::vector3::Vector3};

pub(super) async fn test_npc_lifecycle(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(5, 5, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;

    let name = "GameTestPlayer";
    let start = test.absolute_vec(Vector3::new(1.5, 1.0, 1.5));
    let destination = test.absolute_vec(Vector3::new(2.5, 1.0, 1.5));
    let second_position = test.absolute_vec(Vector3::new(3.5, 1.0, 1.5));
    let npc = test
        .spawn_npc(name, Vector3::new(1.5, 1.0, 1.5), GameMode::Creative)
        .await?;

    test.assert_true(
        test.npc_position(name).await == Some(start),
        "NPC was not admitted to the world",
    )?;

    let second = test
        .spawn_npc(name, Vector3::new(3.5, 1.0, 1.5), GameMode::Creative)
        .await?;
    test.assert_true(
        npc.name() != second.name() && test.npc_position(name).await == Some(second_position),
        "NPCs with the same requested name were not admitted independently",
    )?;

    npc.move_to(destination).await?;
    test.assert_true(
        npc.position() == destination && test.npc_position(name).await == Some(second_position),
        "NPC movement did not update the world roster",
    )?;

    second.disconnect().await?;
    test.assert_true(
        test.npc_position(name).await == Some(destination),
        "Disconnecting one NPC removed the other",
    )?;
    npc.disconnect().await?;
    test.assert_true(
        test.npc_position(name).await.is_none(),
        "NPC remained in the world after disconnect",
    )?;
    test.succeed();
    Ok(())
}

pub(super) async fn test_npc_actions(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(5, 5, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;
    let npc = test
        .spawn_npc(
            "ActionTester",
            Vector3::new(2.5, 1.0, 2.5),
            GameMode::Survival,
        )
        .await?;

    let main_hand = ItemStack::new(2, &Item::DIAMOND);
    let off_hand = ItemStack::new(1, &Item::SHIELD);
    npc.set_selected_hotbar_slot(4)?;
    npc.set_held_item(Hand::Right, main_hand.clone())?;
    npc.set_held_item(Hand::Left, off_hand.clone())?;
    test.assert_true(
        npc.selected_hotbar_slot() == 4
            && npc.held_item(Hand::Right).are_equal(&main_hand)
            && npc.held_item(Hand::Left).are_equal(&off_hand),
        "NPC did not equip the requested items",
    )?;
    npc.set_selected_hotbar_slot(0)?;
    test.assert_true(
        npc.held_item(Hand::Right).is_empty() && npc.held_item(Hand::Left).are_equal(&off_hand),
        "Changing hotbar slot did not change the held item",
    )?;
    npc.set_selected_hotbar_slot(4)?;
    test.assert_true(
        npc.held_item(Hand::Right).are_equal(&main_hand),
        "Returning to the hotbar slot did not restore its item",
    )?;

    npc.set_sprinting(true)?;
    test.assert_true(
        npc.is_sprinting(),
        "Starting sprint did not update player state",
    )?;
    npc.set_sprinting(false)?;
    test.assert_true(
        !npc.is_sprinting(),
        "Stopping sprint did not update player state",
    )?;
    test.succeed();
    Ok(())
}
