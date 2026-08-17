use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    sync::Arc,
};

use pumpkin_codecs::{DataResult, Decode, DynamicOps, Encode};
use pumpkin_nbt::{nbt_ops::NbtOps, tag::NbtTag};

use crate::attributes::{
    color::{ArgbColor, RgbColor},
    env_attribute::EnvAttribute,
    lerp::Lerp,
};

/// Applies a modifier argument to an environment attribute value.
pub trait TypedAttributeModifier<T: Encode + Decode + 'static, A: Encode + Decode + 'static>:
    Send + Sync + 'static
{
    fn modify(&self, target: T, argument: A) -> T;
    fn lerp(&self, attribute: &EnvAttribute<T>) -> &'static dyn Lerp<A>;
}

/// Type-erased storage for a [`TypedAttributeModifier<T, A>`].
///
/// Only the argument type is erased. The environment attribute value type `T`
/// remains statically known by [`AttributeType`](crate::attributes::AttributeType).
pub struct ErasedAttributeModifier<T: Encode + Decode + 'static> {
    operation: AttributeOperation,
    argument_type_id: TypeId,
    modifier: Arc<dyn Any + Send + Sync>,
    validate_argument: fn(NbtTag) -> DataResult<()>,
    encode_argument: fn(&dyn Any) -> DataResult<NbtTag>,
    apply: fn(&dyn Any, T, NbtTag) -> DataResult<T>,
    #[allow(clippy::type_complexity)]
    interpolate: fn(&dyn Any, &EnvAttribute<T>, f32, NbtTag, NbtTag) -> DataResult<NbtTag>,
    marker: PhantomData<fn(T) -> T>,
}

impl<T: Encode + Decode + 'static> Clone for ErasedAttributeModifier<T> {
    fn clone(&self) -> Self {
        Self {
            operation: self.operation,
            argument_type_id: self.argument_type_id,
            modifier: Arc::clone(&self.modifier),
            validate_argument: self.validate_argument,
            encode_argument: self.encode_argument,
            apply: self.apply,
            interpolate: self.interpolate,
            marker: PhantomData,
        }
    }
}

impl<T: Encode + Decode + 'static> ErasedAttributeModifier<T> {
    #[must_use]
    pub fn new<A: Encode + Decode + 'static, M: TypedAttributeModifier<T, A>>(
        operation: AttributeOperation,
        modifier: M,
    ) -> Self {
        Self {
            operation,
            argument_type_id: TypeId::of::<A>(),
            modifier: Arc::new(modifier),
            validate_argument: validate_argument::<A>,
            encode_argument: encode_argument::<A>,
            apply: apply::<T, A, M>,
            interpolate: interpolate::<T, A, M>,
            marker: PhantomData,
        }
    }

    #[must_use]
    pub const fn operation(&self) -> AttributeOperation {
        self.operation
    }

    #[must_use]
    pub const fn argument_type_id(&self) -> TypeId {
        self.argument_type_id
    }

    #[must_use]
    pub fn downcast<A: Encode + Decode + 'static, M: TypedAttributeModifier<T, A>>(
        &self,
    ) -> Option<&M> {
        (self.argument_type_id == TypeId::of::<A>())
            .then(|| self.modifier.downcast_ref::<M>())
            .flatten()
    }

    pub fn validate_argument(&self, input: NbtTag) -> DataResult<()> {
        (self.validate_argument)(input)
    }

    pub fn encode_argument(&self, argument: &dyn Any) -> DataResult<NbtTag> {
        (self.encode_argument)(argument)
    }

    pub fn apply(&self, target: T, input: NbtTag) -> DataResult<T> {
        (self.apply)(self.modifier.as_ref(), target, input)
    }

    pub fn interpolate(
        &self,
        attribute: &EnvAttribute<T>,
        t: f32,
        from: NbtTag,
        to: NbtTag,
    ) -> DataResult<NbtTag> {
        (self.interpolate)(self.modifier.as_ref(), attribute, t, from, to)
    }
}

fn validate_argument<A: Decode + 'static>(input: NbtTag) -> DataResult<()> {
    A::parse(input, &NbtOps).map(drop)
}

fn encode_argument<A: Encode + 'static>(argument: &dyn Any) -> DataResult<NbtTag> {
    let Some(argument) = argument.downcast_ref::<A>() else {
        return DataResult::new_error("attribute modifier argument type mismatch");
    };
    argument.encode_start(&NbtOps)
}

fn apply<
    T: Encode + Decode + 'static,
    A: Encode + Decode + 'static,
    M: TypedAttributeModifier<T, A>,
>(
    modifier: &dyn Any,
    target: T,
    input: NbtTag,
) -> DataResult<T> {
    let Some(modifier) = modifier.downcast_ref::<M>() else {
        return DataResult::new_error("attribute modifier type mismatch");
    };
    A::parse(input, &NbtOps).map(|argument| modifier.modify(target, argument))
}

fn interpolate<
    T: Encode + Decode + 'static,
    A: Encode + Decode + 'static,
    M: TypedAttributeModifier<T, A>,
>(
    modifier: &dyn Any,
    attribute: &EnvAttribute<T>,
    t: f32,
    from: NbtTag,
    to: NbtTag,
) -> DataResult<NbtTag> {
    let Some(modifier) = modifier.downcast_ref::<M>() else {
        return DataResult::new_error("attribute modifier type mismatch");
    };
    A::parse(from, &NbtOps)
        .apply_2(
            |from, to| modifier.lerp(attribute).lerp(t, from, to),
            A::parse(to, &NbtOps),
        )
        .flat_map(|value| value.encode_start(&NbtOps))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AttributeModifier {
    operation: AttributeOperation,
}

impl AttributeModifier {
    #[must_use]
    pub const fn new(operation: AttributeOperation) -> Self {
        Self { operation }
    }

    #[must_use]
    pub const fn operation(self) -> AttributeOperation {
        self.operation
    }

    #[must_use]
    pub const fn is_override(self) -> bool {
        matches!(self.operation, AttributeOperation::Override)
    }

    pub fn validate_argument(
        self,
        attribute: &crate::attributes::registry::EnvironmentAttributeEntry,
        argument: NbtTag,
    ) -> DataResult<()> {
        attribute.validate_modifier_argument(self.operation, argument)
    }

    pub fn interpolate_argument(
        self,
        attribute: &crate::attributes::registry::EnvironmentAttributeEntry,
        t: f32,
        from: NbtTag,
        to: NbtTag,
    ) -> DataResult<NbtTag> {
        attribute.interpolate_modifier_argument(self.operation, t, from, to)
    }
}

impl Default for AttributeModifier {
    fn default() -> Self {
        Self::new(AttributeOperation::Override)
    }
}

impl Encode for AttributeModifier {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        self.operation.encode(ops, prefix)
    }
}

impl Decode for AttributeModifier {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        AttributeOperation::decode(input, ops)
            .map(|(operation, remaining)| (Self::new(operation), remaining))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeOperation {
    Override,
    AlphaBlend,
    Add,
    Subtract,
    Multiply,
    BlendToGray,
    Minimum,
    Maximum,
    And,
    Nand,
    Or,
    Nor,
    Xor,
    Xnor,
}

impl<T: Encode + Decode + 'static> std::fmt::Debug for ErasedAttributeModifier<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErasedAttributeModifier")
            .field("operation", &self.operation)
            .finish_non_exhaustive()
    }
}

impl AttributeOperation {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Override => "override",
            Self::AlphaBlend => "alpha_blend",
            Self::Add => "add",
            Self::Subtract => "subtract",
            Self::Multiply => "multiply",
            Self::BlendToGray => "blend_to_gray",
            Self::Minimum => "minimum",
            Self::Maximum => "maximum",
            Self::And => "and",
            Self::Nand => "nand",
            Self::Or => "or",
            Self::Nor => "nor",
            Self::Xor => "xor",
            Self::Xnor => "xnor",
        }
    }

    #[must_use]
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "override" => Some(Self::Override),
            "alpha_blend" => Some(Self::AlphaBlend),
            "add" => Some(Self::Add),
            "subtract" => Some(Self::Subtract),
            "multiply" => Some(Self::Multiply),
            "blend_to_gray" => Some(Self::BlendToGray),
            "minimum" => Some(Self::Minimum),
            "maximum" => Some(Self::Maximum),
            "and" => Some(Self::And),
            "nand" => Some(Self::Nand),
            "or" => Some(Self::Or),
            "nor" => Some(Self::Nor),
            "xor" => Some(Self::Xor),
            "xnor" => Some(Self::Xnor),
            _ => None,
        }
    }
}

impl Encode for AttributeOperation {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        self.as_str().to_string().encode(ops, prefix)
    }
}

impl Decode for AttributeOperation {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        String::decode(input, ops).flat_map(|(value, remaining)| {
            Self::from_name(&value).map_or_else(
                || DataResult::new_error(format!("unknown attribute modifier: {value}")),
                |operation| DataResult::new_success((operation, remaining)),
            )
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Override;

impl<T: Encode + Decode + 'static> TypedAttributeModifier<T, T> for Override {
    fn modify(&self, _target: T, argument: T) -> T {
        argument
    }

    fn lerp(&self, attribute: &EnvAttribute<T>) -> &'static dyn Lerp<T> {
        attribute.value_type().keyframe_lerp()
    }
}

macro_rules! same_type_modifier {
    ($name:ident, $ty:ty, $body:expr) => {
        #[derive(Debug, Clone, Copy, Default)]
        pub struct $name;

        impl TypedAttributeModifier<$ty, $ty> for $name {
            fn modify(&self, target: $ty, argument: $ty) -> $ty {
                ($body)(target, argument)
            }

            fn lerp(&self, attribute: &EnvAttribute<$ty>) -> &'static dyn Lerp<$ty> {
                attribute.value_type().keyframe_lerp()
            }
        }
    };
}

same_type_modifier!(And, bool, |target: bool, argument: bool| target && argument);
same_type_modifier!(Nand, bool, |target: bool, argument: bool| !(target
    && argument));
same_type_modifier!(Or, bool, |target: bool, argument: bool| target || argument);
same_type_modifier!(Nor, bool, |target: bool, argument: bool| !(target
    || argument));
same_type_modifier!(Xor, bool, |target: bool, argument: bool| target ^ argument);
same_type_modifier!(Xnor, bool, |target: bool, argument: bool| !(target
    ^ argument));

same_type_modifier!(Add, f32, |target: f32, argument: f32| target + argument);
same_type_modifier!(Subtract, f32, |target: f32, argument: f32| target
    - argument);
same_type_modifier!(Multiply, f32, |target: f32, argument: f32| target
    * argument);
same_type_modifier!(Minimum, f32, |target: f32, argument: f32| target
    .min(argument));
same_type_modifier!(Maximum, f32, |target: f32, argument: f32| target
    .max(argument));

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatWithAlpha {
    pub value: f32,
    pub alpha: f32,
}

impl pumpkin_codecs::Encode for FloatWithAlpha {
    fn encode<O: pumpkin_codecs::DynamicOps>(
        &self,
        ops: &'static O,
        prefix: O::Value,
    ) -> pumpkin_codecs::DataResult<O::Value> {
        use pumpkin_codecs::codec::FieldEncode;
        use pumpkin_codecs::struct_builder::StructBuilder;

        self.value
            .encode_field("value", ops, ops.map_builder())
            .pipe(|builder| self.alpha.encode_field("alpha", ops, builder))
            .build(prefix)
    }
}

impl pumpkin_codecs::Decode for FloatWithAlpha {
    fn decode<O: pumpkin_codecs::DynamicOps>(
        input: O::Value,
        ops: &'static O,
    ) -> pumpkin_codecs::DataResult<(Self, O::Value)> {
        use pumpkin_codecs::codec::FieldDecode;

        ops.get_map(&input).flat_map(|map| {
            f32::decode_field::<O>("value", &map, ops).flat_map(|value| {
                f32::decode_field::<O>("alpha", &map, ops).flat_map(|alpha| {
                    if (0.0..=1.0).contains(&alpha) {
                        pumpkin_codecs::DataResult::new_success((
                            Self { value, alpha },
                            ops.empty(),
                        ))
                    } else {
                        pumpkin_codecs::DataResult::new_error("alpha must be between 0 and 1")
                    }
                })
            })
        })
    }
}

trait Pipe: Sized {
    fn pipe<R>(self, f: impl FnOnce(Self) -> R) -> R {
        f(self)
    }
}
impl<T> Pipe for T {}

fn lerp_float_with_alpha(t: f32, from: FloatWithAlpha, to: FloatWithAlpha) -> FloatWithAlpha {
    FloatWithAlpha {
        value: from.value + (to.value - from.value) * t,
        alpha: from.alpha + (to.alpha - from.alpha) * t,
    }
}

static FLOAT_WITH_ALPHA_LERP: crate::attributes::lerp::FnLerp<FloatWithAlpha> =
    crate::attributes::lerp::FnLerp::new(lerp_float_with_alpha);

#[derive(Debug, Clone, Copy, Default)]
pub struct AlphaBlend;

impl TypedAttributeModifier<f32, FloatWithAlpha> for AlphaBlend {
    fn modify(&self, target: f32, argument: FloatWithAlpha) -> f32 {
        target + (argument.value - target) * argument.alpha
    }

    fn lerp(&self, _attribute: &EnvAttribute<f32>) -> &'static dyn Lerp<FloatWithAlpha> {
        &FLOAT_WITH_ALPHA_LERP
    }
}

fn clamp_channel(value: i16) -> u8 {
    value.clamp(0, 255) as u8
}

fn multiply_channel(left: u8, right: u8) -> u8 {
    ((u16::from(left) * u16::from(right) + 127) / 255) as u8
}

impl TypedAttributeModifier<RgbColor, RgbColor> for Add {
    fn modify(&self, target: RgbColor, argument: RgbColor) -> RgbColor {
        RgbColor::from_rgb(
            clamp_channel(i16::from(target.r()) + i16::from(argument.r())),
            clamp_channel(i16::from(target.g()) + i16::from(argument.g())),
            clamp_channel(i16::from(target.b()) + i16::from(argument.b())),
        )
    }

    fn lerp(&self, attribute: &EnvAttribute<RgbColor>) -> &'static dyn Lerp<RgbColor> {
        attribute.value_type().keyframe_lerp()
    }
}

impl TypedAttributeModifier<RgbColor, RgbColor> for Subtract {
    fn modify(&self, target: RgbColor, argument: RgbColor) -> RgbColor {
        RgbColor::from_rgb(
            clamp_channel(i16::from(target.r()) - i16::from(argument.r())),
            clamp_channel(i16::from(target.g()) - i16::from(argument.g())),
            clamp_channel(i16::from(target.b()) - i16::from(argument.b())),
        )
    }

    fn lerp(&self, attribute: &EnvAttribute<RgbColor>) -> &'static dyn Lerp<RgbColor> {
        attribute.value_type().keyframe_lerp()
    }
}

impl TypedAttributeModifier<RgbColor, RgbColor> for Multiply {
    fn modify(&self, target: RgbColor, argument: RgbColor) -> RgbColor {
        RgbColor::from_rgb(
            multiply_channel(target.r(), argument.r()),
            multiply_channel(target.g(), argument.g()),
            multiply_channel(target.b(), argument.b()),
        )
    }

    fn lerp(&self, attribute: &EnvAttribute<RgbColor>) -> &'static dyn Lerp<RgbColor> {
        attribute.value_type().keyframe_lerp()
    }
}

fn alpha_blend_rgb(target: RgbColor, argument: ArgbColor) -> RgbColor {
    let alpha = f32::from(argument.a()) / 255.0;
    let blend = |from: u8, to: u8| -> u8 {
        (f32::from(from) + (f32::from(to) - f32::from(from)) * alpha).round() as u8
    };
    RgbColor::from_rgb(
        blend(target.r(), argument.r()),
        blend(target.g(), argument.g()),
        blend(target.b(), argument.b()),
    )
}

impl TypedAttributeModifier<RgbColor, ArgbColor> for AlphaBlend {
    fn modify(&self, target: RgbColor, argument: ArgbColor) -> RgbColor {
        alpha_blend_rgb(target, argument)
    }

    fn lerp(&self, _attribute: &EnvAttribute<RgbColor>) -> &'static dyn Lerp<ArgbColor> {
        &ARGB_LINEAR_LERP
    }
}

impl TypedAttributeModifier<ArgbColor, RgbColor> for Add {
    fn modify(&self, target: ArgbColor, argument: RgbColor) -> ArgbColor {
        ArgbColor::from_argb(
            target.a(),
            clamp_channel(i16::from(target.r()) + i16::from(argument.r())),
            clamp_channel(i16::from(target.g()) + i16::from(argument.g())),
            clamp_channel(i16::from(target.b()) + i16::from(argument.b())),
        )
    }

    fn lerp(&self, _attribute: &EnvAttribute<ArgbColor>) -> &'static dyn Lerp<RgbColor> {
        &RGB_LINEAR_LERP
    }
}

impl TypedAttributeModifier<ArgbColor, RgbColor> for Subtract {
    fn modify(&self, target: ArgbColor, argument: RgbColor) -> ArgbColor {
        ArgbColor::from_argb(
            target.a(),
            clamp_channel(i16::from(target.r()) - i16::from(argument.r())),
            clamp_channel(i16::from(target.g()) - i16::from(argument.g())),
            clamp_channel(i16::from(target.b()) - i16::from(argument.b())),
        )
    }

    fn lerp(&self, _attribute: &EnvAttribute<ArgbColor>) -> &'static dyn Lerp<RgbColor> {
        &RGB_LINEAR_LERP
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgbMultiplyArgument {
    Rgb(RgbColor),
    Argb(ArgbColor),
}

impl pumpkin_codecs::Encode for ArgbMultiplyArgument {
    fn encode<O: pumpkin_codecs::DynamicOps>(
        &self,
        ops: &'static O,
        prefix: O::Value,
    ) -> pumpkin_codecs::DataResult<O::Value> {
        match self {
            Self::Rgb(value) => value.encode(ops, prefix),
            Self::Argb(value) => value.encode(ops, prefix),
        }
    }
}

impl pumpkin_codecs::Decode for ArgbMultiplyArgument {
    fn decode<O: pumpkin_codecs::DynamicOps>(
        input: O::Value,
        ops: &'static O,
    ) -> pumpkin_codecs::DataResult<(Self, O::Value)> {
        let rgb = RgbColor::parse(input.clone(), ops);
        if let Some(value) = rgb.into_result() {
            return pumpkin_codecs::DataResult::new_success((Self::Rgb(value), ops.empty()));
        }

        ArgbColor::parse(input, ops).map(|value| (Self::Argb(value), ops.empty()))
    }
}

fn lerp_argb_multiply_argument(
    t: f32,
    from: ArgbMultiplyArgument,
    to: ArgbMultiplyArgument,
) -> ArgbMultiplyArgument {
    match (from, to) {
        (ArgbMultiplyArgument::Rgb(from), ArgbMultiplyArgument::Rgb(to)) => {
            ArgbMultiplyArgument::Rgb(lerp_rgb(t, from, to))
        }
        (from, to) => {
            let to_argb = |value| match value {
                ArgbMultiplyArgument::Rgb(value) => {
                    ArgbColor::from_argb(255, value.r(), value.g(), value.b())
                }
                ArgbMultiplyArgument::Argb(value) => value,
            };
            ArgbMultiplyArgument::Argb(lerp_argb(t, to_argb(from), to_argb(to)))
        }
    }
}

static ARGB_MULTIPLY_ARGUMENT_LERP: crate::attributes::lerp::FnLerp<ArgbMultiplyArgument> =
    crate::attributes::lerp::FnLerp::new(lerp_argb_multiply_argument);

impl TypedAttributeModifier<ArgbColor, ArgbMultiplyArgument> for Multiply {
    fn modify(&self, target: ArgbColor, argument: ArgbMultiplyArgument) -> ArgbColor {
        match argument {
            ArgbMultiplyArgument::Rgb(argument) => ArgbColor::from_argb(
                target.a(),
                multiply_channel(target.r(), argument.r()),
                multiply_channel(target.g(), argument.g()),
                multiply_channel(target.b(), argument.b()),
            ),
            ArgbMultiplyArgument::Argb(argument) => ArgbColor::from_argb(
                multiply_channel(target.a(), argument.a()),
                multiply_channel(target.r(), argument.r()),
                multiply_channel(target.g(), argument.g()),
                multiply_channel(target.b(), argument.b()),
            ),
        }
    }

    fn lerp(
        &self,
        _attribute: &EnvAttribute<ArgbColor>,
    ) -> &'static dyn Lerp<ArgbMultiplyArgument> {
        &ARGB_MULTIPLY_ARGUMENT_LERP
    }
}

impl TypedAttributeModifier<ArgbColor, ArgbColor> for AlphaBlend {
    fn modify(&self, target: ArgbColor, argument: ArgbColor) -> ArgbColor {
        let alpha = f32::from(argument.a()) / 255.0;
        let blend = |from: u8, to: u8| -> u8 {
            (f32::from(from) + (f32::from(to) - f32::from(from)) * alpha).round() as u8
        };
        ArgbColor::from_argb(
            blend(target.a(), argument.a()),
            blend(target.r(), argument.r()),
            blend(target.g(), argument.g()),
            blend(target.b(), argument.b()),
        )
    }

    fn lerp(&self, _attribute: &EnvAttribute<ArgbColor>) -> &'static dyn Lerp<ArgbColor> {
        &ARGB_LINEAR_LERP
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GrayBlend {
    pub brightness: f32,
    pub factor: f32,
}

impl pumpkin_codecs::Encode for GrayBlend {
    fn encode<O: pumpkin_codecs::DynamicOps>(
        &self,
        ops: &'static O,
        prefix: O::Value,
    ) -> pumpkin_codecs::DataResult<O::Value> {
        use pumpkin_codecs::codec::FieldEncode;
        use pumpkin_codecs::struct_builder::StructBuilder;

        self.brightness
            .encode_field("brightness", ops, ops.map_builder())
            .pipe(|builder| self.factor.encode_field("factor", ops, builder))
            .build(prefix)
    }
}

impl pumpkin_codecs::Decode for GrayBlend {
    fn decode<O: pumpkin_codecs::DynamicOps>(
        input: O::Value,
        ops: &'static O,
    ) -> pumpkin_codecs::DataResult<(Self, O::Value)> {
        use pumpkin_codecs::codec::FieldDecode;

        ops.get_map(&input).flat_map(|map| {
            f32::decode_field::<O>("brightness", &map, ops).flat_map(|brightness| {
                f32::decode_field::<O>("factor", &map, ops).flat_map(|factor| {
                    if (0.0..=1.0).contains(&brightness) && (0.0..=1.0).contains(&factor) {
                        pumpkin_codecs::DataResult::new_success((
                            Self { brightness, factor },
                            ops.empty(),
                        ))
                    } else {
                        pumpkin_codecs::DataResult::new_error(
                            "brightness and factor must be between 0 and 1",
                        )
                    }
                })
            })
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BlendToGray;

fn gray_rgb(color: RgbColor, argument: GrayBlend) -> RgbColor {
    let gray = argument.brightness
        * (0.3 * f32::from(color.r()) + 0.59 * f32::from(color.g()) + 0.11 * f32::from(color.b()));
    let factor = argument.factor.clamp(0.0, 1.0);
    let blend = |channel: u8| -> u8 {
        (f32::from(channel) + (gray - f32::from(channel)) * factor)
            .clamp(0.0, 255.0)
            .round() as u8
    };
    RgbColor::from_rgb(blend(color.r()), blend(color.g()), blend(color.b()))
}

impl TypedAttributeModifier<RgbColor, GrayBlend> for BlendToGray {
    fn modify(&self, target: RgbColor, argument: GrayBlend) -> RgbColor {
        gray_rgb(target, argument)
    }

    fn lerp(&self, _attribute: &EnvAttribute<RgbColor>) -> &'static dyn Lerp<GrayBlend> {
        &GRAY_BLEND_LERP
    }
}

impl TypedAttributeModifier<ArgbColor, GrayBlend> for BlendToGray {
    fn modify(&self, target: ArgbColor, argument: GrayBlend) -> ArgbColor {
        let rgb = gray_rgb(target.rgb(), argument);
        ArgbColor::from_argb(target.a(), rgb.r(), rgb.g(), rgb.b())
    }

    fn lerp(&self, _attribute: &EnvAttribute<ArgbColor>) -> &'static dyn Lerp<GrayBlend> {
        &GRAY_BLEND_LERP
    }
}

fn lerp_rgb(t: f32, from: RgbColor, to: RgbColor) -> RgbColor {
    let blend = |from: u8, to: u8| -> u8 {
        (f32::from(from) + (f32::from(to) - f32::from(from)) * t).round() as u8
    };
    RgbColor::from_rgb(
        blend(from.r(), to.r()),
        blend(from.g(), to.g()),
        blend(from.b(), to.b()),
    )
}

fn lerp_argb(t: f32, from: ArgbColor, to: ArgbColor) -> ArgbColor {
    let blend = |from: u8, to: u8| -> u8 {
        (f32::from(from) + (f32::from(to) - f32::from(from)) * t).round() as u8
    };
    ArgbColor::from_argb(
        blend(from.a(), to.a()),
        blend(from.r(), to.r()),
        blend(from.g(), to.g()),
        blend(from.b(), to.b()),
    )
}

fn lerp_gray_blend(t: f32, from: GrayBlend, to: GrayBlend) -> GrayBlend {
    GrayBlend {
        brightness: from.brightness + (to.brightness - from.brightness) * t,
        factor: from.factor + (to.factor - from.factor) * t,
    }
}

pub static RGB_LINEAR_LERP: crate::attributes::lerp::FnLerp<RgbColor> =
    crate::attributes::lerp::FnLerp::new(lerp_rgb);
pub static ARGB_LINEAR_LERP: crate::attributes::lerp::FnLerp<ArgbColor> =
    crate::attributes::lerp::FnLerp::new(lerp_argb);
static GRAY_BLEND_LERP: crate::attributes::lerp::FnLerp<GrayBlend> =
    crate::attributes::lerp::FnLerp::new(lerp_gray_blend);
