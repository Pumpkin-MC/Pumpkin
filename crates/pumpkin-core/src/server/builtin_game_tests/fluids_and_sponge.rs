use pumpkin_data::{Block, fluid::FluidProperties};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

pub(super) async fn test_water_lava_interactions(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 7, 5).await?;

    for x in 0..7 {
        for z in 0..5 {
            if x == 0 || x == 6 || z == 0 || z == 4 || x == 3 {
                helper
                    .set_block_default(&BlockPos::new(x, 1, z), &Block::STONE)
                    .await?;
            }
        }
    }

    let water_pos = BlockPos::new(1, 1, 1);
    let lava_obsidian_pos = BlockPos::new(2, 1, 1);
    helper.set_block_default(&water_pos, &Block::WATER).await?;
    helper
        .set_block_default(&lava_obsidian_pos, &Block::LAVA)
        .await?;
    helper
        .assert_block_present(&lava_obsidian_pos, &Block::OBSIDIAN)
        .await?;

    let lava_cobble_pos = BlockPos::new(2, 1, 3);
    let water_adj_pos = BlockPos::new(1, 1, 3);
    let mut flowing_lava = pumpkin_data::fluid::FlowingWaterLikeFluidProperties::default(
        &pumpkin_data::fluid::Fluid::FLOWING_LAVA,
    );
    flowing_lava.level = pumpkin_data::fluid::Level::L2;
    flowing_lava.falling = pumpkin_data::fluid::Falling::False;
    helper
        .set_block_default(&water_adj_pos, &Block::WATER)
        .await?;
    helper
        .set_block(
            &lava_cobble_pos,
            flowing_lava.to_state_id(&pumpkin_data::fluid::Fluid::FLOWING_LAVA),
        )
        .await?;
    helper
        .assert_block_present(&lava_cobble_pos, &Block::COBBLESTONE)
        .await?;

    let soul_soil_pos = BlockPos::new(4, 0, 2);
    let basalt_pos = BlockPos::new(4, 1, 2);
    let blue_ice_pos = BlockPos::new(5, 1, 2);
    helper
        .set_block_default(&soul_soil_pos, &Block::SOUL_SOIL)
        .await?;
    helper
        .set_block_default(&blue_ice_pos, &Block::BLUE_ICE)
        .await?;
    helper
        .set_block(
            &basalt_pos,
            flowing_lava.to_state_id(&pumpkin_data::fluid::Fluid::FLOWING_LAVA),
        )
        .await?;
    helper
        .assert_block_present(&basalt_pos, &Block::BASALT)
        .await?;

    helper.succeed();
    Ok(())
}

pub(super) async fn test_sponge_absorbs_water(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    for x in 0..5 {
        for z in 0..5 {
            if x == 0 || x == 4 || z == 0 || z == 4 {
                helper
                    .set_block_default(&BlockPos::new(x, 1, z), &Block::STONE)
                    .await?;
            }
        }
    }

    let water_1 = BlockPos::new(1, 1, 2);
    let water_2 = BlockPos::new(2, 1, 1);
    let water_3 = BlockPos::new(3, 1, 2);
    let sponge_pos = BlockPos::new(2, 1, 2);

    helper.set_block_default(&water_1, &Block::WATER).await?;
    helper.set_block_default(&water_2, &Block::WATER).await?;
    helper.set_block_default(&water_3, &Block::WATER).await?;

    helper
        .set_block_default(&sponge_pos, &Block::SPONGE)
        .await?;

    helper
        .assert_block_present(&sponge_pos, &Block::WET_SPONGE)
        .await?;
    helper.assert_block_present(&water_1, &Block::AIR).await?;
    helper.assert_block_present(&water_2, &Block::AIR).await?;
    helper.assert_block_present(&water_3, &Block::AIR).await?;

    helper.succeed();
    Ok(())
}
