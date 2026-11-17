use pumpkin_data::chunk::DoublePerlinNoiseParameters;
use pumpkin_util::{
    noise::{
        perlin::{Noise, NormalNoise},
        volume::DensityVolume,
    },
    random::RandomImpl,
};

pub struct DoublePerlinNoiseSampler {
    noise: NormalNoise,
}

impl DoublePerlinNoiseSampler {
    #[must_use]
    pub const fn max_value(&self) -> f32 {
        self.noise.max_value()
    }

    pub fn from_params(
        rand: &mut impl RandomImpl,
        parameters: &DoublePerlinNoiseParameters,
        legacy: bool,
    ) -> Self {
        Self::new(rand, parameters.first_octave, parameters.amplitudes, legacy)
    }

    pub fn new(
        rand: &mut impl RandomImpl,
        first_octave: i32,
        amplitudes: &[f64],
        legacy: bool,
    ) -> Self {
        Self {
            noise: NormalNoise::new(rand, first_octave, amplitudes, legacy),
        }
    }

    #[inline]
    #[must_use]
    pub fn sample(&self, x: f64, y: f64, z: f64) -> f32 {
        self.noise.get(x, y, z)
    }

    #[inline]
    pub fn add_to_volume(
        &self,
        buffer: &mut [f32],
        volume: &DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f32,
    ) {
        self.noise
            .add_to_volume(buffer, volume, xz_scale, y_scale, amplitude);
    }
}
