use super::BlockEntity;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::pin::Pin;
use tokio::sync::Mutex;

pub struct CalibratedSculkSensorBlockEntity {
    pub position: BlockPos,
    pub last_vibration_frequency: Mutex<i32>,
}

impl BlockEntity for CalibratedSculkSensorBlockEntity {
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
            last_vibration_frequency: Mutex::new(last_vibration_frequency),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            nbt.put_int(
                "last_vibration_frequency",
                *self.last_vibration_frequency.lock().await,
            );
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CalibratedSculkSensorBlockEntity {
    pub const ID: &'static str = "minecraft:calibrated_sculk_sensor";
    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            last_vibration_frequency: Mutex::new(0),
        }
    }
}
