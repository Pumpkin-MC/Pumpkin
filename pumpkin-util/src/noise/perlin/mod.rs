use crate::{
    math::lerp3,
    random::{RandomDeriverImpl, RandomImpl},
};

use super::GRADIENTS;

/// A 3D Perlin noise sampler implementation.
///
/// Perlin noise is a gradient noise function commonly used for procedural generation
/// of natural-looking textures, terrain, and other organic patterns. This implementation
/// provides 3D noise with optional vertical scaling.
///
/// The sampler uses a permutation table to hash coordinates and interpolates between
/// gradient vectors at integer lattice points to produce smooth, continuous noise.
#[derive(Clone)]
pub struct PerlinNoiseSampler {
    /// Permutation table for hashing coordinates (size 256, duplicated for easy wrapping).
    permutation: [u8; 256],
    /// X-coordinate origin offset (randomized to avoid symmetry).
    x_origin: f64,
    /// Y-coordinate origin offset (randomized to avoid symmetry).
    y_origin: f64,
    /// Z-coordinate origin offset (randomized to avoid symmetry).
    z_origin: f64,
}

impl PerlinNoiseSampler {
    /// Creates a new Perlin noise sampler with randomized origin and permutation table.
    ///
    /// # Arguments
    /// - `random` – The random number generator to use for initialization.
    ///
    /// # Returns
    /// A new `PerlinNoiseSampler` instance.
    pub fn new(random: &mut impl RandomImpl) -> Self {
        let x_origin = random.next_f64() * 256.0;
        let y_origin = random.next_f64() * 256.0;
        let z_origin = random.next_f64() * 256.0;

        let mut permutation = [0u8; 256];

        permutation
            .iter_mut()
            .enumerate()
            .for_each(|(i, x)| *x = i as u8);

        for i in 0..256 {
            let j = random.next_bounded_i32((256 - i) as i32) as usize;
            permutation.swap(i, i + j);
        }

        Self {
            permutation,
            x_origin,
            y_origin,
            z_origin,
        }
    }

    /// Samples noise at the given coordinates with no vertical scaling.
    ///
    /// This is a convenience method that calls `sample_no_fade` with a zero vertical
    /// scale and maximum Y.
    ///
    /// # Arguments
    /// - `x` – The X coordinate.
    /// - `y` – The Y coordinate.
    /// - `z` – The Z coordinate.
    ///
    /// # Returns
    /// The noise value at the given coordinates, typically in the range [-1, 1].
    #[inline]
    #[must_use]
    pub fn sample_flat_y(&self, x: f64, y: f64, z: f64) -> f64 {
        self.sample_no_fade(x, y, z, 0.0, 0.0)
    }

    /// Samples noise with optional vertical scaling and clamping.
    ///
    /// This method applies the origin offsets, computes the integer lattice points,
    /// and interpolates between gradient values.
    ///
    /// # Arguments
    /// - `x` – The X coordinate.
    /// - `y` – The Y coordinate.
    /// - `z` – The Z coordinate.
    /// - `y_scale` – The vertical scale factor (if 0, vertical noise is not scaled).
    /// - `y_max` – The maximum Y value to clamp to.
    ///
    /// # Returns
    /// The noise value at the given coordinates, typically in the range [-1, 1].
    #[must_use]
    pub fn sample_no_fade(&self, x: f64, y: f64, z: f64, y_scale: f64, y_max: f64) -> f64 {
        let true_x = x + self.x_origin;
        let true_y = y + self.y_origin;
        let true_z = z + self.z_origin;

        let x_floor = true_x.floor();
        let y_floor = true_y.floor();
        let z_floor = true_z.floor();

        let x_dec = true_x - x_floor;
        let y_dec = true_y - y_floor;
        let z_dec = true_z - z_floor;

        let y_noise = if y_scale == 0.0 {
            0.0
        } else {
            let raw_y_dec = if y_max >= 0.0 && y_max < y_dec {
                y_max
            } else {
                y_dec
            };
            (raw_y_dec / y_scale + 1E-7).floor() * y_scale
        };

        self.sample(
            x_floor as i32,
            y_floor as i32,
            z_floor as i32,
            x_dec,
            y_dec - y_noise,
            z_dec,
            y_dec,
        )
    }

    /// Computes the dot product of a gradient vector with the given coordinates.
    ///
    /// # Arguments
    /// - `hash` – The hash value used to select a gradient (lower 4 bits).
    /// - `x` – The X coordinate.
    /// - `y` – The Y coordinate.
    /// - `z` – The Z coordinate.
    ///
    /// # Returns
    /// The dot product of the selected gradient with (x, y, z).
    #[inline]
    const fn grad(hash: i32, x: f64, y: f64, z: f64) -> f64 {
        GRADIENTS[(hash & 15) as usize].dot(x, y, z)
    }

    /// Applies the Perlin fade curve to an interpolant.
    ///
    /// The fade function is 6t⁵ - 15t⁴ + 10t³, which has zero-first and second derivatives
    /// at t=0 and t=1, ensuring smooth interpolation.
    ///
    /// # Arguments
    /// - `value` – The interpolant (typically in [0, 1]).
    ///
    /// # Returns
    /// The faded value.
    #[inline]
    const fn perlin_fade(value: f64) -> f64 {
        value * value * value * (value * (value * 6.0 - 15.0) + 10.0)
    }

    /// Maps an integer coordinate through the permutation table.
    ///
    /// # Arguments
    /// - `input` – The input coordinate.
    ///
    /// # Returns
    /// A hashed value in the range [0, 255].
    #[inline]
    fn map(&self, input: i32) -> i32 {
        i32::from(self.permutation[(input & 0xFF) as usize])
    }

    /// Core sampling function that computes noise at a lattice point.
    ///
    /// This method computes the eight corner gradients of the unit cube surrounding
    /// the sample point and interpolates between them using the fade curves.
    ///
    /// # Arguments
    /// - `x` – The integer X coordinate of the lattice cube corner.
    /// - `y` – The integer Y coordinate of the lattice cube corner.
    /// - `z` – The integer Z coordinate of the lattice cube corner.
    /// - `local_x` – The fractional X offset within the cube [0, 1).
    /// - `local_y` – The fractional Y offset within the cube [0, 1).
    /// - `local_z` – The fractional Z offset within the cube [0, 1).
    /// - `fade_local_y` – The Y value to use for fading (may differ from `local_y` for vertical scaling).
    ///
    /// # Returns
    /// The interpolated noise value.
    #[expect(clippy::too_many_arguments)]
    #[expect(clippy::many_single_char_names)]
    fn sample(
        &self,
        x: i32,
        y: i32,
        z: i32,
        local_x: f64,
        local_y: f64,
        local_z: f64,
        fade_local_y: f64,
    ) -> f64 {
        let i = self.map(x);
        let j = self.map(x + 1);
        let k = self.map(i + y);
        let l = self.map(i + y + 1);

        let m = self.map(j + y);
        let n = self.map(j + y + 1);

        let d = Self::grad(self.map(k + z), local_x, local_y, local_z);
        let e = Self::grad(self.map(m + z), local_x - 1.0, local_y, local_z);
        let f = Self::grad(self.map(l + z), local_x, local_y - 1.0, local_z);
        let g = Self::grad(self.map(n + z), local_x - 1.0, local_y - 1.0, local_z);
        let h = Self::grad(self.map(k + z + 1), local_x, local_y, local_z - 1.0);
        let o = Self::grad(self.map(m + z + 1), local_x - 1.0, local_y, local_z - 1.0);
        let p = Self::grad(self.map(l + z + 1), local_x, local_y - 1.0, local_z - 1.0);
        let q = Self::grad(
            self.map(n + z + 1),
            local_x - 1.0,
            local_y - 1.0,
            local_z - 1.0,
        );
        let r = Self::perlin_fade(local_x);
        let s = Self::perlin_fade(fade_local_y);
        let t = Self::perlin_fade(local_z);

        lerp3(r, s, t, d, e, f, g, h, o, p, q)
    }
}

/// Data for a single octave in an octave Perlin noise sampler.
pub struct SamplerData {
    /// The Perlin noise sampler for this octave.
    pub sampler: PerlinNoiseSampler,
    /// The amplitude multiplier for this octave.
    pub amplitude: f64,
    /// The persistence factor (amplitude scaling between octaves).
    pub persistence: f64,
    /// The lacunarity factor (frequency scaling between octaves).
    pub lacunarity: f64,
}

/// A multi-octave Perlin noise sampler that combines multiple noise layers.
///
/// Octave noise, also known as fractal noise, combines multiple octaves of Perlin noise
/// with different frequencies and amplitudes to create more complex, natural-looking patterns.
/// Each octave adds finer detail to the overall noise.
pub struct OctavePerlinNoiseSampler {
    /// The list of samplers for each octave, with their associated parameters.
    pub samplers: Box<[SamplerData]>,
    /// The maximum possible absolute value this sampler can return.
    max_value: f64,
}

impl OctavePerlinNoiseSampler {
    #[must_use]
    /// Returns the maximum possible absolute value this sampler can return.
    ///
    /// This is useful for normalizing the output to a specific range.
    ///
    /// # Returns
    /// The maximum absolute value.
    pub const fn max_value(&self) -> f64 {
        self.max_value
    }

    /// Calculates the total amplitude for a given scale and set of persistences and amplitudes.
    ///
    /// # Arguments
    /// - `scale` – The overall scale factor.
    /// - `persistences` – The persistence values for each octave.
    /// - `amplitudes` – The amplitude values for each octave.
    ///
    /// # Returns
    /// The sum of scaled amplitudes.
    fn get_total_amplitude_generic(scale: f64, persistences: &[f64], amplitudes: &[f64]) -> f64 {
        amplitudes
            .iter()
            .zip(persistences)
            .map(|(amplitude, persistence)| {
                if *amplitude == 0.0 {
                    0.0
                } else {
                    scale * *amplitude * *persistence
                }
            })
            .sum()
    }

    /// Maintains precision by wrapping large values to avoid floating-point artifacts.
    ///
    /// This method subtracts multiples of 2²⁵ to keep values in a manageable range
    /// while preserving the noise pattern.
    ///
    /// # Arguments
    /// - `value` – The value to wrap.
    ///
    /// # Returns
    /// The wrapped value.
    #[inline]
    #[must_use]
    pub const fn maintain_precision(value: f64) -> f64 {
        value - (value / 3.355_443_2E7 + 0.5).floor() * 3.355_443_2E7
    }

    /// Calculates the starting octave and amplitude array from a list of octaves.
    ///
    /// This method sorts the octaves and creates an amplitude array where indices
    /// corresponding to present octaves have amplitude 1.0.
    ///
    /// # Arguments
    /// - `octaves` – The list of octave indices.
    ///
    /// # Returns
    /// A tuple containing:
    /// - The negative of the smallest octave (starting offset).
    /// - A vector of amplitudes for each octave in the range.
    #[must_use]
    pub fn calculate_amplitudes(octaves: &[i32]) -> (i32, Vec<f64>) {
        let mut octaves = Vec::from_iter(octaves);
        octaves.sort();

        let i = -**octaves.first().expect("we should have some octaves");
        let j = **octaves.last().expect("we should have some octaves");
        let k = i + j + 1;

        let mut double_list = vec![0.0; k as usize];

        for l in octaves {
            double_list[(l + i) as usize] = 1.0;
        }

        (-i, double_list)
    }

    /// Creates a new octave Perlin noise sampler.
    ///
    /// # Arguments
    /// - `random` – The random number generator to use.
    /// - `first_octave` – The index of the first octave (can be negative).
    /// - `amplitudes` – The amplitude for each octave (starting from `first_octave`).
    /// - `legacy` – Whether to use legacy initialization (compatible with older Minecraft versions).
    ///
    /// # Returns
    /// A new `OctavePerlinNoiseSampler` instance.
    pub fn new(
        random: &mut impl RandomImpl,
        first_octave: i32,
        amplitudes: &[f64],
        legacy: bool,
    ) -> Self {
        let i = amplitudes.len();
        let j = -first_octave;

        let mut samplers: Box<[Option<PerlinNoiseSampler>]> = vec![None; i].into();

        if legacy {
            let sampler = PerlinNoiseSampler::new(random);
            if j >= 0 && j < i as i32 {
                let d = amplitudes[j as usize];
                if d != 0.0 {
                    samplers[j as usize] = Some(sampler);
                }
            }

            for kx in (0..j as usize).rev() {
                if kx < i {
                    let e = amplitudes[kx];
                    if e == 0.0 {
                        random.skip(262);
                    } else {
                        samplers[kx] = Some(PerlinNoiseSampler::new(random));
                    }
                } else {
                    random.skip(262);
                }
            }
        } else {
            let splitter = random.next_splitter();
            for k in 0..i {
                if amplitudes[k] != 0.0 {
                    let l = first_octave + k as i32;
                    samplers[k] = Some(PerlinNoiseSampler::new(
                        &mut splitter.split_string(&format!("octave_{l}")),
                    ));
                }
            }
        }

        let mut persistence = 2f64.powi(i as i32 - 1) / (2f64.powi(i as i32) - 1.0);
        let mut lacunarity = 2f64.powi(-j);

        let persistences: Vec<f64> = (0..amplitudes.len())
            .map(|_| {
                let result = persistence;
                persistence /= 2.0;
                result
            })
            .collect();
        let lacunarities = (0..amplitudes.len()).map(|_| {
            let result = lacunarity;
            lacunarity *= 2.0;
            result
        });

        let max_value = Self::get_total_amplitude_generic(2.0, &persistences, amplitudes);

        let samplers = samplers
            .into_iter()
            .zip(amplitudes)
            .zip(persistences)
            .zip(lacunarities)
            .filter_map(|(((sampler, amplitude), persistence), lacunarity)| {
                sampler.map(|sampler| SamplerData {
                    sampler,
                    amplitude: *amplitude,
                    persistence,
                    lacunarity,
                })
            })
            .collect();

        Self {
            samplers,
            max_value,
        }
    }

    /// Calculates the total amplitude for a given scale.
    ///
    /// # Arguments
    /// - `scale` – The scale factor to apply.
    ///
    /// # Returns
    /// The sum of amplitudes times scale times persistence for all octaves.
    #[inline]
    #[must_use]
    pub fn get_total_amplitude(&self, scale: f64) -> f64 {
        self.samplers
            .iter()
            .map(|data| data.amplitude * scale * data.persistence)
            .sum()
    }

    /// Samples the noise at the given coordinates.
    ///
    /// This method combines all octaves by sampling each at its respective frequency
    /// and summing the results with appropriate amplitudes.
    ///
    /// # Arguments
    /// - `x` – The X coordinate.
    /// - `y` – The Y coordinate.
    /// - `z` – The Z coordinate.
    ///
    /// # Returns
    /// The combined noise value from all octaves.
    #[inline]
    #[must_use]
    pub fn sample(&self, x: f64, y: f64, z: f64) -> f64 {
        self.samplers
            .iter()
            .map(|data| {
                let mapped_x = Self::maintain_precision(x * data.lacunarity);
                let mapped_y = Self::maintain_precision(y * data.lacunarity);
                let mapped_z = Self::maintain_precision(z * data.lacunarity);

                let sample = data
                    .sampler
                    .sample_no_fade(mapped_x, mapped_y, mapped_z, 0.0, 0.0);

                data.amplitude * sample * data.persistence
            })
            .sum()
    }
}

/// Tests for the perlin noise implementations.
#[cfg(test)]
mod tests;
