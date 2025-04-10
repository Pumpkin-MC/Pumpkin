use std::array;

use pumpkin_data::noise_router::{InterpolatedNoiseSamplerData, NoiseData, ShiftedNoiseData};
use pumpkin_util::{
    math::clamped_lerp, noise::perlin::OctavePerlinNoiseSampler, random::RandomGenerator,
};

use crate::generation::{
    noise::perlin::DoublePerlinNoiseSampler,
    noise_router::{
        chunk_density_function::ChunkNoiseFunctionSampleOptions,
        chunk_noise_router::{ChunkNoiseFunctionComponent, StaticChunkNoiseFunctionComponentImpl},
    },
};

use super::{
    NoiseFunctionComponentRange, NoisePos, StaticIndependentChunkNoiseFunctionComponentImpl,
};

pub struct Noise {
    sampler: DoublePerlinNoiseSampler,
    data: &'static NoiseData,
}

impl Noise {
    pub fn new(sampler: DoublePerlinNoiseSampler, data: &'static NoiseData) -> Self {
        Self { sampler, data }
    }
}

impl NoiseFunctionComponentRange for Noise {
    #[inline]
    fn min(&self) -> f64 {
        -self.max()
    }

    #[inline]
    fn max(&self) -> f64 {
        self.sampler.max_value()
    }
}

impl StaticIndependentChunkNoiseFunctionComponentImpl for Noise {
    fn sample(&self, pos: &impl NoisePos) -> f64 {
        self.sampler.sample(
            pos.x() as f64 * self.data.xz_scale,
            pos.y() as f64 * self.data.y_scale,
            pos.z() as f64 * self.data.xz_scale,
        )
    }
}

#[inline]
fn shift_sample_3d(sampler: &DoublePerlinNoiseSampler, x: f64, y: f64, z: f64) -> f64 {
    sampler.sample(x * 0.25f64, y * 0.25f64, z * 0.25f64) * 4f64
}

pub struct ShiftA {
    sampler: DoublePerlinNoiseSampler,
}

impl ShiftA {
    pub fn new(sampler: DoublePerlinNoiseSampler) -> Self {
        Self { sampler }
    }
}

impl NoiseFunctionComponentRange for ShiftA {
    #[inline]
    fn min(&self) -> f64 {
        -self.max()
    }

    #[inline]
    fn max(&self) -> f64 {
        self.sampler.max_value() * 4.0
    }
}

impl StaticIndependentChunkNoiseFunctionComponentImpl for ShiftA {
    fn sample(&self, pos: &impl NoisePos) -> f64 {
        shift_sample_3d(&self.sampler, pos.x() as f64, 0.0, pos.z() as f64)
    }
}

pub struct ShiftB {
    sampler: DoublePerlinNoiseSampler,
}

impl ShiftB {
    pub fn new(sampler: DoublePerlinNoiseSampler) -> Self {
        Self { sampler }
    }
}

impl NoiseFunctionComponentRange for ShiftB {
    #[inline]
    fn min(&self) -> f64 {
        -self.max()
    }

    #[inline]
    fn max(&self) -> f64 {
        self.sampler.max_value() * 4.0
    }
}

impl StaticIndependentChunkNoiseFunctionComponentImpl for ShiftB {
    fn sample(&self, pos: &impl NoisePos) -> f64 {
        shift_sample_3d(&self.sampler, pos.z() as f64, pos.x() as f64, 0.0)
    }
}

pub struct ShiftedNoise {
    input_x_index: usize,
    input_y_index: usize,
    input_z_index: usize,
    sampler: DoublePerlinNoiseSampler,
    data: &'static ShiftedNoiseData,
}

impl NoiseFunctionComponentRange for ShiftedNoise {
    #[inline]
    fn min(&self) -> f64 {
        -self.max()
    }

    #[inline]
    fn max(&self) -> f64 {
        self.sampler.max_value()
    }
}

impl StaticChunkNoiseFunctionComponentImpl for ShiftedNoise {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &impl NoisePos,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        let translated_x = pos.x() as f64 * self.data.xz_scale
            + ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=self.input_x_index],
                pos,
                sample_options,
            );
        let translated_y = pos.y() as f64 * self.data.y_scale
            + ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=self.input_y_index],
                pos,
                sample_options,
            );
        let translated_z = pos.z() as f64 * self.data.xz_scale
            + ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=self.input_z_index],
                pos,
                sample_options,
            );

        self.sampler
            .sample(translated_x, translated_y, translated_z)
    }
}

impl ShiftedNoise {
    pub fn new(
        input_x_index: usize,
        input_y_index: usize,
        input_z_index: usize,
        sampler: DoublePerlinNoiseSampler,
        data: &'static ShiftedNoiseData,
    ) -> Self {
        Self {
            input_x_index,
            input_y_index,
            input_z_index,
            sampler,
            data,
        }
    }
}

pub struct InterpolatedNoiseSampler {
    lower_noise: Box<OctavePerlinNoiseSampler>,
    upper_noise: Box<OctavePerlinNoiseSampler>,
    noise: Box<OctavePerlinNoiseSampler>,
    data: &'static InterpolatedNoiseSamplerData,
    fractions: [f64; 16],
    max_value: f64,
}

impl InterpolatedNoiseSampler {
    pub fn new(data: &'static InterpolatedNoiseSamplerData, random: &mut RandomGenerator) -> Self {
        let big_start = -15;
        let big_amplitudes = [1.0; 16];

        let little_start = -7;
        let little_amplitudes = [1.0; 8];

        let lower_noise = Box::new(OctavePerlinNoiseSampler::new(
            random,
            big_start,
            &big_amplitudes,
            true,
        ));
        let upper_noise = Box::new(OctavePerlinNoiseSampler::new(
            random,
            big_start,
            &big_amplitudes,
            true,
        ));
        let noise = Box::new(OctavePerlinNoiseSampler::new(
            random,
            little_start,
            &little_amplitudes,
            true,
        ));

        let max_value = lower_noise.get_total_amplitude(data.scaled_y_scale + 2.0);

        let fractions = array::from_fn(|index| {
            let mut o = 1.0;
            for _ in 0..index {
                o /= 2.0;
            }
            o
        });

        Self {
            lower_noise,
            upper_noise,
            noise,
            data,
            fractions,
            max_value,
        }
    }
}

impl NoiseFunctionComponentRange for InterpolatedNoiseSampler {
    #[inline]
    fn min(&self) -> f64 {
        -self.max()
    }

    #[inline]
    fn max(&self) -> f64 {
        self.max_value
    }
}

impl StaticIndependentChunkNoiseFunctionComponentImpl for InterpolatedNoiseSampler {
    fn sample(&self, pos: &impl NoisePos) -> f64 {
        let d = pos.x() as f64 * self.data.scaled_xz_scale;
        let e = pos.y() as f64 * self.data.scaled_y_scale;
        let f = pos.z() as f64 * self.data.scaled_xz_scale;

        let g = d / self.data.xz_factor;
        let h = e / self.data.y_factor;
        let i = f / self.data.xz_factor;

        let j = self.data.scaled_y_scale * self.data.smear_scale_multiplier;
        let k = j / self.data.y_factor;

        // It's ok the the fractions are more than this; zip will cut it short
        let n: f64 = self
            .noise
            .samplers
            .iter()
            .rev()
            .zip(self.fractions)
            .map(|(data, fraction)| {
                let mapped_x = OctavePerlinNoiseSampler::maintain_precision(g * fraction);
                let mapped_y = OctavePerlinNoiseSampler::maintain_precision(h * fraction);
                let mapped_z = OctavePerlinNoiseSampler::maintain_precision(i * fraction);

                data.sampler.sample_no_fade(
                    mapped_x,
                    mapped_y,
                    mapped_z,
                    k * fraction,
                    h * fraction,
                ) / fraction
            })
            .sum();

        let q = (n / 10f64 + 1f64) / 2f64;
        let bl2 = q >= 1f64;
        let bl3 = q <= 0f64;

        let l = if !bl2 {
            self.lower_noise
                .samplers
                .iter()
                .rev()
                .zip(self.fractions)
                .map(|(data, fraction)| {
                    let mapped_x = OctavePerlinNoiseSampler::maintain_precision(d * fraction);
                    let mapped_y = OctavePerlinNoiseSampler::maintain_precision(e * fraction);
                    let mapped_z = OctavePerlinNoiseSampler::maintain_precision(f * fraction);

                    data.sampler.sample_no_fade(
                        mapped_x,
                        mapped_y,
                        mapped_z,
                        j * fraction,
                        e * fraction,
                    ) / fraction
                })
                .sum()
        } else {
            0.0
        };

        let m = if !bl3 {
            self.upper_noise
                .samplers
                .iter()
                .rev()
                .zip(self.fractions)
                .map(|(data, fraction)| {
                    let mapped_x = OctavePerlinNoiseSampler::maintain_precision(d * fraction);
                    let mapped_y = OctavePerlinNoiseSampler::maintain_precision(e * fraction);
                    let mapped_z = OctavePerlinNoiseSampler::maintain_precision(f * fraction);

                    data.sampler.sample_no_fade(
                        mapped_x,
                        mapped_y,
                        mapped_z,
                        j * fraction,
                        e * fraction,
                    ) / fraction
                })
                .sum()
        } else {
            0.0
        };

        clamped_lerp(l / 512f64, m / 512f64, q) / 128f64
    }
}
