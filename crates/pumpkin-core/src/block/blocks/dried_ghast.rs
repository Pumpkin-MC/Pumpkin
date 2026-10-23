use pumpkin_data::{
    BlockState, BlockStateId,
    block_properties::{DriedGhastLikeProperties, HorizontalFacing},
    entity::EntityType,
    fluid::Fluid,
    sound::{Sound, SoundCategory},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::{tick::TickPriority, world::BlockFlags};
use uuid::Uuid;

use crate::{
    block::{
        BlockBehaviour, GetStateForNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs,
        PathComputationType, PlayerPlacedArgs, RandomTickArgs,
    },
    entity::{EntityBase, r#type::from_type},
};

const MAX_HYDRATION_LEVEL: u8 = 3;
/// Clamped to `u8::MAX` due to Pumpkin's block tick delay representation.
const HYDRATION_TICK_DELAY: u8 = u8::MAX;

#[pumpkin_block("minecraft:dried_ghast")]
pub struct DriedGhastBlock;

const fn horizontal_facing_to_yaw(facing: HorizontalFacing) -> f32 {
    match facing {
        HorizontalFacing::South => 0.0,
        HorizontalFacing::West => 90.0,
        HorizontalFacing::North => 180.0,
        HorizontalFacing::East => 270.0,
    }
}

impl BlockBehaviour for DriedGhastBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = DriedGhastLikeProperties::default(args.block);
        props.waterlogged = args.replacing.water_source();
        props.facing = args.player.get_entity().get_horizontal_facing().opposite();
        props.to_state_id(args.block)
    }

    fn player_placed(&self, args: PlayerPlacedArgs<'_>) {
        let props = DriedGhastLikeProperties::from_state_id(args.state_id);
        let sound = if props.waterlogged {
            Sound::BlockDriedGhastPlaceInWater
        } else {
            Sound::BlockDriedGhastPlace
        };
        args.world
            .play_sound(sound, SoundCategory::Blocks, &args.position.to_f64());
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let props = DriedGhastLikeProperties::from_state_id(args.state_id);
        if props.waterlogged {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }
        args.state_id
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.position);
        let props = DriedGhastLikeProperties::from_state_id(state_id);
        if (props.waterlogged || props.hydration > 0)
            && !args
                .world
                .is_block_tick_scheduled(args.position, args.block)
        {
            args.world.schedule_block_tick(
                args.block,
                *args.position,
                HYDRATION_TICK_DELAY,
                TickPriority::Normal,
            );
        }
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.position);
        let mut props = DriedGhastLikeProperties::from_state_id(state_id);
        if props.waterlogged {
            if props.hydration < MAX_HYDRATION_LEVEL {
                args.world.play_sound(
                    Sound::BlockDriedGhastTransition,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );
                props.hydration += 1;
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_LISTENERS,
                );
            } else {
                args.world
                    .break_block(args.position, None, BlockFlags::SKIP_DROPS);
                let spawn_pos = Vector3::new(
                    f64::from(args.position.0.x) + 0.5,
                    f64::from(args.position.0.y),
                    f64::from(args.position.0.z) + 0.5,
                );
                let ghastling = from_type(
                    &EntityType::HAPPY_GHAST,
                    spawn_pos,
                    args.world,
                    Uuid::new_v4(),
                );
                let entity = ghastling.get_entity();
                entity.set_age(-24000);
                entity.set_rotation(horizontal_facing_to_yaw(props.facing), 0.0);
                args.world.play_sound(
                    Sound::EntityGhastlingSpawn,
                    SoundCategory::Blocks,
                    &entity.pos.load(),
                );
                args.world.spawn_entity(ghastling);
            }
        } else if props.hydration > 0 {
            args.world.play_sound(
                Sound::BlockDriedGhastTransition,
                SoundCategory::Blocks,
                &args.position.to_f64(),
            );
            props.hydration -= 1;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_LISTENERS,
            );
        }
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}
