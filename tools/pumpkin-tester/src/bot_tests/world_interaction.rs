use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use pumpkin_core::{server::Server, world::BlockFlags};
use pumpkin_data::{Block, block_properties::OakTrapdoorLikeProperties, dimension::Dimension};
use pumpkin_util::math::position::BlockPos;

use crate::bot::{HeadlessBot, MAX_POLL_TICKS};

pub async fn run_world_interaction_tests(
    server: &Arc<Server>,
    bot1: &mut HeadlessBot,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let start = Instant::now();
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);

    let base_x = bot1.position.x.floor() as i32;
    let base_y = bot1.position.y.floor() as i32;
    let base_z = bot1.position.z.floor() as i32;

    let trapdoor_pos = BlockPos::new(base_x + 1, base_y, base_z);
    let break_pos = BlockPos::new(base_x - 1, base_y + 1, base_z);

    world.set_block_state(
        &trapdoor_pos,
        Block::OAK_TRAPDOOR.default_state.id,
        BlockFlags::NOTIFY_ALL,
    );
    world.set_block_state(
        &break_pos,
        Block::GOLD_BLOCK.default_state.id,
        BlockFlags::NOTIFY_ALL,
    );

    // 1. Right-click oak trapdoor via SUseItemOn -> should toggle open
    bot1.use_block(trapdoor_pos, 1).await?;

    // 2. Left-click gold block in Creative mode via SPlayerAction(StartedDigging) -> should break to AIR
    bot1.start_digging(break_pos, 2).await?;

    let mut trapdoor_opened = false;
    let mut block_broken = false;
    for _ in 0..MAX_POLL_TICKS {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();

        let state = world.get_block_state(&trapdoor_pos);
        if world.get_block(&trapdoor_pos).id == Block::OAK_TRAPDOOR.id
            && OakTrapdoorLikeProperties::from_state_id(state.id).open
        {
            trapdoor_opened = true;
        }

        if world.get_block(&break_pos).id == Block::AIR.id {
            block_broken = true;
        }

        if trapdoor_opened && block_broken {
            break;
        }
    }

    if !trapdoor_opened {
        return Err(
            "Bot SUseItemOn right-click did not open the oak trapdoor in the world".to_string(),
        );
    }
    if !block_broken {
        return Err(
            "Bot SPlayerAction(StartedDigging) in Creative mode did not break the gold block"
                .to_string(),
        );
    }

    summary_lines.push(format!(
        "[PASS] bot:world_block_use_and_break ({}ms)",
        start.elapsed().as_millis()
    ));
    Ok(())
}
