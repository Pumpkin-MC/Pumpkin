/// Sunlight from here on counts as daylight. Spiders calm down at it, undead burn above it.
pub const DAYLIGHT_BRIGHTNESS: f32 = 0.5;

/// Vanilla `getLightLevelDependentMagicValue` curve. `raw_brightness` is 0–15.
#[must_use]
pub fn light_level_curve(raw_brightness: u8, ambient_light: f32) -> f32 {
    let brightness = f32::from(raw_brightness) / 15.0;
    let curved = brightness / 3.0f32.mul_add(-brightness, 4.0);
    (1.0 - curved).mul_add(ambient_light, curved)
}

#[cfg(test)]
mod tests {
    use super::{DAYLIGHT_BRIGHTNESS, light_level_curve};

    #[test]
    fn full_light_is_full_brightness_in_any_dimension() {
        for ambient in [0.0, 0.1, 1.0] {
            assert!((light_level_curve(15, ambient) - 1.0).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn darkness_is_zero_without_ambient_light() {
        assert!(light_level_curve(0, 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn ambient_light_lifts_the_floor() {
        assert!((light_level_curve(0, 0.1) - 0.1).abs() < 1e-6);
    }

    #[test]
    fn spider_daylight_cutoff_sits_between_light_11_and_12() {
        // Spiders stop hunting at 0.5: raw light 12 is the first level that reaches it.
        assert!(light_level_curve(11, 0.0) < DAYLIGHT_BRIGHTNESS);
        assert!(light_level_curve(12, 0.0) >= DAYLIGHT_BRIGHTNESS);
    }

    #[test]
    fn brighter_never_reads_darker() {
        let mut previous = light_level_curve(0, 0.0);
        for raw in 1..=15 {
            let current = light_level_curve(raw, 0.0);
            assert!(current > previous);
            previous = current;
        }
    }
}
