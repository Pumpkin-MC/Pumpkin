use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
use tokio::sync::Mutex;

use crate::chunk::ChunkData;
use crate::level::Level;
use crate::world::SimpleWorld;

use super::BlockEntity;

pub struct PistonBlockEntity {
    pub position: BlockPos,
    pub facing: i8,
    pub current_progress: Mutex<f32>,
    pub last_progress: Mutex<f32>,
    pub extending: bool,
    pub source: bool,
}

impl PistonBlockEntity {
    pub const ID: &'static str = "minecraft:moving_piston";
}

const FACING: &str = "facing";
const LAST_PROGRESS: &str = "progress";
const EXTENDING: &str = "extending";
const SOURCE: &str = "source";

#[async_trait]
impl BlockEntity for PistonBlockEntity {
    fn identifier(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    async fn tick(&self, world: Arc<dyn SimpleWorld>) {
        let mut last_progress = self.last_progress.lock().await;
        let mut current_progress = self.current_progress.lock().await;
        *last_progress = *current_progress;
        if *last_progress >= 1.0 {
            world.remove_block_entity(&self.position);
        }
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let facing = nbt.get_byte(FACING).unwrap_or(0);
        let last_progress = nbt.get_float(LAST_PROGRESS).unwrap_or(0.0);
        let extending = nbt.get_bool(EXTENDING).unwrap_or(false);
        let source = nbt.get_bool(SOURCE).unwrap_or(false);
        Self {
            position,
            facing,
            current_progress: 0.0.into(),
            last_progress: last_progress.into(),
            extending,
            source,
        }
    }

    fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        nbt.put_byte(FACING, self.facing);
        // nbt.put_float(LAST_PROGRESS, *self.last_progress.lock().await);
        nbt.put_bool(EXTENDING, self.extending);
        nbt.put_bool(SOURCE, self.source);
    }
}
