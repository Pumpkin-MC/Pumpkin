use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use pumpkin_data::noise_router::{self, BaseNoiseFunctionComponent};
use serde::Deserialize;

// I do a lot of leaking here, but its only for testing

pub struct HashableF32(pub f32);

// Normally this is bad, but we just care about checking if components are the same
impl Hash for HashableF32 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_le_bytes().hash(state);
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

pub struct HashableF64(pub f64);

// Normally this is bad, but we just care about checking if components are the same
impl Hash for HashableF64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_le_bytes().hash(state);
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

#[derive(Deserialize, Hash)]
#[serde(tag = "_type", content = "value")]
pub enum SplineRepr {
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
    fn as_base_component(
        &self,
        stack: &mut Vec<BaseNoiseFunctionComponent>,
        map: &mut HashMap<u64, usize>,
    ) -> &'static noise_router::SplineRepr {
        let value = match self {
            SplineRepr::Standard {
                location_function,
                locations,
                values,
                derivatives,
            } => {
                let function_index = location_function.index(stack, map);

                // All this leaking... oh well its just for tests
                let points: Box<[noise_router::SplinePoint]> = locations
                    .iter()
                    .zip(values)
                    .zip(derivatives)
                    .map(|((l, v), d)| noise_router::SplinePoint {
                        location: l.0,
                        value: Box::leak(Box::new(v.as_base_component(stack, map))),
                        derivative: d.0,
                    })
                    .collect();

                noise_router::SplineRepr::Standard {
                    location_function_index: function_index,
                    points: Box::leak(points),
                }
            }
            SplineRepr::Fixed { value } => noise_router::SplineRepr::Fixed { value: value.0 },
        };

        Box::leak(Box::new(value))
    }
}

#[derive(Deserialize, Hash, Copy, Clone)]
pub enum BinaryOperation {
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
    fn as_base_component(&self) -> noise_router::BinaryOperation {
        match self {
            Self::Add => noise_router::BinaryOperation::Add,
            Self::Mul => noise_router::BinaryOperation::Mul,
            Self::Max => noise_router::BinaryOperation::Max,
            Self::Min => noise_router::BinaryOperation::Min,
        }
    }
}

#[derive(Deserialize, Hash, Copy, Clone)]
pub enum LinearOperation {
    #[serde(rename(deserialize = "ADD"))]
    Add,
    #[serde(rename(deserialize = "MUL"))]
    Mul,
}

impl LinearOperation {
    fn as_base_component(&self) -> noise_router::LinearOperation {
        match self {
            Self::Add => noise_router::LinearOperation::Add,
            Self::Mul => noise_router::LinearOperation::Mul,
        }
    }
}

#[derive(Deserialize, Hash, Copy, Clone)]
pub enum UnaryOperation {
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
    fn as_base_component(&self) -> noise_router::UnaryOperation {
        match self {
            Self::Abs => noise_router::UnaryOperation::Abs,
            Self::Square => noise_router::UnaryOperation::Square,
            Self::Cube => noise_router::UnaryOperation::Cube,
            Self::HalfNegative => noise_router::UnaryOperation::HalfNegative,
            Self::QuarterNegative => noise_router::UnaryOperation::QuarterNegative,
            Self::Squeeze => noise_router::UnaryOperation::Squeeze,
        }
    }
}

#[derive(Deserialize, Hash, Copy, Clone)]
pub enum WeirdScaledMapper {
    #[serde(rename(deserialize = "TYPE2"))]
    Caves,
    #[serde(rename(deserialize = "TYPE1"))]
    Tunnels,
}

impl WeirdScaledMapper {
    fn as_base_component(&self) -> noise_router::WeirdScaledMapper {
        match self {
            Self::Caves => noise_router::WeirdScaledMapper::Caves,
            Self::Tunnels => noise_router::WeirdScaledMapper::Tunnels,
        }
    }
}

#[derive(Copy, Clone, Deserialize, PartialEq, Eq, Hash)]
pub enum WrapperType {
    Interpolated,
    #[serde(rename(deserialize = "FlatCache"))]
    CacheFlat,
    Cache2D,
    CacheOnce,
    CellCache,
}

impl WrapperType {
    fn as_base_component(&self) -> noise_router::WrapperType {
        match self {
            Self::Interpolated => noise_router::WrapperType::Interpolated,
            Self::CacheFlat => noise_router::WrapperType::CacheFlat,
            Self::Cache2D => noise_router::WrapperType::Cache2D,
            Self::CacheOnce => noise_router::WrapperType::CacheOnce,
            Self::CellCache => noise_router::WrapperType::CellCache,
        }
    }
}

#[derive(Deserialize, Hash)]
pub struct NoiseData {
    #[serde(rename(deserialize = "noise"))]
    pub noise_id: String,
    #[serde(rename(deserialize = "xzScale"))]
    pub xz_scale: HashableF64,
    #[serde(rename(deserialize = "yScale"))]
    pub y_scale: HashableF64,
}

impl NoiseData {
    fn as_base_component(&self) -> &'static noise_router::NoiseData {
        let value = noise_router::NoiseData {
            noise_id: Box::leak(Box::new(self.noise_id.clone())),
            xz_scale: self.xz_scale.0,
            y_scale: self.y_scale.0,
        };

        Box::leak(Box::new(value))
    }
}

#[derive(Deserialize, Hash)]
pub struct ShiftedNoiseData {
    #[serde(rename(deserialize = "xzScale"))]
    pub xz_scale: HashableF64,
    #[serde(rename(deserialize = "yScale"))]
    pub y_scale: HashableF64,
    #[serde(rename(deserialize = "noise"))]
    pub noise_id: String,
}

impl ShiftedNoiseData {
    fn as_base_component(&self) -> &'static noise_router::ShiftedNoiseData {
        let value = noise_router::ShiftedNoiseData {
            noise_id: Box::leak(Box::new(self.noise_id.clone())),
            xz_scale: self.xz_scale.0,
            y_scale: self.y_scale.0,
        };

        Box::leak(Box::new(value))
    }
}

#[derive(Deserialize, Hash)]
pub struct WeirdScaledData {
    #[serde(rename(deserialize = "noise"))]
    pub noise_id: String,
    #[serde(rename(deserialize = "rarityValueMapper"))]
    pub mapper: WeirdScaledMapper,
}

impl WeirdScaledData {
    fn as_base_component(&self) -> &'static noise_router::WeirdScaledData {
        let value = noise_router::WeirdScaledData {
            noise_id: Box::leak(Box::new(self.noise_id.clone())),
            mapper: self.mapper.as_base_component(),
        };

        Box::leak(Box::new(value))
    }
}

#[derive(Deserialize, Hash)]
pub struct InterpolatedNoiseSamplerData {
    #[serde(rename(deserialize = "scaledXzScale"))]
    pub scaled_xz_scale: HashableF64,
    #[serde(rename(deserialize = "scaledYScale"))]
    pub scaled_y_scale: HashableF64,
    #[serde(rename(deserialize = "xzFactor"))]
    pub xz_factor: HashableF64,
    #[serde(rename(deserialize = "yFactor"))]
    pub y_factor: HashableF64,
    #[serde(rename(deserialize = "smearScaleMultiplier"))]
    pub smear_scale_multiplier: HashableF64,
    #[serde(rename(deserialize = "maxValue"))]
    pub max_value: HashableF64,
    // These are unused currently
    //#[serde(rename(deserialize = "xzScale"))]
    //xz_scale: HashableF64,
    //#[serde(rename(deserialize = "yScale"))]
    //y_scale: HashableF64,
}

impl InterpolatedNoiseSamplerData {
    fn as_base_component(&self) -> &'static noise_router::InterpolatedNoiseSamplerData {
        let value = noise_router::InterpolatedNoiseSamplerData {
            scaled_xz_scale: self.scaled_xz_scale.0,
            scaled_y_scale: self.scaled_y_scale.0,
            xz_factor: self.xz_factor.0,
            y_factor: self.y_factor.0,
            smear_scale_multiplier: self.smear_scale_multiplier.0,
        };

        Box::leak(Box::new(value))
    }
}

#[derive(Deserialize, Hash)]
pub struct ClampedYGradientData {
    #[serde(rename(deserialize = "fromY"))]
    pub from_y: i32,
    #[serde(rename(deserialize = "toY"))]
    pub to_y: i32,
    #[serde(rename(deserialize = "fromValue"))]
    pub from_value: HashableF64,
    #[serde(rename(deserialize = "toValue"))]
    pub to_value: HashableF64,
}

impl ClampedYGradientData {
    fn as_base_component(&self) -> &'static noise_router::ClampedYGradientData {
        let value = noise_router::ClampedYGradientData {
            from_y: self.from_y as f64,
            to_y: self.to_y as f64,
            from_value: self.from_value.0,
            to_value: self.to_value.0,
        };

        Box::leak(Box::new(value))
    }
}

#[derive(Deserialize, Hash)]
pub struct BinaryData {
    #[serde(rename(deserialize = "type"))]
    pub operation: BinaryOperation,
    #[serde(rename(deserialize = "minValue"))]
    pub min_value: HashableF64,
    #[serde(rename(deserialize = "maxValue"))]
    pub max_value: HashableF64,
}

impl BinaryData {
    fn as_base_component(&self) -> &'static noise_router::BinaryData {
        let value = noise_router::BinaryData {
            operation: self.operation.as_base_component(),
        };

        Box::leak(Box::new(value))
    }
}

#[derive(Deserialize, Hash)]
pub struct LinearData {
    #[serde(rename(deserialize = "specificType"))]
    pub operation: LinearOperation,
    pub argument: HashableF64,
    #[serde(rename(deserialize = "minValue"))]
    pub min_value: HashableF64,
    #[serde(rename(deserialize = "maxValue"))]
    pub max_value: HashableF64,
}

impl LinearData {
    fn as_base_component(&self) -> &'static noise_router::LinearData {
        let value = noise_router::LinearData {
            operation: self.operation.as_base_component(),
            argument: self.argument.0,
        };

        Box::leak(Box::new(value))
    }
}

#[derive(Deserialize, Hash)]
pub struct UnaryData {
    #[serde(rename(deserialize = "type"))]
    pub operation: UnaryOperation,
    #[serde(rename(deserialize = "minValue"))]
    pub min_value: HashableF64,
    #[serde(rename(deserialize = "maxValue"))]
    pub max_value: HashableF64,
}

impl UnaryData {
    fn as_base_component(&self) -> &'static noise_router::UnaryData {
        let value = noise_router::UnaryData {
            operation: self.operation.as_base_component(),
        };

        Box::leak(Box::new(value))
    }
}

#[derive(Deserialize, Hash)]
pub struct ClampData {
    #[serde(rename(deserialize = "minValue"))]
    pub min_value: HashableF64,
    #[serde(rename(deserialize = "maxValue"))]
    pub max_value: HashableF64,
}

impl ClampData {
    fn as_base_component(&self) -> &'static noise_router::ClampData {
        let value = noise_router::ClampData {
            min_value: self.min_value.0,
            max_value: self.max_value.0,
        };

        Box::leak(Box::new(value))
    }
}

#[derive(Deserialize, Hash)]
pub struct RangeChoiceData {
    #[serde(rename(deserialize = "minInclusive"))]
    pub min_inclusive: HashableF64,
    #[serde(rename(deserialize = "maxExclusive"))]
    pub max_exclusive: HashableF64,
}

impl RangeChoiceData {
    fn as_base_component(&self) -> &'static noise_router::RangeChoiceData {
        let value = noise_router::RangeChoiceData {
            min_inclusive: self.min_inclusive.0,
            max_exclusive: self.max_exclusive.0,
        };

        Box::leak(Box::new(value))
    }
}

#[derive(Deserialize, Hash)]
pub struct SplineData {
    #[serde(rename(deserialize = "minValue"))]
    pub min_value: HashableF64,
    #[serde(rename(deserialize = "maxValue"))]
    pub max_value: HashableF64,
}

#[derive(Deserialize, Hash)]
#[serde(tag = "_class", content = "value")]
pub enum DensityFunctionRepr {
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
    fn as_base_component(
        &self,
        stack: &mut Vec<BaseNoiseFunctionComponent>,
        hash_to_index_map: &mut HashMap<u64, usize>,
    ) -> BaseNoiseFunctionComponent {
        match self {
            Self::Spline { spline, .. } => BaseNoiseFunctionComponent::Spline {
                spline: spline.as_base_component(stack, hash_to_index_map),
            },
            Self::EndIslands => BaseNoiseFunctionComponent::EndIslands,
            Self::Noise { data } => BaseNoiseFunctionComponent::Noise {
                data: data.as_base_component(),
            },
            Self::ShiftA { noise_id } => BaseNoiseFunctionComponent::ShiftA {
                noise_id: Box::leak(Box::new(noise_id.clone())),
            },
            Self::ShiftB { noise_id } => BaseNoiseFunctionComponent::ShiftB {
                noise_id: Box::leak(Box::new(noise_id.clone())),
            },
            Self::BlendDensity { input } => {
                let index = input.index(stack, hash_to_index_map);
                BaseNoiseFunctionComponent::BlendDensity { input_index: index }
            }
            Self::BlendAlpha => BaseNoiseFunctionComponent::BlendAlpha,
            Self::BlendOffset => BaseNoiseFunctionComponent::BlendOffset,
            Self::Beardifier => BaseNoiseFunctionComponent::Beardifier,
            Self::ShiftedNoise {
                shift_x,
                shift_y,
                shift_z,
                data,
            } => {
                let x_index = shift_x.index(stack, hash_to_index_map);
                let y_index = shift_y.index(stack, hash_to_index_map);
                let z_index = shift_z.index(stack, hash_to_index_map);

                BaseNoiseFunctionComponent::ShiftedNoise {
                    shift_x_index: x_index,
                    shift_y_index: y_index,
                    shift_z_index: z_index,
                    data: data.as_base_component(),
                }
            }
            Self::RangeChoice {
                input,
                when_in_range,
                when_out_range,
                data,
            } => {
                let input_index = input.index(stack, hash_to_index_map);
                let in_index = when_in_range.index(stack, hash_to_index_map);
                let out_index = when_out_range.index(stack, hash_to_index_map);

                BaseNoiseFunctionComponent::RangeChoice {
                    input_index,
                    when_in_range_index: in_index,
                    when_out_range_index: out_index,
                    data: data.as_base_component(),
                }
            }
            Self::WeirdScaled { input, data } => {
                let input_index = input.index(stack, hash_to_index_map);

                BaseNoiseFunctionComponent::WeirdScaled {
                    input_index,
                    data: data.as_base_component(),
                }
            }
            Self::Linear { input, data } => {
                let input_index = input.index(stack, hash_to_index_map);

                BaseNoiseFunctionComponent::Linear {
                    input_index,
                    data: data.as_base_component(),
                }
            }
            Self::Unary { input, data } => {
                let input_index = input.index(stack, hash_to_index_map);

                BaseNoiseFunctionComponent::Unary {
                    input_index,
                    data: data.as_base_component(),
                }
            }
            Self::InterpolatedNoiseSampler { data } => {
                BaseNoiseFunctionComponent::InterpolatedNoiseSampler {
                    data: data.as_base_component(),
                }
            }
            Self::Binary {
                argument1,
                argument2,
                data,
            } => {
                let index_1 = argument1.index(stack, hash_to_index_map);
                let index_2 = argument2.index(stack, hash_to_index_map);

                BaseNoiseFunctionComponent::Binary {
                    argument1_index: index_1,
                    argument2_index: index_2,
                    data: data.as_base_component(),
                }
            }

            Self::Clamp { input, data } => {
                let input_index = input.index(stack, hash_to_index_map);

                BaseNoiseFunctionComponent::Clamp {
                    input_index,
                    data: data.as_base_component(),
                }
            }
            Self::Constant { value } => BaseNoiseFunctionComponent::Constant { value: value.0 },
            Self::ClampedYGradient { data } => BaseNoiseFunctionComponent::ClampedYGradient {
                data: data.as_base_component(),
            },
            Self::Wrapper { input, wrapper } => {
                let input_index = input.index(stack, hash_to_index_map);

                BaseNoiseFunctionComponent::Wrapper {
                    input_index,
                    wrapper: wrapper.as_base_component(),
                }
            }
        }
    }

    fn index(
        &self,
        stack: &mut Vec<BaseNoiseFunctionComponent>,
        hash_to_index_map: &mut HashMap<u64, usize>,
    ) -> usize {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let hash = hasher.finish();

        if let Some(index) = hash_to_index_map.get(&hash) {
            *index
        } else {
            let component = self.as_base_component(stack, hash_to_index_map);
            stack.push(component);
            let index = stack.len() - 1;
            hash_to_index_map.insert(hash, index);
            index
        }
    }

    pub fn base_component_stack(&self) -> Box<[BaseNoiseFunctionComponent]> {
        let mut stack = Vec::new();
        let mut map = HashMap::new();

        let _ = self.index(&mut stack, &mut map);
        stack.into()
    }
}
