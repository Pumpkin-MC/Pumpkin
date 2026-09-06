use pumpkin_data::{Block, BlockState, block_properties::OakFenceLikeProperties};
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use crate::generation::proto_chunk::GenerationCache;
use crate::{generation::section_coords, world::WorldPortalExt};

pub struct EndSpikeFeature {
    pub crystal_invulnerable: bool,
    pub spikes: Vec<Spike>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spike {
    pub center_x: i32,
    pub center_z: i32,
    pub radius: i32,
    pub height: i32,
    pub guarded: bool,
}

impl Spike {
    #[must_use]
    pub const fn is_in_chunk(&self, pos: &BlockPos) -> bool {
        section_coords::block_to_section(pos.0.x) == section_coords::block_to_section(self.center_x)
            && section_coords::block_to_section(pos.0.z)
                == section_coords::block_to_section(self.center_z)
    }

    #[must_use]
    pub const fn is_center_within_chunk(&self, chunk_origin: &BlockPos) -> bool {
        section_coords::block_to_section(chunk_origin.0.x)
            == section_coords::block_to_section(self.center_x)
            && section_coords::block_to_section(chunk_origin.0.z)
                == section_coords::block_to_section(self.center_z)
    }

    #[must_use]
    pub const fn top_bounding_box(&self) -> (f64, f64, f64, f64, f64, f64) {
        (
            (self.center_x - self.radius) as f64,
            0.0,
            (self.center_z - self.radius) as f64,
            (self.center_x + self.radius) as f64 + 1.0,
            256.0,
            (self.center_z + self.radius) as f64 + 1.0,
        )
    }
}

pub struct JavaRandom {
    seed: u64,
}

impl JavaRandom {
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self {
            seed: (seed ^ 0x5DEECE66D) & ((1u64 << 48) - 1),
        }
    }

    pub const fn next(&mut self, bits: u32) -> i32 {
        self.seed = (self.seed.wrapping_mul(0x5DEECE66D).wrapping_add(0xB)) & ((1u64 << 48) - 1);
        (self.seed >> (48 - bits)) as i32
    }

    pub const fn next_i32(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            return 0;
        }
        if (bound & -bound) == bound {
            return ((bound as i64 * self.next(31) as i64) >> 31) as i32;
        }
        let mut bits = self.next(31);
        let mut val = bits % bound;
        while bits - val + (bound - 1) < 0 {
            bits = self.next(31);
            val = bits % bound;
        }
        val
    }

    pub const fn next_i64(&mut self) -> i64 {
        ((self.next(32) as i64) << 32) + self.next(32) as i64
    }
}

impl EndSpikeFeature {
    #[must_use]
    pub fn get_spikes_for_seed(seed: u64) -> [Spike; 10] {
        let mut random = JavaRandom::new(seed);
        let key = (random.next_i64() & 65535) as u64;
        let mut key_random = JavaRandom::new(key);

        let mut sizes: [i32; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        for i in (1..10usize).rev() {
            let j = key_random.next_i32(i as i32 + 1) as usize;
            sizes.swap(i, j);
        }

        let mut spikes = [Spike {
            center_x: 0,
            center_z: 0,
            radius: 0,
            height: 0,
            guarded: false,
        }; 10];

        for i in 0..10 {
            let angle = 2.0 * (-std::f64::consts::PI + (std::f64::consts::PI / 10.0) * i as f64);
            let center_x = (42.0 * angle.cos()).floor() as i32;
            let center_z = (42.0 * angle.sin()).floor() as i32;
            let size = sizes[i];
            let radius = 2 + size / 3;
            let height = 76 + size * 3;
            let guarded = size == 1 || size == 2;
            spikes[i] = Spike {
                center_x,
                center_z,
                radius,
                height,
                guarded,
            };
        }

        spikes
    }

    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _block_registry: &dyn WorldPortalExt,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let spikes = if self.spikes.is_empty() {
            Self::get_spikes_for_seed(0).to_vec()
        } else {
            self.spikes.clone()
        };

        for spike in spikes {
            if !spike.is_center_within_chunk(&pos) {
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

        if spike.guarded {
            for dy in 0i32..=3 {
                for dx in -2i32..=2 {
                    for dz in -2i32..=2 {
                        let x_wall_present = dx.abs() == 2;
                        let z_wall_present = dz.abs() == 2;
                        let on_top = dy == 3;
                        if !x_wall_present && !z_wall_present && !on_top {
                            continue;
                        }

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
