use pumpkin_data::BlockState;
use pumpkin_util::{
    DoublePerlinNoiseParametersCodec,
    math::{
        clamped_map,
        int_provider::IntProvider,
        pool::{Pool, Weighted},
        position::BlockPos,
    },
    random::{RandomGenerator, RandomImpl, legacy_rand::LegacyRand},
};

use crate::block::BlockStateCodec;

use super::noise::perlin::DoublePerlinNoiseSampler;

pub enum BlockStateProvider {
    Simple(SimpleStateProvider),
    Weighted(WeightedBlockStateProvider),
    NoiseThreshold(NoiseThresholdBlockStateProvider),
    NoiseProvider(NoiseBlockStateProvider),
    DualNoise(DualNoiseBlockStateProvider),
    Pillar(PillarBlockStateProvider),
    RandomizedInt(RandomizedIntBlockStateProvider),
}

impl BlockStateProvider {
    pub fn get(&self, random: &mut RandomGenerator, pos: BlockPos) -> &'static BlockState {
        match self {
            Self::NoiseThreshold(provider) => provider.get(random, pos),
            Self::NoiseProvider(provider) => provider.get(pos),
            Self::Simple(provider) => provider.get(pos),
            Self::Weighted(provider) => provider.get(random),
            Self::DualNoise(provider) => provider.get(pos),
            Self::Pillar(provider) => provider.get(pos),
            Self::RandomizedInt(provider) => provider.get(random, pos),
        }
    }
}

pub struct RandomizedIntBlockStateProvider {
    pub source: Box<BlockStateProvider>,
    pub property: String,
    pub values: IntProvider,
}

impl RandomizedIntBlockStateProvider {
    pub fn get(&self, random: &mut RandomGenerator, pos: BlockPos) -> &'static BlockState {
        // TODO
        self.source.get(random, pos)
    }
}

pub struct PillarBlockStateProvider {
    pub state: BlockStateCodec,
}

impl PillarBlockStateProvider {
    pub fn get(&self, _pos: BlockPos) -> &'static BlockState {
        // TODO: random axis
        self.state.get_state()
    }
}

pub struct DualNoiseBlockStateProvider {
    pub base: NoiseBlockStateProvider,
    pub variety: [u32; 2],
    pub slow_noise: DoublePerlinNoiseParametersCodec,
    pub slow_scale: f64,
}

impl DualNoiseBlockStateProvider {
    pub fn get(&self, pos: BlockPos) -> &'static BlockState {
        let sampler = DoublePerlinNoiseSampler::new(
            &mut RandomGenerator::Legacy(LegacyRand::from_seed(self.base.base.seed as u64)),
            self.slow_noise.first_octave,
            &self.slow_noise.amplitudes,
            false,
        );
        let slow_noise =
            self.get_slow_noise(pos.0.x as f64, pos.0.y as f64, pos.0.z as f64, &sampler);
        let mapped = clamped_map(
            slow_noise,
            -1.0,
            1.0,
            self.variety[0] as f64,
            self.variety[1] as f64 + 1.0,
        ) as i32;
        let mut list = Vec::with_capacity(mapped as usize);
        for i in 0..mapped {
            let value = self.get_slow_noise(i as f64 * 54545.0, 0.0, i as f64 * 34234.0, &sampler);
            list.push(self.base.get_state_by_value(&self.base.states, value));
        }
        let value = self.base.base.get_noise(pos);
        self.base.get_state_by_value_2(&list, value).get_state()
    }

    fn get_slow_noise(&self, x: f64, y: f64, z: f64, sampler: &DoublePerlinNoiseSampler) -> f64 {
        sampler.sample(
            x * self.slow_scale,
            y * self.slow_scale,
            z * self.slow_scale,
        )
    }
}

pub struct WeightedBlockStateProvider {
    pub entries: Vec<Weighted<BlockStateCodec>>,
}

impl WeightedBlockStateProvider {
    pub fn get(&self, random: &mut RandomGenerator) -> &'static BlockState {
        Pool::get(&self.entries, random).unwrap().get_state()
    }
}

pub struct SimpleStateProvider {
    pub state: BlockStateCodec,
}

impl SimpleStateProvider {
    pub fn get(&self, _pos: BlockPos) -> &'static BlockState {
        self.state.get_state()
    }
}

pub struct NoiseBlockStateProviderBase {
    pub seed: i64,
    pub noise: DoublePerlinNoiseParametersCodec,
    pub scale: f32,
}

impl NoiseBlockStateProviderBase {
    pub fn get_noise(&self, pos: BlockPos) -> f64 {
        let sampler = DoublePerlinNoiseSampler::new(
            &mut RandomGenerator::Legacy(LegacyRand::from_seed(self.seed as u64)),
            self.noise.first_octave,
            &self.noise.amplitudes,
            false,
        );
        sampler.sample(
            pos.0.x as f64 * self.scale as f64,
            pos.0.y as f64 * self.scale as f64,
            pos.0.z as f64 * self.scale as f64,
        )
    }
}

pub struct NoiseBlockStateProvider {
    pub base: NoiseBlockStateProviderBase,
    pub states: Vec<BlockStateCodec>,
}

impl NoiseBlockStateProvider {
    pub fn get(&self, pos: BlockPos) -> &'static BlockState {
        let value = self.base.get_noise(pos);
        self.get_state_by_value(&self.states, value).get_state()
    }

    fn get_state_by_value<'a>(
        &self,
        states: &'a [BlockStateCodec],
        value: f64,
    ) -> &'a BlockStateCodec {
        let val = f64::midpoint(1.0, value).clamp(0.0, 0.9999);
        &states[(val * states.len() as f64) as usize]
    }

    fn get_state_by_value_2<'a>(
        &self,
        states: &'a [&BlockStateCodec],
        value: f64,
    ) -> &'a BlockStateCodec {
        let val = f64::midpoint(1.0, value).clamp(0.0, 0.9999);
        states[(val * states.len() as f64) as usize]
    }
}

pub struct NoiseThresholdBlockStateProvider {
    pub base: NoiseBlockStateProviderBase,
    pub threshold: f32,
    pub high_chance: f32,
    pub default_state: BlockStateCodec,
    pub low_states: Vec<BlockStateCodec>,
    pub high_states: Vec<BlockStateCodec>,
}

impl NoiseThresholdBlockStateProvider {
    pub fn get(&self, random: &mut RandomGenerator, pos: BlockPos) -> &'static BlockState {
        let value = self.base.get_noise(pos);
        if value < self.threshold as f64 {
            return self.low_states[random.next_bounded_i32(self.low_states.len() as i32) as usize]
                .get_state();
        }
        if random.next_f32() < self.high_chance {
            return self.high_states
                [random.next_bounded_i32(self.high_states.len() as i32) as usize]
                .get_state();
        }
        self.default_state.get_state()
    }
}
