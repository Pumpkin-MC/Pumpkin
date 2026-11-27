use std::sync::{Arc, PoisonError, atomic::Ordering};

use pumpkin_data::{
    Block, block_properties::OakSignLikeProperties, item::Item, item_stack::ItemStack,
};
use pumpkin_gametest::{GameTestContext, GameTestError, GameTestResult, SimulatedPlayer};
use pumpkin_util::{
    GameMode, Hand, PermissionLvl,
    math::{position::BlockPos, vector3::Vector3},
    text::TextComponent,
};

use crate::{
    block::entities::{
        hanging_sign::HangingSignBlockEntity,
        sign::{DyeColor, SignBlockEntity, SignEntityRef},
    },
    entity::player::Player,
    server::server_test_manager::server_world,
    world::World,
};

pub(super) async fn test_sign_authorization_click_commands_and_applicators(
    test: GameTestContext,
) -> GameTestResult<()> {
    test.set_dimensions(21, 7, 21).await?;
    let pos = BlockPos::new(10, 1, 10);
    test.set_block_default(&pos.down(), &Block::STONE).await?;
    let mut properties = OakSignLikeProperties::from_state_id(Block::OAK_SIGN.default_state.id);
    properties.rotation = 0;
    test.set_block(&pos, properties.to_state_id(&Block::OAK_SIGN))
        .await?;

    let hanging = pos.east();
    test.set_block_default(&hanging.up(), &Block::STONE).await?;
    test.set_block_default(&hanging, &Block::OAK_HANGING_SIGN)
        .await?;
    let marker = pos.offset(Vector3::new(0, 4, 0));
    test.set_air(&marker).await?;

    let world = server_world(&test)?;
    let sign = Arc::new(SignBlockEntity::empty(test.absolute_pos(&pos)));
    world.add_block_entity(sign.clone());
    let hanging_sign = Arc::new(HangingSignBlockEntity::empty(test.absolute_pos(&hanging)));
    world.add_block_entity(hanging_sign.clone());

    // Rotation zero faces south, including when the test layout is rotated.
    let home = test.absolute_pos(&pos).to_f64().add_raw(0.0, 0.0, 2.0);
    let player = test
        .spawn_simulated_player(
            "SignTester",
            Vector3::new(10.5, 1.0, 12.5),
            GameMode::Survival,
        )
        .await?;
    player.teleport_to(home)?;
    player.set_permission_level(PermissionLvl::Zero)?;
    let server_player = world
        .get_player_by_id(player.entity_id())
        .ok_or_else(|| GameTestError::World("Simulated sign editor is missing".to_string()))?;

    check_authorization(
        &test,
        player.as_ref(),
        &server_player,
        &SignEntityRef::Sign(&sign),
    )?;
    let hanging_sign = SignEntityRef::Hanging(&hanging_sign);
    submit(&server_player, &hanging_sign, "HANGING");
    test.assert_true(
        stored_text(&hanging_sign) == "HANGING",
        "The registered editor could not update a hanging sign",
    )?;

    let marker = test.absolute_pos(&marker);
    check_click_commands(
        &test,
        player.as_ref(),
        &server_player,
        &sign,
        &world,
        marker,
    )?;
    check_applicators(
        &test,
        player.as_ref(),
        &server_player,
        &sign,
        &world,
        marker,
    )?;
    test.succeed();
    Ok(())
}

fn check_authorization(
    test: &GameTestContext,
    player: &dyn SimulatedPlayer,
    server_player: &Player,
    sign: &SignEntityRef<'_>,
) -> GameTestResult<()> {
    let pos = sign.block_pos();
    server_player.update_sign_text(pos, true, ["NO_EDITOR", "", "", ""]);
    test.assert_true(
        stored_text(sign).is_empty(),
        "A sign without an editor accepted an update",
    )?;

    set_editor(sign, Some(uuid::Uuid::new_v4()));
    server_player.update_sign_text(pos, true, ["WRONG_EDITOR", "", "", ""]);
    test.assert_true(
        stored_text(sign).is_empty(),
        "A sign accepted an update from someone other than its editor",
    )?;

    let home = player.position();
    let far = pos
        .to_f64()
        .add_raw(server_player.block_interaction_range() + 6.0, 0.0, 0.0);
    player.teleport_to(far)?;
    submit(server_player, sign, "OUT_OF_RANGE");
    test.assert_true(
        stored_text(sign).is_empty(),
        "A sign accepted an update past its edit range",
    )?;
    test.assert_true(
        editor(sign).is_none(),
        "The out-of-range sign editor was not cleared",
    )?;

    player.teleport_to(home)?;
    server_player.update_sign_text(pos, true, ["RETURNED", "", "", ""]);
    test.assert_true(
        stored_text(sign).is_empty(),
        "Returning to range restored a cleared editor",
    )?;

    player.teleport_to(pos.to_f64().add_raw(6.0, 0.0, 0.0))?;
    test.assert_true(
        !server_player.can_interact_with_block_at(&pos, 0.0)
            && server_player.can_interact_with_block_at(&pos, 4.0),
        "The editor must be outside interaction range and inside the edit buffer",
    )?;
    submit(server_player, sign, "BUFFER");
    test.assert_true(
        stored_text(sign) == "BUFFER",
        "An update inside the edit buffer was rejected",
    )?;
    player.teleport_to(far)?;
    submit(server_player, sign, "BEYOND");
    test.assert_true(
        stored_text(sign) == "BUFFER",
        "An update past the edit buffer was accepted",
    )?;
    player.teleport_to(home)?;

    sign.set_waxed(true);
    submit(server_player, sign, "WAXED");
    test.assert_true(
        stored_text(sign) == "BUFFER",
        "A waxed sign accepted an update",
    )?;
    sign.set_waxed(false);

    submit(server_player, sign, "LEGIT");
    test.assert_true(
        stored_text(sign) == "LEGIT",
        "A registered editor in range could not update the sign",
    )?;
    server_player.open_sign_editor(pos, true);
    server_player.update_sign_text(pos, true, ["PLUGIN", "", "", ""]);
    test.assert_true(
        stored_text(sign) == "PLUGIN",
        "Opening the sign through the plugin API did not authorize editing",
    )?;
    player.use_block(&pos, Hand::Right)?;
    server_player.update_sign_text(pos, true, ["OPENED", "", "", ""]);
    test.assert_true(
        stored_text(sign) == "OPENED",
        "Interacting with the sign did not authorize editing",
    )
}

fn check_click_commands(
    test: &GameTestContext,
    player: &dyn SimulatedPlayer,
    server_player: &Player,
    sign: &SignBlockEntity,
    world: &Arc<World>,
    marker: BlockPos,
) -> GameTestResult<()> {
    let sign_ref = SignEntityRef::Sign(sign);
    let pos = sign_ref.block_pos();
    let crafted = crafted_click_line(marker);

    submit(server_player, &sign_ref, &crafted);
    test.assert_true(
        stored_text(&sign_ref) == crafted,
        "Client text was not stored literally",
    )?;
    player.use_block(&pos, Hand::Right)?;
    test.assert_true(
        world.get_block(&marker) == &Block::AIR,
        "Client text injected a sign command",
    )?;

    set_click_component(sign, &crafted, true);
    player.use_block(&pos, Hand::Right)?;
    test.assert_true(
        world.get_block(&marker) == &Block::EMERALD_BLOCK,
        "A sign with operator features did not run its click command",
    )?;

    world.set_block_state(
        &marker,
        Block::AIR.default_state.id,
        crate::world::BlockFlags::NOTIFY_ALL,
    );
    set_click_component(sign, &crafted, false);
    player.use_block(&pos, Hand::Right)?;
    test.assert_true(
        world.get_block(&marker) == &Block::AIR,
        "A sign without operator features ran a click command",
    )?;
    test.assert_true(
        editor(&sign_ref).is_none(),
        "A disabled command click opened the sign editor",
    )?;

    set_click_component(sign, "", true);
    submit(server_player, &sign_ref, "trusted");
    submit(server_player, &sign_ref, &crafted);
    test.assert_true(
        stored_text(&sign_ref) == crafted,
        "Client text on a sign with operator features was not stored literally",
    )?;
    player.use_block(&pos, Hand::Right)?;
    test.assert_true(
        world.get_block(&marker) == &Block::AIR,
        "Editing a sign with operator features injected a command",
    )?;

    let dialog = r#"{"text":"dialog","click_event":{"action":"show_dialog","dialog":{"type":"minecraft:notice","title":"x"}}}"#;
    set_click_component(sign, dialog, false);
    player.use_block(&pos, Hand::Right)?;
    test.assert_true(
        editor(&sign_ref).is_none(),
        "A disabled dialog click opened the sign editor",
    )
}

fn check_applicators(
    test: &GameTestContext,
    player: &dyn SimulatedPlayer,
    server_player: &Player,
    sign: &SignBlockEntity,
    world: &World,
    marker: BlockPos,
) -> GameTestResult<()> {
    let sign_ref = SignEntityRef::Sign(sign);
    let text = sign_ref.front_text();
    let pos = sign_ref.block_pos();
    set_click_component(sign, &crafted_click_line(marker), false);
    apply(player, pos, &Item::RED_DYE)?;
    test.assert_true(
        text.get_color() == DyeColor::Red,
        "Dye did not apply to a sign with a click command",
    )?;
    test.assert_true(
        world.get_block(&marker) == &Block::AIR,
        "Dye ran a disabled sign click command",
    )?;

    set_click_component(sign, "", false);
    text.set_color(DyeColor::Black);
    submit(server_player, &sign_ref, "APPLY");

    apply(player, pos, &Item::RED_DYE)?;
    test.assert_true(
        text.get_color() == DyeColor::Red,
        "Dye did not apply to the sign",
    )?;
    apply(player, pos, &Item::GLOW_INK_SAC)?;
    test.assert_true(
        text.has_glowing_text(),
        "Glow ink did not make the sign text glow",
    )?;
    apply(player, pos, &Item::INK_SAC)?;
    test.assert_true(
        !text.has_glowing_text(),
        "Ink did not remove the sign text glow",
    )?;
    apply(player, pos, &Item::HONEYCOMB)?;
    test.assert_true(sign_ref.is_waxed(), "Honeycomb did not wax the sign")
}

fn apply(player: &dyn SimulatedPlayer, pos: BlockPos, item: &'static Item) -> GameTestResult<()> {
    player.set_held_item(Hand::Right, ItemStack::new(1, item))?;
    player.use_block(&pos, Hand::Right)
}

fn submit(player: &Player, sign: &SignEntityRef<'_>, line: &str) {
    set_editor(sign, Some(player.gameprofile.id));
    player.update_sign_text(sign.block_pos(), true, [line, "", "", ""]);
}

fn editor(sign: &SignEntityRef<'_>) -> Option<uuid::Uuid> {
    *sign
        .currently_editing_player()
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

fn set_editor(sign: &SignEntityRef<'_>, editor: Option<uuid::Uuid>) {
    *sign
        .currently_editing_player()
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = editor;
}

fn stored_text(sign: &SignEntityRef<'_>) -> String {
    let raw = sign.front_text().get_message(0, false);
    serde_json::from_str::<TextComponent>(&raw)
        .map_or_else(|_| raw.to_string(), TextComponent::get_text)
}

fn set_click_component(sign: &SignBlockEntity, line: &str, allow_op_features: bool) {
    let messages = [line, "", "", ""].map(Box::<str>::from);
    sign.front_text
        .messages
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone_from(&messages);
    *sign
        .front_text
        .filtered_messages
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = messages;
    sign.allow_op_features
        .store(allow_op_features, Ordering::Relaxed);
    set_editor(&SignEntityRef::Sign(sign), None);
}

fn crafted_click_line(marker: BlockPos) -> String {
    format!(
        r#"{{"text":"click","click_event":{{"action":"run_command","command":"/setblock {} {} {} minecraft:emerald_block"}}}}"#,
        marker.0.x, marker.0.y, marker.0.z
    )
}
