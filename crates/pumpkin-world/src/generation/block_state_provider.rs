use pumpkin_data::{Block, BlockState};
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

use super::noise::perlin::DoublePerlinNoiseSampler;
use crate::generation::block_predicate::BlockPredicate;
use crate::generation::proto_chunk::GenerationCache;
use crate::world::WorldPortalExt;

pub enum BlockStateProvider {
    Simple(SimpleStateProvider),
    Weighted(WeightedBlockStateProvider),
    NoiseThreshold(NoiseThresholdBlockStateProvider),
    NoiseProvider(NoiseBlockStateProvider),
    DualNoise(DualNoiseBlockStateProvider),
    Pillar(PillarBlockStateProvider),
    RandomizedInt(RandomizedIntBlockStateProvider),
    Rule(RuleBasedBlockStateProvider),
}

impl BlockStateProvider {
    pub fn get<T: GenerationCache>(
        &self,
        random: &mut RandomGenerator,
        pos: BlockPos,
        chunk: &T,
        block_registry: &dyn WorldPortalExt,
    ) -> &'static BlockState {
        match self {
            Self::NoiseThreshold(provider) => provider.get(random, pos),
            Self::NoiseProvider(provider) => provider.get(pos),
            Self::Simple(provider) => provider.get(pos),
            Self::Weighted(provider) => provider.get(random),
            Self::DualNoise(provider) => provider.get(pos),
            Self::Pillar(provider) => provider.get(random),
            Self::RandomizedInt(provider) => provider.get(random, pos, chunk, block_registry),
            Self::Rule(provider) => provider.get(block_registry, chunk, random, pos),
        }
    }

    pub fn get_with_context<T: GenerationCache>(
        &self,
        block_registry: &dyn WorldPortalExt,
        chunk: &T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> &'static BlockState {
        match self {
            Self::Rule(provider) => provider.get(block_registry, chunk, random, pos),
            _ => self.get(random, pos, chunk, block_registry),
        }
    }

    pub fn get_optional<T: GenerationCache>(
        &self,
        block_registry: &dyn WorldPortalExt,
        chunk: &T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> Option<&'static BlockState> {
        match self {
            Self::Rule(provider) => provider.get_optional(block_registry, chunk, random, pos),
            _ => Some(self.get(random, pos, chunk, block_registry)),
        }
    }
}

pub struct RuleBasedBlockStateProvider {
    pub fallback: Option<Box<BlockStateProvider>>,
    pub rules: Vec<BlockStateRule>,
}

impl RuleBasedBlockStateProvider {
    pub fn get<T: GenerationCache>(
        &self,
        block_registry: &dyn WorldPortalExt,
        chunk: &T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> &'static BlockState {
        if let Some(optional) = self.get_optional(block_registry, chunk, random, pos) {
            return optional;
        }
        GenerationCache::get_block_state(chunk, &pos.0).to_state()
    }
    pub fn get_optional<T: GenerationCache>(
        &self,
        block_registry: &dyn WorldPortalExt,
        chunk: &T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> Option<&'static BlockState> {
        for rule in &self.rules {
            if rule.if_true.test(block_registry, chunk, &pos) {
                return Some(
                    rule.then
                        .get_with_context(block_registry, chunk, random, pos),
                );
            }
        }
        self.fallback
            .as_ref()
            .map(|f| f.get(random, pos, chunk, block_registry))
    }
}

pub struct BlockStateRule {
    pub if_true: BlockPredicate,
    pub then: BlockStateProvider,
}

pub struct RandomizedIntBlockStateProvider {
    pub source: Box<BlockStateProvider>,
    pub property: String,
    pub values: IntProvider,
}

impl RandomizedIntBlockStateProvider {
    // net.minecraft.world.level.levelgen.feature.stateproviders.RandomizedIntStateProvider#getState:
    // samples `values` and writes it into the named integer property of the state returned by
    // `source`, falling back to the unmodified state if the property doesn't exist on it.
    pub fn get<T: GenerationCache>(
        &self,
        random: &mut RandomGenerator,
        pos: BlockPos,
        chunk: &T,
        block_registry: &dyn WorldPortalExt,
    ) -> &'static BlockState {
        let source_state = self.source.get(random, pos, chunk, block_registry);
        let value = self.values.get(random);

        let block = Block::from_state_id(source_state.id);
        let Some(props_source) = block.properties(source_state.id) else {
            return source_state;
        };
        let props = props_source.to_props();
        if !props.iter().any(|(key, _)| *key == self.property) {
            return source_state;
        }

        let value_str = value.to_string();
        let new_props: Vec<(&str, &str)> = props
            .iter()
            .map(|(key, value)| {
                if *key == self.property {
                    (*key, value_str.as_str())
                } else {
                    (*key, *value)
                }
            })
            .collect();

        let new_state_id = block.from_properties(&new_props).to_state_id(block);
        BlockState::from_id(new_state_id)
    }
}

pub struct PillarBlockStateProvider {
    pub state: &'static BlockState,
}

impl PillarBlockStateProvider {
    // net.minecraft.world.level.levelgen.feature.stateproviders.RotatedBlockProvider#getState:
    // picks a uniformly random Direction.Axis (X, Y, Z in declaration order) via
    // Util.getRandom, then sets it on the block's `axis` property.
    pub fn get(&self, random: &mut RandomGenerator) -> &'static BlockState {
        let block = Block::from_state_id(self.state.id);
        let Some(props_source) = block.properties(self.state.id) else {
            return self.state;
        };
        let mut props = props_source.to_props();
        let Some(idx) = props.iter().position(|(key, _)| *key == "axis") else {
            return self.state;
        };

        let axis_str = match random.next_bounded_i32(3) {
            0 => "x",
            1 => "y",
            _ => "z",
        };
        props[idx] = ("axis", axis_str);

        let new_state_id = block.from_properties(&props).to_state_id(block);
        BlockState::from_id(new_state_id)
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
            self.slow_noise.amplitude,
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
            list.push(NoiseBlockStateProvider::get_state_by_value(
                &self.base.states,
                value,
            ));
        }
        let value = self.base.base.get_noise(pos);
        NoiseBlockStateProvider::get_state_by_value(&list, value)
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
    pub entries: Vec<Weighted<&'static BlockState>>,
}

impl WeightedBlockStateProvider {
    pub fn get(&self, random: &mut RandomGenerator) -> &'static BlockState {
        Pool::get(&self.entries, random).unwrap()
    }
}

pub struct SimpleStateProvider {
    pub state: &'static BlockState,
}

impl SimpleStateProvider {
    pub const fn get(&self, _pos: BlockPos) -> &'static BlockState {
        self.state
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
            self.noise.amplitude,
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
    pub states: Vec<&'static BlockState>,
}

impl NoiseBlockStateProvider {
    pub fn get(&self, pos: BlockPos) -> &'static BlockState {
        let value = self.base.get_noise(pos);
        Self::get_state_by_value(&self.states, value)
    }

    fn get_state_by_value(states: &[&'static BlockState], value: f64) -> &'static BlockState {
        let val = f64::midpoint(1.0, value).clamp(0.0, 0.9999);
        states[(val * states.len() as f64) as usize]
    }
}

pub struct NoiseThresholdBlockStateProvider {
    pub base: NoiseBlockStateProviderBase,
    pub threshold: f32,
    pub high_chance: f32,
    pub default_state: &'static BlockState,
    pub low_states: Vec<&'static BlockState>,
    pub high_states: Vec<&'static BlockState>,
}

impl NoiseThresholdBlockStateProvider {
    pub fn get(&self, random: &mut RandomGenerator, pos: BlockPos) -> &'static BlockState {
        let value = self.base.get_noise(pos);
        if value < self.threshold as f64 {
            return self.low_states[random.next_bounded_i32(self.low_states.len() as i32) as usize];
        }
        if random.next_f32() < self.high_chance {
            return self.high_states
                [random.next_bounded_i32(self.high_states.len() as i32) as usize];
        }
        self.default_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProtoChunk;
    use crate::generation::generator::{GeneratorInit, VanillaGenerator, WorldGenerator};
    use crate::world::BlockAccessor;
    use pumpkin_data::chunk::Biome;
    use pumpkin_data::dimension::Dimension;
    use pumpkin_data::{Mirror, Rotation};
    use pumpkin_util::random::legacy_rand::LegacyRand;
    use pumpkin_util::world_seed::Seed;

    struct NoopWorldPortal;
    impl WorldPortalExt for NoopWorldPortal {
        fn can_place_at(
            &self,
            _block: &Block,
            _state: &BlockState,
            _block_accessor: &dyn BlockAccessor,
            _block_pos: &BlockPos,
        ) -> bool {
            true
        }

        fn mirror(
            &self,
            block: &Block,
            state_id: pumpkin_data::BlockStateId,
            mirror: Mirror,
        ) -> &'static BlockState {
            block.mirror(state_id, mirror)
        }

        fn rotate(
            &self,
            block: &Block,
            state_id: pumpkin_data::BlockStateId,
            rotation: Rotation,
        ) -> &'static BlockState {
            block.rotate(state_id, rotation)
        }

        fn spawn_mobs_for_chunk_generation(
            &self,
            _cache: &mut dyn GenerationCache,
            _biome: &'static Biome,
            _chunk_x: i32,
            _chunk_z: i32,
        ) {
        }
    }

    #[test]
    fn randomized_int_provider_sets_sampled_age() {
        let world_gen = WorldGenerator::Noise(Box::new(VanillaGenerator::new(
            Seed(1),
            Dimension::OVERWORLD,
        )));
        let chunk = ProtoChunk::new(0, 0, &world_gen);
        let registry = NoopWorldPortal;
        let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(1));

        let provider = RandomizedIntBlockStateProvider {
            source: Box::new(BlockStateProvider::Simple(SimpleStateProvider {
                state: Block::CAVE_VINES.default_state,
            })),
            property: "age".to_string(),
            values: IntProvider::Constant(7),
        };

        let pos = BlockPos::new(0, 0, 0);
        let result = provider.get(&mut random, pos, &chunk, &registry);

        let block = Block::from_state_id(result.id);
        let props = block.properties(result.id).unwrap().to_props();
        let age = props
            .iter()
            .find(|(key, _)| *key == "age")
            .map(|(_, value)| *value);
        assert_eq!(age, Some("7"));
    }

    #[test]
    fn pillar_provider_randomizes_axis() {
        let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(1));
        let provider = PillarBlockStateProvider {
            state: Block::BASALT.default_state,
        };

        let mut saw_axis = std::collections::HashSet::new();
        for _ in 0..64 {
            let state = provider.get(&mut random);
            let block = Block::from_state_id(state.id);
            let props = block.properties(state.id).unwrap().to_props();
            let axis = props
                .iter()
                .find(|(key, _)| *key == "axis")
                .map(|(_, value)| (*value).to_string());
            if let Some(axis) = axis {
                saw_axis.insert(axis);
            }
        }

        assert!(
            saw_axis.len() > 1,
            "expected multiple distinct axes to be sampled, got {saw_axis:?}"
        );
    }
}
