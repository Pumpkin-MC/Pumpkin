use std::sync::Arc;

use pumpkin_data::{
    Block, BlockId, BlockStateId,
    dimension::Dimension,
    effect::StatusEffect,
    entity::EntityType,
    particle::Particle,
    sound::{Sound, SoundCategory},
};
use pumpkin_util::{
    Difficulty,
    math::{position::BlockPos, vector3::Vector3},
};
use pumpkin_world::{tick::TickPriority, world::BlockFlags};
use rand::RngExt;

use crate::{
    block::{
        BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
        OnEntityCollisionArgs, OnScheduledTickArgs, RandomTickArgs, blocks::plant::PlantBlockBase,
    },
    world::World,
};

const EYEBLOSSOM_XZ_RANGE: i32 = 3;
const EYEBLOSSOM_Y_RANGE: i32 = 2;

pub struct EyeblossomBlock;

impl BlockMetadata for EyeblossomBlock {
    fn ids() -> Box<[BlockId]> {
        Box::new([BlockId::OPEN_EYEBLOSSOM, BlockId::CLOSED_EYEBLOSSOM])
    }
}

impl BlockBehaviour for EyeblossomBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position)
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

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !<Self as PlantBlockBase>::can_place_at(self, args.world.as_ref(), args.position) {
                args.world
                    .break_block(args.position, None, BlockFlags::empty())
                    .await;
                return;
            }

            let was_open = args.block == &Block::OPEN_EYEBLOSSOM;
            if try_changing_state(args.world, args.block, args.position).await {
                let sound = if was_open {
                    Sound::BlockEyeblossomClose
                } else {
                    Sound::BlockEyeblossomOpen
                };
                args.world.play_sound(
                    sound,
                    SoundCategory::Blocks,
                    &args.position.to_centered_f64(),
                );
            }
        })
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let was_open = args.block == &Block::OPEN_EYEBLOSSOM;
            if try_changing_state(args.world, args.block, args.position).await {
                let sound = if was_open {
                    Sound::BlockEyeblossomCloseLong
                } else {
                    Sound::BlockEyeblossomOpenLong
                };
                args.world.play_sound(
                    sound,
                    SoundCategory::Blocks,
                    &args.position.to_centered_f64(),
                );
            }
        })
    }

    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if args.world.level_info.load().difficulty == Difficulty::Peaceful {
                return;
            }

            if args.entity.get_entity().entity_type == &EntityType::BEE
                && let Some(living_entity) = args.entity.get_living_entity()
            {
                let effect = pumpkin_data::potion::Effect {
                    effect_type: &StatusEffect::POISON,
                    duration: 25,
                    amplifier: 0,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: true,
                };
                living_entity.add_effect(effect).await;
            }
        })
    }
}

impl PlantBlockBase for EyeblossomBlock {}

pub async fn try_changing_state(world: &Arc<World>, current_block: &Block, pos: &BlockPos) -> bool {
    let is_open = current_block == &Block::OPEN_EYEBLOSSOM;
    let should_be_open = if world.dimension == Dimension::OVERWORLD
        || world.dimension == Dimension::OVERWORLD_CAVES
    {
        world.level_time.lock().await.is_night()
    } else {
        is_open
    };

    if should_be_open == is_open {
        return false;
    }

    let new_block = if is_open {
        &Block::CLOSED_EYEBLOSSOM
    } else {
        &Block::OPEN_EYEBLOSSOM
    };

    world
        .set_block_state(pos, new_block.default_state.id, BlockFlags::NOTIFY_ALL)
        .await;

    world.spawn_particle(
        pos.to_centered_f64(),
        Vector3::new(0.0, 0.0, 0.0),
        0.0,
        1,
        Particle::Trail,
    );

    let mut rng = rand::rng();
    for dx in -EYEBLOSSOM_XZ_RANGE..=EYEBLOSSOM_XZ_RANGE {
        for dy in -EYEBLOSSOM_Y_RANGE..=EYEBLOSSOM_Y_RANGE {
            for dz in -EYEBLOSSOM_XZ_RANGE..=EYEBLOSSOM_XZ_RANGE {
                if dx == 0 && dy == 0 && dz == 0 {
                    continue;
                }
                let nearby_pos = pos.offset(Vector3::new(dx, dy, dz));
                let nearby_block = world.get_block(&nearby_pos);
                if nearby_block == current_block {
                    let dist_sqr = (dx * dx + dy * dy + dz * dz) as f64;
                    let distance = dist_sqr.sqrt();
                    let min_delay = (distance * 5.0) as u8;
                    let max_delay = (distance * 10.0) as u8;
                    let delay = if min_delay >= max_delay {
                        min_delay
                    } else {
                        rng.random_range(min_delay..=max_delay)
                    };
                    world.schedule_block_tick(
                        current_block,
                        nearby_pos,
                        delay.max(1),
                        TickPriority::Normal,
                    );
                }
            }
        }
    }

    true
}
