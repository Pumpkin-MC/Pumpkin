use std::{
    any::Any,
    fmt::{self, Debug},
    sync::Arc,
};

use pumpkin_codecs::{DataResult, Decode, DynamicOps, Encode, json_ops::JsonOps};
use pumpkin_nbt::{nbt_ops::NbtOps, tag::NbtTag};
use pumpkin_registry::{
    ROOT, Registry, RegistryBuilder, bootstrap::RegistryEntry, bootstrap_provider,
};
use pumpkin_util::{identifier::Identifier, random::RandomGenerator};
use serde_json::Value;

mod biased_to_bottom;
mod clamped;
mod clamped_normal;
mod constant;
mod trapezoid;
mod uniform;
mod weighted_list;

pub use biased_to_bottom::BiasedToBottomIntProvider;
pub use clamped::ClampedIntProvider;
pub use clamped_normal::ClampedNormalIntProvider;
pub use constant::ConstantIntProvider;
pub use trapezoid::TrapezoidIntProvider;
pub use uniform::UniformIntProvider;
pub use weighted_list::{WeightedEntry, WeightedListIntProvider};

pub trait IntProvider: Any + Debug + Send + Sync {
    fn get_min(&self) -> i32;
    fn get(&self, random: &mut RandomGenerator) -> i32;
    fn get_max(&self) -> i32;
    fn provider_type(&self) -> Identifier;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Clone)]
pub enum IntProviderValue {
    Static(&'static dyn IntProvider),
    Owned(Arc<dyn IntProvider>),
}

impl Debug for IntProviderValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.provider().fmt(f)
    }
}

impl IntProviderValue {
    #[must_use]
    pub fn new<T: IntProvider + 'static>(provider: T) -> Self {
        Self::Owned(Arc::new(provider))
    }

    #[must_use]
    pub const fn from_static(provider: &'static dyn IntProvider) -> Self {
        Self::Static(provider)
    }

    #[must_use]
    pub fn constant(value: i32) -> Self {
        Self::new(ConstantIntProvider::new(value))
    }

    #[must_use]
    pub fn provider(&self) -> &dyn IntProvider {
        match self {
            Self::Static(provider) => *provider,
            Self::Owned(provider) => provider.as_ref(),
        }
    }

    #[must_use]
    pub fn downcast_ref<T: IntProvider + 'static>(&self) -> Option<&T> {
        self.provider().as_any().downcast_ref::<T>()
    }

    #[must_use]
    pub fn get_min(&self) -> i32 {
        self.provider().get_min()
    }

    pub fn get(&self, random: &mut RandomGenerator) -> i32 {
        self.provider().get(random)
    }

    #[must_use]
    pub fn get_max(&self) -> i32 {
        self.provider().get_max()
    }
}

impl<T: IntProvider + 'static> From<T> for IntProviderValue {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

type DecodeProvider = fn(NbtTag) -> DataResult<Arc<dyn IntProvider>>;
type EncodeProvider = fn(&dyn IntProvider) -> DataResult<NbtTag>;

pub trait IntProviderDecode: IntProvider + Sized + Decode {}
pub trait IntProviderEncode: IntProvider + Sized + Encode {}

impl<T: IntProvider + Sized + Decode> IntProviderDecode for T {}
impl<T: IntProvider + Sized + Encode> IntProviderEncode for T {}

#[derive(Clone)]
pub struct IntProviderType {
    decode: DecodeProvider,
    encode: EncodeProvider,
}

impl IntProviderType {
    pub const fn new<T: IntProviderDecode + IntProviderEncode + 'static>() -> Self {
        Self {
            decode: decode_provider::<T>,
            encode: encode_provider::<T>,
        }
    }

    pub fn decode<O: DynamicOps>(
        &self,
        input: O::Value,
        ops: &'static O,
    ) -> DataResult<Arc<dyn IntProvider>> {
        let input = ops.convert_to(&NbtOps, input);
        (self.decode)(input)
    }

    pub fn encode<O: DynamicOps>(
        &self,
        input: &dyn IntProvider,
        ops: &'static O,
    ) -> DataResult<O::Value> {
        (self.encode)(input).map(|tag| NbtOps.convert_to(ops, tag))
    }
}

fn decode_provider<T: IntProviderDecode + 'static>(
    input: NbtTag,
) -> DataResult<Arc<dyn IntProvider>> {
    T::parse(input, &NbtOps).map(|provider| Arc::new(provider) as Arc<dyn IntProvider>)
}

fn encode_provider<T: IntProviderEncode + 'static>(input: &dyn IntProvider) -> DataResult<NbtTag> {
    let Some(input) = input.as_any().downcast_ref::<T>() else {
        return DataResult::new_error(format!(
            "Int provider type mismatch: expected {}",
            std::any::type_name::<T>()
        ));
    };
    input.encode_start(&NbtOps)
}

fn provider_type(identifier: &Identifier) -> Option<IntProviderType> {
    let root = ROOT.get()?;
    let registry_id = root.get_id(&Identifier::vanilla_static("int_provider_type"))?;
    let registry = {
        let value = root.by_id_erased(registry_id)?;
        value.downcast_ref::<Arc<dyn Registry>>()?.clone()
    };
    let provider_id = registry.get_id(identifier)?;
    let provider_type = {
        let provider = registry.by_id_erased(provider_id)?;
        provider.downcast_ref::<IntProviderType>()?.clone()
    };
    Some(provider_type)
}

impl Encode for IntProviderValue {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        if let Some(constant) = self.downcast_ref::<ConstantIntProvider>() {
            return constant.value.encode(ops, prefix);
        }

        let identifier = self.provider().provider_type();
        let Some(provider_type) = provider_type(&identifier) else {
            return DataResult::new_error(format!("Unknown int provider type: {identifier}"));
        };

        provider_type
            .encode(self.provider(), &JsonOps)
            .flat_map(|mut encoded| {
                let Value::Object(ref mut map) = encoded else {
                    return DataResult::new_error(format!(
                        "Int provider codec for {identifier} did not encode an object"
                    ));
                };
                map.insert("type".to_string(), Value::String(identifier.to_string()));
                DataResult::new_success(JsonOps.convert_to(ops, encoded))
            })
    }
}

impl Decode for IntProviderValue {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        if let Some(value) = i32::parse(input.clone(), ops).into_result() {
            return DataResult::new_success((Self::constant(value), ops.empty()));
        }

        let json = ops.convert_to(&JsonOps, input);
        let Value::Object(map) = &json else {
            return DataResult::new_error(format!("Expected int provider object, got {json}"));
        };
        let Some(Value::String(kind)) = map.get("type") else {
            return DataResult::new_error("Missing int provider type");
        };
        let Ok(identifier) = Identifier::parse(kind) else {
            return DataResult::new_error(format!("Invalid int provider type: {kind}"));
        };
        let Some(provider_type) = provider_type(&identifier) else {
            return DataResult::new_error(format!("Unknown int provider type: {identifier}"));
        };

        provider_type
            .decode(json, &JsonOps)
            .map(|provider| (Self::Owned(provider), ops.empty()))
    }
}

bootstrap_provider! {
    INT_PROVIDER_TYPES: IntProviderType => "minecraft:int_provider_type",
    || {
        vec![
            RegistryEntry::new(Identifier::vanilla_static("constant"), IntProviderType::new::<ConstantIntProvider>()),
            RegistryEntry::new(Identifier::vanilla_static("uniform"), IntProviderType::new::<UniformIntProvider>()),
            RegistryEntry::new(Identifier::vanilla_static("biased_to_bottom"), IntProviderType::new::<BiasedToBottomIntProvider>()),
            RegistryEntry::new(Identifier::vanilla_static("clamped"), IntProviderType::new::<ClampedIntProvider>()),
            RegistryEntry::new(Identifier::vanilla_static("trapezoid"), IntProviderType::new::<TrapezoidIntProvider>()),
            RegistryEntry::new(Identifier::vanilla_static("clamped_normal"), IntProviderType::new::<ClampedNormalIntProvider>()),
            RegistryEntry::new(Identifier::vanilla_static("weighted_list"), IntProviderType::new::<WeightedListIntProvider>()),
        ]
    }
}

bootstrap_provider! {
    INT_PROVIDER_TYPE_REGISTRY: Arc<dyn Registry> => "minecraft:root",
    || {
        let Ok(registry) = RegistryBuilder::<IntProviderType>::frozen(
            &Identifier::vanilla_static("int_provider_type"),
        ) else {
            return Vec::new();
        };
        vec![RegistryEntry::new(
            Identifier::vanilla_static("int_provider_type"),
            registry.arc_dyn(),
        )]
    }
}

trait Pipe: Sized {
    fn pipe<R>(self, f: impl FnOnce(Self) -> R) -> R {
        f(self)
    }
}
impl<T> Pipe for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_registry::{BOOTSTRAP, bootstrap::BootstrapManager};
    use serde_json::json;

    fn initialize_registry() {
        BOOTSTRAP.get_or_init(BootstrapManager::new);
        ROOT.get_or_init(|| {
            RegistryBuilder::<Arc<dyn Registry>>::frozen(&Identifier::vanilla_static("root"))
                .expect("root registry must bootstrap")
        });
    }

    fn decode_json(input: Value) -> IntProviderValue {
        initialize_registry();
        IntProviderValue::parse(input, &JsonOps)
            .into_result()
            .expect("int provider must decode")
    }

    fn encode_json(provider: &IntProviderValue) -> Value {
        initialize_registry();
        provider
            .encode_start(&JsonOps)
            .into_result()
            .expect("int provider must encode")
    }

    #[test]
    fn constant_round_trip_uses_compact_form() {
        let decoded = decode_json(json!(7));
        assert_eq!(decoded.get_min(), 7);
        assert_eq!(decoded.get_max(), 7);
        assert_eq!(encode_json(&decoded), json!(7));
    }

    #[test]
    fn uniform_round_trip() {
        let input = json!({
            "type": "minecraft:uniform",
            "min_inclusive": 0,
            "max_inclusive": 7
        });
        let decoded = decode_json(input.clone());
        let provider = decoded
            .downcast_ref::<UniformIntProvider>()
            .expect("uniform provider");
        assert_eq!(provider.min_inclusive, 0);
        assert_eq!(provider.max_inclusive, 7);
        assert_eq!(encode_json(&decoded), input);
    }

    #[test]
    fn clamped_nested_round_trip() {
        let input = json!({
            "type": "minecraft:clamped",
            "source": {
                "type": "minecraft:uniform",
                "min_inclusive": -20,
                "max_inclusive": 20
            },
            "min_inclusive": -5,
            "max_inclusive": 5
        });
        let decoded = decode_json(input.clone());
        let provider = decoded
            .downcast_ref::<ClampedIntProvider>()
            .expect("clamped provider");
        assert!(
            provider
                .source
                .downcast_ref::<UniformIntProvider>()
                .is_some()
        );
        assert_eq!(encode_json(&decoded), input);
    }

    #[test]
    fn weighted_list_round_trip() {
        let input = json!({
            "type": "minecraft:weighted_list",
            "distribution": [
                { "data": 1, "weight": 2 },
                {
                    "data": {
                        "type": "minecraft:uniform",
                        "min_inclusive": 3,
                        "max_inclusive": 6
                    },
                    "weight": 4
                }
            ]
        });
        let decoded = decode_json(input.clone());
        let provider = decoded
            .downcast_ref::<WeightedListIntProvider>()
            .expect("weighted provider");
        assert_eq!(provider.distribution.len(), 2);
        assert_eq!(encode_json(&decoded), input);
    }
}
