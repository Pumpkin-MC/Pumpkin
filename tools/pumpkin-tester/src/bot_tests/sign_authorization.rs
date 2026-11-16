use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use pumpkin_core::{
    block::entities::{
        hanging_sign::HangingSignBlockEntity,
        sign::{DyeColor, SignBlockEntity, SignEntityRef},
    },
    entity::EntityBase,
    server::Server,
};
use pumpkin_data::{Block, dimension::Dimension, item::Item, item_stack::ItemStack};
use pumpkin_protocol::java::server::play::SUpdateSign;
use pumpkin_util::{
    GameMode, Hand, PermissionLvl, math::position::BlockPos, math::vector3::Vector3,
};

use crate::bot::{HeadlessBot, MAX_POLL_TICKS};

const BOT_NAME: &str = "SignAuthBot";

fn stored_line(world: &pumpkin_core::world::World, pos: BlockPos) -> String {
    world
        .get_block_entity(&pos)
        .and_then(|entity| {
            SignEntityRef::from_block_entity(&*entity)
                .map(|sign| sign.front_text().get_message(0, false).to_string())
        })
        .unwrap_or_default()
}

fn stored_text(world: &pumpkin_core::world::World, pos: BlockPos) -> String {
    let raw = stored_line(world, pos);
    serde_json::from_str::<pumpkin_util::text::TextComponent>(&raw)
        .map(|component| component.0.to_bedrock_string())
        .unwrap_or(raw)
}

fn sign_editor(world: &pumpkin_core::world::World, pos: BlockPos) -> Option<uuid::Uuid> {
    world.get_block_entity(&pos).and_then(|entity| {
        SignEntityRef::from_block_entity(&*entity).and_then(|sign| {
            *sign
                .currently_editing_player()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        })
    })
}

fn set_editor(world: &pumpkin_core::world::World, pos: BlockPos, editor: Option<uuid::Uuid>) {
    if let Some(entity) = world.get_block_entity(&pos)
        && let Some(sign) = SignEntityRef::from_block_entity(&*entity)
    {
        *sign
            .currently_editing_player()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = editor;
    }
}

fn set_allow_op_features(world: &pumpkin_core::world::World, pos: BlockPos, allow: bool) {
    if let Some(entity) = world.get_block_entity(&pos)
        && let Some(SignEntityRef::Sign(sign)) = SignEntityRef::from_block_entity(&*entity)
    {
        sign.allow_op_features.store(allow, Ordering::Relaxed);
    }
}

fn set_click_component(
    world: &pumpkin_core::world::World,
    pos: BlockPos,
    line: &str,
    allow_op_features: bool,
) {
    if let Some(entity) = world.get_block_entity(&pos)
        && let Some(SignEntityRef::Sign(sign)) = SignEntityRef::from_block_entity(&*entity)
    {
        let messages = [
            Box::<str>::from(line),
            Box::from(""),
            Box::from(""),
            Box::from(""),
        ];
        sign.front_text
            .messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone_from(&messages);
        *sign
            .front_text
            .filtered_messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = messages;
        sign.allow_op_features
            .store(allow_op_features, Ordering::Relaxed);
        *sign
            .currently_editing_player
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }
}

async fn tick(server: &Arc<Server>, count: usize) {
    for _ in 0..count {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
    }
}

fn sign_color(world: &pumpkin_core::world::World, pos: BlockPos) -> Option<DyeColor> {
    world.get_block_entity(&pos).and_then(|entity| {
        SignEntityRef::from_block_entity(&*entity).map(|sign| sign.front_text().get_color())
    })
}

fn sign_is_glowing(world: &pumpkin_core::world::World, pos: BlockPos) -> bool {
    world.get_block_entity(&pos).is_some_and(|entity| {
        SignEntityRef::from_block_entity(&*entity)
            .is_some_and(|sign| sign.front_text().has_glowing_text())
    })
}

fn sign_is_waxed(world: &pumpkin_core::world::World, pos: BlockPos) -> bool {
    world.get_block_entity(&pos).is_some_and(|entity| {
        SignEntityRef::from_block_entity(&*entity).is_some_and(|sign| sign.is_waxed())
    })
}

fn crafted_click_line(marker: BlockPos) -> String {
    format!(
        r#"{{"text":"click","click_event":{{"action":"run_command","command":"/setblock {} {} {} minecraft:emerald_block"}}}}"#,
        marker.0.x, marker.0.y, marker.0.z
    )
}

async fn move_to_and_wait(
    server: &Arc<Server>,
    bot: &mut HeadlessBot,
    player: &pumpkin_core::entity::player::Player,
    target: Vector3<f64>,
) -> Result<(), String> {
    bot.move_to(target, true).await?;
    for _ in 0..MAX_POLL_TICKS {
        tick(server, 1).await;
        let pos = player.get_entity().pos.load();
        if (pos.x - target.x).abs() < 0.5 && (pos.z - target.z).abs() < 0.5 {
            return Ok(());
        }
    }
    Err("bot did not reach the requested position".to_string())
}

async fn place_sign(server: &Arc<Server>, world: &pumpkin_core::world::World, pos: BlockPos) {
    world
        .level
        .set_block_state(&pos.down(), Block::STONE.default_state.id);
    world
        .level
        .set_block_state(&pos, Block::OAK_SIGN.default_state.id);
    world.add_block_entity(Arc::new(SignBlockEntity::empty(pos)));
    tick(server, 2).await;
}

async fn send_update(bot: &mut HeadlessBot, pos: BlockPos, line: &str) -> Result<(), String> {
    bot.send_packet(&SUpdateSign {
        location: pos,
        is_front_text: true,
        line_1: line,
        line_2: "",
        line_3: "",
        line_4: "",
    })
    .await
}

async fn click_and_wait(
    server: &Arc<Server>,
    bot: &mut HeadlessBot,
    pos: BlockPos,
    sequence: i32,
) -> Result<(), String> {
    bot.use_block(pos, sequence).await?;
    // The click command runs during the interaction, so a short window is enough.
    tick(server, 40).await;
    Ok(())
}

async fn run_out_of_range_case(
    server: &Arc<Server>,
    world: &pumpkin_core::world::World,
    bot: &mut HeadlessBot,
    player: &pumpkin_core::entity::player::Player,
    base: BlockPos,
) -> Result<(), String> {
    // The registered editor must still be within edit range.
    let out_of_range = base.offset(Vector3::new(4, 0, 0));
    place_sign(server, world, out_of_range).await;
    let chunk = out_of_range.chunk_position();
    let inserted = world
        .forced_chunks
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(chunk);
    let before = stored_text(world, out_of_range);
    if !before.is_empty() {
        return Err(format!("sign was not empty after placement: {before:?}"));
    }
    let pos = player.get_entity().pos.load();
    let far = Vector3::new(pos.x + 100.0, pos.y, pos.z);
    move_to_and_wait(server, bot, player, far).await?;
    set_editor(world, out_of_range, Some(player.gameprofile.id));
    send_update(bot, out_of_range, "OUT_OF_RANGE").await?;
    tick(server, 20).await;
    if !stored_text(world, out_of_range).is_empty() {
        return Err(format!(
            "SIGN_UPDATE changed a sign from out of edit range (player at {:?}, stored {:?})",
            player.get_entity().pos.load(),
            stored_text(world, out_of_range)
        ));
    }
    if sign_editor(world, out_of_range).is_some() {
        return Err("out-of-range sign editor was not cleared".to_string());
    }
    let home = Vector3::new(
        f64::from(base.0.x) + 0.5,
        f64::from(base.0.y),
        f64::from(base.0.z) + 0.5,
    );
    move_to_and_wait(server, bot, player, home).await?;

    // Returning to range must not reauthorize the cleared editor.
    send_update(bot, out_of_range, "RETURNED").await?;
    tick(server, 20).await;
    if !stored_text(world, out_of_range).is_empty() {
        return Err("returning to range re-authorized a cleared sign editor".to_string());
    }

    if inserted {
        world
            .forced_chunks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&chunk);
    }

    Ok(())
}

async fn run_range_buffer_case(
    server: &Arc<Server>,
    world: &pumpkin_core::world::World,
    bot: &mut HeadlessBot,
    player: &pumpkin_core::entity::player::Player,
    base: BlockPos,
) -> Result<(), String> {
    // Vanilla's `playerIsTooFarAwayToEdit` allows four more blocks than the
    // normal interaction range, so an update inside that buffer is still valid.
    let pos = base.offset(Vector3::new(0, 0, 4));
    place_sign(server, world, pos).await;
    let target = Vector3::new(
        f64::from(pos.0.x) + 6.5,
        f64::from(pos.0.y),
        f64::from(pos.0.z) + 0.5,
    );
    move_to_and_wait(server, bot, player, target).await?;
    if player.can_interact_with_block_at(&pos, 0.0) {
        return Err("range buffer case started inside the base interaction range".to_string());
    }
    if !player.can_interact_with_block_at(&pos, 4.0) {
        return Err("range buffer case started outside the edit buffer".to_string());
    }
    set_editor(world, pos, Some(player.gameprofile.id));
    send_update(bot, pos, "BUFFER").await?;
    tick(server, 20).await;
    if stored_text(world, pos) != "BUFFER" {
        return Err(format!(
            "SIGN_UPDATE inside the edit buffer was rejected (player at {:?})",
            player.get_entity().pos.load()
        ));
    }

    // Past the four-block buffer must still be rejected.
    let beyond = 1.0 + player.block_interaction_range() + 4.0 + 1.0;
    let target = Vector3::new(
        f64::from(pos.0.x) + beyond,
        f64::from(pos.0.y),
        f64::from(pos.0.z) + 0.5,
    );
    move_to_and_wait(server, bot, player, target).await?;
    send_update(bot, pos, "BEYOND").await?;
    tick(server, 20).await;
    if stored_text(world, pos) != "BUFFER" {
        return Err("SIGN_UPDATE past the edit buffer was accepted".to_string());
    }
    let home = Vector3::new(
        f64::from(base.0.x) + 0.5,
        f64::from(base.0.y),
        f64::from(base.0.z) + 0.5,
    );
    move_to_and_wait(server, bot, player, home).await?;

    Ok(())
}

async fn run_authorization_cases(
    server: &Arc<Server>,
    world: &pumpkin_core::world::World,
    bot: &mut HeadlessBot,
    player: &pumpkin_core::entity::player::Player,
    base: BlockPos,
) -> Result<(), String> {
    // A sign nobody has opened must not accept an update.
    let no_editor = base.offset(Vector3::new(2, 0, 0));
    place_sign(server, world, no_editor).await;
    send_update(bot, no_editor, "NO_EDITOR").await?;
    tick(server, 20).await;
    if !stored_text(world, no_editor).is_empty() {
        return Err("SIGN_UPDATE changed a sign with no registered editor".to_string());
    }

    // A different player registered as editor must not accept an update.
    let wrong_editor = base.offset(Vector3::new(3, 0, 0));
    place_sign(server, world, wrong_editor).await;
    set_editor(world, wrong_editor, Some(uuid::Uuid::new_v4()));
    send_update(bot, wrong_editor, "WRONG_EDITOR").await?;
    tick(server, 20).await;
    if !stored_text(world, wrong_editor).is_empty() {
        return Err("SIGN_UPDATE changed a sign registered to another player".to_string());
    }

    run_out_of_range_case(server, world, bot, player, base).await?;
    run_range_buffer_case(server, world, bot, player, base).await?;

    // A waxed sign must not accept an update even from its registered editor.
    let waxed = base.offset(Vector3::new(2, 0, 1));
    place_sign(server, world, waxed).await;
    set_editor(world, waxed, Some(player.gameprofile.id));
    if let Some(entity) = world.get_block_entity(&waxed)
        && let Some(sign) = SignEntityRef::from_block_entity(&*entity)
    {
        sign.set_waxed(true);
    }
    send_update(bot, waxed, "WAXED").await?;
    tick(server, 20).await;
    if !stored_text(world, waxed).is_empty() {
        return Err("SIGN_UPDATE changed a waxed sign".to_string());
    }

    // A legitimate editor update still applies.
    let editable = base.offset(Vector3::new(4, 0, 1));
    place_sign(server, world, editable).await;
    set_editor(world, editable, Some(player.gameprofile.id));
    send_update(bot, editable, "LEGIT").await?;
    tick(server, 20).await;
    if stored_text(world, editable) != "LEGIT" {
        return Err("SIGN_UPDATE from the registered editor in range was rejected".to_string());
    }

    // Opening through the plugin API registers the player as the editor.
    let plugin_editor = base.offset(Vector3::new(1, 0, 2));
    place_sign(server, world, plugin_editor).await;
    player.open_sign_editor(plugin_editor, true);
    tick(server, 2).await;
    send_update(bot, plugin_editor, "PLUGIN").await?;
    tick(server, 20).await;
    if stored_text(world, plugin_editor) != "PLUGIN" {
        return Err("plugin-opened sign editor rejected its update".to_string());
    }

    // Opening through player interaction registers the editor too.
    let interacted_editor = base.offset(Vector3::new(3, 0, 2));
    place_sign(server, world, interacted_editor).await;
    bot.use_block(interacted_editor, 899).await?;
    tick(server, 2).await;
    send_update(bot, interacted_editor, "OPENED").await?;
    tick(server, 20).await;
    if stored_text(world, interacted_editor) != "OPENED" {
        return Err("player-opened sign editor rejected its update".to_string());
    }

    // Hanging signs use the same gate.
    let hanging = base.offset(Vector3::new(5, 0, 1));
    world
        .level
        .set_block_state(&hanging.up(), Block::STONE.default_state.id);
    world
        .level
        .set_block_state(&hanging, Block::OAK_HANGING_SIGN.default_state.id);
    world.add_block_entity(Arc::new(HangingSignBlockEntity::empty(hanging)));
    tick(server, 2).await;
    set_editor(world, hanging, Some(player.gameprofile.id));
    send_update(bot, hanging, "HANGING").await?;
    tick(server, 20).await;
    if stored_text(world, hanging) != "HANGING" {
        return Err("SIGN_UPDATE on a hanging sign was rejected".to_string());
    }

    Ok(())
}

async fn run_click_command_cases(
    server: &Arc<Server>,
    world: &pumpkin_core::world::World,
    bot: &mut HeadlessBot,
    player: &pumpkin_core::entity::player::Player,
    base: BlockPos,
) -> Result<(), String> {
    let marker = base.offset(Vector3::new(0, 5, 0));
    world
        .level
        .set_block_state(&marker, Block::AIR.default_state.id);
    let crafted = crafted_click_line(marker);

    // A client-submitted line that looks like a component must stay literal.
    let crafted_pos = base.offset(Vector3::new(5, 0, 0));
    place_sign(server, world, crafted_pos).await;
    set_editor(world, crafted_pos, Some(player.gameprofile.id));
    send_update(bot, crafted_pos, &crafted).await?;
    tick(server, 20).await;

    click_and_wait(server, bot, crafted_pos, 900).await?;
    if world.get_block(&marker).id == Block::EMERALD_BLOCK.id {
        return Err("a normal sign executed a run_command click event".to_string());
    }

    // A sign that allows op features follows vanilla and does execute.
    set_click_component(world, crafted_pos, &crafted, true);
    click_and_wait(server, bot, crafted_pos, 901).await?;
    if world.get_block(&marker).id != Block::EMERALD_BLOCK.id {
        return Err("an allow_op_features sign did not execute its click command".to_string());
    }

    // The same stored component without allow_op_features must not execute.
    world
        .level
        .set_block_state(&marker, Block::AIR.default_state.id);
    set_click_component(world, crafted_pos, &crafted, false);
    click_and_wait(server, bot, crafted_pos, 902).await?;
    if world.get_block(&marker).id == Block::EMERALD_BLOCK.id {
        return Err("a sign without allow_op_features executed a click command".to_string());
    }
    if sign_editor(world, crafted_pos).is_some() {
        return Err("a disabled sign click opened the editor instead of the overlay".to_string());
    }

    // Editing a sign that allows op features must not let a client inject a new
    // click command; the submitted line is still stored literally.
    place_sign(server, world, crafted_pos).await;
    set_editor(world, crafted_pos, Some(player.gameprofile.id));
    send_update(bot, crafted_pos, "trusted").await?;
    tick(server, 20).await;
    set_allow_op_features(world, crafted_pos, true);
    set_editor(world, crafted_pos, Some(player.gameprofile.id));
    send_update(bot, crafted_pos, &crafted).await?;
    tick(server, 20).await;
    if stored_text(world, crafted_pos) != crafted {
        return Err(
            "client text on an allow_op_features sign was not stored literally".to_string(),
        );
    }

    world
        .level
        .set_block_state(&marker, Block::AIR.default_state.id);
    click_and_wait(server, bot, crafted_pos, 903).await?;
    if world.get_block(&marker).id == Block::EMERALD_BLOCK.id {
        return Err("editing an allow_op_features sign injected a click command".to_string());
    }

    // show_dialog must also take the disabled-click branch.
    let show_dialog = r#"{"text":"dialog","click_event":{"action":"show_dialog","dialog":{"type":"minecraft:notice","title":"x"}}}"#;
    set_click_component(world, crafted_pos, show_dialog, false);
    click_and_wait(server, bot, crafted_pos, 904).await?;
    if sign_editor(world, crafted_pos).is_some() {
        return Err("a disabled show_dialog click opened the editor".to_string());
    }

    Ok(())
}

async fn run_applicator_cases(
    server: &Arc<Server>,
    world: &pumpkin_core::world::World,
    bot: &mut HeadlessBot,
    player: &pumpkin_core::entity::player::Player,
    base: BlockPos,
) -> Result<(), String> {
    // Dyeing a sign with a disabled click command must not run it.
    let marker = base.offset(Vector3::new(0, 5, 0));
    world
        .level
        .set_block_state(&marker, Block::AIR.default_state.id);
    let crafted = crafted_click_line(marker);
    let click_apply_pos = base.offset(Vector3::new(3, 0, 1));
    place_sign(server, world, click_apply_pos).await;
    set_editor(world, click_apply_pos, Some(player.gameprofile.id));
    send_update(bot, click_apply_pos, "apply click").await?;
    tick(server, 20).await;
    set_click_component(world, click_apply_pos, &crafted, false);

    let inventory = player.inventory();
    inventory.set_stack_in_hand(Hand::Right, ItemStack::new(1, &Item::RED_DYE));
    click_and_wait(server, bot, click_apply_pos, 905).await?;
    if sign_color(world, click_apply_pos) != Some(DyeColor::Red) {
        return Err("dye did not apply to a sign with a click command".to_string());
    }
    if world.get_block(&marker).id == Block::EMERALD_BLOCK.id {
        return Err("dye application executed a disabled click command".to_string());
    }
    inventory.set_stack_in_hand(Hand::Right, ItemStack::EMPTY.clone());

    let pos = base.offset(Vector3::new(1, 0, 1));
    place_sign(server, world, pos).await;
    set_editor(world, pos, Some(player.gameprofile.id));
    send_update(bot, pos, "APPLY").await?;
    tick(server, 20).await;

    inventory.set_stack_in_hand(Hand::Right, ItemStack::new(1, &Item::RED_DYE));
    click_and_wait(server, bot, pos, 906).await?;
    if sign_color(world, pos) != Some(DyeColor::Red) {
        return Err("dye did not apply to a sign".to_string());
    }

    inventory.set_stack_in_hand(Hand::Right, ItemStack::new(1, &Item::GLOW_INK_SAC));
    click_and_wait(server, bot, pos, 907).await?;
    if !sign_is_glowing(world, pos) {
        return Err("glow ink sac did not apply to a sign".to_string());
    }

    inventory.set_stack_in_hand(Hand::Right, ItemStack::new(1, &Item::INK_SAC));
    click_and_wait(server, bot, pos, 908).await?;
    if sign_is_glowing(world, pos) {
        return Err("ink sac did not remove glowing text".to_string());
    }

    inventory.set_stack_in_hand(Hand::Right, ItemStack::new(1, &Item::HONEYCOMB));
    click_and_wait(server, bot, pos, 909).await?;
    if !sign_is_waxed(world, pos) {
        return Err("honeycomb did not wax a sign".to_string());
    }
    inventory.set_stack_in_hand(Hand::Right, ItemStack::EMPTY.clone());

    Ok(())
}

pub async fn run_sign_authorization_tests(
    server: &Arc<Server>,
    addr: std::net::SocketAddr,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let mut bot = HeadlessBot::connect_and_join(addr, BOT_NAME).await?;

    let Some(player) = world.get_player_by_name(BOT_NAME) else {
        return Err(format!("Could not find '{BOT_NAME}' in Overworld"));
    };
    let dispatcher = server.command_dispatcher.load();
    player.set_permission_lvl(server, PermissionLvl::Zero, &dispatcher);
    player.set_gamemode(GameMode::Survival);

    let base = BlockPos::new(
        bot.position.x.floor() as i32,
        bot.position.y.floor() as i32,
        bot.position.z.floor() as i32,
    );

    run_authorization_cases(server, &world, &mut bot, &player, base).await?;
    run_click_command_cases(server, &world, &mut bot, &player, base).await?;
    run_applicator_cases(server, &world, &mut bot, &player, base).await?;

    summary_lines.push("[PASS] bot:sign_authorization_click_commands_and_applicators".to_string());
    Ok(())
}
