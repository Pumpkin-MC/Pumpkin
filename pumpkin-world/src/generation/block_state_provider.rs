use pumpkin_data::{BlockState, chunk::DoublePerlinNoiseParameters};
use pumpkin_util::{
    DoublePerlinNoiseParametersCodec,
    math::{
        clamped_map,
        int_provider::IntProvider,
        pool::{Pool, Weighted},
        position::BlockPos,
        vector3::Vector3,
    },
    random::{RandomGenerator, RandomImpl, legacy_rand::LegacyRand},
};
use serde::Deserialize;

use crate::block::BlockStateCodec;

use super::noise::perlin::DoublePerlinNoiseSampler;

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum BlockStateProvider {
    #[serde(rename = "minecraft:simple_state_provider")]
    Simple(SimpleStateProvider),
    #[serde(rename = "minecraft:weighted_state_provider")]
    Weighted(WeightedBlockStateProvider),
    #[serde(rename = "minecraft:noise_threshold_provider")]
    NoiseThreshold(NoiseThresholdBlockStateProvider),
    #[serde(rename = "minecraft:noise_provider")]
    NoiseProvider(NoiseBlockStateProvider),
    #[serde(rename = "minecraft:dual_noise_provider")]
    DualNoise(DualNoiseBlockStateProvider),
    #[serde(rename = "minecraft:rotated_block_provider")]
    Pillar(PillarBlockStateProvider),
    #[serde(rename = "minecraft:randomized_int_state_provider")]
    RandomizedInt(RandomizedIntBlockStateProvider),
}

impl BlockStateProvider {
    pub fn get(&self, random: &mut RandomGenerator, pos: BlockPos) -> BlockState {
        match self {
            BlockStateProvider::NoiseThreshold(provider) => provider.get(random, pos),
            BlockStateProvider::NoiseProvider(provider) => provider.get(pos),
            BlockStateProvider::Simple(provider) => provider.get(pos),
            BlockStateProvider::Weighted(provider) => provider.get(random),
            BlockStateProvider::DualNoise(provider) => provider.get(pos),
            BlockStateProvider::Pillar(provider) => provider.get(pos),
            BlockStateProvider::RandomizedInt(provider) => provider.get(random, pos),
        }
    }
}

#[derive(Deserialize)]
pub struct RandomizedIntBlockStateProvider {
    source: Box<BlockStateProvider>,
    property: String,
    values: IntProvider,
}

impl RandomizedIntBlockStateProvider {
    pub fn get(&self, random: &mut RandomGenerator, pos: BlockPos) -> BlockState {
        // TODO
        self.source.get(random, pos)
    }
}

#[derive(Deserialize)]
pub struct PillarBlockStateProvider {
    state: BlockStateCodec,
}

impl PillarBlockStateProvider {
    pub fn get(&self, _pos: BlockPos) -> BlockState {
        // TODO: random axis
        self.state.get_state().unwrap()
    }
}

#[derive(Deserialize)]
pub struct DualNoiseBlockStateProvider {
    #[serde(flatten)]
    base: NoiseBlockStateProvider,
    variety: [u32; 2],
    slow_noise: DoublePerlinNoiseParametersCodec,
    slow_scale: f32,
}

impl DualNoiseBlockStateProvider {
    pub fn get(&self, pos: BlockPos) -> BlockState {
        let noise = perlin_codec_to_static(self.slow_noise.clone());
        let sampler = DoublePerlinNoiseSampler::new(
            &mut RandomGenerator::Legacy(LegacyRand::from_seed(self.base.base.seed as u64)),
            &noise,
            false,
        );
        let slow_noise = self.get_slow_noise(&pos, &sampler);
        let mapped = clamped_map(
            slow_noise,
            -1.0,
            1.0,
            self.variety[0] as f64,
            self.variety[1] as f64 + 1.0,
        ) as i32;
        let mut list = Vec::with_capacity(mapped as usize);
        for i in 0..mapped {
            let value = self.get_slow_noise(
                &BlockPos(pos.0.add(&Vector3::new(i * 54545, 0, i * 34234))),
                &sampler,
            );
            list.push(self.base.get_state_by_value(&self.base.states, value));
        }
        let value = self.base.base.get_noise(pos);
        self.base
            .get_state_by_value(&list, value)
            .get_state()
            .unwrap()
    }

    fn get_slow_noise(&self, pos: &BlockPos, sampler: &DoublePerlinNoiseSampler) -> f64 {
        sampler.sample(
            pos.0.x as f64 * self.slow_scale as f64,
            pos.0.y as f64 * self.slow_scale as f64,
            pos.0.z as f64 * self.slow_scale as f64,
        )
    }
}

#[derive(Deserialize)]
pub struct WeightedBlockStateProvider {
    entries: Vec<Weighted<BlockStateCodec>>,
}

impl WeightedBlockStateProvider {
    pub fn get(&self, random: &mut RandomGenerator) -> BlockState {
        Pool.get(&self.entries, random)
            .unwrap()
            .get_state()
            .unwrap()
    }
}

#[derive(Deserialize)]
pub struct SimpleStateProvider {
    state: BlockStateCodec,
}

impl SimpleStateProvider {
    pub fn get(&self, _pos: BlockPos) -> BlockState {
        self.state.get_state().unwrap()
    }
}

#[derive(Deserialize)]
pub struct NoiseBlockStateProviderBase {
    seed: i64,
    noise: DoublePerlinNoiseParametersCodec,
    scale: f32,
}

fn perlin_codec_to_static(noise: DoublePerlinNoiseParametersCodec) -> DoublePerlinNoiseParameters {
    let amplitudes_static: &'static [f64] = noise.amplitudes.leak();
    DoublePerlinNoiseParameters::new(noise.first_octave, amplitudes_static, "none")
}

impl NoiseBlockStateProviderBase {
    pub fn get_noise(&self, pos: BlockPos) -> f64 {
        let noise = perlin_codec_to_static(self.noise.clone());
        let sampler = DoublePerlinNoiseSampler::new(
            &mut RandomGenerator::Legacy(LegacyRand::from_seed(self.seed as u64)),
            &noise,
            false,
        );
        sampler.sample(
            pos.0.x as f64 * self.scale as f64,
            pos.0.y as f64 * self.scale as f64,
            pos.0.z as f64 * self.scale as f64,
        )
    }
}

#[derive(Deserialize)]
pub struct NoiseBlockStateProvider {
    #[serde(flatten)]
    base: NoiseBlockStateProviderBase,
    states: Vec<BlockStateCodec>,
}

impl NoiseBlockStateProvider {
    pub fn get(&self, pos: BlockPos) -> BlockState {
        let value = self.base.get_noise(pos);
        self.get_state_by_value(&self.states, value)
            .get_state()
            .unwrap()
    }

    fn get_state_by_value(&self, states: &[BlockStateCodec], value: f64) -> BlockStateCodec {
        let val = ((1.0 + value) / 2.0).clamp(0.0, 0.9999);
        states[(val * states.len() as f64) as usize].clone()
    }
}

#[derive(Deserialize)]
pub struct NoiseThresholdBlockStateProvider {
    #[serde(flatten)]
    base: NoiseBlockStateProviderBase,
    threshold: f32,
    high_chance: f32,
    default_state: BlockStateCodec,
    low_states: Vec<BlockStateCodec>,
    high_states: Vec<BlockStateCodec>,
}

impl NoiseThresholdBlockStateProvider {
    pub fn get(&self, random: &mut RandomGenerator, pos: BlockPos) -> BlockState {
        let value = self.base.get_noise(pos);
        if value < self.threshold as f64 {
            return self.low_states[random.next_bounded_i32(self.low_states.len() as i32) as usize]
                .get_state()
                .unwrap();
        }
        if random.next_f32() < self.high_chance {
            return self.high_states
                [random.next_bounded_i32(self.high_states.len() as i32) as usize]
                .get_state()
                .unwrap();
        }
        self.default_state.get_state().unwrap()
    }
}
