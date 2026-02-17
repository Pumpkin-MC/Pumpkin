use std::pin::Pin;
use std::sync::Arc;

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_data::block_properties::{Integer0To15, EnumVariants, BlockProperties};
use pumpkin_data::Block;

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
                self.update_power(world, &self.position);
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

    async fn update_power(
        &self,
        world: &Arc<World>,
        block_pos: &BlockPos,
    ) {
        let state = world.get_block_state(block_pos).await;
        let block = &Block::DAYLIGHT_DETECTOR;
        let mut props = DaylightDetectorProperties::from_state_id(state.id, block);

        // TODO: finish power calculation
        // for this we need to get the ambient darkness which is not implemented yet in the light engine
        // and the sun angle attribute
        let sky_light_level = world
            .level
            .light_engine
            .get_sky_light_level(&world.level, block_pos)
            .await
            .unwrap();
        let ambient_darkness = 0; // TODO
        let effective_sky_light = sky_light_level - ambient_darkness;
        // let sun_angle;
        let inverted = props.inverted;

        let mut power = 0;
        if inverted {
            power = 15 - effective_sky_light;
        } else if effective_sky_light > 0 {
            // TODO:
            // some math
            // see source code: net.minecraft.block.DaylightDetectorBlock.java
        }

        power = 15;

        let power = Integer0To15::from_index(power.clamp(0, 15).into());
        if power != props.power {
            props.power = power;
            let state = props.to_state_id(block);
            world
                .set_block_state(block_pos, state, BlockFlags::NOTIFY_ALL)
                .await;
        }
    }
}
