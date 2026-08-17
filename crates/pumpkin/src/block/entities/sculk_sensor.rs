use super::BlockEntity;
use crate::world::World;
use crate::world::game_event::vibration::{
    SculkSensorVibrationUser, VibrationListener, vibration_tick,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

pub type SculkSensorBlockEntity = SculkSensorBlockEntityImpl<false>;

pub struct SculkSensorBlockEntityImpl<const CALIBRATED: bool> {
    pub position: BlockPos,
    pub last_vibration_frequency: AtomicI32,
    pub listener: VibrationListener,
}

impl<const CALIBRATED: bool> BlockEntity for SculkSensorBlockEntityImpl<CALIBRATED> {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let last_vibration_frequency = nbt.get_int("last_vibration_frequency").unwrap_or(0);
        Self {
            position,
            last_vibration_frequency: AtomicI32::new(last_vibration_frequency),
            listener: nbt.get_compound("listener").map_or_else(
                || VibrationListener::new(position),
                |listener| VibrationListener::from_nbt(position, listener),
            ),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            nbt.put_int(
                "last_vibration_frequency",
                self.last_vibration_frequency.load(Ordering::Relaxed),
            );
            nbt.put_compound(
                "listener",
                self.listener
                    .data
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .to_nbt(),
            );
        })
    }
    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        // write_internal is async only for trait compatibility; our write_nbt
        // body is now purely synchronous (atomic load), so block_on is safe.
        futures::executor::block_on(async {
            self.write_internal(&mut nbt).await;
        });
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tick<'a>(
        &'a self,
        world: &'a Arc<World>,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let radius = if CALIBRATED { 16 } else { 8 };
            let user = SculkSensorVibrationUser::new(self.position, radius);
            vibration_tick(world, &self.listener, &user).await;
        })
    }
}

impl<const CALIBRATED: bool> SculkSensorBlockEntityImpl<CALIBRATED> {
    pub const ID: &'static str = if CALIBRATED {
        "minecraft:calibrated_sculk_sensor"
    } else {
        "minecraft:sculk_sensor"
    };

    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            last_vibration_frequency: AtomicI32::new(0),
            listener: VibrationListener::new(position),
        }
    }
}
