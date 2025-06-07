use crate::random::RandomImpl;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use serde::Deserialize;
use syn::LitFloat;

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum NormalFloatProvider {
    #[serde(rename = "minecraft:constant")]
    Constant(ConstantFloatProvider),
    #[serde(rename = "minecraft:uniform")]
    Uniform(UniformFloatProvider),
    #[serde(rename = "minecraft:clamped_normal")]
    ClampedNormal(ClampedNormalFloatProvider),
    #[serde(rename = "minecraft:trapezoid")]
    Trapezoid(TrapezoidFloatProvider),
}

impl ToTokens for NormalFloatProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            NormalFloatProvider::Constant(constant) => {
                tokens.extend(quote! {
                    NormalFloatProvider::Constant(#constant)
                });
            }
            NormalFloatProvider::Uniform(uniform) => {
                tokens.extend(quote! {
                    NormalFloatProvider::Uniform(#uniform)
                });
            }
            NormalFloatProvider::ClampedNormal(clamped_normal) => {
                tokens.extend(quote! {
                    NormalFloatProvider::ClampedNormal(#clamped_normal)
                });
            }
            NormalFloatProvider::Trapezoid(trapezoid) => {
                tokens.extend(quote! {
                    NormalFloatProvider::Trapezoid(#trapezoid)
                });
            }
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum FloatProvider {
    Object(NormalFloatProvider),
    Constant(f32),
}

impl ToTokens for FloatProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            FloatProvider::Object(float_provider) => {
                tokens.extend(quote! {
                    FloatProvider::Object(#float_provider)
                });
            }
            FloatProvider::Constant(f) => tokens.extend(quote! {
                FloatProvider::Constant(#f)
            }),
        }
    }
}

impl FloatProvider {
    pub fn get_min(&self) -> f32 {
        match self {
            FloatProvider::Object(inv_provider) => match inv_provider {
                NormalFloatProvider::Constant(constant) => constant.get_min(),
                NormalFloatProvider::Uniform(uniform) => uniform.get_min(),
                NormalFloatProvider::ClampedNormal(clamped_normal) => clamped_normal.get_min(),
                NormalFloatProvider::Trapezoid(trapezoid) => trapezoid.get_min(),
            },
            FloatProvider::Constant(i) => *i,
        }
    }

    pub fn get(&self, random: &mut impl RandomImpl) -> f32 {
        match self {
            FloatProvider::Object(inv_provider) => match inv_provider {
                NormalFloatProvider::Constant(constant) => constant.get(random),
                NormalFloatProvider::Uniform(uniform) => uniform.get(random),
                NormalFloatProvider::ClampedNormal(clamped_normal) => clamped_normal.get(random),
                NormalFloatProvider::Trapezoid(trapezoid) => trapezoid.get(random),
            },
            FloatProvider::Constant(i) => *i,
        }
    }

    pub fn get_max(&self) -> f32 {
        match self {
            FloatProvider::Object(inv_provider) => match inv_provider {
                NormalFloatProvider::Constant(constant) => constant.get_max(),
                NormalFloatProvider::Uniform(uniform) => uniform.get_max(),
                NormalFloatProvider::ClampedNormal(clamped_normal) => clamped_normal.get_max(),
                NormalFloatProvider::Trapezoid(trapezoid) => trapezoid.get_max(),
            },
            FloatProvider::Constant(i) => *i,
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct ConstantFloatProvider {
    value: f32,
}

impl ToTokens for ConstantFloatProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let value = LitFloat::new(&self.value.to_string(), Span::call_site());
        tokens.extend(quote! {
            ConstantFloatProvider { value: #value }
        });
    }
}

impl ConstantFloatProvider {
    pub fn new(value: f32) -> Self {
        Self { value }
    }

    pub fn get_min(&self) -> f32 {
        self.value
    }

    pub fn get(&self, _random: &mut impl RandomImpl) -> f32 {
        self.value
    }

    pub fn get_max(&self) -> f32 {
        self.value
    }
}

#[derive(Deserialize, Clone)]
pub struct UniformFloatProvider {
    min_inclusive: f32,
    max_exclusive: f32,
}

impl ToTokens for UniformFloatProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let min_inclusive = LitFloat::new(&self.min_inclusive.to_string(), Span::call_site());
        let max_exclusive = LitFloat::new(&self.max_exclusive.to_string(), Span::call_site());
        tokens.extend(quote! {
            UniformFloatProvider { min_inclusive: #min_inclusive, max_exclusive: #max_exclusive }
        });
    }
}

impl UniformFloatProvider {
    pub fn new(min_inclusive: f32, max_exclusive: f32) -> Self {
        Self {
            min_inclusive,
            max_exclusive,
        }
    }

    pub fn get_min(&self) -> f32 {
        self.min_inclusive
    }

    pub fn get(&self, random: &mut impl RandomImpl) -> f32 {
        // Use the random range in [min_inclusive, max_exclusive)
        let range = self.max_exclusive - self.min_inclusive;
        self.min_inclusive + random.next_f32() * range
    }

    pub fn get_max(&self) -> f32 {
        self.max_exclusive
    }
}

#[derive(Deserialize, Clone)]
pub struct ClampedNormalFloatProvider {
    mean: f32,
    deviation: f32,
    min: f32,
    max: f32,
}

impl ToTokens for ClampedNormalFloatProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mean = LitFloat::new(&self.mean.to_string(), Span::call_site());
        let deviation = LitFloat::new(&self.deviation.to_string(), Span::call_site());
        let min = LitFloat::new(&self.min.to_string(), Span::call_site());
        let max = LitFloat::new(&self.max.to_string(), Span::call_site());
        tokens.extend(quote! {
            ClampedNormalFloatProvider {
                mean: #mean,
                deviation: #deviation,
                min: #min,
                max: #max
            }
        });
    }
}

impl ClampedNormalFloatProvider {
    pub fn new(mean: f32, deviation: f32, min: f32, max: f32) -> Self {
        Self {
            mean,
            deviation,
            min,
            max,
        }
    }

    pub fn get_min(&self) -> f32 {
        self.min
    }

    pub fn get(&self, random: &mut impl RandomImpl) -> f32 {
        // Generate normal distribution value
        let gaussian = random.next_gaussian() as f32;
        let value = self.mean + gaussian * self.deviation;

        // Clamp to min/max range
        value.clamp(self.min, self.max)
    }

    pub fn get_max(&self) -> f32 {
        self.max
    }
}

#[derive(Deserialize, Clone)]
pub struct TrapezoidFloatProvider {
    min: f32,
    max: f32,
    plateau: f32,
}

impl ToTokens for TrapezoidFloatProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let min = LitFloat::new(&self.min.to_string(), Span::call_site());
        let max = LitFloat::new(&self.max.to_string(), Span::call_site());
        let plateau = LitFloat::new(&self.plateau.to_string(), Span::call_site());
        tokens.extend(quote! {
            TrapezoidFloatProvider {
                min: #min,
                max: #max,
                plateau: #plateau
            }
        });
    }
}

impl TrapezoidFloatProvider {
    pub fn new(min: f32, max: f32, plateau: f32) -> Self {
        Self { min, max, plateau }
    }

    pub fn get_min(&self) -> f32 {
        self.min
    }

    pub fn get(&self, random: &mut impl RandomImpl) -> f32 {
        // Trapezoid distribution: flat plateau in the middle, linear ramps on sides
        let range = self.max - self.min;
        let plateau_range = range * self.plateau;
        let ramp_range = (range - plateau_range) * 0.5;

        let random_value = random.next_f32();

        if random_value < 0.5 - self.plateau * 0.5 {
            // Left ramp: quadratic distribution biased toward plateau
            let scaled = random_value / (0.5 - self.plateau * 0.5);
            let sqrt_scaled = scaled.sqrt();
            self.min + ramp_range * sqrt_scaled
        } else if random_value > 0.5 + self.plateau * 0.5 {
            // Right ramp: quadratic distribution biased toward plateau
            let scaled = (random_value - (0.5 + self.plateau * 0.5)) / (0.5 - self.plateau * 0.5);
            let sqrt_scaled = (1.0 - scaled).sqrt();
            self.max - ramp_range * sqrt_scaled
        } else {
            // Plateau: uniform distribution
            let plateau_pos = (random_value - (0.5 - self.plateau * 0.5)) / self.plateau;
            self.min + ramp_range + plateau_pos * plateau_range
        }
    }

    pub fn get_max(&self) -> f32 {
        self.max
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::{RandomGenerator, get_seed};

    #[test]
    fn test_constant_float_provider() {
        let mut random = RandomGenerator::Xoroshiro(
            crate::random::xoroshiro128::Xoroshiro::from_seed(get_seed()),
        );
        let provider = ConstantFloatProvider::new(5.5);

        assert_eq!(provider.get_min(), 5.5);
        assert_eq!(provider.get_max(), 5.5);
        assert_eq!(provider.get(&mut random), 5.5);
        assert_eq!(provider.get(&mut random), 5.5); // Should always return the same value
    }

    #[test]
    fn test_uniform_float_provider() {
        let mut random = RandomGenerator::Xoroshiro(
            crate::random::xoroshiro128::Xoroshiro::from_seed(get_seed()),
        );
        let provider = UniformFloatProvider::new(1.0, 5.0);

        assert_eq!(provider.get_min(), 1.0);
        assert_eq!(provider.get_max(), 5.0);

        // Test that values are within range
        for _ in 0..100 {
            let value = provider.get(&mut random);
            assert!(
                (1.0..5.0).contains(&value),
                "Value {} is outside range [1.0, 5.0)",
                value
            );
        }
    }

    #[test]
    fn test_clamped_normal_float_provider() {
        let mut random = RandomGenerator::Xoroshiro(
            crate::random::xoroshiro128::Xoroshiro::from_seed(get_seed()),
        );
        let provider = ClampedNormalFloatProvider::new(3.0, 1.0, 1.0, 5.0);

        assert_eq!(provider.get_min(), 1.0);
        assert_eq!(provider.get_max(), 5.0);

        // Test that values are within range
        for _ in 0..100 {
            let value = provider.get(&mut random);
            assert!(
                (1.0..=5.0).contains(&value),
                "Value {} is outside range [1.0, 5.0]",
                value
            );
        }
    }

    #[test]
    fn test_trapezoid_float_provider() {
        let mut random = RandomGenerator::Xoroshiro(
            crate::random::xoroshiro128::Xoroshiro::from_seed(get_seed()),
        );
        let provider = TrapezoidFloatProvider::new(0.0, 10.0, 0.5);

        assert_eq!(provider.get_min(), 0.0);
        assert_eq!(provider.get_max(), 10.0);

        // Test that values are within range
        for _ in 0..100 {
            let value = provider.get(&mut random);
            assert!(
                (0.0..=10.0).contains(&value),
                "Value {} is outside range [0.0, 10.0]",
                value
            );
        }
    }

    #[test]
    fn test_float_provider_enum_constant() {
        let mut random = RandomGenerator::Xoroshiro(
            crate::random::xoroshiro128::Xoroshiro::from_seed(get_seed()),
        );
        let provider = FloatProvider::Constant(7.5);

        assert_eq!(provider.get_min(), 7.5);
        assert_eq!(provider.get_max(), 7.5);
        assert_eq!(provider.get(&mut random), 7.5);
    }

    #[test]
    fn test_float_provider_enum_object() {
        let mut random = RandomGenerator::Xoroshiro(
            crate::random::xoroshiro128::Xoroshiro::from_seed(get_seed()),
        );
        let uniform = UniformFloatProvider::new(2.0, 8.0);
        let provider = FloatProvider::Object(NormalFloatProvider::Uniform(uniform));

        assert_eq!(provider.get_min(), 2.0);
        assert_eq!(provider.get_max(), 8.0);

        let value = provider.get(&mut random);
        assert!(
            (2.0..8.0).contains(&value),
            "Value {} is outside range [2.0, 8.0)",
            value
        );
    }
}
