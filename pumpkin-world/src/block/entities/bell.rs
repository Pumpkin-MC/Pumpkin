use crate::block::entities::BlockEntity;
use crate::world::SimpleWorld;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::any::Any;
use std::pin::Pin;
use std::sync::Arc;

pub struct BellBlockEntity{
    pub position: BlockPos,
    pub last_side_hit: AtomicCell<Option<HorizontalFacing>>,
    #[allow(dead_code)]
    last_ring_time: AtomicCell<i64>,
    pub ring_ticks: AtomicCell<i32>,
    pub ringing: AtomicCell<bool>,
    resonating: AtomicCell<bool>,
    resonate_time: AtomicCell<i32>
}

impl BellBlockEntity {
    pub const ID: &'static str = "minecraft:bell";
    pub fn new(position: BlockPos) -> Self {
        BellBlockEntity {
            position,
            last_side_hit: AtomicCell::new(None),
            ring_ticks: AtomicCell::new(0),
            resonate_time: AtomicCell::new(0),
            resonating: AtomicCell::new(false),
            ringing: AtomicCell::new(false),
            last_ring_time: AtomicCell::new(0),
        }
    }
    pub fn activate(&self, direction: HorizontalFacing) {
        self.last_side_hit.store(Some(direction));
        if self.ringing.load() {
            self.ring_ticks.store(0);
        } else {
            self.ringing.store(true);
        }
    }
    pub fn raiders_hear_bell(&self) -> bool {
        //TODO

        false

    }
}

impl BlockEntity for BellBlockEntity {

    fn write_nbt<'a>(&'a self, _nbt: &'a mut NbtCompound) -> Pin<Box<dyn Future<Output=()> + Send + 'a>> {
        Box::pin(async move {
        })
    }

    fn from_nbt(_nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized
    {

        Self::new(position)

    }

    fn tick<'a>(&'a self, world: &'a Arc<dyn SimpleWorld>,) -> Pin<Box<dyn Future<Output=()> + Send + 'a>> {
        Box::pin(async move {
            if self.ringing.load() {
                self.ring_ticks.fetch_add(1);
            }
            if self.ring_ticks.load() >= 50 {
                self.ringing.store(false);
                self.ring_ticks.store(0);
            }
            if self.ring_ticks.load()>=5 && self.resonate_time.load()==0 && self.raiders_hear_bell() {
                self.resonating.store(true);
                world.play_sound_fine(Sound::BlockBellResonate, SoundCategory::Blocks, &self.position.to_f64(), 1.0, 1.0).await;
            }

            if self.resonating.load() {
                if self.resonate_time.load() < 40 {
                    self.resonate_time.fetch_add(1);
                } else {
                    self.resonating.store(false);
                }
            }
        })

    }

    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}