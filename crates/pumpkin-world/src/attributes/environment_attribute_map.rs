use std::sync::LazyLock;

use pumpkin_codecs::{
    DataResult, Decode, DynamicOps, Encode, MapLike, struct_builder::StructBuilder,
};
use pumpkin_nbt::{compound::NbtCompound, nbt_ops::NbtOps, tag::NbtTag};
use pumpkin_registry::{Registry, RegistryResolvable};
use rustc_hash::FxHashMap;

use crate::attributes::{EnvironmentAttributeEntry, attribute_modifier::AttributeOperation};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EnvironmentAttributeMap {
    entries: FxHashMap<RegistryResolvable<EnvironmentAttributeEntry>, NbtTag>,
}

pub static EMPTY_ENVIRONMENT_ATTRIBUTE_MAP: LazyLock<EnvironmentAttributeMap> =
    LazyLock::new(EnvironmentAttributeMap::default);

impl EnvironmentAttributeMap {
    #[must_use]
    pub fn builder() -> EnvironmentAttributeMapBuilder {
        EnvironmentAttributeMapBuilder::default()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn get_raw(
        &self,
        attribute: &RegistryResolvable<EnvironmentAttributeEntry>,
    ) -> Option<&NbtTag> {
        self.entries.get(attribute)
    }

    #[must_use]
    pub fn contains(&self, attribute: &RegistryResolvable<EnvironmentAttributeEntry>) -> bool {
        self.entries.contains_key(attribute)
    }

    pub fn keys(&self) -> impl Iterator<Item = &RegistryResolvable<EnvironmentAttributeEntry>> {
        self.entries.keys()
    }

    pub fn validate(&self, registry: &dyn Registry) -> DataResult<()> {
        let mut result = DataResult::new_success(());
        for (attribute, input) in &self.entries {
            let resolved = match attribute.resolve(registry) {
                Ok(resolved) => resolved,
                Err(error) => {
                    result = result.add_message(&DataResult::<()>::new_error(error.to_string()));
                    continue;
                }
            };
            result = result.add_message(&resolved.decode_map_entry(input.clone()).map(drop));
        }
        result
    }

    pub fn apply_modifier<T: 'static>(
        &self,
        registry: &dyn Registry,
        attribute: &RegistryResolvable<EnvironmentAttributeEntry>,
        base_value: T,
    ) -> DataResult<T> {
        let Some(input) = self.entries.get(attribute) else {
            return DataResult::new_success(base_value);
        };
        let resolved = match attribute.resolve(registry) {
            Ok(resolved) => resolved,
            Err(error) => return DataResult::new_error(error.to_string()),
        };
        resolved
            .decode_map_entry(input.clone())
            .flat_map(|entry| resolved.apply_map_entry(entry, base_value))
    }

    pub fn filter_syncable(&self, registry: &dyn Registry) -> DataResult<Self> {
        let mut entries = FxHashMap::default();
        let mut result = DataResult::new_success(());
        for (attribute, input) in &self.entries {
            match attribute.resolve(registry) {
                Ok(resolved) if resolved.is_syncable() => {
                    entries.insert(attribute.clone(), input.clone());
                }
                Ok(_) => {}
                Err(error) => {
                    result = result.add_message(&DataResult::<()>::new_error(error.to_string()));
                }
            }
        }
        result.map(|()| Self { entries })
    }

    pub fn validate_only_positional(&self, registry: &dyn Registry) -> DataResult<()> {
        let mut result = DataResult::new_success(());
        for attribute in self.entries.keys() {
            match attribute.resolve(registry) {
                Ok(resolved) if !resolved.is_positional() => {
                    result = result.add_message(&DataResult::<()>::new_error(format!(
                        "environment attribute {} is not positional",
                        attribute.identifier()
                    )));
                }
                Ok(_) => {}
                Err(error) => {
                    result = result.add_message(&DataResult::<()>::new_error(error.to_string()));
                }
            }
        }
        result
    }
}

impl Encode for EnvironmentAttributeMap {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        let mut builder = ops.map_builder();
        for (attribute, value) in &self.entries {
            builder = builder.add_key_result_value_result(
                attribute.identifier().encode_start(ops),
                DataResult::new_success(NbtOps.convert_to(ops, value.clone())),
            );
        }
        builder.build(prefix)
    }
}

impl Decode for EnvironmentAttributeMap {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).flat_map(|map| {
            let mut entries = FxHashMap::default();
            let mut result = DataResult::new_success(());
            for (key, value) in map.iter() {
                let key = <pumpkin_util::identifier::Identifier as Decode>::parse(key, ops);
                result = result.add_message(&key);
                if let Some(key) = key.into_result_or_partial() {
                    entries.insert(
                        RegistryResolvable::new(key),
                        ops.convert_to(&NbtOps, value.clone()),
                    );
                }
            }
            result.with_complete_or_partial((Self { entries }, ops.empty()))
        })
    }
}

#[derive(Debug, Default)]
pub struct EnvironmentAttributeMapBuilder {
    entries: FxHashMap<RegistryResolvable<EnvironmentAttributeEntry>, NbtTag>,
}

impl EnvironmentAttributeMapBuilder {
    #[must_use]
    pub fn put_all(mut self, map: &EnvironmentAttributeMap) -> Self {
        self.entries.extend(map.entries.clone());
        self
    }

    pub fn set<T: 'static>(
        mut self,
        registry: &dyn Registry,
        attribute: RegistryResolvable<EnvironmentAttributeEntry>,
        value: &T,
    ) -> DataResult<Self> {
        let resolved = match attribute.resolve(registry) {
            Ok(resolved) => resolved,
            Err(error) => return DataResult::new_error(error.to_string()),
        };
        resolved.encode_value(value).map(|value| {
            self.entries.insert(attribute, value);
            self
        })
    }

    pub fn modify<A: 'static>(
        mut self,
        registry: &dyn Registry,
        attribute: RegistryResolvable<EnvironmentAttributeEntry>,
        operation: AttributeOperation,
        argument: &A,
    ) -> DataResult<Self> {
        let resolved = match attribute.resolve(registry) {
            Ok(resolved) => resolved,
            Err(error) => return DataResult::new_error(error.to_string()),
        };
        resolved
            .encode_modifier_argument(operation, argument)
            .map(|argument| {
                let mut compound = NbtCompound::new();
                compound.put_string("modifier", operation.as_str().to_string());
                compound.put("argument", argument);
                self.entries.insert(attribute, NbtTag::Compound(compound));
                self
            })
    }

    #[must_use]
    pub fn build(self) -> EnvironmentAttributeMap {
        EnvironmentAttributeMap {
            entries: self.entries,
        }
    }
}
