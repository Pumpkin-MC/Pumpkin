use std::sync::Arc;

use pumpkin_data::{
    Block, BlockId, BlockStateId,
    dimension::Dimension,
    effect::StatusEffect,
    entity::EntityType,
    sound::{Sound, SoundCategory},
};
use pumpkin_protocol::java::client::play::ParticleOptions;
use pumpkin_util::{
    Difficulty,
    math::{position::BlockPos, vector3::Vector3},
};
use pumpkin_world::{tick::TickPriority, world::BlockFlags};
use rand::RngExt;

use crate::{
    block::{
        BlockBehaviour, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
        OnEntityCollisionArgs, OnScheduledTickArgs, RandomTickArgs, blocks::plant::PlantBlockBase,
    },
    world::World,
};

const EYEBLOSSOM_XZ_RANGE: i32 = 3;
const EYEBLOSSOM_Y_RANGE: i32 = 2;

/// Trail particle colors from vanilla `EyeblossomBlock.Type.particleColor`.
const OPEN_PARTICLE_COLOR: i32 = 16_545_810;
const CLOSED_PARTICLE_COLOR: i32 = 6_250_335;

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

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        <Self as PlantBlockBase>::get_state_for_neighbor_update(
            self,
            args.world,
            args.position,
            args.state_id,
        )
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !<Self as PlantBlockBase>::can_place_at(self, args.world.as_ref(), args.position) {
            args.world
                .break_block(args.position, None, BlockFlags::empty());
            return;
        }

        let was_open = args.block == &Block::OPEN_EYEBLOSSOM;
        if try_changing_state(args.world, args.block, args.position) {
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
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        let was_open = args.block == &Block::OPEN_EYEBLOSSOM;
        if try_changing_state(args.world, args.block, args.position) {
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
    }

    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        {
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
                living_entity.add_effect(effect);
            }
        }
    }
}

impl PlantBlockBase for EyeblossomBlock {}

pub fn try_changing_state(world: &Arc<World>, current_block: &Block, pos: &BlockPos) -> bool {
    let is_open = current_block == &Block::OPEN_EYEBLOSSOM;
    let should_be_open = if world.dimension == Dimension::OVERWORLD
        || world.dimension == Dimension::OVERWORLD_CAVES
    {
        world
            .level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_night()
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

    world.set_block_state(pos, new_block.default_state.id, BlockFlags::NOTIFY_ALL);

    let mut rng = rand::rng();
    spawn_transform_particle(world, new_block, pos, &mut rng);

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

/// Emits the `minecraft:trail` particle that marks a flower changing state,
/// mirroring vanilla `EyeblossomBlock.Type.spawnTransformParticle`.
fn spawn_transform_particle(
    world: &Arc<World>,
    new_block: &Block,
    pos: &BlockPos,
    rng: &mut impl RngExt,
) {
    world.spawn_particle_with_options(
        pos.to_centered_f64(),
        Vector3::new(0.0, 0.0, 0.0),
        0.0,
        1,
        &transform_particle(new_block, pos, rng),
    );
}

/// Builds that particle. Vanilla runs this on the type the flower turned *into*,
/// so `new_block` is the new state and the color comes from it.
fn transform_particle(new_block: &Block, pos: &BlockPos, rng: &mut impl RngExt) -> ParticleOptions {
    let color = if new_block == &Block::OPEN_EYEBLOSSOM {
        OPEN_PARTICLE_COLOR
    } else {
        CLOSED_PARTICLE_COLOR
    };

    let start = pos.to_centered_f64();
    let lifetime = 0.5 + rng.random::<f64>();
    let velocity = Vector3::new(
        rng.random::<f64>() - 0.5,
        rng.random::<f64>() + 1.0,
        rng.random::<f64>() - 0.5,
    );

    ParticleOptions::Trail {
        target: start.add(&velocity.multiply(lifetime, lifetime, lifetime)),
        color,
        duration: (20.0 * lifetime) as i32,
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_protocol::java::client::play::ParticleOptions;
    use rand::{SeedableRng, rngs::StdRng};

    use super::{Block, BlockPos, CLOSED_PARTICLE_COLOR, OPEN_PARTICLE_COLOR, transform_particle};

    /// Vanilla bounds: `lifetime` is `0.5 + random()`, so it lies in `[0.5, 1.5)`,
    /// and the velocity components in `[-0.5, 0.5)` and `[1.0, 2.0)`.
    #[test]
    fn transform_particle_stays_within_the_vanilla_envelope() {
        let mut rng = StdRng::seed_from_u64(0x376C_0554);
        let pos = BlockPos::new(12, 70, -34);
        let center = pos.to_centered_f64();

        for _ in 0..10_000 {
            let ParticleOptions::Trail {
                target,
                color,
                duration,
            } = transform_particle(&Block::OPEN_EYEBLOSSOM, &pos, &mut rng);

            assert_eq!(color, OPEN_PARTICLE_COLOR);
            // duration is `(20.0 * lifetime) as i32` over `[0.5, 1.5)`.
            assert!((10..=29).contains(&duration), "duration {duration}");

            let offset = target.sub(&center);
            assert!(offset.x.abs() < 0.75, "x {}", offset.x);
            assert!(offset.z.abs() < 0.75, "z {}", offset.z);
            assert!(
                offset.y > 0.0 && offset.y < 3.0,
                "y {} should rise but stay near the flower",
                offset.y
            );
        }
    }

    /// The color identifies the state the flower turned into, not the one it left.
    #[test]
    fn transform_particle_takes_the_color_of_the_new_state() {
        let mut rng = StdRng::seed_from_u64(0x0EFE_B105);
        let pos = BlockPos::new(0, 64, 0);

        let closed = transform_particle(&Block::CLOSED_EYEBLOSSOM, &pos, &mut rng);
        let ParticleOptions::Trail { color, .. } = closed;
        assert_eq!(color, CLOSED_PARTICLE_COLOR);
        assert_ne!(CLOSED_PARTICLE_COLOR, OPEN_PARTICLE_COLOR);
    }
}
