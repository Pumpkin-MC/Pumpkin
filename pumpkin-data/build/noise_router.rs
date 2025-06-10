use std::{
    collections::HashMap,
    fs,
    hash::{DefaultHasher, Hash, Hasher},
};

use proc_macro2::{Punct, Spacing, Span, TokenStream};
use quote::{ToTokens, TokenStreamExt, quote};
use serde::Deserialize;
use syn::Ident;

#[derive(Clone)]
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
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        f32::deserialize(deserializer).map(Self)
    }
}

#[derive(Clone)]
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
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        f64::deserialize(deserializer).map(Self)
    }
}

#[derive(Deserialize, Hash, Clone)]
#[serde(tag = "_type", content = "value")]
enum SplineRepr {
    #[serde(rename(deserialize = "standard"))]
    Standard {
        #[serde(rename(deserialize = "locationFunction"))]
        location_function: Box<DensityFunctionRepr>,
        locations: Box<[HashableF32]>,
        values: Box<[SplineRepr]>,
        derivatives: Box<[HashableF32]>,
    },
    #[serde(rename(deserialize = "fixed"))]
    Fixed { value: HashableF32 },
}

impl SplineRepr {
    fn into_token_stream(
        self,
        stack: &mut Vec<TokenStream>,
        hash_to_index_map: &mut HashMap<u64, usize>,
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
                    .map(|((location, value), derivative)| (location, value, derivative))
                    .collect::<Vec<_>>();

                let function_index =
                    location_function.get_index_for_component(stack, hash_to_index_map);

                let point_reprs = points
                    .into_iter()
                    .map(|(location, value, derivative)| {
                        let value_repr = value.into_token_stream(stack, hash_to_index_map);

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

#[derive(Deserialize, Hash, Copy, Clone)]
enum BinaryOperation {
    #[serde(rename(deserialize = "ADD"))]
    Add,
    #[serde(rename(deserialize = "MUL"))]
    Mul,
    #[serde(rename(deserialize = "MIN"))]
    Min,
    #[serde(rename(deserialize = "MAX"))]
    Max,
}

impl BinaryOperation {
    fn into_token_stream(self) -> TokenStream {
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

#[derive(Deserialize, Hash, Copy, Clone)]
enum LinearOperation {
    #[serde(rename(deserialize = "ADD"))]
    Add,
    #[serde(rename(deserialize = "MUL"))]
    Mul,
}

impl LinearOperation {
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

#[derive(Deserialize, Hash, Copy, Clone)]
enum UnaryOperation {
    #[serde(rename(deserialize = "ABS"))]
    Abs,
    #[serde(rename(deserialize = "SQUARE"))]
    Square,
    #[serde(rename(deserialize = "CUBE"))]
    Cube,
    #[serde(rename(deserialize = "HALF_NEGATIVE"))]
    HalfNegative,
    #[serde(rename(deserialize = "QUARTER_NEGATIVE"))]
    QuarterNegative,
    #[serde(rename(deserialize = "SQUEEZE"))]
    Squeeze,
}

impl UnaryOperation {
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

#[derive(Deserialize, Hash, Copy, Clone)]
enum WeirdScaledMapper {
    #[serde(rename(deserialize = "TYPE2"))]
    Caves,
    #[serde(rename(deserialize = "TYPE1"))]
    Tunnels,
}

impl WeirdScaledMapper {
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

#[derive(Copy, Clone, Deserialize, PartialEq, Eq, Hash)]
enum WrapperType {
    Interpolated,
    #[serde(rename(deserialize = "FlatCache"))]
    CacheFlat,
    Cache2D,
    CacheOnce,
    CellCache,
}

impl WrapperType {
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

#[derive(Deserialize, Hash, Clone)]
struct NoiseData {
    #[serde(rename(deserialize = "noise"))]
    noise_id: String,
    #[serde(rename(deserialize = "xzScale"))]
    xz_scale: HashableF64,
    #[serde(rename(deserialize = "yScale"))]
    y_scale: HashableF64,
}

#[derive(Deserialize, Hash, Clone)]
struct ShiftedNoiseData {
    #[serde(rename(deserialize = "xzScale"))]
    xz_scale: HashableF64,
    #[serde(rename(deserialize = "yScale"))]
    y_scale: HashableF64,
    #[serde(rename(deserialize = "noise"))]
    noise_id: String,
}

#[derive(Deserialize, Hash, Clone)]
struct WeirdScaledData {
    #[serde(rename(deserialize = "noise"))]
    noise_id: String,
    #[serde(rename(deserialize = "rarityValueMapper"))]
    mapper: WeirdScaledMapper,
}

#[derive(Deserialize, Hash, Clone)]
struct InterpolatedNoiseSamplerData {
    #[serde(rename(deserialize = "scaledXzScale"))]
    scaled_xz_scale: HashableF64,
    #[serde(rename(deserialize = "scaledYScale"))]
    scaled_y_scale: HashableF64,
    #[serde(rename(deserialize = "xzFactor"))]
    xz_factor: HashableF64,
    #[serde(rename(deserialize = "yFactor"))]
    y_factor: HashableF64,
    #[serde(rename(deserialize = "smearScaleMultiplier"))]
    smear_scale_multiplier: HashableF64,
    #[serde(rename(deserialize = "maxValue"))]
    max_value: HashableF64,
    // These are unused currently
    //#[serde(rename(deserialize = "xzScale"))]
    //xz_scale: HashableF64,
    //#[serde(rename(deserialize = "yScale"))]
    //y_scale: HashableF64,
}

#[derive(Deserialize, Hash, Clone)]
struct ClampedYGradientData {
    #[serde(rename(deserialize = "fromY"))]
    from_y: i32,
    #[serde(rename(deserialize = "toY"))]
    to_y: i32,
    #[serde(rename(deserialize = "fromValue"))]
    from_value: HashableF64,
    #[serde(rename(deserialize = "toValue"))]
    to_value: HashableF64,
}

#[derive(Deserialize, Hash, Clone)]
struct BinaryData {
    #[serde(rename(deserialize = "type"))]
    operation: BinaryOperation,
    #[serde(rename(deserialize = "minValue"))]
    min_value: HashableF64,
    #[serde(rename(deserialize = "maxValue"))]
    max_value: HashableF64,
}

#[derive(Deserialize, Hash, Clone)]
struct LinearData {
    #[serde(rename(deserialize = "specificType"))]
    operation: LinearOperation,
    argument: HashableF64,
    #[serde(rename(deserialize = "minValue"))]
    min_value: HashableF64,
    #[serde(rename(deserialize = "maxValue"))]
    max_value: HashableF64,
}

#[derive(Deserialize, Hash, Clone)]
struct UnaryData {
    #[serde(rename(deserialize = "type"))]
    operation: UnaryOperation,
    #[serde(rename(deserialize = "minValue"))]
    min_value: HashableF64,
    #[serde(rename(deserialize = "maxValue"))]
    max_value: HashableF64,
}

#[derive(Deserialize, Hash, Clone)]
struct ClampData {
    #[serde(rename(deserialize = "minValue"))]
    min_value: HashableF64,
    #[serde(rename(deserialize = "maxValue"))]
    max_value: HashableF64,
}

#[derive(Deserialize, Hash, Clone)]
struct RangeChoiceData {
    #[serde(rename(deserialize = "minInclusive"))]
    min_inclusive: HashableF64,
    #[serde(rename(deserialize = "maxExclusive"))]
    max_exclusive: HashableF64,
}

#[derive(Deserialize, Hash, Clone)]
struct SplineData {
    #[serde(rename(deserialize = "minValue"))]
    min_value: HashableF64,
    #[serde(rename(deserialize = "maxValue"))]
    max_value: HashableF64,
}

#[derive(Deserialize, Hash, Clone)]
#[serde(tag = "_class", content = "value")]
enum DensityFunctionRepr {
    // This is a placeholder for leaving space for world structures
    Beardifier,
    // These functions is initialized by a seed at runtime
    BlendAlpha,
    BlendOffset,
    BlendDensity {
        input: Box<DensityFunctionRepr>,
    },
    EndIslands,
    Noise {
        #[serde(flatten)]
        data: NoiseData,
    },
    ShiftA {
        #[serde(rename(deserialize = "offsetNoise"))]
        noise_id: String,
    },
    ShiftB {
        #[serde(rename(deserialize = "offsetNoise"))]
        noise_id: String,
    },
    ShiftedNoise {
        #[serde(rename(deserialize = "shiftX"))]
        shift_x: Box<DensityFunctionRepr>,
        #[serde(rename(deserialize = "shiftY"))]
        shift_y: Box<DensityFunctionRepr>,
        #[serde(rename(deserialize = "shiftZ"))]
        shift_z: Box<DensityFunctionRepr>,
        #[serde(flatten)]
        data: ShiftedNoiseData,
    },
    InterpolatedNoiseSampler {
        #[serde(flatten)]
        data: InterpolatedNoiseSamplerData,
    },
    #[serde(rename(deserialize = "WeirdScaledSampler"))]
    WeirdScaled {
        input: Box<DensityFunctionRepr>,
        #[serde(flatten)]
        data: WeirdScaledData,
    },
    // The wrapped function is wrapped in a new wrapper at runtime
    #[serde(rename(deserialize = "Wrapping"))]
    Wrapper {
        #[serde(rename(deserialize = "wrapped"))]
        input: Box<DensityFunctionRepr>,
        #[serde(rename(deserialize = "type"))]
        wrapper: WrapperType,
    },
    // These functions are unchanged except possibly for internal functions
    Constant {
        value: HashableF64,
    },
    #[serde(rename(deserialize = "YClampedGradient"))]
    ClampedYGradient {
        #[serde(flatten)]
        data: ClampedYGradientData,
    },
    #[serde(rename(deserialize = "BinaryOperation"))]
    Binary {
        argument1: Box<DensityFunctionRepr>,
        argument2: Box<DensityFunctionRepr>,
        #[serde(flatten)]
        data: BinaryData,
    },
    #[serde(rename(deserialize = "LinearOperation"))]
    Linear {
        input: Box<DensityFunctionRepr>,
        #[serde(flatten)]
        data: LinearData,
    },
    #[serde(rename(deserialize = "UnaryOperation"))]
    Unary {
        input: Box<DensityFunctionRepr>,
        #[serde(flatten)]
        data: UnaryData,
    },
    Clamp {
        input: Box<DensityFunctionRepr>,
        #[serde(flatten)]
        data: ClampData,
    },
    RangeChoice {
        input: Box<DensityFunctionRepr>,
        #[serde(rename(deserialize = "whenInRange"))]
        when_in_range: Box<DensityFunctionRepr>,
        #[serde(rename(deserialize = "whenOutOfRange"))]
        when_out_range: Box<DensityFunctionRepr>,
        #[serde(flatten)]
        data: RangeChoiceData,
    },
    Spline {
        spline: SplineRepr,
        #[serde(flatten)]
        data: SplineData,
    },
}

impl DensityFunctionRepr {
    fn unique_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn get_index_for_component(
        self,
        stack: &mut Vec<TokenStream>,
        hash_to_index_map: &mut HashMap<u64, usize>,
    ) -> usize {
        if let Some(index) = hash_to_index_map.get(&self.unique_id()) {
            *index
        } else {
            let id = self.unique_id();
            let repr = self.into_token_stream(stack, hash_to_index_map);
            stack.push(repr);
            let index = stack.len() - 1;
            hash_to_index_map.insert(id, index);
            index
        }
    }

    fn into_token_stream(
        self,
        stack: &mut Vec<TokenStream>,
        hash_to_index_map: &mut HashMap<u64, usize>,
    ) -> TokenStream {
        match self {
            Self::Spline { spline, data } => {
                let _ = data;
                let spline_repr = spline.into_token_stream(stack, hash_to_index_map);

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
                let noise_id = data.noise_id;
                let xz_scale = data.xz_scale;
                let y_scale = data.y_scale;

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

                let xz_scale = data.xz_scale;
                let y_scale = data.y_scale;
                let noise_id = data.noise_id;

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

                let min_inclusive = data.min_inclusive;
                let max_exclusive = data.max_exclusive;

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

                let action = data.operation.into_token_stream();
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
                let from_y = data.from_y as f64;
                let to_y = data.to_y as f64;
                let from_value = data.from_value;
                let to_value = data.to_value;

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
                let argument = data.argument;
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

                let min_value = data.min_value;
                let max_value = data.max_value;

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

                let noise_id = data.noise_id;
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
                let scaled_xz_scale = data.scaled_xz_scale;
                let scaled_y_scale = data.scaled_y_scale;
                let xz_factor = data.xz_factor;
                let y_factor = data.y_factor;
                let smear_scale_multiplier = data.smear_scale_multiplier;

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

#[derive(Deserialize)]
struct NoiseRouterReprs {
    overworld: NoiseRouterRepr,
    #[serde(rename(deserialize = "large_biomes"))]
    overworld_large_biomes: NoiseRouterRepr,
    #[serde(rename(deserialize = "amplified"))]
    overworld_amplified: NoiseRouterRepr,
    nether: NoiseRouterRepr,
    end: NoiseRouterRepr,
    #[serde(rename(deserialize = "floating_islands"))]
    end_islands: NoiseRouterRepr,
}

#[derive(Deserialize)]
struct NoiseRouterRepr {
    #[serde(rename(deserialize = "barrierNoise"))]
    barrier_noise: DensityFunctionRepr,
    #[serde(rename(deserialize = "fluidLevelFloodednessNoise"))]
    fluid_level_floodedness_noise: DensityFunctionRepr,
    #[serde(rename(deserialize = "fluidLevelSpreadNoise"))]
    fluid_level_spread_noise: DensityFunctionRepr,
    #[serde(rename(deserialize = "lavaNoise"))]
    lava_noise: DensityFunctionRepr,
    temperature: DensityFunctionRepr,
    vegetation: DensityFunctionRepr,
    continents: DensityFunctionRepr,
    erosion: DensityFunctionRepr,
    depth: DensityFunctionRepr,
    ridges: DensityFunctionRepr,
    #[serde(rename(deserialize = "initialDensityWithoutJaggedness"))]
    initial_density_without_jaggedness: DensityFunctionRepr,
    #[serde(rename(deserialize = "finalDensity"))]
    final_density: DensityFunctionRepr,
    #[serde(rename(deserialize = "veinToggle"))]
    vein_toggle: DensityFunctionRepr,
    #[serde(rename(deserialize = "veinRidged"))]
    vein_ridged: DensityFunctionRepr,
    #[serde(rename(deserialize = "veinGap"))]
    vein_gap: DensityFunctionRepr,
}

impl NoiseRouterRepr {
    fn into_token_stream(self) -> TokenStream {
        let mut noise_component_stack = Vec::new();
        let mut noise_lookup_map = HashMap::new();

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
            .clone()
            .get_index_for_component(&mut noise_component_stack, &mut noise_lookup_map);
        let noise_depth = self
            .depth
            .clone()
            .get_index_for_component(&mut noise_component_stack, &mut noise_lookup_map);

        let mut surface_component_stack = Vec::new();
        let mut surface_lookup_map = HashMap::new();
        let _ = self
            .initial_density_without_jaggedness
            .get_index_for_component(&mut surface_component_stack, &mut surface_lookup_map);

        let mut multinoise_component_stack = Vec::new();
        let mut multinoise_lookup_map = HashMap::new();
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

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/density_function.json");

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
            // These functions is initialized by a seed at runtime
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
