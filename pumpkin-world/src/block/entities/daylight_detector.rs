use std::pin::Pin;
use std::sync::Arc;

use pumpkin_data::block_properties::{BlockProperties, EnumVariants, Integer0To15};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

use crate::world::{BlockFlags, SimpleWorld};

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

    fn tick<'a>(
        &'a self,
        world: &'a Arc<dyn SimpleWorld>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {
            if world.get_world_age().await % 20 == 0 {
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

    pub async fn update_power<W: SimpleWorld + ?Sized>(world: &Arc<W>, block_pos: &BlockPos) {
        use std::f32::consts::PI;

        let (block, state) = world.get_block_and_state(block_pos).await;
        let mut props = DaylightDetectorProperties::from_state_id(state.id, block);

        let level = world.get_level().await;

        let inverted = props.inverted;

        // TODO: finish power signal calculation
        // see: https://minecraft.wiki/w/Light#Internal_sky_light
        /*
         * THIS IS A PLACE HOLDER
         *
         * For now the light engine is not finished and we do not have access to such things as
         * getAmbientDarkness() or EnvironmentAttributes.SUN_ANGLE_VISUAL as in java code from yarn:
         * see: net.minecraft.blocks.DaylightDetectorBlock.java
         *
         * so this is a fake implementation using the time of day...
         * is not as accurate as vanilla and doesnt consider weather
         */

        let time_of_day = world.get_time_of_day().await;

        // Sun Angle
        let sun_angle_fraction = (time_of_day as f32 / 24000.0) - 0.25;
        let mut sun_angle_radians = sun_angle_fraction * (PI * 2.0);

        // Ambient darkness (considering only night, not weather)
        let cos_angle = sun_angle_radians.cos();
        let ambient_darkness = if cos_angle < 0.0 {
            // night time = ~11 darkness
            (cos_angle.abs() * 11.0) as u8
        } else {
            0 // full daylight
        };

        let sky_light_level = level
            .light_engine
            .get_sky_light_level(level, block_pos)
            .await
            .unwrap();

        let mut power = sky_light_level - ambient_darkness;

        if inverted {
            power = 15 - power;
        } else if power > 0 {
            let transition_offset = if sun_angle_radians < PI {
                0.0
            } else {
                PI * 2.0
            };

            sun_angle_radians += (transition_offset - sun_angle_radians) * 0.2;
            power = (power as f32 * sun_angle_radians.cos()).round() as u8;
        }

        let power = Integer0To15::from_index(power.clamp(0, 15).into());
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
