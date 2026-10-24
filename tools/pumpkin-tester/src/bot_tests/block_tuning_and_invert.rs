use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use pumpkin_core::{server::Server, world::BlockFlags};
use pumpkin_data::{
    Block,
    block_properties::{
        AttachFace, DaylightDetectorLikeProperties, LeverLikeProperties, NoteBlockLikeProperties,
    },
    dimension::Dimension,
};
use pumpkin_util::math::position::BlockPos;

use crate::bot::{HeadlessBot, MAX_POLL_TICKS};

pub async fn run_block_tuning_and_invert_tests(
    server: &Arc<Server>,
    bot1: &mut HeadlessBot,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let start = Instant::now();
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);

    let base_x = bot1.position.x.floor() as i32;
    let base_y = bot1.position.y.floor() as i32;
    let base_z = bot1.position.z.floor() as i32;

    let note_pos = BlockPos::new(base_x, base_y, base_z + 2);
    let detector_pos = BlockPos::new(base_x + 1, base_y, base_z + 2);
    let floor_pos = BlockPos::new(base_x - 1, base_y - 1, base_z + 2);
    let lever_pos = BlockPos::new(base_x - 1, base_y, base_z + 2);

    world.set_block_state(
        &note_pos,
        Block::NOTE_BLOCK.default_state.id,
        BlockFlags::NOTIFY_ALL,
    );
    world.set_block_state(
        &detector_pos,
        Block::DAYLIGHT_DETECTOR.default_state.id,
        BlockFlags::NOTIFY_ALL,
    );
    world.set_block_state(
        &floor_pos,
        Block::STONE.default_state.id,
        BlockFlags::NOTIFY_ALL,
    );
    let mut lever_props = LeverLikeProperties::default(&Block::LEVER);
    lever_props.face = AttachFace::Floor;
    lever_props.powered = false;
    world.set_block_state(
        &lever_pos,
        lever_props.to_state_id(&Block::LEVER),
        BlockFlags::NOTIFY_ALL,
    );

    // Switch to an empty hotbar slot before right-clicking interactive blocks
    bot1.set_held_item(4).await?;

    // Tune the note block twice (0 -> 1 -> 2), invert the daylight detector, and pull the lever
    bot1.use_block(note_pos, 10).await?;
    bot1.use_block(note_pos, 11).await?;
    bot1.use_block(detector_pos, 12).await?;
    bot1.use_block(lever_pos, 13).await?;

    let mut note_tuned = false;
    let mut detector_inverted = false;
    let mut lever_powered = false;

    for _ in 0..MAX_POLL_TICKS {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();

        let note_state = world.get_block_state(&note_pos);
        if world.get_block(&note_pos).id == Block::NOTE_BLOCK.id
            && NoteBlockLikeProperties::from_state_id(note_state.id).note == 2
        {
            note_tuned = true;
        }

        let detector_state = world.get_block_state(&detector_pos);
        if world.get_block(&detector_pos).id == Block::DAYLIGHT_DETECTOR.id
            && DaylightDetectorLikeProperties::from_state_id(detector_state.id).inverted
        {
            detector_inverted = true;
        }

        let lever_state = world.get_block_state(&lever_pos);
        if world.get_block(&lever_pos).id == Block::LEVER.id
            && LeverLikeProperties::from_state_id(lever_state.id).powered
        {
            lever_powered = true;
        }

        if note_tuned && detector_inverted && lever_powered {
            break;
        }
    }

    if !note_tuned {
        return Err("Bot SUseItemOn right-clicks did not tune note_block to note == 2".to_string());
    }
    if !detector_inverted {
        return Err("Bot SUseItemOn right-click did not invert daylight_detector".to_string());
    }
    if !lever_powered {
        return Err("Bot SUseItemOn right-click did not toggle lever to powered".to_string());
    }

    summary_lines.push(format!(
        "[PASS] bot:note_block_tune_and_daylight_invert ({}ms)",
        start.elapsed().as_millis()
    ));
    Ok(())
}
