use pumpkin_data::noise_router::{InterpolatedNoiseSamplerData, NoiseData, ShiftedNoiseData};
use pumpkin_util::{
    math::{lerp, vector3::Vector3},
    noise::perlin::{Layer, Noise as _, NoiseStack, SmearedPerlinNoise},
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

const SHIFT_COORDINATE_FACTOR: f64 = 0.25;
const SHIFT_VALUE_FACTOR: f32 = 4.0;
const BLENDED_BASE_SCALE: f64 = 684.412;
const BLENDED_LIMIT_FACTOR: f64 = 0.999_984_74f32 as f64;
const BLENDED_MAIN_FACTOR: f64 = 12.75;
const BLENDED_LIMIT_FIRST_OCTAVE: i32 = -15;
const BLENDED_MAIN_FIRST_OCTAVE: i32 = -7;
const SMEARED_RANGE_BASE: f64 = 2.0;

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
            f64::from(pos.x) * self.data.xz_scale,
            f64::from(pos.y) * self.data.y_scale,
            f64::from(pos.z) * self.data.xz_scale,
        )
    }

    fn sample_volume(&self, buffer: &mut [f32], volume: &DensityVolume) {
        buffer.fill(0.0);
        self.sampler
            .add_to_volume(buffer, volume, self.data.xz_scale, self.data.y_scale, 1.0);
    }
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
        self.sampler.max_value() * SHIFT_VALUE_FACTOR
    }
}

impl StaticIndependentChunkNoiseFunctionComponentImpl for ShiftA {
    fn sample(&self, pos: &Vector3<i32>) -> f32 {
        self.sampler.sample(
            f64::from(pos.x) * SHIFT_COORDINATE_FACTOR,
            f64::from(pos.y) * 0.0,
            f64::from(pos.z) * SHIFT_COORDINATE_FACTOR,
        ) * SHIFT_VALUE_FACTOR
    }

    fn sample_volume(&self, buffer: &mut [f32], volume: &DensityVolume) {
        buffer.fill(0.0);
        self.sampler
            .add_to_volume(buffer, volume, SHIFT_COORDINATE_FACTOR, 0.0, 1.0);
        for value in buffer.iter_mut() {
            *value *= SHIFT_VALUE_FACTOR;
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
        self.sampler.max_value() * SHIFT_VALUE_FACTOR
    }
}

impl StaticIndependentChunkNoiseFunctionComponentImpl for ShiftB {
    fn sample(&self, pos: &Vector3<i32>) -> f32 {
        self.sampler.sample(
            f64::from(pos.z) * SHIFT_COORDINATE_FACTOR,
            f64::from(pos.x) * SHIFT_COORDINATE_FACTOR,
            0.0,
        ) * SHIFT_VALUE_FACTOR
    }

    fn sample_volume(&self, buffer: &mut [f32], volume: &DensityVolume) {
        let transposed = DensityVolume::new(
            volume.size_z,
            volume.size_x,
            1,
            volume.min_block_z,
            volume.min_block_x,
            0,
            volume.step_block_z,
            volume.step_block_x,
            1,
        );
        let mut columns = DensityBuffer::acquire(&transposed);
        columns.fill(0.0);
        self.sampler.add_to_volume(
            &mut columns,
            &transposed,
            SHIFT_COORDINATE_FACTOR,
            SHIFT_COORDINATE_FACTOR,
            SHIFT_VALUE_FACTOR,
        );
        for z in 0..volume.size_z {
            for x in 0..volume.size_x {
                let value = columns[transposed.index_unchecked(z, x, 0)];
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
        self.sampler.sample(
            f64::from(pos.x) * self.data.xz_scale + f64::from(x_shift),
            f64::from(pos.y) * self.data.y_scale + f64::from(y_shift),
            f64::from(pos.z) * self.data.xz_scale + f64::from(z_shift),
        )
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
            let base_noise_z = f64::from(volume.block_z(z)) * self.data.xz_scale;
            for x in 0..volume.size_x {
                let base_noise_x = f64::from(volume.block_x(x)) * self.data.xz_scale;
                for y in 0..volume.size_y {
                    let noise_x = base_noise_x + f64::from(buffer[index]);
                    let noise_y = f64::from(volume.block_y(y)) * self.data.y_scale
                        + f64::from(y_shifts[index]);
                    let noise_z = base_noise_z + f64::from(z_shifts[index]);
                    buffer[index] = self.sampler.sample(noise_x, noise_y, noise_z);
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

fn create_fbm(
    random: &mut impl RandomImpl,
    first_octave: i32,
    smear_scale_y: f64,
    value_factor: f64,
) -> NoiseStack<SmearedPerlinNoise> {
    let octaves = -first_octave + 1;
    let mut factor = 1.0;
    let mut value_factor = value_factor / (2f64.powi(octaves) - 1.0);
    let mut layers = Vec::with_capacity(octaves as usize);
    for _ in (0..octaves).rev() {
        layers.push(Layer {
            noise: SmearedPerlinNoise::new(random, smear_scale_y * factor),
            frequency: factor,
            amplitude: value_factor as f32,
        });
        factor /= 2.0;
        value_factor *= 2.0;
    }
    NoiseStack::new(layers)
}

fn fbm_range_max(first_octave: i32, smear_scale_y: f64, value_factor: f64) -> f32 {
    let octaves = -first_octave + 1;
    let mut factor = 1.0;
    let mut value_factor = value_factor / (2f64.powi(octaves) - 1.0);
    let mut range = 0.0f32;
    for _ in (0..octaves).rev() {
        let layer_range = ((smear_scale_y * factor).abs() + SMEARED_RANGE_BASE) as f32;
        range += layer_range * value_factor as f32;
        factor /= 2.0;
        value_factor *= 2.0;
    }
    range
}

pub struct InterpolatedNoiseSampler {
    min_limit_noise: NoiseStack<SmearedPerlinNoise>,
    max_limit_noise: NoiseStack<SmearedPerlinNoise>,
    main_noise: NoiseStack<SmearedPerlinNoise>,
    xz_multiplier: f64,
    y_multiplier: f64,
    main_xz_multiplier: f64,
    main_y_multiplier: f64,
    max_value: f32,
}

impl InterpolatedNoiseSampler {
    pub fn new(data: &'static InterpolatedNoiseSamplerData, random: &mut impl RandomImpl) -> Self {
        let xz_multiplier = BLENDED_BASE_SCALE * data.xz_scale;
        let y_multiplier = BLENDED_BASE_SCALE * data.y_scale;
        let limit_smear_scale_y = y_multiplier * data.smear_scale_multiplier;
        let main_smear_scale_y = limit_smear_scale_y / data.y_factor;
        let min_limit_noise = create_fbm(
            random,
            BLENDED_LIMIT_FIRST_OCTAVE,
            limit_smear_scale_y,
            BLENDED_LIMIT_FACTOR,
        );
        let max_limit_noise = create_fbm(
            random,
            BLENDED_LIMIT_FIRST_OCTAVE,
            limit_smear_scale_y,
            BLENDED_LIMIT_FACTOR,
        );
        let main_noise = create_fbm(
            random,
            BLENDED_MAIN_FIRST_OCTAVE,
            main_smear_scale_y,
            BLENDED_MAIN_FACTOR,
        );
        Self {
            min_limit_noise,
            max_limit_noise,
            main_noise,
            xz_multiplier,
            y_multiplier,
            main_xz_multiplier: xz_multiplier / data.xz_factor,
            main_y_multiplier: y_multiplier / data.y_factor,
            max_value: fbm_range_max(
                BLENDED_LIMIT_FIRST_OCTAVE,
                limit_smear_scale_y,
                BLENDED_LIMIT_FACTOR,
            ),
        }
    }

    #[inline]
    fn choice(main: f32) -> f32 {
        (main + 0.5).clamp(0.0, 1.0)
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
        let x = f64::from(pos.x);
        let y = f64::from(pos.y);
        let z = f64::from(pos.z);
        let alpha = Self::choice(self.main_noise.get(
            x * self.main_xz_multiplier,
            y * self.main_y_multiplier,
            z * self.main_xz_multiplier,
        ));
        if alpha == 0.0 {
            return self.min_limit_noise.get(
                x * self.xz_multiplier,
                y * self.y_multiplier,
                z * self.xz_multiplier,
            );
        }
        if alpha == 1.0 {
            return self.max_limit_noise.get(
                x * self.xz_multiplier,
                y * self.y_multiplier,
                z * self.xz_multiplier,
            );
        }
        lerp(
            alpha,
            self.min_limit_noise.get(
                x * self.xz_multiplier,
                y * self.y_multiplier,
                z * self.xz_multiplier,
            ),
            self.max_limit_noise.get(
                x * self.xz_multiplier,
                y * self.y_multiplier,
                z * self.xz_multiplier,
            ),
        )
    }

    fn sample_volume(&self, buffer: &mut [f32], volume: &DensityVolume) {
        buffer.fill(0.0);
        self.main_noise.add_to_volume(
            buffer,
            volume,
            self.main_xz_multiplier,
            self.main_y_multiplier,
            1.0,
        );
        for value in buffer.iter_mut() {
            *value = Self::choice(*value);
        }
        let mut first = DensityBuffer::acquire(volume);
        first.fill(0.0);
        self.min_limit_noise.add_to_volume(
            &mut first,
            volume,
            self.xz_multiplier,
            self.y_multiplier,
            1.0,
        );
        let mut second = DensityBuffer::acquire(volume);
        second.fill(0.0);
        self.max_limit_noise.add_to_volume(
            &mut second,
            volume,
            self.xz_multiplier,
            self.y_multiplier,
            1.0,
        );
        for ((value, first), second) in buffer.iter_mut().zip(first.iter()).zip(second.iter()) {
            let alpha = *value;
            *value = if alpha == 0.0 {
                *first
            } else if alpha == 1.0 {
                *second
            } else {
                lerp(alpha, *first, *second)
            };
        }
    }
}
