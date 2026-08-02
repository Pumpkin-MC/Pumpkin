use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_data::block_properties::BlockProperties;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

use crate::world::{BlockFlags, World};

use super::BlockEntity;

type DaylightDetectorProperties = pumpkin_data::block_properties::DaylightDetectorLikeProperties;

pub struct DaylightDetectorBlockEntity {
    pub position: BlockPos,
}

impl BlockEntity for DaylightDetectorBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(_nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        Self { position }
    }

    fn write_nbt<'a>(
        &'a self,
        _nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tick<'a>(&'a self, world: &'a Arc<World>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {
            if world.get_world_age().await % 20 == 0 && world.dimension.has_skylight {
                Self::update_power(world, &self.position).await;
            }
        })
    }
}

impl DaylightDetectorBlockEntity {
    pub const ID: &'static str = "minecraft:daylight_detector";

    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self { position }
    }

    pub async fn update_power(world: &Arc<World>, block_pos: &BlockPos) {
        use std::f32::consts::PI;

        let (block, state) = world.get_block_and_state(block_pos);
        let mut props = DaylightDetectorProperties::from_state_id(state.id, block);

        let inverted = props.inverted;
        let time_of_day = world.get_time_of_day().await;
        let sun_angle = ((time_of_day as f32 / 24_000.0) - 0.25) * (PI * 2.0);
        let power = daylight_detector_power(
            world.get_sky_light_level(block_pos),
            world.sky_darken.load(Ordering::Relaxed),
            sun_angle,
            inverted,
        );
        if power != props.power {
            props.power = power;
            let state = props.to_state_id(block);
            world
                .clone()
                .set_block_state(block_pos, state, BlockFlags::NOTIFY_ALL)
                .await;
        }
    }
}

fn daylight_detector_power(
    sky_light: u8,
    sky_darken: u8,
    mut sun_angle: f32,
    inverted: bool,
) -> u8 {
    let mut target = sky_light.saturating_sub(sky_darken);
    if inverted {
        target = 15 - target;
    } else if target > 0 {
        let offset = if sun_angle < std::f32::consts::PI {
            0.0
        } else {
            std::f32::consts::PI * 2.0
        };
        sun_angle += (offset - sun_angle) * 0.2;
        target = (target as f32 * sun_angle.cos()).round().clamp(0.0, 15.0) as u8;
    }
    target
}

#[cfg(test)]
mod tests {
    use super::daylight_detector_power;

    #[test]
    fn uses_effective_sky_brightness_before_sun_angle_adjustment() {
        assert_eq!(daylight_detector_power(15, 0, 0.0, false), 15);
        assert_eq!(daylight_detector_power(15, 11, 0.0, false), 4);
    }

    #[test]
    fn inverted_power_complements_effective_sky_brightness() {
        assert_eq!(daylight_detector_power(15, 0, 0.0, true), 0);
        assert_eq!(daylight_detector_power(15, 11, 0.0, true), 11);
    }
}
