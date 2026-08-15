use std::fmt::{Debug, Formatter};

use pumpkin_codecs::{Decode, Encode};
use rustc_hash::FxHashMap;

use crate::attributes::{
    attribute_modifier::{AttributeOperation, ErasedAttributeModifier},
    lerp::Lerp,
};

/// Describes how values of an environment attribute are serialized, modified,
/// and interpolated.
pub struct AttributeType<T: Encode + Decode + 'static> {
    modifier_library: FxHashMap<AttributeOperation, ErasedAttributeModifier<T>>,
    keyframe_lerp: &'static dyn Lerp<T>,
    state_change_lerp: &'static dyn Lerp<T>,
    spatial_lerp: &'static dyn Lerp<T>,
    partial_tick_lerp: &'static dyn Lerp<T>,
}

impl<T: Encode + Decode + 'static> Debug for AttributeType<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttributeType")
            .field(
                "modifier_operations",
                &self.modifier_library.keys().collect::<Vec<_>>(),
            )
            .finish_non_exhaustive()
    }
}

impl<T: Encode + Decode + 'static> AttributeType<T> {
    #[must_use]
    pub fn new(
        modifier_library: FxHashMap<AttributeOperation, ErasedAttributeModifier<T>>,
        keyframe_lerp: &'static dyn Lerp<T>,
        state_change_lerp: &'static dyn Lerp<T>,
        spatial_lerp: &'static dyn Lerp<T>,
        partial_tick_lerp: &'static dyn Lerp<T>,
    ) -> Self {
        Self {
            modifier_library,
            keyframe_lerp,
            state_change_lerp,
            spatial_lerp,
            partial_tick_lerp,
        }
    }

    #[must_use]
    pub fn modifier_library(&self) -> &FxHashMap<AttributeOperation, ErasedAttributeModifier<T>> {
        &self.modifier_library
    }

    #[must_use]
    pub fn modifier(&self, operation: AttributeOperation) -> Option<&ErasedAttributeModifier<T>> {
        self.modifier_library.get(&operation)
    }

    #[must_use]
    pub const fn keyframe_lerp(&self) -> &'static dyn Lerp<T> {
        self.keyframe_lerp
    }

    #[must_use]
    pub const fn state_change_lerp(&self) -> &'static dyn Lerp<T> {
        self.state_change_lerp
    }

    #[must_use]
    pub const fn spatial_lerp(&self) -> &'static dyn Lerp<T> {
        self.spatial_lerp
    }

    #[must_use]
    pub const fn partial_tick_lerp(&self) -> &'static dyn Lerp<T> {
        self.partial_tick_lerp
    }
}
