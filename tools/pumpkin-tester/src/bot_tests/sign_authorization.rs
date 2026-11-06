use std::{sync::Arc, time::Duration};

use pumpkin_core::{
    block::entities::{
        hanging_sign::HangingSignBlockEntity,
        sign::{SignBlockEntity, SignEntityRef},
    },
    entity::EntityBase,
    server::Server,
};
use pumpkin_data::{Block, dimension::Dimension};
use pumpkin_protocol::java::server::play::SUpdateSign;
use pumpkin_util::{GameMode, PermissionLvl, math::position::BlockPos, math::vector3::Vector3};

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

async fn tick(server: &Arc<Server>, count: usize) {
    for _ in 0..count {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
    }
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

    // The registered editor must still be within edit range.
    let out_of_range = base.offset(Vector3::new(4, 0, 0));
    place_sign(server, world, out_of_range).await;
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
    let home = Vector3::new(
        f64::from(base.0.x) + 0.5,
        f64::from(base.0.y),
        f64::from(base.0.z) + 0.5,
    );
    move_to_and_wait(server, bot, player, home).await?;

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

    summary_lines.push("[PASS] bot:sign_authorization".to_string());
    Ok(())
}
