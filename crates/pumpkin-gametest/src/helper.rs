use pumpkin_data::{Block, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::error::{GameTestError, GameTestResult};
use crate::structure::TestStructureInstance;
use crate::world::GameTestWorld;

pub struct GameTestHelper<'a> {
    world: &'a dyn GameTestWorld,
    placement: &'a TestStructureInstance,
    tick: u32,
}

impl<'a> GameTestHelper<'a> {
    #[must_use]
    pub const fn new(
        world: &'a dyn GameTestWorld,
        placement: &'a TestStructureInstance,
        tick: u32,
    ) -> Self {
        Self {
            world,
            placement,
            tick,
        }
    }

    #[must_use]
    pub const fn tick(&self) -> u32 {
        self.tick
    }

    #[must_use]
    pub const fn absolute_pos(&self, relative: &BlockPos) -> BlockPos {
        self.placement.transform(relative)
    }

    pub async fn block_state_id(&self, relative: &BlockPos) -> BlockStateId {
        let position = self.absolute_pos(relative);
        self.world.block_state_id(&position).await
    }

    pub async fn set_block(
        &self,
        relative: &BlockPos,
        block_state_id: BlockStateId,
    ) -> GameTestResult<()> {
        let position = self.absolute_pos(relative);
        self.world
            .set_block_state(&position, block_state_id, BlockFlags::NOTIFY_ALL)
            .await
    }

    pub async fn set_air(&self, relative: &BlockPos) -> GameTestResult<()> {
        self.set_block(relative, Block::AIR.default_state.id).await
    }

    pub async fn assert_block_state(
        &self,
        relative: &BlockPos,
        expected: BlockStateId,
    ) -> GameTestResult<()> {
        let position = self.absolute_pos(relative);
        let actual = self.world.block_state_id(&position).await;

        if actual == expected {
            return Ok(());
        }

        Err(GameTestError::Assertion {
            tick: self.tick,
            position: Some(position),
            message: format!("expected block state {expected:?}, found {actual:?}"),
        })
    }
}
