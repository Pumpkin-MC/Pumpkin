use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};

use pumpkin_codecs::{DataResult, Decode, DynamicOps, Encode};

use crate::attributes::{
    attribute_range::{ANY_RANGE, AttributeRange},
    attribute_type::AttributeType,
};

/// Definition of a typed environment attribute.
///
/// Values are decoded through the attribute's [`AttributeType`] and then validated
/// by its independent [`AttributeRange`].
#[derive(Debug)]
pub struct EnvAttribute<T: Encode + Decode + 'static> {
    value_type: Arc<AttributeType<T>>,
    default_value: T,
    value_range: &'static dyn AttributeRange<T>,
    is_syncable: bool,
    is_positional: bool,
    is_spatially_interpolated: bool,
}

impl<T: Encode + Decode + 'static> EnvAttribute<T> {
    #[must_use]
    pub fn builder(value_type: Arc<AttributeType<T>>) -> EnvAttributeBuilder<T> {
        EnvAttributeBuilder::new(value_type)
    }

    #[must_use]
    pub fn value_type(&self) -> &AttributeType<T> {
        &self.value_type
    }

    #[must_use]
    pub const fn default_value(&self) -> &T {
        &self.default_value
    }

    #[must_use]
    pub const fn is_syncable(&self) -> bool {
        self.is_syncable
    }

    #[must_use]
    pub const fn is_positional(&self) -> bool {
        self.is_positional
    }

    #[must_use]
    pub const fn is_spatially_interpolated(&self) -> bool {
        self.is_spatially_interpolated
    }

    pub fn validate(&self, value: T) -> DataResult<T> {
        self.value_range.validate(value)
    }

    #[must_use]
    pub fn sanitize(&self, value: T) -> T {
        self.value_range.sanitize(value)
    }
}

impl<T: Encode + Decode + 'static> EnvAttribute<T> {
    pub fn parse<O: DynamicOps>(&self, input: O::Value, ops: &'static O) -> DataResult<T> {
        T::parse(input, ops).flat_map(|value| self.validate(value))
    }
}

impl<T: Encode + Decode + 'static> EnvAttribute<T> {
    pub fn encode_start<O: DynamicOps>(&self, value: &T, ops: &'static O) -> DataResult<O::Value> {
        value.encode_start(ops)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvAttributeBuildError {
    MissingDefaultValue,
}

impl Display for EnvAttributeBuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingDefaultValue => {
                f.write_str("environment attribute is missing a default value")
            }
        }
    }
}

impl std::error::Error for EnvAttributeBuildError {}

pub struct EnvAttributeBuilder<T: Encode + Decode + 'static> {
    value_type: Arc<AttributeType<T>>,
    default_value: Option<T>,
    value_range: &'static dyn AttributeRange<T>,
    is_syncable: bool,
    is_positional: bool,
    is_spatially_interpolated: bool,
}

impl<T: Encode + Decode + 'static> EnvAttributeBuilder<T> {
    #[must_use]
    pub fn new(value_type: Arc<AttributeType<T>>) -> Self {
        Self {
            value_type,
            default_value: None,
            value_range: &ANY_RANGE,
            is_syncable: false,
            is_positional: true,
            is_spatially_interpolated: false,
        }
    }

    #[must_use]
    pub fn default_value(mut self, default_value: T) -> Self {
        self.default_value = Some(default_value);
        self
    }

    #[must_use]
    pub fn value_range(mut self, value_range: &'static dyn AttributeRange<T>) -> Self {
        self.value_range = value_range;
        self
    }

    #[must_use]
    pub fn syncable(mut self) -> Self {
        self.is_syncable = true;
        self
    }

    #[must_use]
    pub fn not_positional(mut self) -> Self {
        self.is_positional = false;
        self
    }

    #[must_use]
    pub fn spatially_interpolated(mut self) -> Self {
        self.is_spatially_interpolated = true;
        self
    }

    pub fn build(self) -> Result<EnvAttribute<T>, EnvAttributeBuildError> {
        let Some(default_value) = self.default_value else {
            return Err(EnvAttributeBuildError::MissingDefaultValue);
        };

        Ok(EnvAttribute {
            value_type: self.value_type,
            default_value,
            value_range: self.value_range,
            is_syncable: self.is_syncable,
            is_positional: self.is_positional,
            is_spatially_interpolated: self.is_spatially_interpolated,
        })
    }
}
