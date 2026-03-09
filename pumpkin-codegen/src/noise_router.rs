use std::{
    collections::BTreeMap,
    fs,
    hash::{DefaultHasher, Hash, Hasher},
};

use proc_macro2::{Punct, Spacing, Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use serde::Deserialize;
use syn::Ident;

/// Wraps an `f32` to provide a bitwise-exact `Hash` implementation for use as a map key.
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
#[derive(Deserialize, Hash)]
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
    /// Emits the `SplineRepr` token stream, registering any subcomponents into `stack`.
    ///
    /// # Arguments
    /// – `stack` – accumulator of all unique density-function component token streams.
    /// – `hash_to_index_map` – maps a component's hash to its index in `stack`.
    fn get_token_stream(
        &self,
        stack: &mut Vec<TokenStream>,
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
                    location_function.get_index_for_component(stack, hash_to_index_map);

                let point_reprs = points
                    .into_iter()
                    .map(|(location, value, derivative)| {
                        let value_repr = value.get_token_stream(stack, hash_to_index_map);

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

/// Rarity-value mapper used by the `WeirdScaled` density function.
#[derive(Deserialize, Hash, Copy, Clone)]
enum WeirdScaledMapper {
    /// Cave-type scaling curve (TYPE2).
    #[serde(rename(deserialize = "TYPE2"))]
    Caves,
    /// Tunnel-type scaling curve (TYPE1).
    #[serde(rename(deserialize = "TYPE1"))]
    Tunnels,
}

impl WeirdScaledMapper {
    /// Emits the token stream for this mapper variant.
    fn into_token_stream(self) -> TokenStream {
        match self {
            Self::Caves => {
                quote! {
                    WeirdScaledMapper::Caves
                }
            }
            Self::Tunnels => {
                quote! {
                    WeirdScaledMapper::Tunnels
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
#[derive(Deserialize, Hash)]
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
#[derive(Deserialize, Hash)]
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

/// Deserialized parameters for a weird-scaled-sampler density function.
#[derive(Deserialize, Hash, Clone)]
struct WeirdScaledData {
    /// Resource location ID of the noise generator.
    #[serde(rename(deserialize = "noise"))]
    noise_id: String,
    /// The rarity-value mapper that scales the noise output.
    #[serde(rename(deserialize = "rarityValueMapper"))]
    mapper: WeirdScaledMapper,
}

/// Deserialized parameters for the interpolated noise sampler density function.
#[derive(Deserialize, Hash)]
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
    // These are unused currently
    //#[serde(rename(deserialize = "xzScale"))]
    //xz_scale: HashableF64,
    //#[serde(rename(deserialize = "yScale"))]
    //y_scale: HashableF64,
}

/// Deserialized parameters for a clamped Y-gradient density function.
#[derive(Deserialize, Hash)]
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
#[derive(Deserialize, Hash)]
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
#[derive(Deserialize, Hash)]
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

/// Deserialized parameters for a unary density function transformation.
#[derive(Deserialize, Hash)]
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
#[derive(Deserialize, Hash)]
struct ClampData {
    /// Lower bound of the clamp range.
    #[serde(rename(deserialize = "minValue"))]
    min_value: HashableF64,
    /// Upper bound of the clamp range.
    #[serde(rename(deserialize = "maxValue"))]
    max_value: HashableF64,
}

/// Deserialized range bounds for the `RangeChoice` density function.
#[derive(Deserialize, Hash)]
struct RangeChoiceData {
    /// Inclusive lower bound of the "in-range" interval.
    #[serde(rename(deserialize = "minInclusive"))]
    min_inclusive: HashableF64,
    /// Exclusive upper bound of the "in-range" interval.
    #[serde(rename(deserialize = "maxExclusive"))]
    max_exclusive: HashableF64,
}

/// Deserialized output-range metadata for a spline density function.
#[derive(Deserialize, Hash)]
struct SplineData {
    /// Minimum possible output value of the spline.
    #[serde(rename(deserialize = "minValue"))]
    min_value: HashableF64,
    /// Maximum possible output value of the spline.
    #[serde(rename(deserialize = "maxValue"))]
    max_value: HashableF64,
}

/// Deserialized representation of any density function node in the noise router tree.
#[derive(Deserialize, Hash)]
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
    #[serde(rename(deserialize = "WeirdScaledSampler"))]
    WeirdScaled {
        /// The density function to scale.
        input: Box<Self>,
        /// Noise ID and mapper type for scaling.
        #[serde(flatten)]
        data: WeirdScaledData,
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
    /// Computes a stable 64-bit hash for this density function node.
    fn unique_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    /// Returns the index of this component in `stack`, inserting it if not yet present.
    ///
    /// # Arguments
    /// – `stack` – accumulator of all unique density-function component token streams.
    /// – `hash_to_index_map` – maps a component's hash to its index in `stack`.
    fn get_index_for_component(
        &self,
        stack: &mut Vec<TokenStream>,
        hash_to_index_map: &mut BTreeMap<u64, usize>,
    ) -> usize {
        if let Some(index) = hash_to_index_map.get(&self.unique_id()) {
            *index
        } else {
            let id = self.unique_id();
            let repr = self.get_token_stream(stack, hash_to_index_map);
            stack.push(repr);
            let index = stack.len() - 1;
            hash_to_index_map.insert(id, index);
            index
        }
    }

    /// Emits the `BaseNoiseFunctionComponent` token stream for this node, registering subcomponents into `stack`.
    ///
    /// # Arguments
    /// – `stack` – accumulator of all unique density-function component token streams.
    /// – `hash_to_index_map` – maps a component's hash to its index in `stack`.
    fn get_token_stream(
        &self,
        stack: &mut Vec<TokenStream>,
        hash_to_index_map: &mut BTreeMap<u64, usize>,
    ) -> TokenStream {
        match self {
            Self::Spline { spline, data } => {
                let _ = data;
                let spline_repr = spline.get_token_stream(stack, hash_to_index_map);

                quote! {
                    BaseNoiseFunctionComponent::Spline {
                        spline: &#spline_repr,
                    }
                }
            }
            Self::EndIslands => quote! {
                BaseNoiseFunctionComponent::EndIslands
            },
            Self::Noise { data } => {
                let noise_id = &data.noise_id;
                let xz_scale = &data.xz_scale;
                let y_scale = &data.y_scale;

                quote! {
                    BaseNoiseFunctionComponent::Noise {
                        data: &NoiseData {
                            noise_id: #noise_id,
                            xz_scale: #xz_scale,
                            y_scale: #y_scale,
                        }
                    }
                }
            }
            Self::ShiftA { noise_id } => {
                quote! {
                    BaseNoiseFunctionComponent::ShiftA {
                        noise_id: #noise_id
                    }
                }
            }
            Self::ShiftB { noise_id } => {
                quote! {
                    BaseNoiseFunctionComponent::ShiftB {
                        noise_id: #noise_id
                    }
                }
            }
            Self::BlendDensity { input } => {
                let input_index = input.get_index_for_component(stack, hash_to_index_map);

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
                let shift_x_index = shift_x.get_index_for_component(stack, hash_to_index_map);
                let shift_y_index = shift_y.get_index_for_component(stack, hash_to_index_map);
                let shift_z_index = shift_z.get_index_for_component(stack, hash_to_index_map);

                let xz_scale = &data.xz_scale;
                let y_scale = &data.y_scale;
                let noise_id = &data.noise_id;

                quote! {
                    BaseNoiseFunctionComponent::ShiftedNoise {
                        shift_x_index: #shift_x_index,
                        shift_y_index: #shift_y_index,
                        shift_z_index: #shift_z_index,
                        data: &ShiftedNoiseData {
                            xz_scale: #xz_scale,
                            y_scale: #y_scale,
                            noise_id: #noise_id,
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
                let input_index = input.get_index_for_component(stack, hash_to_index_map);
                let when_in_index = when_in_range.get_index_for_component(stack, hash_to_index_map);
                let when_out_index =
                    when_out_range.get_index_for_component(stack, hash_to_index_map);

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
                let argument1_index = argument1.get_index_for_component(stack, hash_to_index_map);
                let argument2_index = argument2.get_index_for_component(stack, hash_to_index_map);

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
                let input_index = input.get_index_for_component(stack, hash_to_index_map);
                let wrapper_repr = wrapper.into_token_stream();

                quote! {
                    BaseNoiseFunctionComponent::Wrapper {
                        input_index: #input_index,
                        wrapper: #wrapper_repr,
                    }
                }
            }
            Self::Linear { input, data } => {
                let input_index = input.get_index_for_component(stack, hash_to_index_map);

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
                let input_index = input.get_index_for_component(stack, hash_to_index_map);

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
                let input_index = input.get_index_for_component(stack, hash_to_index_map);

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
            Self::WeirdScaled { input, data } => {
                let input_index = input.get_index_for_component(stack, hash_to_index_map);

                let noise_id = &data.noise_id;
                let action = data.mapper.into_token_stream();

                quote! {
                    BaseNoiseFunctionComponent::WeirdScaled {
                        input_index: #input_index,
                        data: &WeirdScaledData {
                            noise_id: #noise_id,
                            mapper: #action,
                        },
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
    #[serde(rename(deserialize = "initialDensityWithoutJaggedness"))]
    initial_density_without_jaggedness: DensityFunctionRepr,
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
    /// Consumes this router representation and emits the `BaseNoiseRouters` token stream.
    fn into_token_stream(self) -> TokenStream {
        let mut noise_component_stack = Vec::new();
        let mut noise_lookup_map = BTreeMap::new();

        // The aquifer sampler is called most often
        let final_density = self
            .final_density
            .get_index_for_component(&mut noise_component_stack, &mut noise_lookup_map);
        let barrier_noise = self
            .barrier_noise
            .get_index_for_component(&mut noise_component_stack, &mut noise_lookup_map);
        let fluid_level_floodedness_noise = self
            .fluid_level_floodedness_noise
            .get_index_for_component(&mut noise_component_stack, &mut noise_lookup_map);
        let fluid_level_spread_noise = self
            .fluid_level_spread_noise
            .get_index_for_component(&mut noise_component_stack, &mut noise_lookup_map);
        let lava_noise = self
            .lava_noise
            .get_index_for_component(&mut noise_component_stack, &mut noise_lookup_map);

        // Ore sampler is called fewer times than aquifer sampler
        let vein_toggle = self
            .vein_toggle
            .get_index_for_component(&mut noise_component_stack, &mut noise_lookup_map);
        let vein_ridged = self
            .vein_ridged
            .get_index_for_component(&mut noise_component_stack, &mut noise_lookup_map);
        let vein_gap = self
            .vein_gap
            .get_index_for_component(&mut noise_component_stack, &mut noise_lookup_map);

        // These should all be cached so it doesn't matter where their components are
        let noise_erosion = self
            .erosion
            .get_index_for_component(&mut noise_component_stack, &mut noise_lookup_map);
        let noise_depth = self
            .depth
            .get_index_for_component(&mut noise_component_stack, &mut noise_lookup_map);

        let mut surface_component_stack = Vec::new();
        let mut surface_lookup_map = BTreeMap::new();
        let _ = self
            .initial_density_without_jaggedness
            .get_index_for_component(&mut surface_component_stack, &mut surface_lookup_map);

        let mut multinoise_component_stack = Vec::new();
        let mut multinoise_lookup_map = BTreeMap::new();
        let ridges = self
            .ridges
            .get_index_for_component(&mut multinoise_component_stack, &mut multinoise_lookup_map);
        let temperature = self
            .temperature
            .get_index_for_component(&mut multinoise_component_stack, &mut multinoise_lookup_map);
        let vegetation = self
            .vegetation
            .get_index_for_component(&mut multinoise_component_stack, &mut multinoise_lookup_map);
        let continents = self
            .continents
            .get_index_for_component(&mut multinoise_component_stack, &mut multinoise_lookup_map);
        let multi_erosion = self
            .erosion
            .get_index_for_component(&mut multinoise_component_stack, &mut multinoise_lookup_map);
        let multi_depth = self
            .depth
            .get_index_for_component(&mut multinoise_component_stack, &mut multinoise_lookup_map);

        quote! {
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
        }
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

/// Reads `density_function.json` and emits the complete noise-router constants `TokenStream`.
pub fn build() -> TokenStream {
    let mut reprs: NoiseRouterReprs =
        serde_json5::from_str(&fs::read_to_string("../assets/density_function.json").unwrap())
            .expect("could not deserialize density_function.json");

    // The `final_density` function is mutated at runtime for the aquifer generator in Java.
    fix_final_density!(reprs.overworld);
    fix_final_density!(reprs.overworld_amplified);
    fix_final_density!(reprs.overworld_large_biomes);
    fix_final_density!(reprs.nether);

    let _ = reprs.end;
    let _ = reprs.end_islands;

    let overworld_router = reprs.overworld.into_token_stream();
    let nether_router = reprs.nether.into_token_stream();
    let end_router = reprs.end.into_token_stream();

    quote! {
        pub struct NoiseData {
            pub noise_id: &'static str,
            pub xz_scale: f64,
            pub y_scale: f64,
        }

        pub struct ShiftedNoiseData {
            pub xz_scale: f64,
            pub y_scale: f64,
            pub noise_id: &'static str,
        }

        #[derive(Copy, Clone)]
        pub enum WeirdScaledMapper {
            Caves,
            Tunnels,
        }

        impl WeirdScaledMapper {
            #[inline]
            pub fn max_multiplier(&self) -> f64 {
                match self {
                    Self::Tunnels => 2.0,
                    Self::Caves => 3.0,
                }
            }

            #[inline]
            pub fn scale(&self, value: f64) -> f64 {
                match self {
                    Self::Tunnels => {
                        if value < -0.5 {
                            0.75
                        } else if value < 0.0 {
                            1.0
                        } else if value < 0.5 {
                            1.5
                        } else {
                            2.0
                        }
                    }
                    Self::Caves => {
                        if value < -0.75 {
                            0.5
                        } else if value < -0.5 {
                            0.75
                        } else if value < 0.5 {
                            1.0
                        } else if value < 0.75 {
                            2.0
                        } else {
                            3.0
                        }
                    }
                }
            }
        }

        pub struct WeirdScaledData {
            pub noise_id: &'static str,
            pub mapper: WeirdScaledMapper,
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
            pub fn apply_density(&self, density: f64) -> f64 {
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
        }

        pub struct UnaryData {
            pub operation: UnaryOperation,
        }

        impl UnaryData {
            #[inline]
            pub fn apply_density(&self, density: f64) -> f64 {
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
                }
            }
        }

        pub struct ClampData {
            pub min_value: f64,
            pub max_value: f64,
        }

        impl ClampData {
            #[inline]
            pub fn apply_density(&self, density: f64) -> f64 {
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
            EndIslands,
            Noise {
                data: &'static NoiseData,
            },
            ShiftA {
                noise_id: &'static str,
            },
            ShiftB {
                noise_id: &'static str,
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
            WeirdScaled {
                input_index: usize,
                data: &'static WeirdScaledData,
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
