use crate::random::RandomImpl;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use serde::Deserialize;
use syn::LitInt;

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum NormalIntProvider {
    #[serde(rename = "minecraft:constant")]
    Constant(ConstantIntProvider),
    #[serde(rename = "minecraft:uniform")]
    Uniform(UniformIntProvider),
    #[serde(rename = "minecraft:biased_to_bottom")]
    BiasedToBottom(BiasedToBottomIntProvider),
    #[serde(rename = "minecraft:clamped")]
    Clamped(ClampedIntProvider),
    #[serde(rename = "minecraft:clamped_normal")]
    ClampedNormal(ClampedNormalIntProvider),
    #[serde(rename = "minecraft:weighted_list")]
    WeightedList(WeightedListIntProvider),
}

impl ToTokens for NormalIntProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            NormalIntProvider::Constant(constant) => {
                tokens.extend(quote! {
                    NormalIntProvider::Constant(#constant)
                });
            }
            NormalIntProvider::Uniform(uniform) => {
                tokens.extend(quote! {
                    NormalIntProvider::Uniform(#uniform)
                });
            }
            NormalIntProvider::BiasedToBottom(biased) => {
                tokens.extend(quote! {
                    NormalIntProvider::BiasedToBottom(#biased)
                });
            }
            NormalIntProvider::Clamped(clamped) => {
                tokens.extend(quote! {
                    NormalIntProvider::Clamped(#clamped)
                });
            }
            NormalIntProvider::ClampedNormal(clamped_normal) => {
                tokens.extend(quote! {
                    NormalIntProvider::ClampedNormal(#clamped_normal)
                });
            }
            NormalIntProvider::WeightedList(weighted_list) => {
                tokens.extend(quote! {
                    NormalIntProvider::WeightedList(#weighted_list)
                });
            }
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum IntProvider {
    Object(NormalIntProvider),
    Constant(i32),
}

impl ToTokens for IntProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            IntProvider::Object(int_provider) => {
                tokens.extend(quote! {
                    IntProvider::Object(#int_provider)
                });
            }
            IntProvider::Constant(i) => tokens.extend(quote! {
                IntProvider::Constant(#i)
            }),
        }
    }
}

impl IntProvider {
    pub fn get_min(&self) -> i32 {
        match self {
            IntProvider::Object(int_provider) => match int_provider {
                NormalIntProvider::Constant(constant) => constant.get_min(),
                NormalIntProvider::Uniform(uniform) => uniform.get_min(),
                NormalIntProvider::BiasedToBottom(biased) => biased.get_min(),
                NormalIntProvider::Clamped(clamped) => clamped.get_min(),
                NormalIntProvider::ClampedNormal(clamped_normal) => clamped_normal.get_min(),
                NormalIntProvider::WeightedList(weighted_list) => weighted_list.get_min(),
            },
            IntProvider::Constant(i) => *i,
        }
    }

    pub fn get(&self, random: &mut impl RandomImpl) -> i32 {
        match self {
            IntProvider::Object(int_provider) => match int_provider {
                NormalIntProvider::Constant(constant) => constant.get(random),
                NormalIntProvider::Uniform(uniform) => uniform.get(random),
                NormalIntProvider::BiasedToBottom(biased) => biased.get(random),
                NormalIntProvider::Clamped(clamped) => clamped.get(random),
                NormalIntProvider::ClampedNormal(clamped_normal) => clamped_normal.get(random),
                NormalIntProvider::WeightedList(weighted_list) => weighted_list.get(random),
            },
            IntProvider::Constant(i) => *i,
        }
    }

    pub fn get_max(&self) -> i32 {
        match self {
            IntProvider::Object(int_provider) => match int_provider {
                NormalIntProvider::Constant(constant) => constant.get_max(),
                NormalIntProvider::Uniform(uniform) => uniform.get_max(),
                NormalIntProvider::BiasedToBottom(biased) => biased.get_max(),
                NormalIntProvider::Clamped(clamped) => clamped.get_max(),
                NormalIntProvider::ClampedNormal(clamped_normal) => clamped_normal.get_max(),
                NormalIntProvider::WeightedList(weighted_list) => weighted_list.get_max(),
            },
            IntProvider::Constant(i) => *i,
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ConstantIntProvider {
    value: i32,
}

impl ToTokens for ConstantIntProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let value = LitInt::new(&self.value.to_string(), Span::call_site());
        tokens.extend(quote! {
            ConstantIntProvider { value: #value }
        });
    }
}

impl ConstantIntProvider {
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    pub fn get_min(&self) -> i32 {
        self.value
    }

    pub fn get(&self, _random: &mut impl RandomImpl) -> i32 {
        self.value
    }

    pub fn get_max(&self) -> i32 {
        self.value
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct BiasedToBottomIntProvider {
    pub min_inclusive: i32,
    pub max_inclusive: i32,
}

impl ToTokens for BiasedToBottomIntProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let min_inclusive = LitInt::new(&self.min_inclusive.to_string(), Span::call_site());
        let max_inclusive = LitInt::new(&self.max_inclusive.to_string(), Span::call_site());
        tokens.extend(quote! {
            BiasedToBottomIntProvider { min_inclusive: #min_inclusive, max_inclusive: #max_inclusive }
        });
    }
}

impl BiasedToBottomIntProvider {
    pub fn new(min_inclusive: i32, max_inclusive: i32) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
        }
    }

    pub fn get_min(&self) -> i32 {
        self.min_inclusive
    }

    pub fn get(&self, random: &mut impl RandomImpl) -> i32 {
        // Similar to uniform but biased toward lower values
        // Uses triangular distribution with mode at min
        let range = (self.max_inclusive - self.min_inclusive + 1) as f64;
        let triangular = random.next_triangular(0.0, range);
        self.min_inclusive + (triangular.abs() as i32).min(self.max_inclusive - self.min_inclusive)
    }

    pub fn get_max(&self) -> i32 {
        self.max_inclusive
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ClampedIntProvider {
    source: Box<IntProvider>,
    min_inclusive: i32,
    max_inclusive: i32,
}

impl ToTokens for ClampedIntProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let source = &self.source;
        let min_inclusive = LitInt::new(&self.min_inclusive.to_string(), Span::call_site());
        let max_inclusive = LitInt::new(&self.max_inclusive.to_string(), Span::call_site());
        tokens.extend(quote! {
            ClampedIntProvider {
                source: Box::new(#source),
                min_inclusive: #min_inclusive,
                max_inclusive: #max_inclusive
            }
        });
    }
}

impl ClampedIntProvider {
    pub fn new(source: IntProvider, min_inclusive: i32, max_inclusive: i32) -> Self {
        Self {
            source: Box::new(source),
            min_inclusive,
            max_inclusive,
        }
    }

    pub fn get_min(&self) -> i32 {
        self.min_inclusive.max(self.source.get_min())
    }

    pub fn get(&self, random: &mut impl RandomImpl) -> i32 {
        self.source
            .get(random)
            .clamp(self.min_inclusive, self.max_inclusive)
    }

    pub fn get_max(&self) -> i32 {
        self.max_inclusive.min(self.source.get_max())
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ClampedNormalIntProvider {
    mean: f32,
    deviation: f32,
    min_inclusive: i32,
    max_inclusive: i32,
}

impl ToTokens for ClampedNormalIntProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mean = syn::LitFloat::new(&self.mean.to_string(), Span::call_site());
        let deviation = syn::LitFloat::new(&self.deviation.to_string(), Span::call_site());
        let min_inclusive = LitInt::new(&self.min_inclusive.to_string(), Span::call_site());
        let max_inclusive = LitInt::new(&self.max_inclusive.to_string(), Span::call_site());
        tokens.extend(quote! {
            ClampedNormalIntProvider {
                mean: #mean,
                deviation: #deviation,
                min_inclusive: #min_inclusive,
                max_inclusive: #max_inclusive
            }
        });
    }
}

impl ClampedNormalIntProvider {
    pub fn new(mean: f32, deviation: f32, min_inclusive: i32, max_inclusive: i32) -> Self {
        Self {
            mean,
            deviation,
            min_inclusive,
            max_inclusive,
        }
    }

    pub fn get_min(&self) -> i32 {
        self.min_inclusive
    }

    pub fn get(&self, random: &mut impl RandomImpl) -> i32 {
        // Generate normal distribution value and clamp to range
        let gaussian = random.next_gaussian() as f32;
        let value = (self.mean + gaussian * self.deviation).round() as i32;
        value.clamp(self.min_inclusive, self.max_inclusive)
    }

    pub fn get_max(&self) -> i32 {
        self.max_inclusive
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct WeightedEntry {
    data: IntProvider,
    weight: i32,
}

impl ToTokens for WeightedEntry {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let data = &self.data;
        let weight = LitInt::new(&self.weight.to_string(), Span::call_site());
        tokens.extend(quote! {
            WeightedEntry { data: #data, weight: #weight }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct WeightedListIntProvider {
    distribution: Vec<WeightedEntry>,
}

impl ToTokens for WeightedListIntProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let distribution = &self.distribution;
        tokens.extend(quote! {
            WeightedListIntProvider { distribution: vec![#(#distribution),*] }
        });
    }
}

impl WeightedListIntProvider {
    pub fn new(distribution: Vec<WeightedEntry>) -> Self {
        Self { distribution }
    }

    pub fn get_min(&self) -> i32 {
        self.distribution
            .iter()
            .map(|entry| entry.data.get_min())
            .min()
            .unwrap_or(0)
    }

    pub fn get(&self, random: &mut impl RandomImpl) -> i32 {
        if self.distribution.is_empty() {
            return 0;
        }

        // Calculate total weight
        let total_weight: i32 = self.distribution.iter().map(|entry| entry.weight).sum();

        if total_weight == 0 {
            return 0;
        }

        // Choose random weight
        let chosen_weight = random.next_bounded_i32(total_weight);
        let mut current_weight = 0;

        // Find the entry corresponding to the chosen weight
        for entry in &self.distribution {
            current_weight += entry.weight;
            if chosen_weight < current_weight {
                return entry.data.get(random);
            }
        }

        // Fallback to last entry
        self.distribution.last().unwrap().data.get(random)
    }

    pub fn get_max(&self) -> i32 {
        self.distribution
            .iter()
            .map(|entry| entry.data.get_max())
            .max()
            .unwrap_or(0)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct UniformIntProvider {
    pub min_inclusive: i32,
    pub max_inclusive: i32,
}

impl ToTokens for UniformIntProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let min_inclusive = LitInt::new(&self.min_inclusive.to_string(), Span::call_site());
        let max_inclusive = LitInt::new(&self.max_inclusive.to_string(), Span::call_site());

        tokens.extend(quote! {
            UniformIntProvider { min_inclusive: #min_inclusive, max_inclusive: #max_inclusive }
        });
    }
}

impl UniformIntProvider {
    pub fn new(min_inclusive: i32, max_inclusive: i32) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
        }
    }

    pub fn get_min(&self) -> i32 {
        self.min_inclusive
    }

    pub fn get(&self, random: &mut impl RandomImpl) -> i32 {
        random.next_inbetween_i32(self.min_inclusive, self.max_inclusive)
    }

    pub fn get_max(&self) -> i32 {
        self.max_inclusive
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::{RandomGenerator, get_seed};

    #[test]
    fn test_constant_int_provider() {
        let mut random = RandomGenerator::Xoroshiro(
            crate::random::xoroshiro128::Xoroshiro::from_seed(get_seed()),
        );
        let provider = ConstantIntProvider::new(42);

        assert_eq!(provider.get_min(), 42);
        assert_eq!(provider.get_max(), 42);
        assert_eq!(provider.get(&mut random), 42);
        assert_eq!(provider.get(&mut random), 42); // Should always return the same value
    }

    #[test]
    fn test_uniform_int_provider() {
        let mut random = RandomGenerator::Xoroshiro(
            crate::random::xoroshiro128::Xoroshiro::from_seed(get_seed()),
        );
        let provider = UniformIntProvider::new(1, 10);

        assert_eq!(provider.get_min(), 1);
        assert_eq!(provider.get_max(), 10);

        // Test that values are within range
        for _ in 0..100 {
            let value = provider.get(&mut random);
            assert!(
                (1..=10).contains(&value),
                "Value {} is outside range [1, 10]",
                value
            );
        }
    }

    #[test]
    fn test_biased_to_bottom_int_provider() {
        let mut random = RandomGenerator::Xoroshiro(
            crate::random::xoroshiro128::Xoroshiro::from_seed(get_seed()),
        );
        let provider = BiasedToBottomIntProvider::new(1, 20);

        assert_eq!(provider.get_min(), 1);
        assert_eq!(provider.get_max(), 20);

        // Test that values are within range (biased toward lower values)
        for _ in 0..100 {
            let value = provider.get(&mut random);
            assert!(
                (1..=20).contains(&value),
                "Value {} is outside range [1, 20]",
                value
            );
        }
    }

    #[test]
    fn test_clamped_normal_int_provider() {
        let mut random = RandomGenerator::Xoroshiro(
            crate::random::xoroshiro128::Xoroshiro::from_seed(get_seed()),
        );
        let provider = ClampedNormalIntProvider::new(5.0, 2.0, 1, 10);

        assert_eq!(provider.get_min(), 1);
        assert_eq!(provider.get_max(), 10);

        // Test that values are within range
        for _ in 0..100 {
            let value = provider.get(&mut random);
            assert!(
                (1..=10).contains(&value),
                "Value {} is outside range [1, 10]",
                value
            );
        }
    }

    #[test]
    fn test_clamped_int_provider() {
        let mut random = RandomGenerator::Xoroshiro(
            crate::random::xoroshiro128::Xoroshiro::from_seed(get_seed()),
        );
        let source =
            IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider::new(1, 100)));
        let provider = ClampedIntProvider::new(source, 5, 15);

        assert_eq!(provider.get_min(), 5);
        assert_eq!(provider.get_max(), 15);

        // Test that values are within clamped range
        for _ in 0..100 {
            let value = provider.get(&mut random);
            assert!(
                (5..=15).contains(&value),
                "Value {} is outside clamped range [5, 15]",
                value
            );
        }
    }

    #[test]
    fn test_weighted_list_int_provider() {
        let mut random = RandomGenerator::Xoroshiro(
            crate::random::xoroshiro128::Xoroshiro::from_seed(get_seed()),
        );

        let entries = vec![
            WeightedEntry {
                data: IntProvider::Constant(1),
                weight: 10,
            },
            WeightedEntry {
                data: IntProvider::Constant(2),
                weight: 20,
            },
            WeightedEntry {
                data: IntProvider::Constant(3),
                weight: 5,
            },
        ];

        let provider = WeightedListIntProvider::new(entries);

        assert_eq!(provider.get_min(), 1);
        assert_eq!(provider.get_max(), 3);

        // Test that values are from the weighted list
        for _ in 0..100 {
            let value = provider.get(&mut random);
            assert!(
                (1..=3).contains(&value),
                "Value {} is not from the weighted list",
                value
            );
        }
    }

    #[test]
    fn test_int_provider_enum_constant() {
        let mut random = RandomGenerator::Xoroshiro(
            crate::random::xoroshiro128::Xoroshiro::from_seed(get_seed()),
        );
        let provider = IntProvider::Constant(25);

        assert_eq!(provider.get_min(), 25);
        assert_eq!(provider.get_max(), 25);
        assert_eq!(provider.get(&mut random), 25);
    }

    #[test]
    fn test_int_provider_enum_object() {
        let mut random = RandomGenerator::Xoroshiro(
            crate::random::xoroshiro128::Xoroshiro::from_seed(get_seed()),
        );
        let uniform = UniformIntProvider::new(5, 15);
        let provider = IntProvider::Object(NormalIntProvider::Uniform(uniform));

        assert_eq!(provider.get_min(), 5);
        assert_eq!(provider.get_max(), 15);

        let value = provider.get(&mut random);
        assert!(
            (5..=15).contains(&value),
            "Value {} is outside range [5, 15]",
            value
        );
    }
}
