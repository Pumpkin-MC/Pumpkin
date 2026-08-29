use std::{
    collections::BTreeMap,
    fs,
    hash::{DefaultHasher, Hash, Hasher},
};

use heck::ToShoutySnakeCase;

use proc_macro2::{Punct, Spacing, Span, TokenStream};
use quote::{ToTokens, TokenStreamExt, quote};
use serde::Deserialize;
use syn::Ident;

/// Wraps an `f32` to provide a bitwise-exact `Hash` implementation for use as a map key.
#[derive(Clone, Copy)]
struct HashableF32(pub f32);

// Normally this is bad, but we just care about checking if components are the same
impl Hash for HashableF32 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_le_bytes().hash(state);
    }
}

impl ToTokens for HashableF32 {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let value = self.0;
        if value.is_finite() {
            value.to_tokens(tokens);
        } else {
            tokens.append(Ident::new("f32", Span::call_site()));
            tokens.append(Punct::new(':', Spacing::Joint));
            tokens.append(Punct::new(':', Spacing::Joint));
            if value.is_nan() {
                tokens.append(Ident::new("NAN", Span::call_site()));
            } else if value > 0.0 {
                tokens.append(Ident::new("INFINITY", Span::call_site()));
            } else {
                tokens.append(Ident::new("NEG_INFINITY", Span::call_site()));
            }
        }
    }
}

impl<'de> Deserialize<'de> for HashableF32 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        f32::deserialize(deserializer).map(Self)
    }
}

/// Wraps an `f64` to provide a bitwise-exact `Hash` implementation for use as a map key.
#[derive(Clone, Copy)]
struct HashableF64(pub f64);

// Normally this is bad, but we just care about checking if components are the same
impl Hash for HashableF64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_le_bytes().hash(state);
    }
}

impl ToTokens for HashableF64 {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let value = self.0;
        if value.is_finite() {
            value.to_tokens(tokens);
        } else {
            tokens.append(Ident::new("f64", Span::call_site()));
            tokens.append(Punct::new(':', Spacing::Joint));
            tokens.append(Punct::new(':', Spacing::Joint));
            if value.is_nan() {
                tokens.append(Ident::new("NAN", Span::call_site()));
            } else if value > 0.0 {
                tokens.append(Ident::new("INFINITY", Span::call_site()));
            } else {
                tokens.append(Ident::new("NEG_INFINITY", Span::call_site()));
            }
        }
    }
}

impl<'de> Deserialize<'de> for HashableF64 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        f64::deserialize(deserializer).map(Self)
    }
}

/// Deserialized representation of a cubic spline used inside density functions.
#[derive(Deserialize, Hash, Clone)]
#[serde(tag = "_type", content = "value")]
enum SplineRepr {
    /// A standard multipoint spline evaluated against a location density function.
    #[serde(rename(deserialize = "standard"))]
    Standard {
        /// The density function that drives the spline location axis.
        #[serde(rename(deserialize = "locationFunction"))]
        location_function: Box<DensityFunctionRepr>,
        /// X-axis sample locations for each spline segment.
        locations: Box<[HashableF32]>,
        /// Nested spline values at each sample location.
        values: Box<[Self]>,
        /// Derivative (tangent) values at each sample location.
        derivatives: Box<[HashableF32]>,
    },
    /// A spline that returns a single constant value regardless of input.
    #[serde(rename(deserialize = "fixed"))]
    Fixed {
        /// The constant output value.
        value: HashableF32,
    },
}

impl SplineRepr {
    fn get_token_stream(
        &self,
        stack: &mut Vec<TokenStream>,
        nodes: &mut Vec<DensityFunctionRepr>,
        hash_to_index_map: &mut BTreeMap<u64, usize>,
    ) -> TokenStream {
        match self {
            Self::Fixed { value } => {
                quote! {
                    SplineRepr::Fixed {value: #value}
                }
            }
            Self::Standard {
                location_function,
                locations,
                values,
                derivatives,
            } => {
                assert_eq!(values.len(), locations.len());
                assert_eq!(values.len(), derivatives.len());

                let points = locations
                    .into_iter()
                    .zip(values)
                    .zip(derivatives)
                    .map(|((location, value), derivative)| (location, value, derivative));

                let function_index =
                    location_function.get_index_for_component(stack, nodes, hash_to_index_map);

                let point_reprs = points
                    .into_iter()
                    .map(|(location, value, derivative)| {
                        let value_repr = value.get_token_stream(stack, nodes, hash_to_index_map);

                        quote! {
                            SplinePoint {
                                location: #location,
                                value: &#value_repr,
                                derivative: #derivative,
                            }
                        }
                    })
                    .collect::<Vec<_>>();

                quote! {
                    SplineRepr::Standard {
                        location_function_index: #function_index,
                        points: &[#(#point_reprs),*],
                    }
                }
            }
        }
    }
}

/// Arithmetic operation applied to two density function arguments.
#[derive(Deserialize, Hash, Copy, Clone)]
enum BinaryOperation {
    /// Adds the two arguments.
    #[serde(rename(deserialize = "ADD"))]
    Add,
    /// Multiplies the two arguments.
    #[serde(rename(deserialize = "MUL"))]
    Mul,
    /// Takes the minimum of the two arguments.
    #[serde(rename(deserialize = "MIN"))]
    Min,
    /// Takes the maximum of the two arguments.
    #[serde(rename(deserialize = "MAX"))]
    Max,
}

impl BinaryOperation {
    /// Emits the token stream for this binary operation variant.
    fn get_token_stream(&self) -> TokenStream {
        match self {
            Self::Add => {
                quote! {
                    BinaryOperation::Add
                }
            }
            Self::Mul => {
                quote! {
                    BinaryOperation::Mul
                }
            }
            Self::Min => {
                quote! {
                    BinaryOperation::Min
                }
            }
            Self::Max => {
                quote! {
                    BinaryOperation::Max
                }
            }
        }
    }
}

/// Arithmetic operation applied to a single density function argument and a scalar.
#[derive(Deserialize, Hash, Copy, Clone)]
enum LinearOperation {
    /// Adds the scalar argument to the density value.
    #[serde(rename(deserialize = "ADD"))]
    Add,
    /// Multiplies the density value by the scalar argument.
    #[serde(rename(deserialize = "MUL"))]
    Mul,
}

impl LinearOperation {
    /// Emits the token stream for this linear operation variant.
    fn into_token_stream(self) -> TokenStream {
        match self {
            Self::Add => {
                quote! {
                    LinearOperation::Add
                }
            }
            Self::Mul => {
                quote! {
                    LinearOperation::Mul
                }
            }
        }
    }
}

/// Single-argument transformation applied to a density value.
#[derive(Deserialize, Hash, Copy, Clone)]
enum UnaryOperation {
    /// Returns the reciprocal (1/x) of the value.
    #[serde(rename(deserialize = "INVERT"))]
    Invert,
    /// Returns the absolute value.
    #[serde(rename(deserialize = "ABS"))]
    Abs,
    /// Squares the value.
    #[serde(rename(deserialize = "SQUARE"))]
    Square,
    /// Cubes the value.
    #[serde(rename(deserialize = "CUBE"))]
    Cube,
    /// Halves the value only if it is negative, passes it through otherwise.
    #[serde(rename(deserialize = "HALF_NEGATIVE"))]
    HalfNegative,
    /// Quarters the value only if it is negative, passes it through otherwise.
    #[serde(rename(deserialize = "QUARTER_NEGATIVE"))]
    QuarterNegative,
    /// Applies a smooth cubic "squeeze" mapping to `[-1, 1]`.
    #[serde(rename(deserialize = "SQUEEZE"))]
    Squeeze,
}

impl UnaryOperation {
    /// Emits the token stream for this unary operation variant.
    fn into_token_stream(self) -> TokenStream {
        match self {
            Self::Invert => {
                quote! {
                    UnaryOperation::Invert
                }
            }
            Self::Abs => {
                quote! {
                    UnaryOperation::Abs
                }
            }
            Self::Square => {
                quote! {
                    UnaryOperation::Square
                }
            }
            Self::Cube => {
                quote! {
                    UnaryOperation::Cube
                }
            }
            Self::HalfNegative => {
                quote! {
                    UnaryOperation::HalfNegative
                }
            }
            Self::QuarterNegative => {
                quote! {
                    UnaryOperation::QuarterNegative
                }
            }
            Self::Squeeze => {
                quote! {
                    UnaryOperation::Squeeze
                }
            }
        }
    }
}

/// Caching or interpolation wrapper applied around an inner density function.
#[derive(Copy, Clone, Deserialize, PartialEq, Eq, Hash)]
enum WrapperType {
    /// Trilinear interpolation over noise cells.
    Interpolated,
    /// Flat (2D) per-column cache.
    #[serde(rename(deserialize = "FlatCache"))]
    CacheFlat,
    /// 2D (XZ) per-chunk cache.
    Cache2D,
    /// Evaluate once and cache for the entire invocation.
    CacheOnce,
    /// Per-noise-cell cache.
    CellCache,
}

impl WrapperType {
    /// Emits the token stream for this wrapper type variant.
    fn into_token_stream(self) -> TokenStream {
        match self {
            Self::Interpolated => {
                quote! {
                    WrapperType::Interpolated
                }
            }
            Self::CacheFlat => {
                quote! {
                    WrapperType::CacheFlat
                }
            }
            Self::Cache2D => {
                quote! {
                    WrapperType::Cache2D
                }
            }
            Self::CacheOnce => {
                quote! {
                    WrapperType::CacheOnce
                }
            }
            Self::CellCache => {
                quote! {
                    WrapperType::CellCache
                }
            }
        }
    }
}

/// Deserialized parameters for a simple noise density function.
#[derive(Deserialize, Hash, Clone)]
struct NoiseData {
    /// Resource location ID of the noise generator.
    #[serde(rename(deserialize = "noise"))]
    noise_id: String,
    /// Horizontal (XZ) frequency scale factor.
    #[serde(rename(deserialize = "xzScale"))]
    xz_scale: HashableF64,
    /// Vertical (Y) frequency scale factor.
    #[serde(rename(deserialize = "yScale"))]
    y_scale: HashableF64,
}

/// Deserialized parameters for a shifted-noise density function.
#[derive(Deserialize, Hash, Clone)]
struct ShiftedNoiseData {
    /// Horizontal (XZ) frequency scale factor.
    #[serde(rename(deserialize = "xzScale"))]
    xz_scale: HashableF64,
    /// Vertical (Y) frequency scale factor.
    #[serde(rename(deserialize = "yScale"))]
    y_scale: HashableF64,
    /// Resource location ID of the noise generator.
    #[serde(rename(deserialize = "noise"))]
    noise_id: String,
}

/// Deserialized parameters for the interpolated noise sampler density function.
#[derive(Deserialize, Hash, Clone)]
struct InterpolatedNoiseSamplerData {
    /// XZ scale after cell-size scaling has been applied.
    #[serde(rename(deserialize = "scaledXzScale"))]
    scaled_xz_scale: HashableF64,
    /// Y scale after cell-size scaling has been applied.
    #[serde(rename(deserialize = "scaledYScale"))]
    scaled_y_scale: HashableF64,
    /// Horizontal cell-size factor.
    #[serde(rename(deserialize = "xzFactor"))]
    xz_factor: HashableF64,
    /// Vertical cell-size factor.
    #[serde(rename(deserialize = "yFactor"))]
    y_factor: HashableF64,
    /// Multiplier applied to smear-scale for blending.
    #[serde(rename(deserialize = "smearScaleMultiplier"))]
    smear_scale_multiplier: HashableF64,
    /// Maximum possible output value.
    #[serde(rename(deserialize = "maxValue"))]
    max_value: HashableF64,
}

/// Deserialized parameters for a clamped Y-gradient density function.
#[derive(Deserialize, Hash, Clone)]
struct ClampedYGradientData {
    /// Y coordinate at which the gradient starts.
    #[serde(rename(deserialize = "fromY"))]
    from_y: i32,
    /// Y coordinate at which the gradient ends.
    #[serde(rename(deserialize = "toY"))]
    to_y: i32,
    /// Density value at `from_y`.
    #[serde(rename(deserialize = "fromValue"))]
    from_value: HashableF64,
    /// Density value at `to_y`.
    #[serde(rename(deserialize = "toValue"))]
    to_value: HashableF64,
}

/// Deserialized parameters for a binary density function operation.
#[derive(Deserialize, Hash, Clone)]
struct BinaryData {
    /// The binary operation to apply to the two arguments.
    #[serde(rename(deserialize = "type"))]
    operation: BinaryOperation,
    /// Minimum possible output value (informational, not enforced).
    #[serde(rename(deserialize = "minValue"))]
    min_value: HashableF64,
    /// Maximum possible output value (informational, not enforced).
    #[serde(rename(deserialize = "maxValue"))]
    max_value: HashableF64,
}

/// Deserialized parameters for a linear density function operation.
#[derive(Deserialize, Hash, Clone)]
struct LinearData {
    /// The linear operation (add or multiply) to apply with `argument`.
    #[serde(rename(deserialize = "specificType"))]
    operation: LinearOperation,
    /// The scalar operand for the linear operation.
    argument: HashableF64,
    /// Minimum possible output value (informational, not enforced).
    #[serde(rename(deserialize = "minValue"))]
    min_value: HashableF64,
    /// Maximum possible output value (informational, not enforced).
    #[serde(rename(deserialize = "maxValue"))]
    max_value: HashableF64,
}

#[derive(Deserialize, Hash, Clone)]
struct FindTopSurfaceData {
    /// Lower Y bound to stop searching at.
    #[serde(rename(deserialize = "lowerBound"))]
    lower_bound: i32,
    /// Step size between Y levels when searching.
    #[serde(rename(deserialize = "cellHeight"))]
    cell_height: i32,
}

/// Deserialized parameters for a unary density function transformation.
#[derive(Deserialize, Hash, Clone)]
struct UnaryData {
    /// The unary transformation to apply.
    #[serde(rename(deserialize = "type"))]
    operation: UnaryOperation,
    /// Minimum possible output value (informational, not enforced).
    #[serde(rename(deserialize = "minValue"))]
    min_value: HashableF64,
    /// Maximum possible output value (informational, not enforced).
    #[serde(rename(deserialize = "maxValue"))]
    max_value: HashableF64,
}

/// Deserialized parameters for a clamp density function.
#[derive(Deserialize, Hash, Clone)]
struct ClampData {
    /// Lower bound of the clamp range.
    #[serde(rename(deserialize = "minValue"))]
    min_value: HashableF64,
    /// Upper bound of the clamp range.
    #[serde(rename(deserialize = "maxValue"))]
    max_value: HashableF64,
}

/// Deserialized range bounds for the `RangeChoice` density function.
#[derive(Deserialize, Hash, Clone)]
struct RangeChoiceData {
    /// Inclusive lower bound of the "in-range" interval.
    #[serde(rename(deserialize = "minInclusive"))]
    min_inclusive: HashableF64,
    /// Exclusive upper bound of the "in-range" interval.
    #[serde(rename(deserialize = "maxExclusive"))]
    max_exclusive: HashableF64,
}

/// Deserialized output-range metadata for a spline density function.
#[derive(Deserialize, Hash, Clone)]
struct SplineData {
    /// Minimum possible output value of the spline.
    #[serde(rename(deserialize = "minValue"))]
    min_value: HashableF64,
    /// Maximum possible output value of the spline.
    #[serde(rename(deserialize = "maxValue"))]
    max_value: HashableF64,
}

/// Deserialized representation of any density function node in the noise router tree.
#[derive(Deserialize, Hash, Clone)]
#[serde(tag = "_class", content = "value")]
enum DensityFunctionRepr {
    /// Placeholder that leaves space for world-structure contributions at runtime.
    // This is a placeholder for leaving space for world structures
    Beardifier,
    /// Blending alpha factor, initialized from a world seed at runtime.
    // These functions are initialized by a seed at runtime
    BlendAlpha,
    /// Blending offset factor, initialized from a world seed at runtime.
    BlendOffset,
    /// Blends the density from an inner function.
    BlendDensity {
        /// The inner density function to blend.
        input: Box<Self>,
    },
    FindTopSurface {
        /// The density function to test for solidity.
        density: Box<Self>,
        /// The density function providing the upper Y bound.
        #[serde(rename(deserialize = "upperBound"))]
        upper_bound: Box<Self>,
        /// Lower bound and step size parameters.
        #[serde(flatten)]
        data: FindTopSurfaceData,
    },
    /// End-islands noise sampler, seeded at runtime.
    EndIslands,
    /// A standard noise sampler.
    Noise {
        /// Noise parameters (ID and frequency scales).
        #[serde(flatten)]
        data: NoiseData,
    },
    /// Horizontal shift noise along the A axis.
    ShiftA {
        /// Noise ID for the offset generator.
        #[serde(rename(deserialize = "offsetNoise"))]
        noise_id: String,
    },
    /// Horizontal shift noise along the B axis.
    ShiftB {
        /// Noise ID for the offset generator.
        #[serde(rename(deserialize = "offsetNoise"))]
        noise_id: String,
    },
    /// A noise sample shifted in XYZ by three inner density functions.
    ShiftedNoise {
        /// Density function providing the X shift.
        #[serde(rename(deserialize = "shiftX"))]
        shift_x: Box<Self>,
        /// Density function providing the Y shift.
        #[serde(rename(deserialize = "shiftY"))]
        shift_y: Box<Self>,
        /// Density function providing the Z shift.
        #[serde(rename(deserialize = "shiftZ"))]
        shift_z: Box<Self>,
        /// Noise ID and frequency scales for the shifted sample.
        #[serde(flatten)]
        data: ShiftedNoiseData,
    },
    /// A trilinearly interpolated multi-octave noise sampler.
    InterpolatedNoiseSampler {
        /// Sampler configuration parameters.
        #[serde(flatten)]
        data: InterpolatedNoiseSamplerData,
    },
    /// Scales an input density function by a cave/tunnel rarity curve.
    #[serde(rename(deserialize = "IntervalSelect"))]
    IntervalSelect {
        input: Box<Self>,
        thresholds: Box<[HashableF64]>,
        functions: Box<[Self]>,
    },
    /// Wraps an inner function with a caching or interpolation layer.
    // The wrapped function is wrapped in a new wrapper at runtime
    #[serde(rename(deserialize = "Wrapping"))]
    Wrapper {
        /// The inner density function to wrap.
        #[serde(rename(deserialize = "wrapped"))]
        input: Box<Self>,
        /// The type of wrapper to apply.
        #[serde(rename(deserialize = "type"))]
        wrapper: WrapperType,
    },
    /// Returns a constant density value.
    // These functions are unchanged except possibly for internal functions
    Constant {
        /// The constant output value.
        value: HashableF64,
    },
    /// A linear gradient clamped between two Y levels.
    #[serde(rename(deserialize = "YClampedGradient"))]
    ClampedYGradient {
        /// Gradient parameters.
        #[serde(flatten)]
        data: ClampedYGradientData,
    },
    /// Applies a binary operation to two inner density functions.
    #[serde(rename(deserialize = "BinaryOperation"))]
    Binary {
        /// First argument density function.
        argument1: Box<Self>,
        /// Second argument density function.
        argument2: Box<Self>,
        /// Operation type and output range metadata.
        #[serde(flatten)]
        data: BinaryData,
    },
    /// Applies a linear (add or multiply) operation with a scalar.
    #[serde(rename(deserialize = "LinearOperation"))]
    Linear {
        /// The inner density function to transform.
        input: Box<Self>,
        /// Operation type, scalar argument, and output range metadata.
        #[serde(flatten)]
        data: LinearData,
    },
    /// Applies a unary transformation to an inner density function.
    #[serde(rename(deserialize = "UnaryOperation"))]
    Unary {
        /// The inner density function to transform.
        input: Box<Self>,
        /// Transformation type and output range metadata.
        #[serde(flatten)]
        data: UnaryData,
    },
    /// Clamps an inner density function's output to a range.
    Clamp {
        /// The inner density function to clamp.
        input: Box<Self>,
        /// Clamp range parameters.
        #[serde(flatten)]
        data: ClampData,
    },
    /// Selects one of two density functions based on whether the input is within a range.
    RangeChoice {
        /// The density function to evaluate for range testing.
        input: Box<Self>,
        /// Density function used when `input` is within the range.
        #[serde(rename(deserialize = "whenInRange"))]
        when_in_range: Box<Self>,
        /// Density function used when `input` is outside the range.
        #[serde(rename(deserialize = "whenOutOfRange"))]
        when_out_range: Box<Self>,
        /// Range bounds and output metadata.
        #[serde(flatten)]
        data: RangeChoiceData,
    },
    /// Evaluates a cubic spline over a location density function.
    Spline {
        /// The spline structure.
        spline: SplineRepr,
        /// Output range metadata.
        #[serde(flatten)]
        data: SplineData,
    },
}

impl DensityFunctionRepr {
    /// Simplifies and constant-folds the density function tree at codegen time.
    fn optimize(&mut self) {
        match self {
            Self::BlendDensity { input } => input.optimize(),
            Self::FindTopSurface {
                density,
                upper_bound,
                ..
            } => {
                density.optimize();
                upper_bound.optimize();
            }
            Self::ShiftedNoise {
                shift_x,
                shift_y,
                shift_z,
                ..
            } => {
                shift_x.optimize();
                shift_y.optimize();
                shift_z.optimize();
            }
            Self::IntervalSelect {
                input, functions, ..
            } => {
                input.optimize();
                for f in functions.iter_mut() {
                    f.optimize();
                }
            }
            Self::Wrapper { input, .. } => input.optimize(),
            Self::RangeChoice {
                input,
                when_in_range,
                when_out_range,
                ..
            } => {
                input.optimize();
                when_in_range.optimize();
                when_out_range.optimize();
            }
            Self::Linear { input, data } => {
                input.optimize();
                if let Self::Constant { value } = &**input {
                    let val = match data.operation {
                        LinearOperation::Add => value.0 + data.argument.0,
                        LinearOperation::Mul => value.0 * data.argument.0,
                    };
                    *self = Self::Constant {
                        value: HashableF64(val),
                    };
                    return;
                }
                match data.operation {
                    LinearOperation::Add => {
                        if data.argument.0 == 0.0 {
                            *self = *input.clone();
                        }
                    }
                    LinearOperation::Mul => {
                        if data.argument.0 == 1.0 {
                            *self = *input.clone();
                        } else if data.argument.0 == 0.0 {
                            *self = Self::Constant {
                                value: HashableF64(0.0),
                            };
                        }
                    }
                }
            }
            Self::Binary {
                argument1,
                argument2,
                data,
            } => {
                argument1.optimize();
                argument2.optimize();
                if let (Self::Constant { value: v1 }, Self::Constant { value: v2 }) =
                    (&**argument1, &**argument2)
                {
                    let res = match data.operation {
                        BinaryOperation::Add => v1.0 + v2.0,
                        BinaryOperation::Mul => v1.0 * v2.0,
                        BinaryOperation::Min => v1.0.min(v2.0),
                        BinaryOperation::Max => v1.0.max(v2.0),
                    };
                    *self = Self::Constant {
                        value: HashableF64(res),
                    };
                    return;
                }
                match data.operation {
                    BinaryOperation::Add => {
                        if let Self::Constant { value } = &**argument1 {
                            if value.0 == 0.0 {
                                *self = *argument2.clone();
                                return;
                            }
                        }
                        if let Self::Constant { value } = &**argument2 {
                            if value.0 == 0.0 {
                                *self = *argument1.clone();
                                return;
                            }
                        }
                    }
                    BinaryOperation::Mul => {
                        if let Self::Constant { value } = &**argument1 {
                            if value.0 == 1.0 {
                                *self = *argument2.clone();
                                return;
                            } else if value.0 == 0.0 {
                                *self = Self::Constant {
                                    value: HashableF64(0.0),
                                };
                                return;
                            }
                        }
                        if let Self::Constant { value } = &**argument2 {
                            if value.0 == 1.0 {
                                *self = *argument1.clone();
                                return;
                            } else if value.0 == 0.0 {
                                *self = Self::Constant {
                                    value: HashableF64(0.0),
                                };
                                return;
                            }
                        }
                    }
                    _ => {}
                }
            }
            Self::Unary { input, data } => {
                input.optimize();
                if let Self::Constant { value } = &**input {
                    let val = match data.operation {
                        UnaryOperation::Abs => value.0.abs(),
                        UnaryOperation::Square => value.0 * value.0,
                        UnaryOperation::Cube => value.0 * value.0 * value.0,
                        UnaryOperation::HalfNegative => {
                            if value.0 > 0.0 {
                                value.0
                            } else {
                                value.0 * 0.5
                            }
                        }
                        UnaryOperation::QuarterNegative => {
                            if value.0 > 0.0 {
                                value.0
                            } else {
                                value.0 * 0.25
                            }
                        }
                        UnaryOperation::Squeeze => {
                            let c = value.0.clamp(-1.0, 1.0);
                            c / 2.0 - c * c * c / 24.0
                        }
                        UnaryOperation::Invert => {
                            if value.0 == 0.0 {
                                f64::INFINITY
                            } else {
                                1.0 / value.0
                            }
                        }
                    };
                    *self = Self::Constant {
                        value: HashableF64(val),
                    };
                }
            }
            Self::Clamp { input, data } => {
                input.optimize();
                if let Self::Constant { value } = &**input {
                    *self = Self::Constant {
                        value: HashableF64(value.0.clamp(data.min_value.0, data.max_value.0)),
                    };
                }
            }
            _ => {}
        }
    }

    fn get_index_for_component_readonly(&self, hash_to_index_map: &BTreeMap<u64, usize>) -> usize {
        *hash_to_index_map.get(&self.unique_id()).unwrap_or(&0)
    }

    fn emit_compiled_eval_fn(
        &self,
        index: usize,
        fn_prefix: &str,
        hash_to_index_map: &BTreeMap<u64, usize>,
    ) -> TokenStream {
        let fn_name = syn::Ident::new(&format!("{}_{}", fn_prefix, index), Span::call_site());
        match self {
            Self::Constant { value } => {
                let val = value.0;
                quote! {
                    #[inline(always)]
                    pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                        let _ = (pos, ctx);
                        #val
                    }
                }
            }
            Self::ClampedYGradient { data } => {
                let from_y = f64::from(data.from_y);
                let to_y = f64::from(data.to_y);
                let from_val = data.from_value.0;
                let to_val = data.to_value.0;
                quote! {
                    #[inline(always)]
                    pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                        let _ = ctx;
                        let y = pos.y as f64;
                        let clamped = y.clamp(#from_y, #to_y);
                        let delta = (clamped - #from_y) / (#to_y - #from_y);
                        #from_val + delta * (#to_val - #from_val)
                    }
                }
            }
            Self::Linear { input, data } => {
                let child_idx = input.get_index_for_component_readonly(hash_to_index_map);
                let child_fn =
                    syn::Ident::new(&format!("{}_{}", fn_prefix, child_idx), Span::call_site());
                let arg = data.argument.0;
                let body = match data.operation {
                    LinearOperation::Add => quote! { #child_fn(pos, ctx) + #arg },
                    LinearOperation::Mul => quote! { #child_fn(pos, ctx) * #arg },
                };
                quote! {
                    #[inline(always)]
                    pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                        #body
                    }
                }
            }
            Self::Unary { input, data } => {
                let child_idx = input.get_index_for_component_readonly(hash_to_index_map);
                let child_fn =
                    syn::Ident::new(&format!("{}_{}", fn_prefix, child_idx), Span::call_site());
                let body = match data.operation {
                    UnaryOperation::Abs => quote! { #child_fn(pos, ctx).abs() },
                    UnaryOperation::Square => quote! { let v = #child_fn(pos, ctx); v * v },
                    UnaryOperation::Cube => quote! { let v = #child_fn(pos, ctx); v * v * v },
                    UnaryOperation::HalfNegative => {
                        quote! { let v = #child_fn(pos, ctx); if v > 0.0 { v } else { v * 0.5 } }
                    }
                    UnaryOperation::QuarterNegative => {
                        quote! { let v = #child_fn(pos, ctx); if v > 0.0 { v } else { v * 0.25 } }
                    }
                    UnaryOperation::Squeeze => {
                        quote! { let c = #child_fn(pos, ctx).clamp(-1.0, 1.0); c / 2.0 - c * c * c / 24.0 }
                    }
                    UnaryOperation::Invert => {
                        quote! { let v = #child_fn(pos, ctx); if v == 0.0 { f64::INFINITY } else { 1.0 / v } }
                    }
                };
                quote! {
                    #[inline(always)]
                    pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                        #body
                    }
                }
            }
            Self::Clamp { input, data } => {
                let child_idx = input.get_index_for_component_readonly(hash_to_index_map);
                let child_fn =
                    syn::Ident::new(&format!("{}_{}", fn_prefix, child_idx), Span::call_site());
                let min_v = data.min_value.0;
                let max_v = data.max_value.0;
                quote! {
                    #[inline(always)]
                    pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                        #child_fn(pos, ctx).clamp(#min_v, #max_v)
                    }
                }
            }
            Self::Binary {
                argument1,
                argument2,
                data,
            } => {
                let child1_idx = argument1.get_index_for_component_readonly(hash_to_index_map);
                let child2_idx = argument2.get_index_for_component_readonly(hash_to_index_map);
                let child1_fn =
                    syn::Ident::new(&format!("{}_{}", fn_prefix, child1_idx), Span::call_site());
                let child2_fn =
                    syn::Ident::new(&format!("{}_{}", fn_prefix, child2_idx), Span::call_site());
                let body = match data.operation {
                    BinaryOperation::Add => quote! { #child1_fn(pos, ctx) + #child2_fn(pos, ctx) },
                    BinaryOperation::Mul => quote! { #child1_fn(pos, ctx) * #child2_fn(pos, ctx) },
                    BinaryOperation::Min => {
                        quote! { #child1_fn(pos, ctx).min(#child2_fn(pos, ctx)) }
                    }
                    BinaryOperation::Max => {
                        quote! { #child1_fn(pos, ctx).max(#child2_fn(pos, ctx)) }
                    }
                };
                quote! {
                    #[inline(always)]
                    pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                        #body
                    }
                }
            }
            Self::RangeChoice {
                input,
                when_in_range,
                when_out_range,
                data,
            } => {
                let input_idx = input.get_index_for_component_readonly(hash_to_index_map);
                let when_in_idx = when_in_range.get_index_for_component_readonly(hash_to_index_map);
                let when_out_idx =
                    when_out_range.get_index_for_component_readonly(hash_to_index_map);
                let input_fn =
                    syn::Ident::new(&format!("{}_{}", fn_prefix, input_idx), Span::call_site());
                let when_in_fn =
                    syn::Ident::new(&format!("{}_{}", fn_prefix, when_in_idx), Span::call_site());
                let when_out_fn = syn::Ident::new(
                    &format!("{}_{}", fn_prefix, when_out_idx),
                    Span::call_site(),
                );
                let min_inc = data.min_inclusive.0;
                let max_exc = data.max_exclusive.0;
                quote! {
                    #[inline(always)]
                    pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                        let val = #input_fn(pos, ctx);
                        if val >= #min_inc && val < #max_exc {
                            #when_in_fn(pos, ctx)
                        } else {
                            #when_out_fn(pos, ctx)
                        }
                    }
                }
            }
            Self::Noise { data } => {
                let noise_id = quote::format_ident!("{}", data.noise_id.to_shouty_snake_case());
                let xz_scale = data.xz_scale.0;
                let y_scale = data.y_scale.0;
                quote! {
                    #[inline(always)]
                    pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                        ctx.sample_noise(DoublePerlinNoiseParameters::#noise_id, pos.x as f64 * #xz_scale, pos.y as f64 * #y_scale, pos.z as f64 * #xz_scale)
                    }
                }
            }
            Self::ShiftA { noise_id } => {
                let noise_id = quote::format_ident!("{}", noise_id.to_shouty_snake_case());
                quote! {
                    #[inline(always)]
                    pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                        ctx.sample_shift_a(DoublePerlinNoiseParameters::#noise_id, pos)
                    }
                }
            }
            Self::ShiftB { noise_id } => {
                let noise_id = quote::format_ident!("{}", noise_id.to_shouty_snake_case());
                quote! {
                    #[inline(always)]
                    pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                        ctx.sample_shift_b(DoublePerlinNoiseParameters::#noise_id, pos)
                    }
                }
            }
            Self::ShiftedNoise {
                shift_x,
                shift_y,
                shift_z,
                data,
            } => {
                let sx_idx = shift_x.get_index_for_component_readonly(hash_to_index_map);
                let sy_idx = shift_y.get_index_for_component_readonly(hash_to_index_map);
                let sz_idx = shift_z.get_index_for_component_readonly(hash_to_index_map);
                let sx_fn =
                    syn::Ident::new(&format!("{}_{}", fn_prefix, sx_idx), Span::call_site());
                let sy_fn =
                    syn::Ident::new(&format!("{}_{}", fn_prefix, sy_idx), Span::call_site());
                let sz_fn =
                    syn::Ident::new(&format!("{}_{}", fn_prefix, sz_idx), Span::call_site());
                let noise_id = quote::format_ident!("{}", data.noise_id.to_shouty_snake_case());
                let xz_scale = data.xz_scale.0;
                let y_scale = data.y_scale.0;
                quote! {
                    #[inline(always)]
                    pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                        let sx = #sx_fn(pos, ctx);
                        let sy = #sy_fn(pos, ctx);
                        let sz = #sz_fn(pos, ctx);
                        ctx.sample_shifted_noise(DoublePerlinNoiseParameters::#noise_id, sx, sy, sz, #xz_scale, #y_scale)
                    }
                }
            }
            Self::BlendAlpha => {
                quote! {
                    #[inline(always)]
                    pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                        ctx.sample_blend_alpha(pos)
                    }
                }
            }
            Self::BlendOffset => {
                quote! {
                    #[inline(always)]
                    pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                        ctx.sample_blend_offset(pos)
                    }
                }
            }
            Self::BlendDensity { input } => {
                let child_idx = input.get_index_for_component_readonly(hash_to_index_map);
                let child_fn =
                    syn::Ident::new(&format!("{}_{}", fn_prefix, child_idx), Span::call_site());
                quote! {
                    #[inline(always)]
                    pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                        let val = #child_fn(pos, ctx);
                        ctx.sample_blend_density(val, pos)
                    }
                }
            }
            Self::Beardifier => {
                quote! {
                    #[inline(always)]
                    pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                        ctx.sample_beardifier(pos)
                    }
                }
            }
            Self::EndIslands => {
                quote! {
                    #[inline(always)]
                    pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                        ctx.sample_end_islands(pos)
                    }
                }
            }
            Self::Wrapper { input, wrapper } => {
                let child_idx = input.get_index_for_component_readonly(hash_to_index_map);
                let child_fn =
                    syn::Ident::new(&format!("{}_{}", fn_prefix, child_idx), Span::call_site());
                let wrapper_repr = wrapper.into_token_stream();
                let comp_idx = index;
                quote! {
                    #[inline(always)]
                    pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                        ctx.sample_wrapper(#comp_idx, #wrapper_repr, pos, &#child_fn)
                    }
                }
            }
            Self::IntervalSelect {
                input,
                thresholds,
                functions,
            } => {
                let input_idx = input.get_index_for_component_readonly(hash_to_index_map);
                let input_fn =
                    syn::Ident::new(&format!("{}_{}", fn_prefix, input_idx), Span::call_site());
                let func_fns = functions
                    .iter()
                    .map(|f| {
                        let idx = f.get_index_for_component_readonly(hash_to_index_map);
                        syn::Ident::new(&format!("{}_{}", fn_prefix, idx), Span::call_site())
                    })
                    .collect::<Vec<_>>();
                let threshold_values = thresholds.iter().map(|t| t.0).collect::<Vec<_>>();
                let th_indices = (0..threshold_values.len()).collect::<Vec<_>>();
                let last_func_fn = func_fns.last().unwrap();
                let initial_func_fns = if func_fns.len() > 1 {
                    &func_fns[..func_fns.len() - 1]
                } else {
                    &[]
                };
                quote! {
                    #[inline(always)]
                    pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                        let input_val = #input_fn(pos, ctx);
                        let thresholds = &[#(#threshold_values),*];
                        let mut selected = thresholds.len();
                        for (i, &t) in thresholds.iter().enumerate() {
                            if input_val < t {
                                selected = i;
                                break;
                            }
                        }
                        match selected {
                            #( #th_indices => #initial_func_fns(pos, ctx), )*
                            _ => #last_func_fn(pos, ctx),
                        }
                    }
                }
            }
            Self::InterpolatedNoiseSampler { .. } => {
                quote! {
                    #[inline(always)]
                    pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                        ctx.sample_interpolated_noise(pos)
                    }
                }
            }
            Self::Spline { spline, .. } => {
                let loc_idx = match spline {
                    SplineRepr::Fixed { .. } => None,
                    SplineRepr::Standard {
                        location_function, ..
                    } => {
                        Some(location_function.get_index_for_component_readonly(hash_to_index_map))
                    }
                };
                if let Some(loc_idx) = loc_idx {
                    let loc_fn =
                        syn::Ident::new(&format!("{}_{}", fn_prefix, loc_idx), Span::call_site());
                    quote! {
                        #[inline(always)]
                        pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                            let location_val = #loc_fn(pos, ctx);
                            ctx.sample_spline(#index, location_val, pos)
                        }
                    }
                } else {
                    let val = match spline {
                        SplineRepr::Fixed { value } => value.0 as f64,
                        _ => 0.0,
                    };
                    quote! {
                        #[inline(always)]
                        pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                            let _ = (pos, ctx);
                            #val
                        }
                    }
                }
            }
            Self::FindTopSurface {
                density,
                upper_bound,
                data,
            } => {
                let d_idx = density.get_index_for_component_readonly(hash_to_index_map);
                let u_idx = upper_bound.get_index_for_component_readonly(hash_to_index_map);
                let d_fn = syn::Ident::new(&format!("{}_{}", fn_prefix, d_idx), Span::call_site());
                let u_fn = syn::Ident::new(&format!("{}_{}", fn_prefix, u_idx), Span::call_site());
                let lower = data.lower_bound;
                let cell_h = data.cell_height;
                quote! {
                    #[inline(always)]
                    pub fn #fn_name<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                        ctx.sample_find_top_surface(&#d_fn, &#u_fn, #lower, #cell_h, pos)
                    }
                }
            }
        }
    }

    /// Computes a stable 64-bit hash for this density function node.
    fn unique_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    /// Returns the index of this component in `stack`, inserting it if not yet present.
    fn get_index_for_component(
        &self,
        stack: &mut Vec<TokenStream>,
        nodes: &mut Vec<DensityFunctionRepr>,
        hash_to_index_map: &mut BTreeMap<u64, usize>,
    ) -> usize {
        if let Some(index) = hash_to_index_map.get(&self.unique_id()) {
            *index
        } else {
            let id = self.unique_id();
            let repr = self.get_token_stream(stack, nodes, hash_to_index_map);
            stack.push(repr);
            nodes.push(self.clone());
            let index = stack.len() - 1;
            hash_to_index_map.insert(id, index);
            index
        }
    }

    fn get_token_stream(
        &self,
        stack: &mut Vec<TokenStream>,
        nodes: &mut Vec<DensityFunctionRepr>,
        hash_to_index_map: &mut BTreeMap<u64, usize>,
    ) -> TokenStream {
        match self {
            Self::Spline { spline, data } => {
                let _ = data;
                let spline_repr = spline.get_token_stream(stack, nodes, hash_to_index_map);

                quote! {
                    BaseNoiseFunctionComponent::Spline {
                        spline: &#spline_repr,
                    }
                }
            }
            Self::FindTopSurface {
                density,
                upper_bound,
                data,
            } => {
                let density_index =
                    density.get_index_for_component(stack, nodes, hash_to_index_map);
                let upper_bound_index =
                    upper_bound.get_index_for_component(stack, nodes, hash_to_index_map);
                let lower_bound = data.lower_bound;
                let cell_height = data.cell_height;

                quote! {
                    BaseNoiseFunctionComponent::FindTopSurface {
                        density_index: #density_index,
                        upper_bound_index: #upper_bound_index,
                        data: &FindTopSurfaceData {
                            lower_bound: #lower_bound,
                            cell_height: #cell_height,
                        },
                    }
                }
            }
            Self::EndIslands => quote! {
                BaseNoiseFunctionComponent::EndIslands
            },
            Self::Noise { data } => {
                let noise_id = quote::format_ident!("{}", data.noise_id.to_shouty_snake_case());
                let xz_scale = &data.xz_scale;
                let y_scale = &data.y_scale;

                quote! {
                    BaseNoiseFunctionComponent::Noise {
                        data: &NoiseData {
                            noise_id: DoublePerlinNoiseParameters::#noise_id,
                            xz_scale: #xz_scale,
                            y_scale: #y_scale,
                        }
                    }
                }
            }
            Self::ShiftA { noise_id } => {
                let noise_id = quote::format_ident!("{}", noise_id.to_shouty_snake_case());

                quote! {
                    BaseNoiseFunctionComponent::ShiftA {
                        noise_id: DoublePerlinNoiseParameters::#noise_id,
                    }
                }
            }
            Self::ShiftB { noise_id } => {
                let noise_id = quote::format_ident!("{}", noise_id.to_shouty_snake_case());

                quote! {
                    BaseNoiseFunctionComponent::ShiftB {
                        noise_id: DoublePerlinNoiseParameters::#noise_id,
                    }
                }
            }
            Self::BlendDensity { input } => {
                let input_index = input.get_index_for_component(stack, nodes, hash_to_index_map);

                quote! {
                    BaseNoiseFunctionComponent::BlendDensity {
                        input_index: #input_index,
                    }
                }
            }
            Self::BlendAlpha => {
                quote! {
                    BaseNoiseFunctionComponent::BlendAlpha
                }
            }
            Self::BlendOffset => {
                quote! {
                    BaseNoiseFunctionComponent::BlendOffset
                }
            }
            Self::Beardifier => {
                quote! {
                    BaseNoiseFunctionComponent::Beardifier
                }
            }
            Self::ShiftedNoise {
                shift_x,
                shift_y,
                shift_z,
                data,
            } => {
                let shift_x_index =
                    shift_x.get_index_for_component(stack, nodes, hash_to_index_map);
                let shift_y_index =
                    shift_y.get_index_for_component(stack, nodes, hash_to_index_map);
                let shift_z_index =
                    shift_z.get_index_for_component(stack, nodes, hash_to_index_map);

                let xz_scale = &data.xz_scale;
                let y_scale = &data.y_scale;
                let noise_id = quote::format_ident!("{}", data.noise_id.to_shouty_snake_case());

                quote! {
                    BaseNoiseFunctionComponent::ShiftedNoise {
                        shift_x_index: #shift_x_index,
                        shift_y_index: #shift_y_index,
                        shift_z_index: #shift_z_index,
                        data: &ShiftedNoiseData {
                            xz_scale: #xz_scale,
                            y_scale: #y_scale,
                            noise_id: DoublePerlinNoiseParameters::#noise_id,
                        },
                    }
                }
            }
            Self::RangeChoice {
                input,
                when_in_range,
                when_out_range,
                data,
            } => {
                let input_index = input.get_index_for_component(stack, nodes, hash_to_index_map);
                let when_in_index =
                    when_in_range.get_index_for_component(stack, nodes, hash_to_index_map);
                let when_out_index =
                    when_out_range.get_index_for_component(stack, nodes, hash_to_index_map);

                let min_inclusive = &data.min_inclusive;
                let max_exclusive = &data.max_exclusive;

                quote! {
                    BaseNoiseFunctionComponent::RangeChoice {
                        input_index: #input_index,
                        when_in_range_index: #when_in_index,
                        when_out_range_index: #when_out_index,
                        data: &RangeChoiceData {
                            min_inclusive: #min_inclusive,
                            max_exclusive: #max_exclusive,
                        },
                    }
                }
            }
            Self::Binary {
                argument1,
                argument2,
                data,
            } => {
                let argument1_index =
                    argument1.get_index_for_component(stack, nodes, hash_to_index_map);
                let argument2_index =
                    argument2.get_index_for_component(stack, nodes, hash_to_index_map);

                let action = data.operation.get_token_stream();
                quote! {
                    BaseNoiseFunctionComponent::Binary {
                        argument1_index: #argument1_index,
                        argument2_index: #argument2_index,
                        data: &BinaryData {
                            operation: #action,
                        },
                    }
                }
            }
            Self::ClampedYGradient { data } => {
                let from_y = f64::from(data.from_y);
                let to_y = f64::from(data.to_y);
                let from_value = &data.from_value;
                let to_value = &data.to_value;

                quote! {
                    BaseNoiseFunctionComponent::ClampedYGradient {
                        data: &ClampedYGradientData {
                            from_y: #from_y,
                            to_y: #to_y,
                            from_value: #from_value,
                            to_value: #to_value,
                        }
                    }
                }
            }
            Self::Constant { value } => {
                quote! {
                    BaseNoiseFunctionComponent::Constant {
                        value: #value
                    }
                }
            }
            Self::Wrapper { input, wrapper } => {
                let input_index = input.get_index_for_component(stack, nodes, hash_to_index_map);
                let wrapper_repr = wrapper.into_token_stream();

                quote! {
                    BaseNoiseFunctionComponent::Wrapper {
                        input_index: #input_index,
                        wrapper: #wrapper_repr,
                    }
                }
            }
            Self::Linear { input, data } => {
                let input_index = input.get_index_for_component(stack, nodes, hash_to_index_map);

                let action = data.operation.into_token_stream();
                let argument = &data.argument;
                quote! {
                    BaseNoiseFunctionComponent::Linear {
                        input_index: #input_index,
                        data: &LinearData {
                            operation: #action,
                            argument: #argument,
                        },
                    }
                }
            }
            Self::Clamp { input, data } => {
                let input_index = input.get_index_for_component(stack, nodes, hash_to_index_map);

                let min_value = &data.min_value;
                let max_value = &data.max_value;

                quote! {
                    BaseNoiseFunctionComponent::Clamp {
                        input_index: #input_index,
                        data: &ClampData {
                            min_value: #min_value,
                            max_value: #max_value,
                        },
                    }
                }
            }
            Self::Unary { input, data } => {
                let input_index = input.get_index_for_component(stack, nodes, hash_to_index_map);

                let action = data.operation.into_token_stream();

                quote! {
                    BaseNoiseFunctionComponent::Unary {
                        input_index: #input_index,
                        data: &UnaryData {
                            operation: #action,
                        },
                    }
                }
            }
            Self::IntervalSelect {
                input,
                thresholds,
                functions,
            } => {
                let input_index = input.get_index_for_component(stack, nodes, hash_to_index_map);
                let functions_indices = functions
                    .iter()
                    .map(|f| f.get_index_for_component(stack, nodes, hash_to_index_map))
                    .collect::<Vec<_>>();
                let thresholds = thresholds.iter().map(|t| t.0).collect::<Vec<_>>();

                quote! {
                    BaseNoiseFunctionComponent::IntervalSelect {
                        input_index: #input_index,
                        thresholds: &[#(#thresholds),*],
                        functions_indices: &[#(#functions_indices),*],
                    }
                }
            }
            Self::InterpolatedNoiseSampler { data } => {
                let scaled_xz_scale = &data.scaled_xz_scale;
                let scaled_y_scale = &data.scaled_y_scale;
                let xz_factor = &data.xz_factor;
                let y_factor = &data.y_factor;
                let smear_scale_multiplier = &data.smear_scale_multiplier;

                quote! {
                    BaseNoiseFunctionComponent::InterpolatedNoiseSampler {
                        data: &InterpolatedNoiseSamplerData {
                            scaled_xz_scale: #scaled_xz_scale,
                            scaled_y_scale: #scaled_y_scale,
                            xz_factor: #xz_factor,
                            y_factor: #y_factor,
                            smear_scale_multiplier: #smear_scale_multiplier,
                        }
                    }
                }
            }
        }
    }
}

/// Top-level container for all dimension noise router representations deserialized from JSON.
#[derive(Deserialize)]
struct NoiseRouterReprs {
    /// Standard overworld noise router.
    overworld: NoiseRouterRepr,
    /// Large-biomes overworld noise router variant.
    #[serde(rename(deserialize = "large_biomes"))]
    overworld_large_biomes: NoiseRouterRepr,
    /// Amplified overworld noise router variant.
    #[serde(rename(deserialize = "amplified"))]
    overworld_amplified: NoiseRouterRepr,
    /// Nether dimension noise router.
    nether: NoiseRouterRepr,
    /// End dimension noise router.
    end: NoiseRouterRepr,
    /// Floating-islands (End) noise router variant.
    #[serde(rename(deserialize = "floating_islands"))]
    end_islands: NoiseRouterRepr,
}

/// Deserialized noise router for a single dimension, containing all density function roots.
#[derive(Deserialize)]
struct NoiseRouterRepr {
    /// Density function controlling aquifer barrier generation.
    #[serde(rename(deserialize = "barrierNoise"))]
    barrier_noise: DensityFunctionRepr,
    /// Density function controlling fluid-level floodedness.
    #[serde(rename(deserialize = "fluidLevelFloodednessNoise"))]
    fluid_level_floodedness_noise: DensityFunctionRepr,
    /// Density function controlling how fluid levels spread.
    #[serde(rename(deserialize = "fluidLevelSpreadNoise"))]
    fluid_level_spread_noise: DensityFunctionRepr,
    /// Density function controlling lava pocket generation.
    #[serde(rename(deserialize = "lavaNoise"))]
    lava_noise: DensityFunctionRepr,
    /// Density function for biome temperature noise.
    temperature: DensityFunctionRepr,
    /// Density function for biome vegetation noise.
    vegetation: DensityFunctionRepr,
    /// Density function for continental-scale terrain shaping.
    continents: DensityFunctionRepr,
    /// Density function for erosion-based terrain shaping.
    erosion: DensityFunctionRepr,
    /// Density function encoding terrain depth below the surface.
    depth: DensityFunctionRepr,
    /// Density function for terrain ridge shaping.
    ridges: DensityFunctionRepr,
    /// Preliminary surface density used for above-surface checks (without jaggedness).
    #[serde(rename(deserialize = "preliminarySurfaceLevel"))]
    preliminary_surface_level: DensityFunctionRepr,
    /// Final solid/air density used for block placement.
    #[serde(rename(deserialize = "finalDensity"))]
    final_density: DensityFunctionRepr,
    /// Density function toggling ore-vein generation.
    #[serde(rename(deserialize = "veinToggle"))]
    vein_toggle: DensityFunctionRepr,
    /// Density function for ridged ore-vein shaping.
    #[serde(rename(deserialize = "veinRidged"))]
    vein_ridged: DensityFunctionRepr,
    /// Density function controlling gaps within ore veins.
    #[serde(rename(deserialize = "veinGap"))]
    vein_gap: DensityFunctionRepr,
}

impl NoiseRouterRepr {
    fn optimize(&mut self) {
        self.barrier_noise.optimize();
        self.fluid_level_floodedness_noise.optimize();
        self.fluid_level_spread_noise.optimize();
        self.lava_noise.optimize();
        self.temperature.optimize();
        self.vegetation.optimize();
        self.continents.optimize();
        self.erosion.optimize();
        self.depth.optimize();
        self.ridges.optimize();
        self.preliminary_surface_level.optimize();
        self.final_density.optimize();
        self.vein_toggle.optimize();
        self.vein_ridged.optimize();
        self.vein_gap.optimize();
    }

    /// Consumes this router representation and emits the `BaseNoiseRouters` token stream and compiled evaluator modules.
    fn into_token_stream_compiled(mut self, router_name: &str) -> (TokenStream, TokenStream) {
        self.optimize();
        let mut noise_component_stack = Vec::new();
        let mut noise_nodes = Vec::new();
        let mut noise_lookup_map = BTreeMap::new();

        // The aquifer sampler is called most often
        let final_density = self.final_density.get_index_for_component(
            &mut noise_component_stack,
            &mut noise_nodes,
            &mut noise_lookup_map,
        );
        let barrier_noise = self.barrier_noise.get_index_for_component(
            &mut noise_component_stack,
            &mut noise_nodes,
            &mut noise_lookup_map,
        );
        let fluid_level_floodedness_noise =
            self.fluid_level_floodedness_noise.get_index_for_component(
                &mut noise_component_stack,
                &mut noise_nodes,
                &mut noise_lookup_map,
            );
        let fluid_level_spread_noise = self.fluid_level_spread_noise.get_index_for_component(
            &mut noise_component_stack,
            &mut noise_nodes,
            &mut noise_lookup_map,
        );
        let lava_noise = self.lava_noise.get_index_for_component(
            &mut noise_component_stack,
            &mut noise_nodes,
            &mut noise_lookup_map,
        );

        // Ore sampler is called fewer times than aquifer sampler
        let vein_toggle = self.vein_toggle.get_index_for_component(
            &mut noise_component_stack,
            &mut noise_nodes,
            &mut noise_lookup_map,
        );
        let vein_ridged = self.vein_ridged.get_index_for_component(
            &mut noise_component_stack,
            &mut noise_nodes,
            &mut noise_lookup_map,
        );
        let vein_gap = self.vein_gap.get_index_for_component(
            &mut noise_component_stack,
            &mut noise_nodes,
            &mut noise_lookup_map,
        );

        // These should all be cached so it doesn't matter where their components are
        let noise_erosion = self.erosion.get_index_for_component(
            &mut noise_component_stack,
            &mut noise_nodes,
            &mut noise_lookup_map,
        );
        let noise_depth = self.depth.get_index_for_component(
            &mut noise_component_stack,
            &mut noise_nodes,
            &mut noise_lookup_map,
        );

        let mut surface_component_stack = Vec::new();
        let mut surface_nodes = Vec::new();
        let mut surface_lookup_map = BTreeMap::new();
        let _ = self.preliminary_surface_level.get_index_for_component(
            &mut surface_component_stack,
            &mut surface_nodes,
            &mut surface_lookup_map,
        );

        let mut multinoise_component_stack = Vec::new();
        let mut multinoise_nodes = Vec::new();
        let mut multinoise_lookup_map = BTreeMap::new();
        let ridges = self.ridges.get_index_for_component(
            &mut multinoise_component_stack,
            &mut multinoise_nodes,
            &mut multinoise_lookup_map,
        );
        let temperature = self.temperature.get_index_for_component(
            &mut multinoise_component_stack,
            &mut multinoise_nodes,
            &mut multinoise_lookup_map,
        );
        let vegetation = self.vegetation.get_index_for_component(
            &mut multinoise_component_stack,
            &mut multinoise_nodes,
            &mut multinoise_lookup_map,
        );
        let continents = self.continents.get_index_for_component(
            &mut multinoise_component_stack,
            &mut multinoise_nodes,
            &mut multinoise_lookup_map,
        );
        let multi_erosion = self.erosion.get_index_for_component(
            &mut multinoise_component_stack,
            &mut multinoise_nodes,
            &mut multinoise_lookup_map,
        );
        let multi_depth = self.depth.get_index_for_component(
            &mut multinoise_component_stack,
            &mut multinoise_nodes,
            &mut multinoise_lookup_map,
        );

        let base_routers_ts = quote! {
            BaseNoiseRouters {
                noise: BaseNoiseRouter {
                    full_component_stack: &[#(#noise_component_stack),*],
                    barrier_noise: #barrier_noise,
                    fluid_level_floodedness_noise: #fluid_level_floodedness_noise,
                    fluid_level_spread_noise: #fluid_level_spread_noise,
                    lava_noise: #lava_noise,
                    erosion: #noise_erosion,
                    depth: #noise_depth,
                    final_density: #final_density,
                    vein_toggle: #vein_toggle,
                    vein_ridged: #vein_ridged,
                    vein_gap: #vein_gap,
                },
                surface_estimator: BaseSurfaceEstimator {
                    full_component_stack: &[#(#surface_component_stack),*],
                },
                multi_noise: BaseMultiNoiseRouter {
                    full_component_stack: &[#(#multinoise_component_stack),*],
                    temperature: #temperature,
                    vegetation: #vegetation,
                    continents: #continents,
                    erosion: #multi_erosion,
                    depth: #multi_depth,
                    ridges: #ridges,
                },
            }
        };

        let mod_ident = quote::format_ident!("{}_noise_evaluator", router_name);
        let prefix = format!("{}_node", router_name);

        let fn_tokens = noise_nodes
            .iter()
            .enumerate()
            .map(|(idx, node)| node.emit_compiled_eval_fn(idx, &prefix, &noise_lookup_map));

        let final_density_fn = quote::format_ident!("{}_{}", prefix, final_density);
        let barrier_noise_fn = quote::format_ident!("{}_{}", prefix, barrier_noise);
        let fluid_floodedness_fn =
            quote::format_ident!("{}_{}", prefix, fluid_level_floodedness_noise);
        let fluid_spread_fn = quote::format_ident!("{}_{}", prefix, fluid_level_spread_noise);
        let lava_noise_fn = quote::format_ident!("{}_{}", prefix, lava_noise);
        let vein_toggle_fn = quote::format_ident!("{}_{}", prefix, vein_toggle);
        let vein_ridged_fn = quote::format_ident!("{}_{}", prefix, vein_ridged);
        let vein_gap_fn = quote::format_ident!("{}_{}", prefix, vein_gap);
        let erosion_fn = quote::format_ident!("{}_{}", prefix, noise_erosion);
        let depth_fn = quote::format_ident!("{}_{}", prefix, noise_depth);

        let compiled_mod_ts = quote! {
            pub mod #mod_ident {
                use super::*;
                #(#fn_tokens)*

                #[inline(always)]
                pub fn sample_final_density<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                    #final_density_fn(pos, ctx)
                }
                #[inline(always)]
                pub fn sample_barrier_noise<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                    #barrier_noise_fn(pos, ctx)
                }
                #[inline(always)]
                pub fn sample_fluid_level_floodedness_noise<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                    #fluid_floodedness_fn(pos, ctx)
                }
                #[inline(always)]
                pub fn sample_fluid_level_spread_noise<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                    #fluid_spread_fn(pos, ctx)
                }
                #[inline(always)]
                pub fn sample_lava_noise<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                    #lava_noise_fn(pos, ctx)
                }
                #[inline(always)]
                pub fn sample_vein_toggle<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                    #vein_toggle_fn(pos, ctx)
                }
                #[inline(always)]
                pub fn sample_vein_ridged<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                    #vein_ridged_fn(pos, ctx)
                }
                #[inline(always)]
                pub fn sample_vein_gap<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                    #vein_gap_fn(pos, ctx)
                }
                #[inline(always)]
                pub fn sample_erosion<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                    #erosion_fn(pos, ctx)
                }
                #[inline(always)]
                pub fn sample_depth<C: NoiseEvaluationContext>(pos: &pumpkin_util::math::vector3::Vector3<i32>, ctx: &mut C) -> f64 {
                    #depth_fn(pos, ctx)
                }
            }
        };

        (base_routers_ts, compiled_mod_ts)
    }
}

/// Wraps `$router.final_density` in a `Beardifier`-add and `CellCache` wrapper, mirroring the
/// Java runtime mutation applied to aquifer generators.
macro_rules! fix_final_density {
    ($router:expr) => {{
        $router.final_density = DensityFunctionRepr::Wrapper {
            input: Box::new(DensityFunctionRepr::Binary {
                argument1: Box::new($router.final_density),
                argument2: Box::new(DensityFunctionRepr::Beardifier),
                data: BinaryData {
                    operation: BinaryOperation::Add,
                    max_value: HashableF64(f64::INFINITY),
                    min_value: HashableF64(f64::NEG_INFINITY),
                },
            }),
            wrapper: WrapperType::CellCache,
        };
    }};
}

fn load_df_json(base_df_dir: &std::path::Path, name: &str) -> serde_json::Value {
    let clean = name.strip_prefix("minecraft:").unwrap_or(name);
    let path = base_df_dir.join(format!("{clean}.json"));
    let content = fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "Failed to read density function at {}: {err}",
            path.display()
        )
    });
    serde_json::from_str(&content).unwrap_or_else(|err| {
        panic!(
            "Failed to parse density function at {}: {err}",
            path.display()
        )
    })
}

fn clean_noise_name(n: &str) -> String {
    n.strip_prefix("minecraft:").unwrap_or(n).to_string()
}

fn parse_vanilla_df(base_df_dir: &std::path::Path, val: &serde_json::Value) -> DensityFunctionRepr {
    match val {
        serde_json::Value::Number(n) => DensityFunctionRepr::Constant {
            value: HashableF64(n.as_f64().unwrap_or(0.0)),
        },
        serde_json::Value::String(s) => {
            if s == "minecraft:y" {
                DensityFunctionRepr::ClampedYGradient {
                    data: ClampedYGradientData {
                        from_y: -4064,
                        to_y: 4062,
                        from_value: HashableF64(-4064.0),
                        to_value: HashableF64(4062.0),
                    },
                }
            } else if s == "minecraft:zero" {
                DensityFunctionRepr::Constant {
                    value: HashableF64(0.0),
                }
            } else {
                let loaded = load_df_json(base_df_dir, s);
                parse_vanilla_df(base_df_dir, &loaded)
            }
        }
        serde_json::Value::Object(obj) => {
            let type_str = obj
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or_else(|| panic!("Density function object missing type: {val:?}"));
            let clean_type = type_str.strip_prefix("minecraft:").unwrap_or(type_str);

            match clean_type {
                "constant" => {
                    let num = obj
                        .get("value")
                        .or_else(|| obj.get("argument"))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    DensityFunctionRepr::Constant {
                        value: HashableF64(num),
                    }
                }
                "y_clamped_gradient" => {
                    let from_y = obj.get("from_y").and_then(|v| v.as_i64()).unwrap_or(-64) as i32;
                    let to_y = obj.get("to_y").and_then(|v| v.as_i64()).unwrap_or(320) as i32;
                    let from_value = obj
                        .get("from_value")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let to_value = obj.get("to_value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    DensityFunctionRepr::ClampedYGradient {
                        data: ClampedYGradientData {
                            from_y,
                            to_y,
                            from_value: HashableF64(from_value),
                            to_value: HashableF64(to_value),
                        },
                    }
                }
                "old_blended_noise" => {
                    let xz_scale = obj.get("xz_scale").and_then(|v| v.as_f64()).unwrap_or(0.25);
                    let y_scale = obj.get("y_scale").and_then(|v| v.as_f64()).unwrap_or(0.125);
                    let xz_factor = obj
                        .get("xz_factor")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(80.0);
                    let y_factor = obj
                        .get("y_factor")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(160.0);
                    let smear_scale_multiplier = obj
                        .get("smear_scale_multiplier")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(8.0);
                    let max_value = obj
                        .get("max_value")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(f64::INFINITY);

                    let scaled_xz_scale = xz_scale;
                    let scaled_y_scale = y_scale * (y_factor / xz_factor);

                    DensityFunctionRepr::InterpolatedNoiseSampler {
                        data: InterpolatedNoiseSamplerData {
                            scaled_xz_scale: HashableF64(scaled_xz_scale),
                            scaled_y_scale: HashableF64(scaled_y_scale),
                            xz_factor: HashableF64(xz_factor),
                            y_factor: HashableF64(y_factor),
                            smear_scale_multiplier: HashableF64(smear_scale_multiplier),
                            max_value: HashableF64(max_value),
                        },
                    }
                }
                "add" | "mul" | "min" | "max" => {
                    let arg1 = parse_vanilla_df(
                        base_df_dir,
                        obj.get("argument1").expect("Missing argument1"),
                    );
                    let arg2 = parse_vanilla_df(
                        base_df_dir,
                        obj.get("argument2").expect("Missing argument2"),
                    );
                    let min_value = obj
                        .get("min_value")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(f64::NEG_INFINITY);
                    let max_value = obj
                        .get("max_value")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(f64::INFINITY);

                    if clean_type == "add" || clean_type == "mul" {
                        let linear_op = if clean_type == "add" {
                            LinearOperation::Add
                        } else {
                            LinearOperation::Mul
                        };

                        if let DensityFunctionRepr::Constant { value: c } = arg1 {
                            return DensityFunctionRepr::Linear {
                                input: Box::new(arg2),
                                data: LinearData {
                                    operation: linear_op,
                                    argument: c,
                                    min_value: HashableF64(min_value),
                                    max_value: HashableF64(max_value),
                                },
                            };
                        } else if let DensityFunctionRepr::Constant { value: c } = arg2 {
                            return DensityFunctionRepr::Linear {
                                input: Box::new(arg1),
                                data: LinearData {
                                    operation: linear_op,
                                    argument: c,
                                    min_value: HashableF64(min_value),
                                    max_value: HashableF64(max_value),
                                },
                            };
                        }
                    }

                    let op = match clean_type {
                        "add" => BinaryOperation::Add,
                        "mul" => BinaryOperation::Mul,
                        "min" => BinaryOperation::Min,
                        "max" => BinaryOperation::Max,
                        _ => unreachable!(),
                    };

                    DensityFunctionRepr::Binary {
                        argument1: Box::new(arg1),
                        argument2: Box::new(arg2),
                        data: BinaryData {
                            operation: op,
                            min_value: HashableF64(min_value),
                            max_value: HashableF64(max_value),
                        },
                    }
                }
                "abs" | "square" | "cube" | "half_negative" | "quarter_negative" | "squeeze"
                | "invert" => {
                    let op = match clean_type {
                        "abs" => UnaryOperation::Abs,
                        "square" => UnaryOperation::Square,
                        "cube" => UnaryOperation::Cube,
                        "half_negative" => UnaryOperation::HalfNegative,
                        "quarter_negative" => UnaryOperation::QuarterNegative,
                        "squeeze" => UnaryOperation::Squeeze,
                        "invert" => UnaryOperation::Invert,
                        _ => unreachable!(),
                    };
                    let input_node = obj
                        .get("argument")
                        .or_else(|| obj.get("input"))
                        .expect("Missing argument/input");
                    let input = parse_vanilla_df(base_df_dir, input_node);
                    let min_value = obj
                        .get("min_value")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(f64::NEG_INFINITY);
                    let max_value = obj
                        .get("max_value")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(f64::INFINITY);
                    DensityFunctionRepr::Unary {
                        input: Box::new(input),
                        data: UnaryData {
                            operation: op,
                            min_value: HashableF64(min_value),
                            max_value: HashableF64(max_value),
                        },
                    }
                }
                "clamp" => {
                    let input_node = obj
                        .get("input")
                        .or_else(|| obj.get("argument"))
                        .expect("Missing input");
                    let input = parse_vanilla_df(base_df_dir, input_node);
                    let min_val = obj
                        .get("min")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(f64::NEG_INFINITY);
                    let max_val = obj
                        .get("max")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(f64::INFINITY);
                    DensityFunctionRepr::Clamp {
                        input: Box::new(input),
                        data: ClampData {
                            min_value: HashableF64(min_val),
                            max_value: HashableF64(max_val),
                        },
                    }
                }
                "range_choice" => {
                    let input =
                        parse_vanilla_df(base_df_dir, obj.get("input").expect("Missing input"));
                    let min_inc = obj
                        .get("min_inclusive")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(f64::NEG_INFINITY);
                    let max_exc = obj
                        .get("max_exclusive")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(f64::INFINITY);
                    let when_in = parse_vanilla_df(
                        base_df_dir,
                        obj.get("when_in_range").expect("Missing when_in_range"),
                    );
                    let when_out = parse_vanilla_df(
                        base_df_dir,
                        obj.get("when_out_of_range")
                            .expect("Missing when_out_of_range"),
                    );
                    DensityFunctionRepr::RangeChoice {
                        input: Box::new(input),
                        when_in_range: Box::new(when_in),
                        when_out_range: Box::new(when_out),
                        data: RangeChoiceData {
                            min_inclusive: HashableF64(min_inc),
                            max_exclusive: HashableF64(max_exc),
                        },
                    }
                }
                "interval_select" => {
                    let input =
                        parse_vanilla_df(base_df_dir, obj.get("input").expect("Missing input"));
                    let thresholds: Vec<HashableF64> = obj
                        .get("thresholds")
                        .and_then(|v| v.as_array())
                        .expect("Missing thresholds array")
                        .iter()
                        .map(|v| HashableF64(v.as_f64().unwrap_or(0.0)))
                        .collect();
                    let funcs: Vec<DensityFunctionRepr> = obj
                        .get("functions")
                        .and_then(|v| v.as_array())
                        .expect("Missing functions array")
                        .iter()
                        .map(|v| parse_vanilla_df(base_df_dir, v))
                        .collect();

                    DensityFunctionRepr::IntervalSelect {
                        input: Box::new(input),
                        thresholds: thresholds.into_boxed_slice(),
                        functions: funcs.into_boxed_slice(),
                    }
                }
                "spline" => {
                    let spline_node = obj.get("spline").expect("Missing spline");
                    let spline = parse_vanilla_spline(base_df_dir, spline_node);
                    let min_value = obj
                        .get("min_value")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(f64::NEG_INFINITY);
                    let max_value = obj
                        .get("max_value")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(f64::INFINITY);
                    DensityFunctionRepr::Spline {
                        spline,
                        data: SplineData {
                            min_value: HashableF64(min_value),
                            max_value: HashableF64(max_value),
                        },
                    }
                }
                "noise" => {
                    let noise_name = obj
                        .get("noise")
                        .and_then(|v| v.as_str())
                        .expect("Missing noise name");
                    let xz_scale = obj.get("xz_scale").and_then(|v| v.as_f64()).unwrap_or(1.0);
                    let y_scale = obj.get("y_scale").and_then(|v| v.as_f64()).unwrap_or(1.0);
                    DensityFunctionRepr::Noise {
                        data: NoiseData {
                            noise_id: clean_noise_name(noise_name),
                            xz_scale: HashableF64(xz_scale),
                            y_scale: HashableF64(y_scale),
                        },
                    }
                }
                "shifted_noise" => {
                    let shift_x =
                        parse_vanilla_df(base_df_dir, obj.get("shift_x").expect("Missing shift_x"));
                    let shift_y =
                        parse_vanilla_df(base_df_dir, obj.get("shift_y").expect("Missing shift_y"));
                    let shift_z =
                        parse_vanilla_df(base_df_dir, obj.get("shift_z").expect("Missing shift_z"));
                    let noise_name = obj
                        .get("noise")
                        .and_then(|v| v.as_str())
                        .expect("Missing noise name");
                    let xz_scale = obj.get("xz_scale").and_then(|v| v.as_f64()).unwrap_or(1.0);
                    let y_scale = obj.get("y_scale").and_then(|v| v.as_f64()).unwrap_or(1.0);
                    DensityFunctionRepr::ShiftedNoise {
                        shift_x: Box::new(shift_x),
                        shift_y: Box::new(shift_y),
                        shift_z: Box::new(shift_z),
                        data: ShiftedNoiseData {
                            noise_id: clean_noise_name(noise_name),
                            xz_scale: HashableF64(xz_scale),
                            y_scale: HashableF64(y_scale),
                        },
                    }
                }
                "shift_a" => {
                    let offset_noise = obj
                        .get("argument")
                        .and_then(|v| v.as_str())
                        .expect("Missing argument");
                    DensityFunctionRepr::ShiftA {
                        noise_id: clean_noise_name(offset_noise),
                    }
                }
                "shift_b" => {
                    let offset_noise = obj
                        .get("argument")
                        .and_then(|v| v.as_str())
                        .expect("Missing argument");
                    DensityFunctionRepr::ShiftB {
                        noise_id: clean_noise_name(offset_noise),
                    }
                }
                "shift" => {
                    let offset_noise = obj
                        .get("argument")
                        .and_then(|v| v.as_str())
                        .expect("Missing argument");
                    DensityFunctionRepr::ShiftA {
                        noise_id: clean_noise_name(offset_noise),
                    }
                }
                "blend_alpha" => DensityFunctionRepr::BlendAlpha,
                "blend_offset" => DensityFunctionRepr::BlendOffset,
                "blend_density" => {
                    let input = parse_vanilla_df(
                        base_df_dir,
                        obj.get("argument").expect("Missing argument"),
                    );
                    DensityFunctionRepr::BlendDensity {
                        input: Box::new(input),
                    }
                }
                "end_islands" => DensityFunctionRepr::EndIslands,
                "beardifier" => DensityFunctionRepr::Beardifier,
                "interpolated" | "flat_cache" | "cache_flat" | "cache_2d" | "cache_once"
                | "cache_all_in_cell" => {
                    let wrapper = match clean_type {
                        "interpolated" => WrapperType::Interpolated,
                        "flat_cache" | "cache_flat" => WrapperType::CacheFlat,
                        "cache_2d" => WrapperType::Cache2D,
                        "cache_once" => WrapperType::CacheOnce,
                        "cache_all_in_cell" => WrapperType::CellCache,
                        _ => unreachable!(),
                    };
                    let input = parse_vanilla_df(
                        base_df_dir,
                        obj.get("argument").expect("Missing argument"),
                    );
                    DensityFunctionRepr::Wrapper {
                        input: Box::new(input),
                        wrapper,
                    }
                }
                "find_top_surface" | "weird_utility_density" => {
                    let density =
                        parse_vanilla_df(base_df_dir, obj.get("density").expect("Missing density"));
                    let upper_bound = parse_vanilla_df(
                        base_df_dir,
                        obj.get("upper_bound").expect("Missing upper_bound"),
                    );
                    let lower_bound = obj
                        .get("lower_bound")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(-64) as i32;
                    let cell_height =
                        obj.get("cell_height").and_then(|v| v.as_i64()).unwrap_or(8) as i32;
                    DensityFunctionRepr::FindTopSurface {
                        density: Box::new(density),
                        upper_bound: Box::new(upper_bound),
                        data: FindTopSurfaceData {
                            lower_bound,
                            cell_height,
                        },
                    }
                }
                other => panic!("Unknown density function type in datapack: {other}"),
            }
        }
        _ => panic!("Unsupported JSON value in density function: {val:?}"),
    }
}

fn parse_vanilla_spline(base_df_dir: &std::path::Path, val: &serde_json::Value) -> SplineRepr {
    if let Some(n) = val.as_f64() {
        return SplineRepr::Fixed {
            value: HashableF32(n as f32),
        };
    }
    if let Some(obj) = val.as_object() {
        if obj.contains_key("value") && !obj.contains_key("points") {
            return parse_vanilla_spline(base_df_dir, &obj["value"]);
        }
        let loc_fn = parse_vanilla_df(
            base_df_dir,
            obj.get("coordinate").expect("Missing coordinate in spline"),
        );
        let points = obj
            .get("points")
            .and_then(|v| v.as_array())
            .expect("Missing points array in spline");

        let mut locations = Vec::new();
        let mut values = Vec::new();
        let mut derivatives = Vec::new();

        for pt in points {
            locations.push(HashableF32(pt["location"].as_f64().unwrap() as f32));
            values.push(parse_vanilla_spline(base_df_dir, &pt["value"]));
            derivatives.push(HashableF32(pt["derivative"].as_f64().unwrap() as f32));
        }

        return SplineRepr::Standard {
            location_function: Box::new(loc_fn),
            locations: locations.into_boxed_slice(),
            values: values.into_boxed_slice(),
            derivatives: derivatives.into_boxed_slice(),
        };
    }
    panic!("Unsupported spline node: {val:?}");
}

fn load_vanilla_noise_router(
    base_ns_dir: &std::path::Path,
    base_df_dir: &std::path::Path,
    dim_name: &str,
) -> NoiseRouterRepr {
    let path = base_ns_dir.join(format!("{dim_name}.json"));
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("Failed to read noise_settings at {}: {err}", path.display()));
    let val: serde_json::Value = serde_json::from_str(&content).unwrap_or_else(|err| {
        panic!(
            "Failed to parse noise_settings at {}: {err}",
            path.display()
        )
    });
    let nr = val
        .get("noise_router")
        .expect("Missing noise_router in noise_settings");

    let zero = serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap());

    NoiseRouterRepr {
        barrier_noise: parse_vanilla_df(base_df_dir, nr.get("barrier").unwrap_or(&zero)),
        fluid_level_floodedness_noise: parse_vanilla_df(
            base_df_dir,
            nr.get("fluid_level_floodedness").unwrap_or(&zero),
        ),
        fluid_level_spread_noise: parse_vanilla_df(
            base_df_dir,
            nr.get("fluid_level_spread").unwrap_or(&zero),
        ),
        lava_noise: parse_vanilla_df(base_df_dir, nr.get("lava").unwrap_or(&zero)),
        temperature: parse_vanilla_df(base_df_dir, nr.get("temperature").unwrap_or(&zero)),
        vegetation: parse_vanilla_df(base_df_dir, nr.get("vegetation").unwrap_or(&zero)),
        continents: parse_vanilla_df(base_df_dir, nr.get("continents").unwrap_or(&zero)),
        erosion: parse_vanilla_df(base_df_dir, nr.get("erosion").unwrap_or(&zero)),
        depth: parse_vanilla_df(base_df_dir, nr.get("depth").unwrap_or(&zero)),
        ridges: parse_vanilla_df(base_df_dir, nr.get("ridges").unwrap_or(&zero)),
        preliminary_surface_level: parse_vanilla_df(
            base_df_dir,
            nr.get("preliminary_surface_level").unwrap_or(&zero),
        ),
        final_density: parse_vanilla_df(base_df_dir, nr.get("final_density").unwrap_or(&zero)),
        vein_toggle: parse_vanilla_df(base_df_dir, nr.get("vein_toggle").unwrap_or(&zero)),
        vein_ridged: parse_vanilla_df(base_df_dir, nr.get("vein_ridged").unwrap_or(&zero)),
        vein_gap: parse_vanilla_df(base_df_dir, nr.get("vein_gap").unwrap_or(&zero)),
    }
}

fn load_vanilla_noise_routers() -> NoiseRouterReprs {
    let base_ns_dir =
        std::path::Path::new("../../assets/datapacks/26_2/data/minecraft/worldgen/noise_settings");
    let base_df_dir = std::path::Path::new(
        "../../assets/datapacks/26_2/data/minecraft/worldgen/density_function",
    );

    let overworld = load_vanilla_noise_router(base_ns_dir, base_df_dir, "overworld");
    let overworld_large_biomes =
        load_vanilla_noise_router(base_ns_dir, base_df_dir, "large_biomes");
    let overworld_amplified = load_vanilla_noise_router(base_ns_dir, base_df_dir, "amplified");
    let nether = load_vanilla_noise_router(base_ns_dir, base_df_dir, "nether");
    let end = load_vanilla_noise_router(base_ns_dir, base_df_dir, "end");
    let end_islands = load_vanilla_noise_router(base_ns_dir, base_df_dir, "floating_islands");

    NoiseRouterReprs {
        overworld,
        overworld_large_biomes,
        overworld_amplified,
        nether,
        end,
        end_islands,
    }
}

/// Reads vanilla datapack noise_settings and density_function files and emits the complete noise-router constants `TokenStream`.
pub fn build() -> TokenStream {
    let mut reprs: NoiseRouterReprs = load_vanilla_noise_routers();

    // The `final_density` function is mutated at runtime for the aquifer generator in Java.
    fix_final_density!(reprs.overworld);
    fix_final_density!(reprs.overworld_amplified);
    fix_final_density!(reprs.overworld_large_biomes);
    fix_final_density!(reprs.nether);

    let _ = reprs.end;
    let _ = reprs.end_islands;

    let (overworld_router, overworld_compiled) =
        reprs.overworld.into_token_stream_compiled("overworld");
    let (nether_router, nether_compiled) = reprs.nether.into_token_stream_compiled("nether");
    let (end_router, end_compiled) = reprs.end.into_token_stream_compiled("end");

    quote! {
        use crate::chunk::DoublePerlinNoiseParameters;

        pub trait NoiseEvaluationContext {
            fn sample_noise(&mut self, noise_id: DoublePerlinNoiseParameters, x: f64, y: f64, z: f64) -> f64;
            fn sample_shift_a(&mut self, noise_id: DoublePerlinNoiseParameters, pos: &pumpkin_util::math::vector3::Vector3<i32>) -> f64;
            fn sample_shift_b(&mut self, noise_id: DoublePerlinNoiseParameters, pos: &pumpkin_util::math::vector3::Vector3<i32>) -> f64;
            fn sample_shifted_noise(&mut self, noise_id: DoublePerlinNoiseParameters, shift_x: f64, shift_y: f64, shift_z: f64, xz_scale: f64, y_scale: f64) -> f64;
            fn sample_interpolated_noise(&mut self, pos: &pumpkin_util::math::vector3::Vector3<i32>) -> f64;
            fn sample_beardifier(&mut self, pos: &pumpkin_util::math::vector3::Vector3<i32>) -> f64;
            fn sample_blend_alpha(&mut self, pos: &pumpkin_util::math::vector3::Vector3<i32>) -> f64;
            fn sample_blend_offset(&mut self, pos: &pumpkin_util::math::vector3::Vector3<i32>) -> f64;
            fn sample_blend_density(&mut self, input_val: f64, pos: &pumpkin_util::math::vector3::Vector3<i32>) -> f64;
            fn sample_end_islands(&mut self, pos: &pumpkin_util::math::vector3::Vector3<i32>) -> f64;
            fn sample_wrapper(&mut self, wrapper_index: usize, wrapper_type: WrapperType, pos: &pumpkin_util::math::vector3::Vector3<i32>, eval_input: &dyn Fn(&pumpkin_util::math::vector3::Vector3<i32>, &mut Self) -> f64) -> f64;
            fn sample_spline(&mut self, spline_index: usize, location_value: f64, pos: &pumpkin_util::math::vector3::Vector3<i32>) -> f64;
            fn sample_find_top_surface(&mut self, density_fn: &dyn Fn(&pumpkin_util::math::vector3::Vector3<i32>, &mut Self) -> f64, upper_bound_fn: &dyn Fn(&pumpkin_util::math::vector3::Vector3<i32>, &mut Self) -> f64, lower_bound: i32, cell_height: i32, pos: &pumpkin_util::math::vector3::Vector3<i32>) -> f64;
        }

        #overworld_compiled
        #nether_compiled
        #end_compiled

        pub struct NoiseData {
            pub noise_id: DoublePerlinNoiseParameters,
            pub xz_scale: f64,
            pub y_scale: f64,
        }

        pub struct FindTopSurfaceData {
            pub lower_bound: i32,
            pub cell_height: i32,
        }

        pub struct ShiftedNoiseData {
            pub xz_scale: f64,
            pub y_scale: f64,
            pub noise_id: DoublePerlinNoiseParameters,
        }


        pub struct InterpolatedNoiseSamplerData {
            pub scaled_xz_scale: f64,
            pub scaled_y_scale: f64,
            pub xz_factor: f64,
            pub y_factor: f64,
            pub smear_scale_multiplier: f64,
        }

        pub struct ClampedYGradientData {
            pub from_y: f64,
            pub to_y: f64,
            pub from_value: f64,
            pub to_value: f64,
        }

        impl ClampedYGradientData {
            #[inline]
            #[must_use]
            pub fn apply_y(&self, y: f64) -> f64 {
                let clamped = y.clamp(self.from_y, self.to_y);
                let delta = (clamped - self.from_y) / (self.to_y - self.from_y);
                self.from_value + delta * (self.to_value - self.from_value)
            }
        }

        #[derive(Copy, Clone)]
        pub enum BinaryOperation {
            Add,
            Mul,
            Min,
            Max,
        }

        pub struct BinaryData {
            pub operation: BinaryOperation,
        }

        impl BinaryData {
            #[inline]
            #[must_use]
            pub const fn apply_density(&self, a: f64, b: f64) -> f64 {
                match self.operation {
                    BinaryOperation::Add => a + b,
                    BinaryOperation::Mul => a * b,
                    BinaryOperation::Min => a.min(b),
                    BinaryOperation::Max => a.max(b),
                }
            }
        }

        #[derive(Copy, Clone)]
        pub enum LinearOperation {
            Add,
            Mul,
        }

        pub struct LinearData {
            pub operation: LinearOperation,
            pub argument: f64,
        }

        impl LinearData {
            #[inline]
            #[must_use]
            pub const fn apply_density(&self, density: f64) -> f64 {
                match self.operation {
                    LinearOperation::Add => density + self.argument,
                    LinearOperation::Mul => density * self.argument,
                }
            }
        }

        #[derive(Copy, Clone)]
        pub enum UnaryOperation {
            Abs,
            Square,
            Cube,
            HalfNegative,
            QuarterNegative,
            Squeeze,
            Invert,  // new in 26.1
        }

        pub struct UnaryData {
            pub operation: UnaryOperation,
        }

        impl UnaryData {
            #[inline]
            #[must_use]
            #[allow(clippy::too_many_lines)]
            pub const fn apply_density(&self, density: f64) -> f64 {
                match self.operation {
                    UnaryOperation::Abs => density.abs(),
                    UnaryOperation::Square => density * density,
                    UnaryOperation::Cube => density * density * density,
                    UnaryOperation::HalfNegative => {
                        if density > 0.0 {
                            density
                        } else {
                            density * 0.5
                        }
                    }
                    UnaryOperation::QuarterNegative => {
                        if density > 0.0 {
                            density
                        } else {
                            density * 0.25
                        }
                    }
                    UnaryOperation::Squeeze => {
                        let clamped = density.clamp(-1.0, 1.0);
                        clamped / 2.0 - clamped * clamped * clamped / 24.0
                    }
                    UnaryOperation::Invert => {
                        if density == 0.0 { f64::INFINITY } else { 1.0 / density }
                    },
                }
            }
        }

        pub struct ClampData {
            pub min_value: f64,
            pub max_value: f64,
        }

        impl ClampData {
            #[inline]
            #[must_use]
            pub const fn apply_density(&self, density: f64) -> f64 {
                density.clamp(self.min_value, self.max_value)
            }
        }

        pub struct RangeChoiceData {
            pub min_inclusive: f64,
            pub max_exclusive: f64,
        }

        pub struct SplinePoint {
            pub location: f32,
            pub value: &'static SplineRepr,
            pub derivative: f32,
        }

        pub enum SplineRepr {
            Standard {
                location_function_index: usize,
                points: &'static [SplinePoint],
            },
            Fixed { value: f32 },
        }

        #[derive(Copy, Clone)]
        pub enum WrapperType {
            Interpolated,
            CacheFlat,
            Cache2D,
            CacheOnce,
            CellCache,
        }

        pub enum BaseNoiseFunctionComponent {
            // This is a placeholder for leaving space for world structures
            Beardifier,
            // These functions are initialized by a seed at runtime
            BlendAlpha,
            BlendOffset,
            BlendDensity {
                input_index: usize,
            },
            FindTopSurface {
                density_index: usize,
                upper_bound_index: usize,
                data: &'static FindTopSurfaceData,
            },
            EndIslands,
            Noise {
                data: &'static NoiseData,
            },
            ShiftA {
                noise_id: DoublePerlinNoiseParameters,
            },
            ShiftB {
                noise_id: DoublePerlinNoiseParameters,
            },
            ShiftedNoise {
                shift_x_index: usize,
                shift_y_index: usize,
                shift_z_index: usize,
                data: &'static ShiftedNoiseData,
            },
            InterpolatedNoiseSampler {
                data: &'static InterpolatedNoiseSamplerData,
            },
            IntervalSelect {
                input_index: usize,
                thresholds: &'static [f64],
                functions_indices: &'static [usize],
            },
            // The wrapped function is wrapped in a new wrapper at runtime
            Wrapper {
                input_index: usize,
                wrapper: WrapperType,
            },
            // These functions are unchanged except possibly for internal functions
            Constant {
                value: f64,
            },
            ClampedYGradient {
                data: &'static ClampedYGradientData,
            },
            Binary {
                argument1_index: usize,
                argument2_index: usize,
                data: &'static BinaryData,
            },
            Linear {
                input_index: usize,
                data: &'static LinearData,
            },
            Unary {
                input_index: usize,
                data: &'static UnaryData,
            },
            Clamp {
                input_index: usize,
                data: &'static ClampData,
            },
            RangeChoice {
                input_index: usize,
                when_in_range_index: usize,
                when_out_range_index: usize,
                data: &'static RangeChoiceData,
            },
            Spline {
                spline: &'static SplineRepr,
            },
        }

        pub struct BaseNoiseRouter {
            pub full_component_stack: &'static [BaseNoiseFunctionComponent],
            pub barrier_noise: usize,
            pub fluid_level_floodedness_noise: usize,
            pub fluid_level_spread_noise: usize,
            pub lava_noise: usize,
            pub erosion: usize,
            pub depth: usize,
            pub final_density: usize,
            pub vein_toggle: usize,
            pub vein_ridged: usize,
            pub vein_gap: usize,
        }

        pub struct BaseSurfaceEstimator {
            pub full_component_stack: &'static [BaseNoiseFunctionComponent],
        }

        pub struct BaseMultiNoiseRouter {
            pub full_component_stack: &'static [BaseNoiseFunctionComponent],
            pub temperature: usize,
            pub vegetation: usize,
            pub continents: usize,
            pub erosion: usize,
            pub depth: usize,
            pub ridges: usize,
        }

        pub struct BaseNoiseRouters {
            pub noise: BaseNoiseRouter,
            pub surface_estimator: BaseSurfaceEstimator,
            pub multi_noise: BaseMultiNoiseRouter,
        }

        pub const OVERWORLD_BASE_NOISE_ROUTER: BaseNoiseRouters = #overworld_router;
        pub const NETHER_BASE_NOISE_ROUTER: BaseNoiseRouters = #nether_router;
        pub const END_BASE_NOISE_ROUTER: BaseNoiseRouters = #end_router;
    }
}
