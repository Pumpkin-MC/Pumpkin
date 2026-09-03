use std::array;

use pumpkin_data::noise_router::{InterpolatedNoiseSamplerData, NoiseData, ShiftedNoiseData};
use pumpkin_util::{
    math::{clamped_lerp, vector3::Vector3},
    noise::perlin::OctavePerlinNoiseSampler,
    random::RandomImpl,
};

use crate::generation::{
    noise::perlin::DoublePerlinNoiseSampler,
    noise::router::{
        chunk_noise_router::{ChunkNoiseFunctionComponent, StaticChunkNoiseFunctionComponentImpl},
        density_volume::{DensityBuffer, DensityVolume},
    },
};

use super::{NoiseFunctionComponentRange, StaticIndependentChunkNoiseFunctionComponentImpl};

pub struct Noise {
    sampler: DoublePerlinNoiseSampler,
    data: &'static NoiseData,
}

impl Noise {
    pub const fn new(sampler: DoublePerlinNoiseSampler, data: &'static NoiseData) -> Self {
        Self { sampler, data }
    }
}

impl NoiseFunctionComponentRange for Noise {
    #[inline]
    fn min(&self) -> f32 {
        -self.max()
    }

    #[inline]
    fn max(&self) -> f32 {
        self.sampler.max_value()
    }
}

impl StaticIndependentChunkNoiseFunctionComponentImpl for Noise {
    fn sample(&self, pos: &Vector3<i32>) -> f32 {
        self.sampler.sample(
            pos.x as f32 * self.data.xz_scale,
            pos.y as f32 * self.data.y_scale,
            pos.z as f32 * self.data.xz_scale,
        )
    }

    fn sample_volume(&self, buffer: &mut [f32], volume: &DensityVolume) {
        let mut index = 0;
        for z in 0..volume.size_z {
            let noise_z = volume.block_z(z) as f32 * self.data.xz_scale;
            for x in 0..volume.size_x {
                let noise_x = volume.block_x(x) as f32 * self.data.xz_scale;
                for y in 0..volume.size_y {
                    let noise_y = volume.block_y(y) as f32 * self.data.y_scale;
                    buffer[index] = self.sampler.sample(noise_x, noise_y, noise_z);
                    index += 1;
                }
            }
        }
    }
}

#[inline]
fn shift_sample_3d(sampler: &DoublePerlinNoiseSampler, x: f32, y: f32, z: f32) -> f32 {
    sampler.sample(x * 0.25f32, y * 0.25f32, z * 0.25f32) * 4f32
}

pub struct ShiftA {
    sampler: DoublePerlinNoiseSampler,
}

impl ShiftA {
    pub const fn new(sampler: DoublePerlinNoiseSampler) -> Self {
        Self { sampler }
    }
}

impl NoiseFunctionComponentRange for ShiftA {
    #[inline]
    fn min(&self) -> f32 {
        -self.max()
    }

    #[inline]
    fn max(&self) -> f32 {
        self.sampler.max_value() * 4.0
    }
}

impl StaticIndependentChunkNoiseFunctionComponentImpl for ShiftA {
    fn sample(&self, pos: &Vector3<i32>) -> f32 {
        shift_sample_3d(&self.sampler, pos.x as f32, 0.0, pos.z as f32)
    }

    fn sample_volume(&self, buffer: &mut [f32], volume: &DensityVolume) {
        for z in 0..volume.size_z {
            let block_z = volume.block_z(z);
            for x in 0..volume.size_x {
                let value =
                    shift_sample_3d(&self.sampler, volume.block_x(x) as f32, 0.0, block_z as f32);
                let index = volume.index_unchecked(x, 0, z);
                buffer[index..index + volume.size_y].fill(value);
            }
        }
    }
}

pub struct ShiftB {
    sampler: DoublePerlinNoiseSampler,
}

impl ShiftB {
    pub const fn new(sampler: DoublePerlinNoiseSampler) -> Self {
        Self { sampler }
    }
}

impl NoiseFunctionComponentRange for ShiftB {
    #[inline]
    fn min(&self) -> f32 {
        -self.max()
    }

    #[inline]
    fn max(&self) -> f32 {
        self.sampler.max_value() * 4.0
    }
}

impl StaticIndependentChunkNoiseFunctionComponentImpl for ShiftB {
    fn sample(&self, pos: &Vector3<i32>) -> f32 {
        shift_sample_3d(&self.sampler, pos.z as f32, pos.x as f32, 0.0)
    }

    fn sample_volume(&self, buffer: &mut [f32], volume: &DensityVolume) {
        for z in 0..volume.size_z {
            let block_z = volume.block_z(z);
            for x in 0..volume.size_x {
                let value =
                    shift_sample_3d(&self.sampler, block_z as f32, volume.block_x(x) as f32, 0.0);
                let index = volume.index_unchecked(x, 0, z);
                buffer[index..index + volume.size_y].fill(value);
            }
        }
    }
}

pub struct ShiftedNoise {
    pub(crate) input_x_index: usize,
    pub(crate) input_y_index: usize,
    pub(crate) input_z_index: usize,
    sampler: DoublePerlinNoiseSampler,
    data: &'static ShiftedNoiseData,
}

impl ShiftedNoise {
    #[inline]
    pub fn sample_with_shifts(
        &self,
        pos: &Vector3<i32>,
        x_shift: f32,
        y_shift: f32,
        z_shift: f32,
    ) -> f32 {
        let translated_x = pos.x as f32 * self.data.xz_scale + x_shift;
        let translated_y = pos.y as f32 * self.data.y_scale + y_shift;
        let translated_z = pos.z as f32 * self.data.xz_scale + z_shift;
        self.sampler
            .sample(translated_x, translated_y, translated_z)
    }
}

impl NoiseFunctionComponentRange for ShiftedNoise {
    #[inline]
    fn min(&self) -> f32 {
        -self.max()
    }

    #[inline]
    fn max(&self) -> f32 {
        self.sampler.max_value()
    }
}

impl StaticChunkNoiseFunctionComponentImpl for ShiftedNoise {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
    ) -> f32 {
        let x_shift = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input_x_index],
            pos,
        );
        let y_shift = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input_y_index],
            pos,
        );
        let z_shift = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input_z_index],
            pos,
        );

        self.sample_with_shifts(pos, x_shift, y_shift, z_shift)
    }

    fn sample_volume(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        buffer: &mut [f32],
        volume: &DensityVolume,
    ) {
        ChunkNoiseFunctionComponent::sample_volume_from_stack(
            &mut component_stack[..=self.input_x_index],
            buffer,
            volume,
        );
        let mut y_shifts = DensityBuffer::acquire(volume);
        ChunkNoiseFunctionComponent::sample_volume_from_stack(
            &mut component_stack[..=self.input_y_index],
            &mut y_shifts,
            volume,
        );
        let mut z_shifts = DensityBuffer::acquire(volume);
        ChunkNoiseFunctionComponent::sample_volume_from_stack(
            &mut component_stack[..=self.input_z_index],
            &mut z_shifts,
            volume,
        );
        let mut index = 0;
        for z in 0..volume.size_z {
            let block_z = volume.block_z(z);
            for x in 0..volume.size_x {
                let block_x = volume.block_x(x);
                for y in 0..volume.size_y {
                    let pos = Vector3::new(block_x, volume.block_y(y), block_z);
                    buffer[index] = self.sample_with_shifts(
                        &pos,
                        buffer[index],
                        y_shifts[index],
                        z_shifts[index],
                    );
                    index += 1;
                }
            }
        }
    }
}

impl ShiftedNoise {
    pub const fn new(
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
    lower_noise: OctavePerlinNoiseSampler,
    upper_noise: OctavePerlinNoiseSampler,
    noise: OctavePerlinNoiseSampler,
    data: &'static InterpolatedNoiseSamplerData,
    fractions: [f64; 16],
    max_value: f32,
    y_multiplier: f64,
}

impl InterpolatedNoiseSampler {
    pub fn new(data: &'static InterpolatedNoiseSamplerData, random: &mut impl RandomImpl) -> Self {
        let big_start = -15;
        let big_amplitudes = [1.0; 16];

        let little_start = -7;
        let little_amplitudes = [1.0; 8];

        let lower_noise = OctavePerlinNoiseSampler::new(random, big_start, &big_amplitudes, true);
        let upper_noise = OctavePerlinNoiseSampler::new(random, big_start, &big_amplitudes, true);
        let noise = OctavePerlinNoiseSampler::new(random, little_start, &little_amplitudes, true);

        let y_multiplier = (data.scaled_y_scale * data.xz_factor / data.y_factor * 684.412) as f64;
        let max_value = lower_noise.get_total_amplitude(y_multiplier + 2.0) as f32;

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
            y_multiplier,
        }
    }
}

impl NoiseFunctionComponentRange for InterpolatedNoiseSampler {
    #[inline]
    fn min(&self) -> f32 {
        -self.max()
    }

    #[inline]
    fn max(&self) -> f32 {
        self.max_value
    }
}

impl StaticIndependentChunkNoiseFunctionComponentImpl for InterpolatedNoiseSampler {
    fn sample(&self, pos: &Vector3<i32>) -> f32 {
        let xz_multiplier = (self.data.scaled_xz_scale * 684.412) as f64;

        let d = pos.x as f64 * xz_multiplier;
        let e = pos.y as f64 * self.y_multiplier;
        let f = pos.z as f64 * xz_multiplier;

        let g = d / self.data.xz_factor as f64;
        let h = e / self.data.y_factor as f64;
        let i = f / self.data.xz_factor as f64;

        let j = self.y_multiplier * self.data.smear_scale_multiplier as f64;
        let k = j / self.data.y_factor as f64;

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

        let q = f64::midpoint(n / 10.0, 1.0);
        let bl2 = q >= 1.0;
        let bl3 = q <= 0.0;

        let l = if bl2 {
            0.0
        } else {
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
        };

        let m = if bl3 {
            0.0
        } else {
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
        };

        (clamped_lerp(l / 512.0, m / 512.0, q) / 128.0) as f32
    }
}
