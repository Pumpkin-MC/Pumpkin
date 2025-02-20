use pumpkin_util::random::RandomGenerator;

use crate::{
    generation::{
        noise::{
            clamped_lerp,
            perlin::{DoublePerlinNoiseSampler, OctavePerlinNoiseSampler},
        },
        noise_router::{
            chunk_density_function::ChunkNoiseFunctionSampleOptions,
            chunk_noise_router::{
                ChunkNoiseFunctionComponent, StaticChunkNoiseFunctionComponentImpl,
            },
        },
    },
    noise_router::density_function_ast::{
        InterpolatedNoiseSamplerData, NoiseData, ShiftedNoiseData,
    },
};

use super::{
    NoiseFunctionComponentRange, NoisePos, StaticIndependentChunkNoiseFunctionComponentImpl,
};

#[derive(Clone)]
pub struct Noise {
    pub sampler: DoublePerlinNoiseSampler,
    pub xz_scale: f64,
    pub y_scale: f64,
}

impl Noise {
    pub fn new(sampler: DoublePerlinNoiseSampler, data: &NoiseData) -> Self {
        Self {
            sampler,
            xz_scale: data.xz_scale.0,
            y_scale: data.y_scale.0,
        }
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
            pos.x() as f64 * self.xz_scale,
            pos.y() as f64 * self.y_scale,
            pos.z() as f64 * self.xz_scale,
        )
    }
}

#[inline]
fn shift_sample_3d(sampler: &DoublePerlinNoiseSampler, x: f64, y: f64, z: f64) -> f64 {
    sampler.sample(x * 0.25f64, y * 0.25f64, z * 0.25f64) * 4f64
}

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone)]
pub struct ShiftedNoise {
    pub input_x_index: usize,
    pub input_y_index: usize,
    pub input_z_index: usize,
    pub sampler: DoublePerlinNoiseSampler,
    pub xz_scale: f64,
    pub y_scale: f64,
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
        let translated_x = pos.x() as f64 * self.xz_scale
            + ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=self.input_x_index],
                pos,
                sample_options,
            );
        let translated_y = pos.y() as f64 * self.y_scale
            + ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=self.input_y_index],
                pos,
                sample_options,
            );
        let translated_z = pos.z() as f64 * self.xz_scale
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
        data: &ShiftedNoiseData,
    ) -> Self {
        Self {
            input_x_index,
            input_y_index,
            input_z_index,
            sampler,
            xz_scale: data.xz_scale.0,
            y_scale: data.y_scale.0,
        }
    }
}

#[derive(Clone)]
pub struct InterpolatedNoiseSampler {
    pub lower_noise: Box<OctavePerlinNoiseSampler>,
    pub upper_noise: Box<OctavePerlinNoiseSampler>,
    pub noise: Box<OctavePerlinNoiseSampler>,
    pub scaled_xz_scale: f64,
    pub scaled_y_scale: f64,
    pub xz_factor: f64,
    pub y_factor: f64,
    pub smear_scale_multiplier: f64,
    pub max_value: f64,
}

impl InterpolatedNoiseSampler {
    pub fn new(data: &InterpolatedNoiseSamplerData, random: &mut RandomGenerator) -> Self {
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

        Self {
            lower_noise,
            upper_noise,
            noise,
            scaled_xz_scale: data.scaled_xz_scale.0,
            scaled_y_scale: data.scaled_y_scale.0,
            xz_factor: data.xz_factor.0,
            y_factor: data.y_factor.0,
            smear_scale_multiplier: data.smear_scale_multiplier.0,
            max_value: data.max_value.0,
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
        let d = pos.x() as f64 * self.scaled_xz_scale;
        let e = pos.y() as f64 * self.scaled_y_scale;
        let f = pos.z() as f64 * self.scaled_xz_scale;

        let g = d / self.xz_factor;
        let h = e / self.y_factor;
        let i = f / self.xz_factor;

        let j = self.scaled_y_scale * self.smear_scale_multiplier;
        let k = j / self.y_factor;

        let mut n = 0f64;
        let mut o = 1f64;

        for p in 0..8 {
            let sampler = self.noise.get_octave(p);
            if let Some(sampler) = sampler {
                n += sampler.sample_no_fade(
                    OctavePerlinNoiseSampler::maintain_precision(g * o),
                    OctavePerlinNoiseSampler::maintain_precision(h * o),
                    OctavePerlinNoiseSampler::maintain_precision(i * o),
                    k * o,
                    h * o,
                ) / o;
            }

            o /= 2f64;
        }

        let q = (n / 10f64 + 1f64) / 2f64;
        let bl2 = q >= 1f64;
        let bl3 = q <= 0f64;
        let mut o = 1f64;
        let mut l = 0f64;
        let mut m = 0f64;

        for r in 0..16 {
            let s = OctavePerlinNoiseSampler::maintain_precision(d * o);
            let t = OctavePerlinNoiseSampler::maintain_precision(e * o);
            let u = OctavePerlinNoiseSampler::maintain_precision(f * o);
            let v = j * o;

            if !bl2 {
                let sampler = self.lower_noise.get_octave(r);
                if let Some(sampler) = sampler {
                    l += sampler.sample_no_fade(s, t, u, v, e * o) / o;
                }
            }

            if !bl3 {
                let sampler = self.upper_noise.get_octave(r);
                if let Some(sampler) = sampler {
                    m += sampler.sample_no_fade(s, t, u, v, e * o) / o;
                }
            }

            o /= 2f64;
        }

        clamped_lerp(l / 512f64, m / 512f64, q) / 128f64
    }
}
