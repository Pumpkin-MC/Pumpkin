use std::{
    any::{Any, TypeId, type_name},
    sync::{Arc, OnceLock},
};

use pumpkin_codecs::{Decode, Encode};
use rustc_hash::FxHashMap;

use crate::attributes::{
    ArgbColor, AttributeType, RgbColor,
    attribute_modifier::{
        Add, AlphaBlend, And, ArgbMultiplyArgument, AttributeOperation, BlendToGray,
        ErasedAttributeModifier, Maximum, Minimum, Multiply, Nand, Nor, Or, Override, Subtract,
        Xnor, Xor,
    },
    lerp::{FnLerp, Lerp},
    value_types::{
        Activity, AmbientParticles, AmbientSounds, BackgroundMusic, BedRule, MoonPhase,
        ParticleOptions, TriState,
    },
};

#[derive(Debug)]
struct StepLerp;

impl<T> Lerp<T> for StepLerp {
    fn lerp(&self, t: f32, from: T, to: T) -> T {
        if t < 1.0 { from } else { to }
    }
}

static STEP_LERP: StepLerp = StepLerp;

fn linear_float(t: f32, from: f32, to: f32) -> f32 {
    from + (to - from) * t
}

fn angle_float(t: f32, from: f32, to: f32) -> f32 {
    let delta = (to - from + 180.0).rem_euclid(360.0) - 180.0;
    (from + delta * t).rem_euclid(360.0)
}

static FLOAT_LERP: FnLerp<f32> = FnLerp::new(linear_float);
static ANGLE_LERP: FnLerp<f32> = FnLerp::new(angle_float);

fn override_library<T: Encode + Decode + 'static>()
-> FxHashMap<AttributeOperation, ErasedAttributeModifier<T>> {
    let mut modifiers = FxHashMap::default();
    modifiers.insert(
        AttributeOperation::Override,
        ErasedAttributeModifier::new::<T, _>(AttributeOperation::Override, Override),
    );
    modifiers
}

fn boolean_library() -> FxHashMap<AttributeOperation, ErasedAttributeModifier<bool>> {
    let mut modifiers = override_library::<bool>();
    modifiers.insert(
        AttributeOperation::And,
        ErasedAttributeModifier::new::<bool, _>(AttributeOperation::And, And),
    );
    modifiers.insert(
        AttributeOperation::Nand,
        ErasedAttributeModifier::new::<bool, _>(AttributeOperation::Nand, Nand),
    );
    modifiers.insert(
        AttributeOperation::Or,
        ErasedAttributeModifier::new::<bool, _>(AttributeOperation::Or, Or),
    );
    modifiers.insert(
        AttributeOperation::Nor,
        ErasedAttributeModifier::new::<bool, _>(AttributeOperation::Nor, Nor),
    );
    modifiers.insert(
        AttributeOperation::Xor,
        ErasedAttributeModifier::new::<bool, _>(AttributeOperation::Xor, Xor),
    );
    modifiers.insert(
        AttributeOperation::Xnor,
        ErasedAttributeModifier::new::<bool, _>(AttributeOperation::Xnor, Xnor),
    );
    modifiers
}

fn float_library() -> FxHashMap<AttributeOperation, ErasedAttributeModifier<f32>> {
    use crate::attributes::attribute_modifier::FloatWithAlpha;

    let mut modifiers = override_library::<f32>();
    modifiers.insert(
        AttributeOperation::Add,
        ErasedAttributeModifier::new::<f32, _>(AttributeOperation::Add, Add),
    );
    modifiers.insert(
        AttributeOperation::Subtract,
        ErasedAttributeModifier::new::<f32, _>(AttributeOperation::Subtract, Subtract),
    );
    modifiers.insert(
        AttributeOperation::Multiply,
        ErasedAttributeModifier::new::<f32, _>(AttributeOperation::Multiply, Multiply),
    );
    modifiers.insert(
        AttributeOperation::Minimum,
        ErasedAttributeModifier::new::<f32, _>(AttributeOperation::Minimum, Minimum),
    );
    modifiers.insert(
        AttributeOperation::Maximum,
        ErasedAttributeModifier::new::<f32, _>(AttributeOperation::Maximum, Maximum),
    );
    modifiers.insert(
        AttributeOperation::AlphaBlend,
        ErasedAttributeModifier::new::<FloatWithAlpha, _>(
            AttributeOperation::AlphaBlend,
            AlphaBlend,
        ),
    );
    modifiers
}

fn rgb_library() -> FxHashMap<AttributeOperation, ErasedAttributeModifier<RgbColor>> {
    use crate::attributes::attribute_modifier::GrayBlend;

    let mut modifiers = override_library::<RgbColor>();
    modifiers.insert(
        AttributeOperation::Add,
        ErasedAttributeModifier::new::<RgbColor, _>(AttributeOperation::Add, Add),
    );
    modifiers.insert(
        AttributeOperation::Subtract,
        ErasedAttributeModifier::new::<RgbColor, _>(AttributeOperation::Subtract, Subtract),
    );
    modifiers.insert(
        AttributeOperation::Multiply,
        ErasedAttributeModifier::new::<RgbColor, _>(AttributeOperation::Multiply, Multiply),
    );
    modifiers.insert(
        AttributeOperation::AlphaBlend,
        ErasedAttributeModifier::new::<ArgbColor, _>(AttributeOperation::AlphaBlend, AlphaBlend),
    );
    modifiers.insert(
        AttributeOperation::BlendToGray,
        ErasedAttributeModifier::new::<GrayBlend, _>(AttributeOperation::BlendToGray, BlendToGray),
    );
    modifiers
}

fn argb_library() -> FxHashMap<AttributeOperation, ErasedAttributeModifier<ArgbColor>> {
    use crate::attributes::attribute_modifier::GrayBlend;

    let mut modifiers = override_library::<ArgbColor>();
    modifiers.insert(
        AttributeOperation::Add,
        ErasedAttributeModifier::new::<RgbColor, _>(AttributeOperation::Add, Add),
    );
    modifiers.insert(
        AttributeOperation::Subtract,
        ErasedAttributeModifier::new::<RgbColor, _>(AttributeOperation::Subtract, Subtract),
    );
    modifiers.insert(
        AttributeOperation::Multiply,
        ErasedAttributeModifier::new::<ArgbMultiplyArgument, _>(
            AttributeOperation::Multiply,
            Multiply,
        ),
    );
    modifiers.insert(
        AttributeOperation::AlphaBlend,
        ErasedAttributeModifier::new::<ArgbColor, _>(AttributeOperation::AlphaBlend, AlphaBlend),
    );
    modifiers.insert(
        AttributeOperation::BlendToGray,
        ErasedAttributeModifier::new::<GrayBlend, _>(AttributeOperation::BlendToGray, BlendToGray),
    );
    modifiers
}

fn simple_type<T: Encode + Decode + 'static>() -> Arc<AttributeType<T>> {
    Arc::new(AttributeType::new(
        override_library(),
        &STEP_LERP,
        &STEP_LERP,
        &STEP_LERP,
        &STEP_LERP,
    ))
}

pub fn boolean_type() -> Arc<AttributeType<bool>> {
    static VALUE: OnceLock<Arc<AttributeType<bool>>> = OnceLock::new();
    Arc::clone(VALUE.get_or_init(|| {
        Arc::new(AttributeType::new(
            boolean_library(),
            &STEP_LERP,
            &STEP_LERP,
            &STEP_LERP,
            &STEP_LERP,
        ))
    }))
}

pub fn float_type() -> Arc<AttributeType<f32>> {
    static VALUE: OnceLock<Arc<AttributeType<f32>>> = OnceLock::new();
    Arc::clone(VALUE.get_or_init(|| {
        Arc::new(AttributeType::new(
            float_library(),
            &FLOAT_LERP,
            &FLOAT_LERP,
            &FLOAT_LERP,
            &FLOAT_LERP,
        ))
    }))
}

pub fn angle_type() -> Arc<AttributeType<f32>> {
    static VALUE: OnceLock<Arc<AttributeType<f32>>> = OnceLock::new();
    Arc::clone(VALUE.get_or_init(|| {
        Arc::new(AttributeType::new(
            float_library(),
            &ANGLE_LERP,
            &ANGLE_LERP,
            &ANGLE_LERP,
            &ANGLE_LERP,
        ))
    }))
}

pub fn rgb_type() -> Arc<AttributeType<RgbColor>> {
    use crate::attributes::attribute_modifier::RGB_LINEAR_LERP;

    static VALUE: OnceLock<Arc<AttributeType<RgbColor>>> = OnceLock::new();
    Arc::clone(VALUE.get_or_init(|| {
        Arc::new(AttributeType::new(
            rgb_library(),
            &RGB_LINEAR_LERP,
            &RGB_LINEAR_LERP,
            &RGB_LINEAR_LERP,
            &RGB_LINEAR_LERP,
        ))
    }))
}

pub fn argb_type() -> Arc<AttributeType<ArgbColor>> {
    use crate::attributes::attribute_modifier::ARGB_LINEAR_LERP;

    static VALUE: OnceLock<Arc<AttributeType<ArgbColor>>> = OnceLock::new();
    Arc::clone(VALUE.get_or_init(|| {
        Arc::new(AttributeType::new(
            argb_library(),
            &ARGB_LINEAR_LERP,
            &ARGB_LINEAR_LERP,
            &ARGB_LINEAR_LERP,
            &ARGB_LINEAR_LERP,
        ))
    }))
}

macro_rules! cached_simple_type {
    ($name:ident, $ty:ty) => {
        pub fn $name() -> Arc<AttributeType<$ty>> {
            static VALUE: OnceLock<Arc<AttributeType<$ty>>> = OnceLock::new();
            Arc::clone(VALUE.get_or_init(simple_type::<$ty>))
        }
    };
}

cached_simple_type!(tri_state_type, TriState);
cached_simple_type!(moon_phase_type, MoonPhase);
cached_simple_type!(activity_type, Activity);
cached_simple_type!(bed_rule_type, BedRule);
cached_simple_type!(particle_type, ParticleOptions);
cached_simple_type!(ambient_particles_type, AmbientParticles);
cached_simple_type!(background_music_type, BackgroundMusic);
cached_simple_type!(ambient_sounds_type, AmbientSounds);

pub struct AttributeTypeEntry {
    value_type_id: TypeId,
    value_type_name: &'static str,
    value: Arc<dyn Any + Send + Sync>,
}

impl std::fmt::Debug for AttributeTypeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttributeTypeEntry")
            .field("value_type_name", &self.value_type_name)
            .finish_non_exhaustive()
    }
}

impl AttributeTypeEntry {
    #[must_use]
    pub fn new<T: Encode + Decode + Send + Sync + 'static>(value: Arc<AttributeType<T>>) -> Self {
        Self {
            value_type_id: TypeId::of::<T>(),
            value_type_name: type_name::<T>(),
            value,
        }
    }

    #[must_use]
    pub const fn value_type_id(&self) -> TypeId {
        self.value_type_id
    }

    #[must_use]
    pub const fn value_type_name(&self) -> &'static str {
        self.value_type_name
    }

    #[must_use]
    pub fn downcast<T: Encode + Decode + Send + Sync + 'static>(
        &self,
    ) -> Option<&AttributeType<T>> {
        (self.value_type_id == TypeId::of::<T>())
            .then(|| self.value.downcast_ref::<AttributeType<T>>())
            .flatten()
    }
}
