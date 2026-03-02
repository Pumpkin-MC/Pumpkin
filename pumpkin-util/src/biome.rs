use std::sync::LazyLock;

use serde::Deserialize;

use crate::{noise::simplex::OctaveSimplexNoiseSampler, random::legacy_rand::LegacyRand};

/// Global noise sampler for biome temperature variation.
pub static TEMPERATURE_NOISE: LazyLock<OctaveSimplexNoiseSampler> = LazyLock::new(|| {
    let mut rand = LegacyRand::from_seed(1234);
    OctaveSimplexNoiseSampler::new(&mut rand, &[0])
});

/// Global noise sampler for frozen ocean biome adjustments.
pub static FROZEN_OCEAN_NOISE: LazyLock<OctaveSimplexNoiseSampler> = LazyLock::new(|| {
    let mut rand = LegacyRand::from_seed(3456);
    OctaveSimplexNoiseSampler::new(&mut rand, &[-2, -1, 0])
});

/// Global noise sampler for foliage-based temperature adjustments.
pub static FOLIAGE_NOISE: LazyLock<OctaveSimplexNoiseSampler> = LazyLock::new(|| {
    let mut rand = LegacyRand::from_seed(2345);
    OctaveSimplexNoiseSampler::new(&mut rand, &[0])
});

/// Modifiers that adjust the base biome temperature.
#[derive(Clone, Deserialize, Copy, Hash, PartialEq, Eq, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum TemperatureModifier {
    /// No temperature modification.
    None,
    /// Frozen biome adjustment, typically used for frozen oceans and snowy areas.
    Frozen,
}

impl TemperatureModifier {
    /// Applies the temperature modifier to a given position and base temperature.
    ///
    /// # Parameters
    /// - `x`: X coordinate in the world.
    /// - `z`: Z coordinate in the world.
    /// - `temperature`: The base biome temperature.
    ///
    /// # Returns
    /// - The modified temperature as an `f32`.
    pub fn convert_temperature(&self, x: f64, z: f64, temperature: f32) -> f32 {
        match self {
            Self::None => temperature,
            Self::Frozen => {
                let frozen_ocean_sample =
                    FROZEN_OCEAN_NOISE.sample(x * 0.05, z * 0.05, false) * 7.0;
                let foliage_sample = FOLIAGE_NOISE.sample(x * 0.2, z * 0.2, false);

                let threshold = frozen_ocean_sample + foliage_sample;
                if threshold < 0.3 {
                    let foliage_sample = FOLIAGE_NOISE.sample(x * 0.09, z * 0.09, false);
                    if foliage_sample < 0.8 {
                        return 0.2f32;
                    }
                }

                temperature
            }
        }
    }
}

/// Represents weather information for a biome, including temperature and precipitation.
#[derive(Clone, Debug)]
pub struct Weather {
    #[expect(dead_code)]
    has_precipitation: bool,
    /// Base temperature of the biome.
    temperature: f32,
    /// Modifier affecting the base temperature.
    temperature_modifier: TemperatureModifier,
    /// Rate of rainfall or snowfall in the biome.
    #[expect(dead_code)]
    downfall: f32,
}

impl Weather {
    /// Creates a new `Weather` instance.
    ///
    /// # Parameters
    /// - `has_precipitation`: Whether the biome has precipitation.
    /// - `temperature`: Base temperature of the biome.
    /// - `temperature_modifier`: Modifier affecting the temperature.
    /// - `downfall`: Amount of rainfall or snowfall.
    #[must_use]
    pub const fn new(
        has_precipitation: bool,
        temperature: f32,
        temperature_modifier: TemperatureModifier,
        downfall: f32,
    ) -> Self {
        Self {
            has_precipitation,
            temperature,
            temperature_modifier,
            downfall,
        }
    }

    /// Computes the effective temperature at a given position.
    ///
    /// # Parameters
    /// - `x`, `z`: World coordinates.
    /// - `y`: Y-level of the position.
    /// - `sea_level`: Sea level in the world.
    ///
    /// # Returns
    /// - The temperature at the position as `f32`.
    ///
    /// # Notes
    /// - This function is computationally expensive and should be cached.
    /// - Temperature is influenced by `TemperatureModifier` and noise samplers.
    pub fn compute_temperature(&self, x: f64, y: i32, z: f64, sea_level: i32) -> f32 {
        let modified_temperature =
            self.temperature_modifier
                .convert_temperature(x, z, self.temperature);
        let offset_sea_level = sea_level + 17;

        if y > offset_sea_level {
            let temperature_noise =
                (TEMPERATURE_NOISE.sample(x / 8.0, z / 8.0, false) * 8.0) as f32;

            modified_temperature
                - (temperature_noise + y as f32 - offset_sea_level as f32) * 0.05f32 / 40.0f32
        } else {
            modified_temperature
        }
    }
}
