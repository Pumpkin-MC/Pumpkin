use std::sync::Arc;

use async_trait::async_trait;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::{Block, BlockDirection, BlockState, block_properties::get_block_by_state_id};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

use crate::world::{BlockFlags, SimpleWorld};

use super::BlockEntity;

pub struct PistonBlockEntity {
    pub position: BlockPos,
    pub pushed_block_state: &'static BlockState,
    pub facing: BlockDirection,
    pub current_progress: AtomicCell<f32>,
    pub last_progress: AtomicCell<f32>,
    pub extending: bool,
    pub source: bool,
}

impl PistonBlockEntity {
    pub const ID: &'static str = "minecraft:piston";

    pub async fn finish(&self, world: Arc<dyn SimpleWorld>) {
        if self.last_progress.load() < 1.0 {
            let pos = self.position;
            world.remove_block_entity(&pos).await;
            if world.get_block(&pos).await == &Block::MOVING_PISTON {
                let state = if self.source {
                    Block::AIR.default_state.id
                } else {
                    self.pushed_block_state.id
                };
                world
                    .clone()
                    .set_block_state(&pos, state, BlockFlags::NOTIFY_ALL)
                    .await;
                world
                    .update_neighbor(&pos, get_block_by_state_id(state))
                    .await;
            }
        }
    }
}

const FACING: &str = "facing";
const LAST_PROGRESS: &str = "progress";
const EXTENDING: &str = "extending";
const SOURCE: &str = "source";

#[async_trait]
impl BlockEntity for PistonBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    async fn tick(&self, world: &Arc<dyn SimpleWorld>) {
        let current_progress = self.current_progress.load();
        self.last_progress.store(current_progress);
        if current_progress >= 1.0 {
            let pos = self.position;
            world.remove_block_entity(&pos).await;
            if world.get_block(&pos).await == &Block::MOVING_PISTON {
                if self.pushed_block_state.is_air() {
                    world
                        .clone()
                        .set_block_state(
                            &pos,
                            self.pushed_block_state.id,
                            BlockFlags::FORCE_STATE | BlockFlags::MOVED,
                        )
                        .await;
                } else {
                    world
                        .clone()
                        .set_block_state(
                            &pos,
                            self.pushed_block_state.id,
                            BlockFlags::NOTIFY_ALL | BlockFlags::MOVED,
                        )
                        .await;
                    world
                        .clone()
                        .update_neighbor(&pos, get_block_by_state_id(self.pushed_block_state.id))
                        .await;
                }
            }
        }
        self.current_progress.store(current_progress + 0.5);
        if current_progress + 0.5 >= 1.0 {
            self.current_progress.store(1.0);
        }
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        // TODO
        let pushed_block_state = Block::AIR.default_state;
        let facing = nbt.get_byte(FACING).unwrap_or(0);
        let last_progress = nbt.get_float(LAST_PROGRESS).unwrap_or(0.0);
        let extending = nbt.get_bool(EXTENDING).unwrap_or(false);
        let source = nbt.get_bool(SOURCE).unwrap_or(false);
        Self {
            pushed_block_state,
            position,
            facing: BlockDirection::from_index(facing as u8).unwrap_or(BlockDirection::Down),
            current_progress: last_progress.into(),
            last_progress: last_progress.into(),
            extending,
            source,
        }
    }

    async fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        // TODO: pushed_block_state
        nbt.put_byte(FACING, self.facing.to_index() as i8);
        nbt.put_float(LAST_PROGRESS, self.last_progress.load());
        nbt.put_bool(EXTENDING, self.extending);
        nbt.put_bool(SOURCE, self.source);
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        // TODO: pushed_block_state
        nbt.put_byte(FACING, self.facing.to_index() as i8);
        nbt.put_float(LAST_PROGRESS, self.last_progress.load());
        nbt.put_bool(EXTENDING, self.extending);
        nbt.put_bool(SOURCE, self.source);
        // TODO: duplicated code :c
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
