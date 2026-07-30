use std::f64::consts::PI;
use std::sync::Mutex;

use pumpkin_data::{
    Block, BlockState,
    block_properties::{BlockProperties, OakFenceLikeProperties},
};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomImpl, legacy_rand::LegacyRand},
};

use crate::generation::proto_chunk::GenerationCache;
use crate::{generation::section_coords, world::WorldPortalExt};

pub struct EndSpikeFeature {
    pub crystal_invulnerable: bool,
    pub spikes: Vec<Spike>,
}

#[derive(Clone)]
pub struct Spike {
    pub center_x: i32,
    pub center_z: i32,
    pub radius: i32,
    pub height: i32,
    pub guarded: bool,
}

impl Spike {
    pub const fn is_in_chunk(&self, pos: &BlockPos) -> bool {
        section_coords::block_to_section(pos.0.x) == section_coords::block_to_section(self.center_x)
            && section_coords::block_to_section(pos.0.z)
                == section_coords::block_to_section(self.center_z)
    }
}

/// Cached spike list keyed by world seed, matching Java's `SPIKE_CACHE`.
static SPIKE_CACHE: Mutex<Option<(u64, Vec<Spike>)>> = Mutex::new(None);

/// Compute the canonical spike list for a given world seed, using the same
/// 48-bit LCG (`LegacyRand`) that Java's `SingleThreadedRandomSource` uses.
///
/// Results are cached so that both worldgen and dragon-fight code get the
/// exact same list without recomputing.
pub fn get_spikes_for_seed(world_seed: u64) -> Vec<Spike> {
    {
        let cache = SPIKE_CACHE.lock().unwrap();
        if let Some((seed, ref spikes)) = *cache
            && seed == world_seed
        {
            return spikes.clone();
        }
    }

    // Stage 1: derive cache key from world seed
    // Java: RandomSource.createThreadLocalInstance(worldSeed).nextLong() & 65535L
    let mut key_rng = LegacyRand::from_seed(world_seed);
    let cache_key = key_rng.next_i64() as u64 & 0xFFFF;

    // Stage 2: shuffle sizes using the cache key
    // Java: SPIKE_CACHE.load() → Util.toShuffledList(IntStream.range(0,10),
    //       RandomSource.createThreadLocalInstance(seed))
    let mut rng = LegacyRand::from_seed(cache_key);
    let mut sizes: Vec<i32> = (0..10).collect();
    for i in (1..10usize).rev() {
        let j = rng.next_bounded_i32(i as i32 + 1) as usize;
        sizes.swap(i, j);
    }

    let mut spikes = Vec::with_capacity(10);
    for (i, &l) in sizes.iter().enumerate() {
        let angle = 2.0 * (-PI + PI / 10.0 * i as f64);
        let center_x = (42.0 * angle.cos()).floor() as i32;
        let center_z = (42.0 * angle.sin()).floor() as i32;
        let radius = 2 + l / 3;
        let height = 76 + l * 3;
        let guarded = l == 1 || l == 2;
        spikes.push(Spike {
            center_x,
            center_z,
            radius,
            height,
            guarded,
        });
    }

    *SPIKE_CACHE.lock().unwrap() = Some((world_seed, spikes.clone()));
    spikes
}

impl EndSpikeFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _block_registry: &dyn WorldPortalExt,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        _random: &mut pumpkin_util::random::RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut spikes = self.spikes.clone();
        if spikes.is_empty() {
            spikes = get_spikes_for_seed(chunk.world_seed());
        }
        for spike in spikes {
            if !spike.is_in_chunk(&pos) {
                continue;
            }
            Self::gen_spike(&spike, chunk);
        }

        true
    }

    fn gen_spike<T: GenerationCache>(spike: &Spike, chunk: &mut T) {
        let radius = spike.radius;
        for pos in BlockPos::iterate(
            BlockPos::new(
                spike.center_x - radius,
                chunk.bottom_y() as i32,
                spike.center_z - radius,
            ),
            BlockPos::new(
                spike.center_x + radius,
                spike.height + 10,
                spike.center_z + radius,
            ),
        ) {
            if pos
                .0
                .squared_distance_to(spike.center_x, pos.0.y, spike.center_z)
                <= (radius * radius + 1)
                && pos.0.y < spike.height
            {
                chunk.set_block_state(&pos.0, Block::OBSIDIAN.default_state);
                continue;
            }
            if pos.0.y <= 65 {
                continue;
            }
            chunk.set_block_state(&pos.0, Block::AIR.default_state);
        }

        // Bedrock cap serves as the crystal base, fire sits on top of it
        chunk.set_block_state(
            &pumpkin_util::math::vector3::Vector3::new(
                spike.center_x,
                spike.height,
                spike.center_z,
            ),
            Block::BEDROCK.default_state,
        );
        chunk.set_block_state(
            &pumpkin_util::math::vector3::Vector3::new(
                spike.center_x,
                spike.height + 1,
                spike.center_z,
            ),
            Block::FIRE.default_state,
        );

        // Iron-bar cage for guarded spikes: 5x5 walls + open top frame at dy=3.
        if spike.guarded {
            for dy in 0i32..=3 {
                for dx in -2i32..=2 {
                    for dz in -2i32..=2 {
                        // Only place on perimeter walls and the top frame
                        let x_wall_present = dx.abs() == 2;
                        let z_wall_present = dz.abs() == 2;
                        let on_top = dy == 3;
                        if !x_wall_present && !z_wall_present && !on_top {
                            continue;
                        }

                        // Connectivity rules
                        let x_edge = x_wall_present || on_top;
                        let z_edge = z_wall_present || on_top;

                        let mut props = OakFenceLikeProperties::default(&Block::IRON_BARS);
                        props.north = x_edge && dz != -2;
                        props.south = x_edge && dz != 2;
                        props.west = z_edge && dx != -2;
                        props.east = z_edge && dx != 2;

                        let bar_state = BlockState::from_id(props.to_state_id(&Block::IRON_BARS));
                        chunk.set_block_state(
                            &pumpkin_util::math::vector3::Vector3::new(
                                spike.center_x + dx,
                                spike.height + dy,
                                spike.center_z + dz,
                            ),
                            bar_state,
                        );
                    }
                }
            }
        }
    }
}
