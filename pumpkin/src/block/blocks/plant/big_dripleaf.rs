use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::blocks::plant::big_dripleaf_stem::{
    BigDripleafStemLikeProperties, handle_big_dripleaf_breaking,
};
use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::{
    BlockBehaviour, BlockFuture, BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    OnEntityStepArgs, OnNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs, PlacedArgs,
};
use crate::entity::EntityBase;
use crate::entity::ai::pathfinder::node::Coordinate;
use crate::world::World;
use pumpkin_data::block_properties::{
    BigDripleafLikeProperties, BlockProperties, HorizontalFacing, Tilt,
};
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use rand::RngExt;

#[pumpkin_block("minecraft:big_dripleaf")]
pub struct BigDripleafBlock;

impl BlockBehaviour for BigDripleafBlock {
    fn on_entity_step<'a>(&'a self, args: OnEntityStepArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let props = BigDripleafLikeProperties::from_state_id(args.state.id, args.block);
            if props.tilt == Tilt::None
                && can_entity_tilt(args.position, args.entity)
                && !block_receives_redstone_power(args.world, args.position).await
            {
                set_tilt_and_schedule_tick(
                    args.state.id,
                    args.world,
                    args.position,
                    Tilt::Unstable,
                    None,
                )
                .await;
            }
        })
    }
    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position).await;
            if block_receives_redstone_power(args.world, args.position).await {
                reset_tilt(state.id, args.world, args.position).await;
            } else {
                let props =
                    BigDripleafLikeProperties::from_state_id(state.id, &Block::BIG_DRIPLEAF);

                if props.tilt == Tilt::Unstable {
                    set_tilt_and_schedule_tick(
                        state.id,
                        args.world,
                        args.position,
                        Tilt::Partial,
                        Some(Sound::BlockBigDripleafTiltDown),
                    )
                    .await;
                } else if props.tilt == Tilt::Partial {
                    set_tilt_and_schedule_tick(
                        state.id,
                        args.world,
                        args.position,
                        Tilt::Full,
                        Some(Sound::BlockBigDripleafTiltDown),
                    )
                    .await;
                } else if props.tilt == Tilt::Full {
                    reset_tilt(state.id, args.world, args.position).await;
                }
            }
        })
    }
    //TODO: onProjectileHit
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position).await
        })
    }
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let (support_block, support_block_state_id) = args
                .world
                .get_block_and_state_id(&args.position.down())
                .await;
            let facing = if support_block == &Block::BIG_DRIPLEAF {
                get_dripleaf_facing_dir(support_block_state_id)
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
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            <Self as PlantBlockBase>::get_state_for_neighbor_update(
                self,
                args.world,
                args.position,
                args.state_id,
            )
            .await
        })
    }
    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if block_receives_redstone_power(args.world, args.position).await {
                let state_id = args.world.get_block_state_id(args.position).await;
                reset_tilt(state_id, args.world, args.position).await;
            }
        })
    }

    /// if leaf is placed on top of another leaf, turn the lower one into a stem.
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let support_pos = args.position.down();
            let (support_block, support_state_id) =
                args.world.get_block_and_state_id(&support_pos).await;
            if support_block == &Block::BIG_DRIPLEAF {
                let old_dripleaf_props = BigDripleafLikeProperties::from_state_id(
                    support_state_id,
                    &Block::BIG_DRIPLEAF,
                );
                let mut dripleaf_stem_props =
                    BigDripleafStemLikeProperties::default(&Block::BIG_DRIPLEAF_STEM);

                dripleaf_stem_props.facing = old_dripleaf_props.facing;
                dripleaf_stem_props.waterlogged = old_dripleaf_props.waterlogged;
                args.world
                    .set_block_state(
                        &support_pos,
                        dripleaf_stem_props.to_state_id(&Block::BIG_DRIPLEAF_STEM),
                        BlockFlags::empty(),
                    )
                    .await;
            }
        })
    }

    /// if the leaf is broken, turn the stem below into a leaf.
    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move { handle_big_dripleaf_breaking(args.world, args.position).await })
    }
}
async fn set_tilt_and_schedule_tick(
    state_id: BlockStateId,
    world: &Arc<World>,
    pos: &BlockPos,
    tilt: Tilt,
    sound_wrapper: Option<Sound>,
) {
    set_tilt(state_id, world, pos, tilt).await;
    if let Some(tilt_sound) = sound_wrapper {
        play_tilt_sound(world, pos, tilt_sound).await;
    }
    let tick_delay = match tilt {
        Tilt::None => -1,
        Tilt::Unstable | Tilt::Partial => 10,
        Tilt::Full => 100,
    };
    if tick_delay != -1 {
        world
            .schedule_block_tick(
                &Block::BIG_DRIPLEAF,
                *pos,
                tick_delay as u8,
                pumpkin_world::tick::TickPriority::Normal,
            )
            .await;
    }
}
async fn play_tilt_sound(world: &Arc<World>, pos: &BlockPos, tilt_sound: Sound) {
    let pitch = rand::rng().random_range(0.8f32..1.2f32);
    let v = pos.as_vector3();
    let position = Vector3::new(v.x as f64, v.y as f64, v.z as f64);
    world
        .play_sound_fine(tilt_sound, SoundCategory::Blocks, &position, 1f32, pitch)
        .await;
}
async fn reset_tilt(state_id: BlockStateId, world: &Arc<World>, pos: &BlockPos) {
    set_tilt(state_id, world, pos, Tilt::None).await;
    let props = BigDripleafLikeProperties::from_state_id(state_id, &Block::BIG_DRIPLEAF);
    if props.tilt != Tilt::None {
        play_tilt_sound(world, pos, Sound::BlockBigDripleafTiltUp).await;
    }
}
async fn set_tilt(state_id: BlockStateId, world: &Arc<World>, pos: &BlockPos, new_tilt: Tilt) {
    let mut props = BigDripleafLikeProperties::from_state_id(state_id, &Block::BIG_DRIPLEAF);
    props.tilt = new_tilt;
    world
        .set_block_state(
            pos,
            props.to_state_id(&Block::BIG_DRIPLEAF),
            BlockFlags::NOTIFY_ALL,
        )
        .await;
    //todo GameEvents?
}
fn can_entity_tilt<T: EntityBase + ?Sized>(pos: &BlockPos, entity: &T) -> bool {
    entity.get_entity().on_ground.load(Ordering::Relaxed)
        && entity.get_entity().pos.load().y > pos.as_vector3().y as f64 + 0.6875f64
}
fn get_dripleaf_facing_dir(state_id: BlockStateId) -> HorizontalFacing {
    let dripleaf_props = BigDripleafLikeProperties::from_state_id(state_id, &Block::BIG_DRIPLEAF);
    dripleaf_props.facing
}

fn is_dripleaf_waterlogged(state_id: BlockStateId) -> bool {
    let dripleaf_props = BigDripleafLikeProperties::from_state_id(state_id, &Block::BIG_DRIPLEAF);
    dripleaf_props.waterlogged
}
impl PlantBlockBase for BigDripleafBlock {
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let support_block = block_accessor.get_block(pos).await;
        can_plant_dripleaf_on_top(support_block)
    }
    async fn get_state_for_neighbor_update(
        &self,
        block_accessor: &dyn BlockAccessor,
        block_pos: &BlockPos,
        block_state: BlockStateId,
    ) -> BlockStateId {
        if !<Self as PlantBlockBase>::can_place_at(self, block_accessor, block_pos).await {
            if is_dripleaf_waterlogged(block_state) {
                return Block::WATER.default_state.id;
            }
            return Block::AIR.default_state.id;
        }
        block_state
    }
}
#[must_use]
pub fn can_plant_dripleaf_on_top(support_block: &Block) -> bool {
    if support_block == &Block::BIG_DRIPLEAF || support_block == &Block::BIG_DRIPLEAF_STEM {
        return true;
    }

    support_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_BIG_DRIPLEAF)
}
