use pumpkin_data::{
    Block, BlockState,
    block_properties::{BlockProperties, OakFenceLikeProperties},
};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
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

impl EndSpikeFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _block_registry: &dyn WorldPortalExt,
        _min_y: i8,
        _height: u16,
        _feature: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut spikes = self.spikes.clone();
        if spikes.is_empty() {
            let mut sizes: Vec<i32> = (0..10).collect();
            for i in (1..10usize).rev() {
                let j = random.next_bounded_i32(i as i32 + 1) as usize;
                sizes.swap(i, j);
            }

            for (i, &l) in sizes.iter().enumerate() {
                let angle = 2.0 * (-std::f64::consts::PI + 0.3141592653589793 * i as f64);
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
                        let on_x_wall = dx.abs() == 2;
                        let on_z_wall = dz.abs() == 2;
                        let on_top = dy == 3;
                        if !on_x_wall && !on_z_wall && !on_top {
                            continue;
                        }

                        // Connectivity rules
                        let x_edge = on_x_wall || on_top;
                        let z_edge = on_z_wall || on_top;

                        let mut props = OakFenceLikeProperties::default(&Block::IRON_BARS);
                        props.north = x_edge && dz != 2;
                        props.south = x_edge && dz != -2;
                        props.west = z_edge && dx != 2;
                        props.east = z_edge && dx != -2;

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
