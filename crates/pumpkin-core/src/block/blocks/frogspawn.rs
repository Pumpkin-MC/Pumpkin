use pumpkin_data::{
    Block, BlockDirection, BlockStateId,
    entity::EntityType,
    fluid::Fluid,
    sound::{Sound, SoundCategory},
    tag::{self, Taggable},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::{
    tick::TickPriority,
    world::{BlockAccessor, BlockFlags},
};
use rand::RngExt;
use uuid::Uuid;

use crate::{
    block::{
        BlockBehaviour, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnEntityCollisionArgs,
        OnScheduledTickArgs, PlacedArgs,
    },
    entity::r#type::from_type,
    world::World,
};

const MIN_TADPOLES_SPAWN: u8 = 2;
const MAX_TADPOLES_SPAWN: u8 = 5;
/// Clamped to `u8::MAX` due to Pumpkin's block tick delay representation.
const HATCH_DELAY: u8 = u8::MAX;

#[pumpkin_block("minecraft:frogspawn")]
pub struct FrogspawnBlock;

impl FrogspawnBlock {
    #[must_use]
    pub fn may_place_on_world(world: &World, pos: &BlockPos) -> bool {
        let below_pos = pos.down();
        let (below_fluid, below_fluid_state) = world.get_fluid_and_fluid_state(&below_pos);
        let below_block = world.get_block(&below_pos);
        let above_fluid = world.get_fluid(pos);
        ((below_fluid.has_tag(&tag::Fluid::MINECRAFT_SUPPORTS_FROGSPAWN)
            && below_fluid_state.is_source)
            || below_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_FROGSPAWN))
            && *above_fluid == Fluid::EMPTY
    }

    #[must_use]
    pub fn may_place_on_accessor(accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let below_pos = pos.down();
        let (below_block, below_state) = accessor.get_block_and_state(&below_pos);
        let (here_block, _) = accessor.get_block_and_state(pos);
        let is_water_source_below =
            below_block == &Block::WATER && below_state.id == Block::WATER.default_state.id;
        (is_water_source_below || below_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_FROGSPAWN))
            && here_block != &Block::WATER
            && here_block != &Block::LAVA
    }
}

impl BlockBehaviour for FrogspawnBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        args.world.map_or_else(
            || Self::may_place_on_accessor(args.block_accessor, args.position),
            |world| Self::may_place_on_world(world, args.position),
        )
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        args.world.schedule_block_tick(
            args.block,
            *args.position,
            HATCH_DELAY,
            TickPriority::Normal,
        );
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.direction == BlockDirection::Down
            && !Self::may_place_on_world(args.world, args.position)
        {
            return Block::AIR.default_state.id;
        }
        args.state_id
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !Self::may_place_on_world(args.world, args.position) {
            args.world
                .break_block(args.position, None, BlockFlags::SKIP_DROPS);
            return;
        }
        args.world
            .break_block(args.position, None, BlockFlags::SKIP_DROPS);
        args.world.play_sound(
            Sound::BlockFrogspawnHatch,
            SoundCategory::Blocks,
            &args.position.to_f64(),
        );
        let count = rand::rng().random_range(MIN_TADPOLES_SPAWN..=MAX_TADPOLES_SPAWN);
        for _ in 0..count {
            let x_offset = rand::rng().random_range(0.2..0.8);
            let z_offset = rand::rng().random_range(0.2..0.8);
            let spawn_pos = Vector3::new(
                f64::from(args.position.0.x) + x_offset,
                f64::from(args.position.0.y),
                f64::from(args.position.0.z) + z_offset,
            );
            let tadpole = from_type(&EntityType::TADPOLE, spawn_pos, args.world, Uuid::new_v4());
            let yaw = rand::rng().random::<f32>() * 360.0;
            tadpole.get_entity().set_rotation(yaw, 0.0);
            if let Some(mob) = tadpole.get_mob() {
                mob.get_mob_entity()
                    .persistence_required
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
            args.world.spawn_entity(tadpole);
        }
    }

    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        if args.entity.get_entity().entity_type == &EntityType::FALLING_BLOCK {
            args.world
                .break_block(args.position, None, BlockFlags::SKIP_DROPS);
        }
    }
}
