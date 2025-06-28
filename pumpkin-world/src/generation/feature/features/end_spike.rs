use pumpkin_data::Block;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::{height_limit::HeightLimitView, section_coords},
    world::BlockRegistryExt,
};

#[derive(Deserialize)]
pub struct EndSpikeFeature {
    crystal_invulnerable: bool,
    spikes: Vec<Spike>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Spike {
    center_x: i32,
    center_z: i32,
    radius: i32,
    height: i32,
    guarded: bool,
}

impl Spike {
    pub fn is_in_chunk(&self, pos: &BlockPos) -> bool {
        section_coords::block_to_section(pos.0.x) == section_coords::block_to_section(self.center_x)
            && section_coords::block_to_section(pos.0.z)
                == section_coords::block_to_section(self.center_z)
    }
}

impl EndSpikeFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk<'_>,
        _block_registry: &dyn BlockRegistryExt,
        _min_y: i8,
        _height: u16,
        _feature: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut spikes = self.spikes.clone();
        if spikes.is_empty() {
            for i in 0..10 {
                let angle = 2.0 * (-std::f64::consts::PI + 0.3141592653589793 * i as f64);
                let center_x = (42.0 * angle.cos()).floor() as i32;
                let center_z = (42.0 * angle.sin()).floor() as i32;

                let l = random.next_bounded_i32(10); // TODO
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

    fn gen_spike(spike: &Spike, chunk: &mut ProtoChunk<'_>) {
        let radius = spike.radius;
        for pos in BlockPos::iterate(
            BlockPos::new(
                spike.center_x - radius,
                chunk.bottom_y() as i32,
                spike.center_z - radius,
            ),
            BlockPos::new(
                spike.center_x + radius,
                chunk.height() as i32 + 10,
                spike.center_z + radius,
            ),
        ) {
            if pos
                .0
                .squared_distance_to(spike.center_x, pos.0.y, spike.center_z)
                <= (radius * radius + 1)
                && pos.0.y < spike.height
            {
                chunk.set_block_state(&pos.0, &Block::OBSIDIAN.default_state);
                continue;
            }
            if pos.0.y <= 65 {
                continue;
            }
            chunk.set_block_state(&pos.0, &Block::AIR.default_state);
        }
        // TODO
    }
}
