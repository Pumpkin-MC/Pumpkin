use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::blocks::plant::big_dripleaf_stem::{
    BigDripleafStemBlock, BigDripleafStemLikeProperties,
};
use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::{
    BlockBehaviour, BonemealArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    OnEntityCollisionArgs, OnNeighborUpdateArgs, OnPlaceArgs, OnProjectileHitArgs,
    OnScheduledTickArgs,
};
use crate::entity::EntityBase;
use crate::entity::ai::pathfinder::node::Coordinate;
use crate::world::World;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{
    BigDripleafLikeProperties, HorizontalFacing, Tilt, WaterLikeProperties,
};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, tag};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use rand::RngExt;

#[pumpkin_block("minecraft:big_dripleaf")]
pub struct BigDripleafBlock;

impl BigDripleafBlock {
    #[must_use]
    pub fn can_grow_into(world: &World, pos: &BlockPos) -> bool {
        if !world.is_in_build_limit(*pos) {
            return false;
        }
        let (block, state) = world.get_block_and_state(pos);
        state.is_air() || block == &Block::WATER || block == &Block::SMALL_DRIPLEAF
    }

    pub fn place(
        world: &Arc<World>,
        pos: &BlockPos,
        waterlogged: bool,
        facing: HorizontalFacing,
    ) -> bool {
        let mut props = BigDripleafLikeProperties::default(&Block::BIG_DRIPLEAF);
        props.waterlogged = waterlogged;
        props.facing = facing;
        world.set_block_state(
            pos,
            props.to_state_id(&Block::BIG_DRIPLEAF),
            BlockFlags::NOTIFY_ALL,
        );
        true
    }

    pub fn place_with_random_height(
        world: &Arc<World>,
        stem_bottom_pos: &BlockPos,
        facing: HorizontalFacing,
    ) {
        let desired_height = rand::rng().random_range(2..=5);
        let mut pos = *stem_bottom_pos;
        let mut height = 0;

        while height < desired_height && Self::can_grow_into(world, &pos) {
            height += 1;
            pos = pos.up();
        }

        let leaf_y = stem_bottom_pos.0.y + height - 1;
        pos.0.y = stem_bottom_pos.0.y;

        while pos.0.y < leaf_y {
            let waterlogged = is_water_source(world.as_ref(), &pos);
            BigDripleafStemBlock::place(world, &pos, waterlogged, facing);
            pos = pos.up();
        }

        let waterlogged = is_water_source(world.as_ref(), &pos);
        Self::place(world, &pos, waterlogged, facing);
    }
}

impl BlockBehaviour for BigDripleafBlock {
    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        let props = BigDripleafLikeProperties::from_state_id(args.state.id);
        if props.tilt == Tilt::None
            && can_entity_tilt(args.position, args.entity)
            && !block_receives_redstone_power(args.world, args.position)
        {
            set_tilt_and_schedule_tick(
                args.state.id,
                args.world,
                args.position,
                Tilt::Unstable,
                None,
            );
        }
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state = args.world.get_block_state(args.position);
        if block_receives_redstone_power(args.world, args.position) {
            reset_tilt(state.id, args.world, args.position);
            return;
        }
        let props = BigDripleafLikeProperties::from_state_id(state.id);

        if props.tilt == Tilt::Unstable {
            set_tilt_and_schedule_tick(
                state.id,
                args.world,
                args.position,
                Tilt::Partial,
                Some(Sound::BlockBigDripleafTiltDown),
            );
        } else if props.tilt == Tilt::Partial {
            set_tilt_and_schedule_tick(
                state.id,
                args.world,
                args.position,
                Tilt::Full,
                Some(Sound::BlockBigDripleafTiltDown),
            );
        } else if props.tilt == Tilt::Full {
            reset_tilt(state.id, args.world, args.position);
        }
    }

    fn on_projectile_hit(&self, args: OnProjectileHitArgs<'_>) {
        set_tilt_and_schedule_tick(
            args.state.id,
            args.world,
            args.position,
            Tilt::Full,
            Some(Sound::BlockBigDripleafTiltDown),
        );
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position)
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let (support_block, support_block_state_id) =
            args.world.get_block_and_state_id(&args.position.down());
        let facing = if support_block == &Block::BIG_DRIPLEAF {
            get_dripleaf_facing_dir(support_block_state_id)
        } else if support_block == &Block::BIG_DRIPLEAF_STEM {
            BigDripleafStemLikeProperties::from_state_id(support_block_state_id).facing
        } else {
            args.player
                .living_entity
                .entity
                .get_horizontal_facing()
                .opposite()
        };
        let mut dripleaf_props = BigDripleafLikeProperties::default(args.block);

        dripleaf_props.facing = facing;
        dripleaf_props.waterlogged = args.replacing.water_source();

        dripleaf_props.to_state_id(args.block)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.direction == BlockDirection::Down
            && !<Self as PlantBlockBase>::can_place_at(self, args.world, args.position)
        {
            return Block::AIR.default_state.id;
        }
        let props = BigDripleafLikeProperties::from_state_id(args.state_id);
        if props.waterlogged {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }
        if args.direction == BlockDirection::Up
            && args.world.get_block(args.neighbor_position) == &Block::BIG_DRIPLEAF
        {
            let mut stem_props = BigDripleafStemLikeProperties::default(&Block::BIG_DRIPLEAF_STEM);
            stem_props.facing = props.facing;
            stem_props.waterlogged = props.waterlogged;
            return stem_props.to_state_id(&Block::BIG_DRIPLEAF_STEM);
        }
        args.state_id
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        if block_receives_redstone_power(args.world, args.position) {
            let state_id = args.world.get_block_state_id(args.position);
            reset_tilt(state_id, args.world, args.position);
        }
    }

    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        Self::can_grow_into(args.world, &args.position.up())
    }

    fn is_bonemeal_success(&self, _args: BonemealArgs<'_>) -> bool {
        true
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        let above_pos = args.position.up();
        if Self::can_grow_into(args.world, &above_pos) {
            let props = BigDripleafLikeProperties::from_state_id(args.state_id);
            let above_waterlogged = is_water_source(args.world.as_ref(), &above_pos);
            BigDripleafStemBlock::place(args.world, args.position, props.waterlogged, props.facing);
            Self::place(args.world, &above_pos, above_waterlogged, props.facing);
        }
    }
}

#[must_use]
pub fn is_water_source(block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
    let (block, state) = block_accessor.get_block_and_state(pos);
    if block == &Block::WATER {
        return WaterLikeProperties::from_state_id(state.id).level == 0;
    }
    state.id.is_waterlogged()
}
fn set_tilt_and_schedule_tick(
    state_id: BlockStateId,
    world: &Arc<World>,
    pos: &BlockPos,
    tilt: Tilt,
    sound_wrapper: Option<Sound>,
) {
    set_tilt(state_id, world, pos, tilt);
    if let Some(tilt_sound) = sound_wrapper {
        play_tilt_sound(world, pos, tilt_sound);
    }
    let tick_delay = match tilt {
        Tilt::None => -1,
        Tilt::Unstable | Tilt::Partial => 10,
        Tilt::Full => 100,
    };
    if tick_delay != -1 {
        world.schedule_block_tick(
            &Block::BIG_DRIPLEAF,
            *pos,
            tick_delay as u8,
            pumpkin_world::tick::TickPriority::Normal,
        );
    }
}

fn reset_tilt(state_id: BlockStateId, world: &Arc<World>, pos: &BlockPos) {
    set_tilt(state_id, world, pos, Tilt::None);
    let props = BigDripleafLikeProperties::from_state_id(state_id);
    if props.tilt != Tilt::None {
        play_tilt_sound(world, pos, Sound::BlockBigDripleafTiltUp);
    }
}

fn set_tilt(state_id: BlockStateId, world: &Arc<World>, pos: &BlockPos, new_tilt: Tilt) {
    let mut props = BigDripleafLikeProperties::from_state_id(state_id);
    props.tilt = new_tilt;
    world.set_block_state(
        pos,
        props.to_state_id(&Block::BIG_DRIPLEAF),
        BlockFlags::NOTIFY_ALL,
    );
}

fn play_tilt_sound(world: &Arc<World>, pos: &BlockPos, tilt_sound: Sound) {
    let pitch = rand::rng().random_range(0.8f32..1.2f32);
    let v = pos.as_vector3();
    let position = Vector3::new(v.x as f64, v.y as f64, v.z as f64);
    world.play_sound_fine(tilt_sound, SoundCategory::Blocks, &position, 1f32, pitch);
}
fn can_entity_tilt<T: EntityBase + ?Sized>(pos: &BlockPos, entity: &T) -> bool {
    entity.get_entity().on_ground.load(Ordering::Relaxed)
        && entity.get_entity().pos.load().y > pos.as_vector3().y as f64 + 0.6875f64
}
fn get_dripleaf_facing_dir(state_id: BlockStateId) -> HorizontalFacing {
    let dripleaf_props = BigDripleafLikeProperties::from_state_id(state_id);
    dripleaf_props.facing
}

impl PlantBlockBase for BigDripleafBlock {
    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let support_block = block_accessor.get_block(pos);
        can_plant_dripleaf_on_top(support_block)
    }
}
#[must_use]
pub fn can_plant_dripleaf_on_top(support_block: &Block) -> bool {
    if support_block == &Block::BIG_DRIPLEAF || support_block == &Block::BIG_DRIPLEAF_STEM {
        return true;
    }

    support_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_BIG_DRIPLEAF)
}
