use std::sync::Arc;

use pumpkin_data::{Block, fluid::Fluid};
use pumpkin_world::{
    block::entities::BlockEntity,
    chunk::ChunkData,
    tick::{OrderedTick, ScheduledTick},
};
use tokio::sync::RwLock;

#[derive(Default)]
pub struct TickData {
    pub block_ticks: Vec<OrderedTick<&'static Block>>,
    pub fluid_ticks: Vec<OrderedTick<&'static Fluid>>,
    pub random_ticks: Vec<ScheduledTick<()>>,
    pub block_entities: Vec<Arc<dyn BlockEntity>>,
    pub cloned_chunks: Vec<Arc<RwLock<ChunkData>>>,
    pub worker_pool: Vec<TickBatch>,
}

impl TickData {
    pub fn clear(&mut self) {
        self.block_entities.clear();
        self.block_ticks.clear();
        self.fluid_ticks.clear();
        self.random_ticks.clear();
        self.cloned_chunks.clear();
    }
}

pub struct TickBatch {
    pub block_ticks: Vec<OrderedTick<&'static Block>>,
    pub fluid_ticks: Vec<OrderedTick<&'static Fluid>>,
    pub random_ticks: Vec<ScheduledTick<()>>,
    pub block_entities: Vec<Arc<dyn BlockEntity>>,
}

impl TickBatch {
    pub fn clear(&mut self) {
        self.block_ticks.clear();
        self.fluid_ticks.clear();
        self.random_ticks.clear();
        self.block_entities.clear();
    }
}
