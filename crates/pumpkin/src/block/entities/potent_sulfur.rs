use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use pumpkin_data::BlockId;
use pumpkin_data::block_properties::{
    BlockProperties, PotentSulfurLikeProperties, PotentSulfurState,
};
use pumpkin_data::game_event::GameEvent;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use pumpkin_util::random::{RandomGenerator, RandomImpl};
use pumpkin_world::world::BlockFlags;

use crate::block::blocks::potent_sulfur::find_noxious_gas_source_block;
use crate::world::World;
use crate::world::game_event::{GameEventContext, emit_game_event};

use super::BlockEntity;

/// `net.minecraft.world.level.block.entity.PotentSulfurBlockEntity`.
///
/// Ported: the `countdown` persistence and `SERVER_WAITING_COUNTDOWN_TICKER`, which drives
/// the DORMANT <-> ERUPTING geyser cycle off a position-derived random source. Not ported:
/// `LAUNCH_ENTITY_TICKER` (needs collision-shape sweeps and the `NOT_AFFECTED_BY_GEYSERS`
/// entity tag) and `SERVER_NAUSEA_EFFECT_TICKER` (needs `canBeReachedByNoxiousGas`'s
/// line-of-sight clip). The particle and plume tickers are client-side only.
pub struct PotentSulfurBlockEntity {
    pub position: BlockPos,
    /// `waitingCountdown`, which vanilla initialises to -1.
    pub waiting_countdown: AtomicI32,
}

/// `PotentSulfurBlockEntity.GEYSER_SALT`.
const GEYSER_SALT: i64 = -904_011_478;

impl BlockEntity for PotentSulfurBlockEntity {
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
        // loadAdditional: an absent "countdown" leaves the -1 default untouched.
        Self {
            position,
            waiting_countdown: AtomicI32::new(nbt.get_int("countdown").unwrap_or(-1)),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            nbt.put_int("countdown", self.waiting_countdown.load(Ordering::Relaxed));
        })
    }

    /// `SERVER_WAITING_COUNTDOWN_TICKER`, which vanilla attaches only to the DORMANT and
    /// ERUPTING states.
    fn tick<'a>(&'a self, world: &'a Arc<World>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let (block, state) = world.get_block_and_state(&self.position);
            if block.id != BlockId::POTENT_SULFUR {
                return;
            }
            let mut props = PotentSulfurLikeProperties::from_state_id(state.id, block);
            let is_dormant = match props.potent_sulfur_state {
                PotentSulfurState::Dormant => true,
                PotentSulfurState::Erupting => false,
                _ => return,
            };

            if world.level_time.lock().await.world_age % 20 != 0 {
                return;
            }

            let Some(source_block) = find_noxious_gas_source_block(world, &self.position) else {
                return;
            };

            if self.waiting_countdown.load(Ordering::Relaxed) <= 0 {
                let water_blocks = source_block.0.y - self.position.0.y - 1;
                let mut rng = self.geyser_positional(world);
                // The dormant wait scales with the water column; the erupting wait is
                // short and burns one draw first, so the two phases diverge.
                let countdown = if is_dormant {
                    10 * (water_blocks - 1) + next_int_between_inclusive(&mut rng, 15, 30)
                } else {
                    rng.next_i32();
                    water_blocks - 1 + next_int_between_inclusive(&mut rng, 1, 2)
                };
                self.waiting_countdown.store(countdown, Ordering::Relaxed);
            }

            let remaining = self.waiting_countdown.load(Ordering::Relaxed);
            if remaining > 0 {
                self.waiting_countdown
                    .store(remaining - 1, Ordering::Relaxed);
            }

            if self.waiting_countdown.load(Ordering::Relaxed) != 0 {
                return;
            }

            let next_state = if is_dormant {
                PotentSulfurState::Erupting
            } else {
                PotentSulfurState::Dormant
            };
            props.potent_sulfur_state = next_state;
            world
                .set_block_state(
                    &self.position,
                    props.to_state_id(block),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;

            if next_state == PotentSulfurState::Dormant {
                emit_game_event(
                    world,
                    GameEvent::BlockDeactivate,
                    Vector3::new(
                        f64::from(self.position.0.x) + 0.5,
                        f64::from(self.position.0.y) + 0.5,
                        f64::from(self.position.0.z) + 0.5,
                    ),
                    GameEventContext::none(),
                )
                .await;
            }
        })
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        nbt.put_int("countdown", self.waiting_countdown.load(Ordering::Relaxed));
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl PotentSulfurBlockEntity {
    pub const ID: &'static str = "minecraft:potent_sulfur";

    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            waiting_countdown: AtomicI32::new(-1),
        }
    }

    /// `resetCountdown`.
    pub fn reset_countdown(&self) {
        self.waiting_countdown.store(-1, Ordering::Relaxed);
    }

    /// `geyserPositional`: `new XoroshiroRandomSource(level.getSeed() ^ GEYSER_SALT)
    /// .forkPositional().at(pos)`, so a given geyser's timing is deterministic.
    fn geyser_positional(&self, world: &World) -> RandomGenerator {
        let seed = world.level.seed.0 ^ (GEYSER_SALT as u64);
        RandomGenerator::Xoroshiro(Xoroshiro::from_seed(seed).next_splitter().split_pos(
            self.position.0.x,
            self.position.0.y,
            self.position.0.z,
        ))
    }
}

/// `RandomSource.nextIntBetweenInclusive`.
fn next_int_between_inclusive(rng: &mut RandomGenerator, min: i32, max: i32) -> i32 {
    min + rng.next_bounded_i32(max - min + 1)
}
