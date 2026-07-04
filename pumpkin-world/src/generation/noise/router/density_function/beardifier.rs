use crate::generation::noise::router::chunk_density_function::ChunkNoiseFunctionSampleOptions;
use crate::generation::noise::router::chunk_noise_router::{
    ChunkNoiseFunctionComponent, MutableChunkNoiseFunctionComponentImpl,
};
use pumpkin_util::math::{block_box::BlockBox, vector3::Vector3};
use std::sync::OnceLock;

use super::{NoiseFunctionComponentRange, StaticIndependentChunkNoiseFunctionComponentImpl};

const BEARD_KERNEL_RADIUS: i32 = 12;
const BEARD_KERNEL_SIZE: i32 = 24;

static BEARD_KERNEL: OnceLock<[f64; 13824]> = OnceLock::new();

#[expect(clippy::large_stack_arrays)]
fn get_beard_kernel() -> &'static [f64; 13824] {
    BEARD_KERNEL.get_or_init(|| {
        let mut kernel = [0.0; 13824];
        for zi in 0..BEARD_KERNEL_SIZE {
            for xi in 0..BEARD_KERNEL_SIZE {
                for yi in 0..BEARD_KERNEL_SIZE {
                    kernel[(zi * 24 * 24 + xi * 24 + yi) as usize] = compute_beard_contribution(
                        xi - BEARD_KERNEL_RADIUS,
                        (yi - BEARD_KERNEL_RADIUS) as f64 + 0.5,
                        zi - BEARD_KERNEL_RADIUS,
                    );
                }
            }
        }
        kernel
    })
}

fn compute_beard_contribution(dx: i32, dy: f64, dz: i32) -> f64 {
    let distance_sqr = (dx as f64).powi(2) + dy.powi(2) + (dz as f64).powi(2);
    std::f64::consts::E.powf(-distance_sqr / 16.0)
}

fn get_bury_contribution(dx: f64, dy: f64, dz: f64) -> f64 {
    let distance = (dx * dx + dy * dy + dz * dz).sqrt();

    // Equivalent to Mth.clampedMap(distance, 0.0, 6.0, 1.0, 0.0)
    if distance < 0.0 {
        1.0
    } else if distance > 6.0 {
        0.0
    } else {
        1.0 - (distance / 6.0)
    }
}

fn get_beard_contribution(dx: i32, dy: i32, dz: i32, y_to_ground: i32) -> f64 {
    let xi = dx + BEARD_KERNEL_RADIUS;
    let yi = dy + BEARD_KERNEL_RADIUS;
    let zi = dz + BEARD_KERNEL_RADIUS;

    if (0..BEARD_KERNEL_SIZE).contains(&xi)
        && (0..BEARD_KERNEL_SIZE).contains(&yi)
        && (0..BEARD_KERNEL_SIZE).contains(&zi)
    {
        let dy_with_offset = y_to_ground as f64 + 0.5;
        let distance_sqr = (dx as f64).powi(2) + dy_with_offset.powi(2) + (dz as f64).powi(2);

        // Equivalent to: -dyWithOffset * Mth.fastInvSqrt(distanceSqr / 2.0) / 2.0
        let value = -dy_with_offset * (distance_sqr / 2.0).sqrt().recip() / 2.0;

        let kernel = get_beard_kernel();
        value * kernel[(zi * 24 * 24 + xi * 24 + yi) as usize]
    } else {
        0.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TerrainAdaptation {
    None,
    BeardThin,
    BeardBox,
    Bury,
    Encapsulate,
}

impl TerrainAdaptation {
    pub fn from_str(s: &str) -> Self {
        match s {
            "beard_thin" => Self::BeardThin,
            "beard_box" => Self::BeardBox,
            "bury" => Self::Bury,
            "encapsulate" => Self::Encapsulate,
            _ => Self::None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BeardifierJunction {
    pub x: i32,
    pub ground_y: i32,
    pub z: i32,
}

// Corresponds to Beardifier.Rigid in Java
#[derive(Clone, Debug)]
pub struct BeardifierStructure {
    pub bounding_box: BlockBox,
    pub terrain_adaptation: TerrainAdaptation,
    pub ground_level_delta: i32,
}

#[derive(Clone)]
pub struct Beardifier {
    pub structures: Vec<BeardifierStructure>,
    pub junctions: Vec<BeardifierJunction>,
    pub affected_box: Option<BlockBox>,
}

impl Beardifier {
    pub const fn new(
        structures: Vec<BeardifierStructure>,
        junctions: Vec<BeardifierJunction>,
        affected_box: Option<BlockBox>,
    ) -> Self {
        Self {
            structures,
            junctions,
            affected_box,
        }
    }
}

impl StaticIndependentChunkNoiseFunctionComponentImpl for Beardifier {
    fn sample(&self, pos: &Vector3<i32>) -> f64 {
        let Some(affected_box) = self.affected_box else {
            return 0.0;
        };

        let block_x = pos.x;
        let block_y = pos.y;
        let block_z = pos.z;

        if !affected_box.contains(block_x, block_y, block_z) {
            return 0.0;
        }

        let mut weight = 0.0;

        for structure in &self.structures {
            let box_min_x = structure.bounding_box.min.x;
            let box_min_y = structure.bounding_box.min.y;
            let box_min_z = structure.bounding_box.min.z;
            let box_max_x = structure.bounding_box.max.x;
            let box_max_y = structure.bounding_box.max.y;
            let box_max_z = structure.bounding_box.max.z;

            let ground_level_delta = structure.ground_level_delta;

            let dx = 0.max((box_min_x - block_x).max(block_x - box_max_x));
            let dz = 0.max((box_min_z - block_z).max(block_z - box_max_z));
            let ground_y = box_min_y + ground_level_delta;
            let dy_to_ground = block_y - ground_y;

            let dy = match structure.terrain_adaptation {
                TerrainAdaptation::None => 0,
                TerrainAdaptation::Bury | TerrainAdaptation::BeardThin => dy_to_ground,
                TerrainAdaptation::BeardBox => 0.max((ground_y - block_y).max(block_y - box_max_y)),
                TerrainAdaptation::Encapsulate => {
                    0.max((box_min_y - block_y).max(block_y - box_max_y))
                }
            };

            let contrib = match structure.terrain_adaptation {
                TerrainAdaptation::None => 0.0,
                TerrainAdaptation::Bury => {
                    get_bury_contribution(dx as f64, dy as f64 / 2.0, dz as f64)
                }
                TerrainAdaptation::BeardThin | TerrainAdaptation::BeardBox => {
                    get_beard_contribution(dx, dy, dz, dy_to_ground) * 0.8
                }
                TerrainAdaptation::Encapsulate => {
                    get_bury_contribution(dx as f64 / 2.0, dy as f64 / 2.0, dz as f64 / 2.0) * 0.8
                }
            };
            weight += contrib;
        }

        for junction in &self.junctions {
            let j_dx = block_x - junction.x;
            let j_dy = block_y - junction.ground_y;
            let j_dz = block_z - junction.z;

            weight += get_beard_contribution(j_dx, j_dy, j_dz, j_dy) * 0.4;
        }

        weight
    }
}

impl MutableChunkNoiseFunctionComponentImpl for Beardifier {
    fn sample(
        &mut self,
        _component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
        _sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        StaticIndependentChunkNoiseFunctionComponentImpl::sample(self, pos)
    }
}

impl NoiseFunctionComponentRange for Beardifier {
    fn min(&self) -> f64 {
        f64::NEG_INFINITY
    }

    fn max(&self) -> f64 {
        f64::INFINITY
    }
}
