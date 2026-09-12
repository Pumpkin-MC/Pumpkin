use crate::{
    math::lerp3,
    random::{RandomDeriverImpl, RandomImpl},
};

use super::volume::DensityVolume;
use crate::math::smoothstep;

const GRADIENTS: [[f32; 3]; 16] = [
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [-1.0, -1.0, 0.0],
    [1.0, 0.0, 1.0],
    [-1.0, 0.0, 1.0],
    [1.0, 0.0, -1.0],
    [-1.0, 0.0, -1.0],
    [0.0, 1.0, 1.0],
    [0.0, -1.0, 1.0],
    [0.0, 1.0, -1.0],
    [0.0, -1.0, -1.0],
    [1.0, 1.0, 0.0],
    [0.0, -1.0, 1.0],
    [-1.0, 1.0, 0.0],
    [0.0, -1.0, -1.0],
];

const ROUND_OFF: f64 = 33_554_432.0;
const HALF_ROUND_OFF: f64 = f64::from_bits(0x416F_FFFF_FFFF_FFFF);
const NOISE_OFFSET_SCALE: f64 = 256.0;
const SHIFT_UP_EPSILON: f64 = 1.0E-7f32 as f64;
const STANDARD_DEVIATION: f64 = 0.270_224_783_124_521_1;
const TARGET_DEVIATION: f64 = 0.333_333_333_333_333_3;
const INPUT_FACTOR: f64 = 1.018_126_888_217_522_7;
const PERSISTENCE: f64 = 0.5;
const LACUNARITY: f64 = 2.0;

pub trait Noise {
    fn get(&self, x: f64, y: f64, z: f64) -> f32;

    fn add_to_volume(
        &self,
        buffer: &mut [f32],
        volume: &DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f32,
    );
}

#[inline]
fn grad_dot(hash: i32, x: f32, y: f32, z: f32) -> f32 {
    let g = &GRADIENTS[(hash & 15) as usize];
    g[0] * x + g[1] * y + g[2] * z
}

#[inline]
fn grad_dot_xz(g: &[f32; 3], x: f32, z: f32) -> f32 {
    g[0] * x + g[2] * z
}

#[inline]
#[must_use]
pub fn wrap(x: f64) -> f64 {
    if (-HALF_ROUND_OFF..HALF_ROUND_OFF).contains(&x) {
        x
    } else {
        x - (x / ROUND_OFF + 0.5).floor() * ROUND_OFF
    }
}

#[derive(Clone)]
pub struct PerlinNoise {
    perms: [u8; 256],
    pub offset_x: f64,
    pub offset_y: f64,
    pub offset_z: f64,
}

struct Corners {
    d000_xz: f32,
    d100_xz: f32,
    d010_xz: f32,
    d110_xz: f32,
    d001_xz: f32,
    d101_xz: f32,
    d011_xz: f32,
    d111_xz: f32,
    g000_y: f32,
    g100_y: f32,
    g010_y: f32,
    g110_y: f32,
    g001_y: f32,
    g101_y: f32,
    g011_y: f32,
    g111_y: f32,
}

impl Corners {
    const fn zero() -> Self {
        Self {
            d000_xz: 0.0,
            d100_xz: 0.0,
            d010_xz: 0.0,
            d110_xz: 0.0,
            d001_xz: 0.0,
            d101_xz: 0.0,
            d011_xz: 0.0,
            d111_xz: 0.0,
            g000_y: 0.0,
            g100_y: 0.0,
            g010_y: 0.0,
            g110_y: 0.0,
            g001_y: 0.0,
            g101_y: 0.0,
            g011_y: 0.0,
            g111_y: 0.0,
        }
    }

    #[inline]
    fn lerp(&self, alpha_x: f32, alpha_y: f32, alpha_z: f32, relative_y: f32) -> f32 {
        lerp3(
            alpha_x,
            alpha_y,
            alpha_z,
            self.d000_xz + self.g000_y * relative_y,
            self.d100_xz + self.g100_y * relative_y,
            self.d010_xz + self.g010_y * (relative_y - 1.0),
            self.d110_xz + self.g110_y * (relative_y - 1.0),
            self.d001_xz + self.g001_y * relative_y,
            self.d101_xz + self.g101_y * relative_y,
            self.d011_xz + self.g011_y * (relative_y - 1.0),
            self.d111_xz + self.g111_y * (relative_y - 1.0),
        )
    }
}

impl PerlinNoise {
    pub fn new(random: &mut impl RandomImpl) -> Self {
        let offset_x = random.next_f64() * NOISE_OFFSET_SCALE;
        let offset_y = random.next_f64() * NOISE_OFFSET_SCALE;
        let offset_z = random.next_f64() * NOISE_OFFSET_SCALE;
        let mut perms = [0u8; 256];
        for (i, perm) in perms.iter_mut().enumerate() {
            *perm = i as u8;
        }
        for i in 0..256 {
            let offset = random.next_bounded_i32((256 - i) as i32) as usize;
            perms.swap(i, offset + i);
        }
        Self {
            perms,
            offset_x,
            offset_y,
            offset_z,
        }
    }

    #[inline]
    fn permute(&self, x: i32) -> i32 {
        i32::from(self.perms[(x & 0xFF) as usize])
    }

    #[inline]
    fn permute_to_grad(&self, x: i32) -> &'static [f32; 3] {
        &GRADIENTS[(self.permute(x) & 15) as usize]
    }

    #[expect(clippy::too_many_arguments)]
    fn sample_and_lerp(
        &self,
        x: i32,
        y: i32,
        z: i32,
        relative_x: f32,
        relative_y: f32,
        relative_z: f32,
        original_relative_y: f32,
    ) -> f32 {
        let x0 = self.permute(x);
        let x1 = self.permute(x + 1);
        let xy00 = self.permute(x0 + y);
        let xy01 = self.permute(x0 + y + 1);
        let xy10 = self.permute(x1 + y);
        let xy11 = self.permute(x1 + y + 1);
        let d000 = grad_dot(self.permute(xy00 + z), relative_x, relative_y, relative_z);
        let d100 = grad_dot(
            self.permute(xy10 + z),
            relative_x - 1.0,
            relative_y,
            relative_z,
        );
        let d010 = grad_dot(
            self.permute(xy01 + z),
            relative_x,
            relative_y - 1.0,
            relative_z,
        );
        let d110 = grad_dot(
            self.permute(xy11 + z),
            relative_x - 1.0,
            relative_y - 1.0,
            relative_z,
        );
        let d001 = grad_dot(
            self.permute(xy00 + z + 1),
            relative_x,
            relative_y,
            relative_z - 1.0,
        );
        let d101 = grad_dot(
            self.permute(xy10 + z + 1),
            relative_x - 1.0,
            relative_y,
            relative_z - 1.0,
        );
        let d011 = grad_dot(
            self.permute(xy01 + z + 1),
            relative_x,
            relative_y - 1.0,
            relative_z - 1.0,
        );
        let d111 = grad_dot(
            self.permute(xy11 + z + 1),
            relative_x - 1.0,
            relative_y - 1.0,
            relative_z - 1.0,
        );
        let alpha_x = smoothstep(relative_x);
        let alpha_y = smoothstep(original_relative_y);
        let alpha_z = smoothstep(relative_z);
        lerp3(
            alpha_x, alpha_y, alpha_z, d000, d100, d010, d110, d001, d101, d011, d111,
        )
    }

    #[expect(clippy::too_many_arguments)]
    fn corners(
        &self,
        x0: i32,
        x1: i32,
        floor_y: i32,
        floor_z: i32,
        relative_x: f32,
        relative_z: f32,
        corners: &mut Corners,
    ) {
        let xy00 = self.permute(x0 + floor_y);
        let xy01 = self.permute(x0 + floor_y + 1);
        let xy10 = self.permute(x1 + floor_y);
        let xy11 = self.permute(x1 + floor_y + 1);
        let g000 = self.permute_to_grad(xy00 + floor_z);
        corners.d000_xz = grad_dot_xz(g000, relative_x, relative_z);
        corners.g000_y = g000[1];
        let g100 = self.permute_to_grad(xy10 + floor_z);
        corners.d100_xz = grad_dot_xz(g100, relative_x - 1.0, relative_z);
        corners.g100_y = g100[1];
        let g010 = self.permute_to_grad(xy01 + floor_z);
        corners.d010_xz = grad_dot_xz(g010, relative_x, relative_z);
        corners.g010_y = g010[1];
        let g110 = self.permute_to_grad(xy11 + floor_z);
        corners.d110_xz = grad_dot_xz(g110, relative_x - 1.0, relative_z);
        corners.g110_y = g110[1];
        let g001 = self.permute_to_grad(xy00 + floor_z + 1);
        corners.d001_xz = grad_dot_xz(g001, relative_x, relative_z - 1.0);
        corners.g001_y = g001[1];
        let g101 = self.permute_to_grad(xy10 + floor_z + 1);
        corners.d101_xz = grad_dot_xz(g101, relative_x - 1.0, relative_z - 1.0);
        corners.g101_y = g101[1];
        let g011 = self.permute_to_grad(xy01 + floor_z + 1);
        corners.d011_xz = grad_dot_xz(g011, relative_x, relative_z - 1.0);
        corners.g011_y = g011[1];
        let g111 = self.permute_to_grad(xy11 + floor_z + 1);
        corners.d111_xz = grad_dot_xz(g111, relative_x - 1.0, relative_z - 1.0);
        corners.g111_y = g111[1];
    }
}

impl Noise for PerlinNoise {
    fn get(&self, x: f64, y: f64, z: f64) -> f32 {
        let x = wrap(x) + self.offset_x;
        let y = wrap(y) + self.offset_y;
        let z = wrap(z) + self.offset_z;
        let floor_x = x.floor() as i32;
        let floor_y = y.floor() as i32;
        let floor_z = z.floor() as i32;
        let relative_x = (x - f64::from(floor_x)) as f32;
        let relative_y = (y - f64::from(floor_y)) as f32;
        let relative_z = (z - f64::from(floor_z)) as f32;
        self.sample_and_lerp(
            floor_x, floor_y, floor_z, relative_x, relative_y, relative_z, relative_y,
        )
    }

    fn add_to_volume(
        &self,
        buffer: &mut [f32],
        volume: &DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f32,
    ) {
        let mut corners = Corners::zero();
        let mut index = 0;
        for index_z in 0..volume.size_z {
            let z = wrap(f64::from(volume.block_z(index_z)) * xz_scale) + self.offset_z;
            let floor_z = z.floor() as i32;
            let relative_z = (z - f64::from(floor_z)) as f32;
            let alpha_z = smoothstep(relative_z);
            for index_x in 0..volume.size_x {
                let x = wrap(f64::from(volume.block_x(index_x)) * xz_scale) + self.offset_x;
                let floor_x = x.floor() as i32;
                let relative_x = (x - f64::from(floor_x)) as f32;
                let x0 = self.permute(floor_x);
                let x1 = self.permute(floor_x + 1);
                let alpha_x = smoothstep(relative_x);
                let mut last_floor_y = i32::MIN;
                for index_y in 0..volume.size_y {
                    let y = wrap(f64::from(volume.block_y(index_y)) * y_scale) + self.offset_y;
                    let floor_y = y.floor() as i32;
                    let relative_y = (y - f64::from(floor_y)) as f32;
                    let alpha_y = smoothstep(relative_y);
                    if last_floor_y != floor_y {
                        self.corners(
                            x0,
                            x1,
                            floor_y,
                            floor_z,
                            relative_x,
                            relative_z,
                            &mut corners,
                        );
                        last_floor_y = floor_y;
                    }
                    buffer[index] +=
                        amplitude * corners.lerp(alpha_x, alpha_y, alpha_z, relative_y);
                    index += 1;
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct SmearedPerlinNoise {
    noise: PerlinNoise,
    fudge_y_scale: f64,
}

impl SmearedPerlinNoise {
    pub fn new(random: &mut impl RandomImpl, fudge_y_scale: f64) -> Self {
        Self {
            noise: PerlinNoise::new(random),
            fudge_y_scale,
        }
    }

    #[inline]
    fn compute_fudge_y(&self, original_y: f64, relative_y: f64) -> f64 {
        let fudge_limit = if original_y >= 0.0 && original_y < relative_y {
            original_y
        } else {
            relative_y
        };
        (fudge_limit / self.fudge_y_scale + SHIFT_UP_EPSILON).floor() * self.fudge_y_scale
    }
}

impl Noise for SmearedPerlinNoise {
    fn get(&self, x: f64, original_y: f64, z: f64) -> f32 {
        let x = wrap(x) + self.noise.offset_x;
        let y = wrap(original_y) + self.noise.offset_y;
        let z = wrap(z) + self.noise.offset_z;
        let floor_x = x.floor() as i32;
        let floor_y = y.floor() as i32;
        let floor_z = z.floor() as i32;
        let relative_x = (x - f64::from(floor_x)) as f32;
        let relative_y = y - f64::from(floor_y);
        let relative_z = (z - f64::from(floor_z)) as f32;
        let fudged_relative_y = (relative_y - self.compute_fudge_y(original_y, relative_y)) as f32;
        self.noise.sample_and_lerp(
            floor_x,
            floor_y,
            floor_z,
            relative_x,
            fudged_relative_y,
            relative_z,
            relative_y as f32,
        )
    }

    fn add_to_volume(
        &self,
        buffer: &mut [f32],
        volume: &DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f32,
    ) {
        let mut corners = Corners::zero();
        let mut index = 0;
        for index_z in 0..volume.size_z {
            let z = wrap(f64::from(volume.block_z(index_z)) * xz_scale) + self.noise.offset_z;
            let floor_z = z.floor() as i32;
            let relative_z = (z - f64::from(floor_z)) as f32;
            let alpha_z = smoothstep(relative_z);
            for index_x in 0..volume.size_x {
                let x = wrap(f64::from(volume.block_x(index_x)) * xz_scale) + self.noise.offset_x;
                let floor_x = x.floor() as i32;
                let relative_x = (x - f64::from(floor_x)) as f32;
                let x0 = self.noise.permute(floor_x);
                let x1 = self.noise.permute(floor_x + 1);
                let alpha_x = smoothstep(relative_x);
                let mut last_floor_y = i32::MIN;
                for index_y in 0..volume.size_y {
                    let original_y = f64::from(volume.block_y(index_y)) * y_scale;
                    let y = wrap(original_y) + self.noise.offset_y;
                    let floor_y = y.floor() as i32;
                    let relative_y = y - f64::from(floor_y);
                    let alpha_y = smoothstep(relative_y as f32);
                    if last_floor_y != floor_y {
                        self.noise.corners(
                            x0,
                            x1,
                            floor_y,
                            floor_z,
                            relative_x,
                            relative_z,
                            &mut corners,
                        );
                        last_floor_y = floor_y;
                    }
                    let fudged_relative_y =
                        (relative_y - self.compute_fudge_y(original_y, relative_y)) as f32;
                    buffer[index] +=
                        amplitude * corners.lerp(alpha_x, alpha_y, alpha_z, fudged_relative_y);
                    index += 1;
                }
            }
        }
    }
}

pub struct Layer<N> {
    pub noise: N,
    pub frequency: f64,
    pub amplitude: f32,
}

pub struct NoiseStack<N> {
    pub layers: Box<[Layer<N>]>,
}

impl<N: Noise> NoiseStack<N> {
    #[must_use]
    pub fn new(layers: Vec<Layer<N>>) -> Self {
        Self {
            layers: layers.into_boxed_slice(),
        }
    }
}

impl<N: Noise> Noise for NoiseStack<N> {
    fn get(&self, x: f64, y: f64, z: f64) -> f32 {
        let mut value = 0.0f32;
        for layer in &self.layers {
            let frequency = layer.frequency;
            value += layer.amplitude * layer.noise.get(x * frequency, y * frequency, z * frequency);
        }
        value
    }

    fn add_to_volume(
        &self,
        buffer: &mut [f32],
        volume: &DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f32,
    ) {
        for layer in &self.layers {
            let frequency = layer.frequency;
            layer.noise.add_to_volume(
                buffer,
                volume,
                xz_scale * frequency,
                y_scale * frequency,
                amplitude * layer.amplitude,
            );
        }
    }
}

struct OctaveInfo {
    index: i32,
    frequency: f64,
    amplitude: f64,
}

const fn amplitude_modifier(modifiers: &[f64], index: usize) -> f64 {
    if modifiers.is_empty() {
        1.0
    } else {
        modifiers[index]
    }
}

fn build_octaves(
    base_octave: i32,
    base_amplitude: f64,
    octave_count: usize,
    normalize: bool,
    modifiers: &[f64],
) -> Vec<OctaveInfo> {
    let mut frequency = LACUNARITY.powi(base_octave);
    let mut amplitude = if normalize {
        base_amplitude
            * (PERSISTENCE.powi(-(octave_count as i32 - 1))
                / (PERSISTENCE.powi(-(octave_count as i32)) - 1.0))
    } else {
        base_amplitude
    };
    let mut octaves = Vec::with_capacity(octave_count);
    for i in 0..octave_count {
        let modifier = amplitude_modifier(modifiers, i);
        if modifier != 0.0 {
            octaves.push(OctaveInfo {
                index: base_octave + i as i32,
                frequency,
                amplitude: amplitude * modifier,
            });
        }
        frequency *= LACUNARITY;
        amplitude *= PERSISTENCE;
    }
    octaves
}

fn estimate_deviation(octaves: &[OctaveInfo]) -> f64 {
    let mut variance = 0.0;
    for octave in octaves {
        let layer_deviation = STANDARD_DEVIATION * octave.amplitude.abs();
        variance += layer_deviation * layer_deviation;
    }
    variance.sqrt()
}

fn compute_normalization_factor(target_amplitude: f64, octaves: &[OctaveInfo]) -> f64 {
    let input_deviation = estimate_deviation(octaves);
    if input_deviation == 0.0 {
        0.0
    } else {
        let input_sum_deviation = input_deviation * 2.0f64.sqrt();
        let target_deviation = target_amplitude * TARGET_DEVIATION;
        target_deviation / input_sum_deviation
    }
}

fn parity_expected_deviation(octave_span: i32) -> f64 {
    0.1 * (1.0 + 1.0 / f64::from(octave_span + 1))
}

fn parity_normalization_factor(base_amplitude: f64, octave_count: usize, modifiers: &[f64]) -> f64 {
    let mut min_octave = i32::MAX;
    let mut max_octave = i32::MIN;
    for i in 0..octave_count {
        if amplitude_modifier(modifiers, i) != 0.0 {
            min_octave = min_octave.min(i as i32);
            max_octave = max_octave.max(i as i32);
        }
    }
    base_amplitude * 0.5 * TARGET_DEVIATION / parity_expected_deviation(max_octave - min_octave)
}

fn parity_base_amplitude(base_octave: i32, amplitudes: &[f64]) -> f64 {
    let octaves = build_octaves(base_octave, 1.0, amplitudes.len(), true, amplitudes);
    let target_amplitude: f64 = octaves.iter().map(|o| o.amplitude.abs()).sum();
    let new_normalization_factor = compute_normalization_factor(target_amplitude, &octaves);
    if new_normalization_factor == 0.0 {
        1.0
    } else {
        parity_normalization_factor(1.0, amplitudes.len(), amplitudes) / new_normalization_factor
    }
}

pub fn legacy_fbm(
    random: &mut impl RandomImpl,
    first_octave: i32,
    amplitudes: &[f64],
) -> NoiseStack<PerlinNoise> {
    let octaves = amplitudes.len();
    let zero_octave_index = -first_octave;
    let mut noise_levels: Vec<Option<PerlinNoise>> = (0..octaves).map(|_| None).collect();
    let zero_octave = PerlinNoise::new(random);
    if zero_octave_index >= 0
        && zero_octave_index < octaves as i32
        && amplitudes[zero_octave_index as usize] != 0.0
    {
        noise_levels[zero_octave_index as usize] = Some(zero_octave);
    }
    for i in (0..zero_octave_index).rev() {
        if i < octaves as i32 {
            if amplitudes[i as usize] == 0.0 {
                random.skip(262);
            } else {
                noise_levels[i as usize] = Some(PerlinNoise::new(random));
            }
        } else {
            random.skip(262);
        }
    }
    let mut factor = LACUNARITY.powi(-zero_octave_index);
    let mut value_factor =
        LACUNARITY.powi(octaves as i32 - 1) / (LACUNARITY.powi(octaves as i32) - 1.0);
    let mut layers = Vec::with_capacity(octaves);
    for (i, noise) in noise_levels.into_iter().enumerate() {
        if let Some(noise) = noise {
            layers.push(Layer {
                noise,
                frequency: factor,
                amplitude: (value_factor * amplitudes[i]) as f32,
            });
        }
        factor *= LACUNARITY;
        value_factor /= LACUNARITY;
    }
    NoiseStack::new(layers)
}

pub struct NormalNoise {
    stack: NoiseStack<PerlinNoise>,
    max_value: f32,
}

impl NormalNoise {
    pub fn new(
        random: &mut impl RandomImpl,
        first_octave: i32,
        amplitudes: &[f64],
        legacy: bool,
    ) -> Self {
        let octave_count = amplitudes.len();
        if legacy {
            let octaves = build_octaves(first_octave, 1.0, octave_count, true, amplitudes);
            let mut target_amplitude: f64 = octaves.iter().map(|o| o.amplitude.abs()).sum();
            let mut normalization_factor = compute_normalization_factor(target_amplitude, &octaves);
            if normalization_factor != 0.0 {
                let parity = parity_normalization_factor(1.0, octave_count, amplitudes);
                target_amplitude *= parity / normalization_factor;
                normalization_factor = parity;
            }
            let first = legacy_fbm(random, first_octave, amplitudes);
            let second = legacy_fbm(random, first_octave, amplitudes);
            let value_factor = normalization_factor as f32;
            let mut layers = Vec::new();
            for layer in first.layers {
                layers.push(Layer {
                    noise: layer.noise,
                    frequency: layer.frequency,
                    amplitude: layer.amplitude * value_factor,
                });
            }
            for layer in second.layers {
                layers.push(Layer {
                    noise: layer.noise,
                    frequency: layer.frequency * INPUT_FACTOR,
                    amplitude: layer.amplitude * value_factor,
                });
            }
            return Self {
                stack: NoiseStack::new(layers),
                max_value: (target_amplitude * TARGET_DEVIATION * 6.0) as f32,
            };
        }
        let base_amplitude = parity_base_amplitude(first_octave, amplitudes);
        let octaves = build_octaves(first_octave, base_amplitude, octave_count, true, amplitudes);
        let target_amplitude: f64 = octaves.iter().map(|o| o.amplitude.abs()).sum();
        let normalization_factor = compute_normalization_factor(target_amplitude, &octaves);
        let first_random = random.next_splitter();
        let second_random = random.next_splitter();
        let mut layers = Vec::with_capacity(octaves.len() * 2);
        for octave in &octaves {
            let seed = format!("octave_{}", octave.index);
            let value_factor = (normalization_factor * octave.amplitude) as f32;
            layers.push(Layer {
                noise: PerlinNoise::new(&mut first_random.split_string(&seed)),
                frequency: octave.frequency,
                amplitude: value_factor,
            });
            layers.push(Layer {
                noise: PerlinNoise::new(&mut second_random.split_string(&seed)),
                frequency: octave.frequency * INPUT_FACTOR,
                amplitude: value_factor,
            });
        }
        Self {
            stack: NoiseStack::new(layers),
            max_value: (target_amplitude * TARGET_DEVIATION * 6.0) as f32,
        }
    }

    #[must_use]
    pub const fn max_value(&self) -> f32 {
        self.max_value
    }
}

impl Noise for NormalNoise {
    #[inline]
    fn get(&self, x: f64, y: f64, z: f64) -> f32 {
        self.stack.get(x, y, z)
    }

    #[inline]
    fn add_to_volume(
        &self,
        buffer: &mut [f32],
        volume: &DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f32,
    ) {
        self.stack
            .add_to_volume(buffer, volume, xz_scale, y_scale, amplitude);
    }
}
