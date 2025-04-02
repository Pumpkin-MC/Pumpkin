use std::sync::LazyLock;

use serde::Deserialize;

use crate::{
    math::vector3::Vector3, noise::simplex::OctaveSimplexNoiseSampler,
    random::legacy_rand::LegacyRand,
};

pub static TEMPERATURE_NOISE: LazyLock<OctaveSimplexNoiseSampler> = LazyLock::new(|| {
    let mut rand = LegacyRand::from_seed(1234);
    OctaveSimplexNoiseSampler::new(&mut rand, &[0])
});

pub static FROZEN_OCEAN_NOISE: LazyLock<OctaveSimplexNoiseSampler> = LazyLock::new(|| {
    let mut rand = LegacyRand::from_seed(3456);
    OctaveSimplexNoiseSampler::new(&mut rand, &[-2, -1, 0])
});

pub static FOLIAGE_NOISE: LazyLock<OctaveSimplexNoiseSampler> = LazyLock::new(|| {
    let mut rand = LegacyRand::from_seed(2345);
    OctaveSimplexNoiseSampler::new(&mut rand, &[0])
});

#[derive(Clone, Deserialize, Copy, Hash, PartialEq, Eq, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum TemperatureModifier {
    None,
    Frozen,
}

impl TemperatureModifier {
    pub fn convert_temperature(&self, pos: &Vector3<i32>, temperature: f32) -> f32 {
        match self {
            TemperatureModifier::None => temperature,
            TemperatureModifier::Frozen => {
                let frozen_ocean_sample =
                    FROZEN_OCEAN_NOISE.sample(pos.x as f64 * 0.05, pos.z as f64 * 0.05, false)
                        * 7.0;
                let foliage_sample =
                    FOLIAGE_NOISE.sample(pos.x as f64 * 0.2, pos.z as f64 * 0.2, false);

                let threshold = frozen_ocean_sample + foliage_sample;
                if threshold < 0.3 {
                    let foliage_sample =
                        FOLIAGE_NOISE.sample(pos.x as f64 * 0.09, pos.z as f64 * 0.09, false);
                    if foliage_sample < 0.8 {
                        return 0.2f32;
                    }
                }

                temperature
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Weather {
    #[allow(dead_code)]
    has_precipitation: bool,
    temperature: f32,
    temperature_modifier: TemperatureModifier,
    #[allow(dead_code)]
    downfall: f32,
}

impl Weather {
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

    /// This is an expensive function and should be cached
    pub fn compute_temperature(&self, pos: &Vector3<i32>, sea_level: i32) -> f32 {
        let modified_temperature = self
            .temperature_modifier
            .convert_temperature(pos, self.temperature);
        let offset_sea_level = sea_level + 17;

        if pos.y > offset_sea_level {
            let temperature_noise =
                (TEMPERATURE_NOISE.sample(pos.x as f64 / 8.0, pos.z as f64 / 8.0, false) * 8.0)
                    as f32;

            modified_temperature
                - (temperature_noise + pos.y as f32 - offset_sea_level as f32) * 0.05f32 / 40.0f32
        } else {
            modified_temperature
        }
    }
}
