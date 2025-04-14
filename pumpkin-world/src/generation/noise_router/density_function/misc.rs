use std::sync::Arc;

use pumpkin_data::noise_router::{
    ClampedYGradientData, RangeChoiceData, WeirdScaledData, WeirdScaledMapper,
};
use pumpkin_util::{
    math::clamped_map,
    noise::simplex::SimplexNoiseSampler,
    random::{RandomImpl, legacy_rand::LegacyRand},
};

use crate::generation::{
    noise::perlin::DoublePerlinNoiseSampler,
    noise_router::{
        chunk_density_function::ChunkNoiseFunctionSampleOptions,
        chunk_noise_router::{ChunkNoiseFunctionComponent, StaticChunkNoiseFunctionComponentImpl},
    },
};

use super::{
    IndexToNoisePos, NoiseFunctionComponentRange, NoisePos,
    StaticIndependentChunkNoiseFunctionComponentImpl,
};

#[derive(Clone)]
pub struct EndIsland {
    sampler: Arc<SimplexNoiseSampler>,
}

impl EndIsland {
    pub fn new(seed: u64) -> Self {
        let mut rand = LegacyRand::from_seed(seed);
        rand.skip(17292);
        Self {
            sampler: Arc::new(SimplexNoiseSampler::new(&mut rand)),
        }
    }

    fn sample_2d(sampler: &SimplexNoiseSampler, x: i32, z: i32) -> f32 {
        let i = x / 2;
        let j = z / 2;
        let k = x % 2;
        let l = z % 2;

        let f = ((x * x + z * z) as f32).sqrt().mul_add(-8f32, 100f32);
        let mut f = f.clamp(-100f32, 80f32);

        for m in -12..=12 {
            for n in -12..=12 {
                let o = (i + m) as i64;
                let p = (j + n) as i64;

                if (o * o + p * p) > 4096i64
                    && sampler.sample_2d(o as f64, p as f64) < -0.9f32 as f64
                {
                    let g =
                        (o as f32).abs().mul_add(3439f32, (p as f32).abs() * 147f32) % 13f32 + 9f32;
                    let h = (k - m * 2) as f32;
                    let q = (l - n * 2) as f32;
                    let r = h.hypot(q).mul_add(-g, 100f32);
                    let s = r.clamp(-100f32, 80f32);

                    f = f.max(s);
                }
            }
        }

        f
    }
}

// These values are hardcoded from java
impl NoiseFunctionComponentRange for EndIsland {
    #[inline]
    fn min(&self) -> f64 {
        -0.84375
    }

    #[inline]
    fn max(&self) -> f64 {
        0.5625
    }
}

impl StaticIndependentChunkNoiseFunctionComponentImpl for EndIsland {
    fn sample(&self, pos: &impl NoisePos) -> f64 {
        (Self::sample_2d(&self.sampler, pos.x() / 8, pos.z() / 8) as f64 - 8f64) / 128f64
    }
}

pub struct WeirdScaled {
    pub input_index: usize,
    pub sampler: DoublePerlinNoiseSampler,
    pub mapper: WeirdScaledMapper,
}

impl WeirdScaled {
    pub fn new(
        input_index: usize,
        sampler: DoublePerlinNoiseSampler,
        data: &WeirdScaledData,
    ) -> Self {
        Self {
            input_index,
            sampler,
            mapper: data.mapper,
        }
    }
}

impl StaticChunkNoiseFunctionComponentImpl for WeirdScaled {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &impl NoisePos,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        let input_density = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input_index],
            pos,
            sample_options,
        );
        let scaled_density = self.mapper.scale(input_density);
        scaled_density
            * self
                .sampler
                .sample(
                    pos.x() as f64 / scaled_density,
                    pos.y() as f64 / scaled_density,
                    pos.z() as f64 / scaled_density,
                )
                .abs()
    }

    fn fill(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        array: &mut [f64],
        mapper: &impl IndexToNoisePos,
        sample_options: &mut ChunkNoiseFunctionSampleOptions,
    ) {
        ChunkNoiseFunctionComponent::fill_from_stack(
            &mut component_stack[..=self.input_index],
            array,
            mapper,
            sample_options,
        );

        array.iter_mut().enumerate().for_each(|(index, value)| {
            let pos = mapper.at(index, Some(sample_options));
            let scaled_density = self.mapper.scale(*value);
            *value = scaled_density
                * self
                    .sampler
                    .sample(
                        pos.x() as f64 / scaled_density,
                        pos.y() as f64 / scaled_density,
                        pos.z() as f64 / scaled_density,
                    )
                    .abs();
        });
    }
}

impl NoiseFunctionComponentRange for WeirdScaled {
    #[inline]
    fn min(&self) -> f64 {
        -self.max()
    }

    #[inline]
    fn max(&self) -> f64 {
        self.sampler.max_value() * self.mapper.max_multiplier()
    }
}

#[derive(Clone)]
pub struct ClampedYGradient {
    data: &'static ClampedYGradientData,
}

impl ClampedYGradient {
    pub fn new(data: &'static ClampedYGradientData) -> Self {
        Self { data }
    }
}

impl NoiseFunctionComponentRange for ClampedYGradient {
    #[inline]
    fn min(&self) -> f64 {
        self.data.from_value.min(self.data.to_value)
    }

    #[inline]
    fn max(&self) -> f64 {
        self.data.from_value.max(self.data.to_value)
    }
}

impl StaticIndependentChunkNoiseFunctionComponentImpl for ClampedYGradient {
    fn sample(&self, pos: &impl NoisePos) -> f64 {
        clamped_map(
            pos.y() as f64,
            self.data.from_y,
            self.data.to_y,
            self.data.from_value,
            self.data.to_value,
        )
    }
}

#[derive(Clone)]
pub struct RangeChoice {
    input_index: usize,
    pub(crate) when_in_index: usize,
    pub(crate) when_out_index: usize,
    data: &'static RangeChoiceData,
    min_value: f64,
    max_value: f64,
}

impl RangeChoice {
    pub fn new(
        input_index: usize,
        when_in_index: usize,
        when_out_index: usize,
        min_value: f64,
        max_value: f64,
        data: &'static RangeChoiceData,
    ) -> Self {
        Self {
            input_index,
            when_in_index,
            when_out_index,
            min_value,
            max_value,
            data,
        }
    }
}

impl NoiseFunctionComponentRange for RangeChoice {
    #[inline]
    fn min(&self) -> f64 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f64 {
        self.max_value
    }
}

impl StaticChunkNoiseFunctionComponentImpl for RangeChoice {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &impl NoisePos,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        let input_sample = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input_index],
            pos,
            sample_options,
        );

        if self.data.min_inclusive <= input_sample && input_sample < self.data.max_exclusive {
            ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=self.when_in_index],
                pos,
                sample_options,
            )
        } else {
            ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=self.when_out_index],
                pos,
                sample_options,
            )
        }
    }

    fn fill(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        array: &mut [f64],
        mapper: &impl IndexToNoisePos,
        sample_options: &mut ChunkNoiseFunctionSampleOptions,
    ) {
        ChunkNoiseFunctionComponent::fill_from_stack(
            &mut component_stack[..=self.input_index],
            array,
            mapper,
            sample_options,
        );

        array.iter_mut().enumerate().for_each(|(index, value)| {
            let pos = mapper.at(index, Some(sample_options));
            *value = if self.data.min_inclusive <= *value && *value < self.data.max_exclusive {
                ChunkNoiseFunctionComponent::sample_from_stack(
                    &mut component_stack[..=self.when_in_index],
                    &pos,
                    sample_options,
                )
            } else {
                ChunkNoiseFunctionComponent::sample_from_stack(
                    &mut component_stack[..=self.when_out_index],
                    &pos,
                    sample_options,
                )
            };
        });
    }
}
