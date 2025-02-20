use std::hash::{Hash, Hasher};

use serde::Deserialize;

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

#[derive(Deserialize, Hash, Copy, Clone)]
pub enum LinearOperation {
    #[serde(rename(deserialize = "ADD"))]
    Add,
    #[serde(rename(deserialize = "MUL"))]
    Mul,
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

#[derive(Deserialize, Hash, Copy, Clone)]
pub enum WeirdScaledMapper {
    #[serde(rename(deserialize = "TYPE2"))]
    Caves,
    #[serde(rename(deserialize = "TYPE1"))]
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

#[derive(Copy, Clone, Deserialize, PartialEq, Eq, Hash)]
pub enum WrapperType {
    Interpolated,
    #[serde(rename(deserialize = "FlatCache"))]
    CacheFlat,
    Cache2D,
    CacheOnce,
    CellCache,
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

#[derive(Deserialize, Hash)]
pub struct ShiftedNoiseData {
    #[serde(rename(deserialize = "xzScale"))]
    pub xz_scale: HashableF64,
    #[serde(rename(deserialize = "yScale"))]
    pub y_scale: HashableF64,
    #[serde(rename(deserialize = "noise"))]
    pub noise_id: String,
}

#[derive(Deserialize, Hash)]
pub struct WeirdScaledData {
    #[serde(rename(deserialize = "noise"))]
    pub noise_id: String,
    #[serde(rename(deserialize = "rarityValueMapper"))]
    pub mapper: WeirdScaledMapper,
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

#[derive(Deserialize, Hash)]
pub struct BinaryData {
    #[serde(rename(deserialize = "type"))]
    pub operation: BinaryOperation,
    #[serde(rename(deserialize = "minValue"))]
    pub min_value: HashableF64,
    #[serde(rename(deserialize = "maxValue"))]
    pub max_value: HashableF64,
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
    #[inline]
    pub fn apply_density(&self, density: f64) -> f64 {
        match self.operation {
            LinearOperation::Add => density + self.argument.0,
            LinearOperation::Mul => density * self.argument.0,
        }
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

#[derive(Deserialize, Hash)]
pub struct ClampData {
    #[serde(rename(deserialize = "minValue"))]
    pub min_value: HashableF64,
    #[serde(rename(deserialize = "maxValue"))]
    pub max_value: HashableF64,
}

impl ClampData {
    #[inline]
    pub fn apply_density(&self, density: f64) -> f64 {
        density.clamp(self.min_value.0, self.max_value.0)
    }
}

#[derive(Deserialize, Hash)]
pub struct RangeChoiceData {
    #[serde(rename(deserialize = "minInclusive"))]
    pub min_inclusive: HashableF64,
    #[serde(rename(deserialize = "maxExclusive"))]
    pub max_exclusive: HashableF64,
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

/*
impl DensityFunctionRepr {
    #[allow(unused_variables)]
    pub fn as_str(&self) -> &str {
        match self {
            DensityFunctionRepr::BlendAlpha => "BlendAlpha",
            DensityFunctionRepr::Linear { input, data } => "Linear",
            DensityFunctionRepr::ClampedYGradient { data } => "ClampedYGradient",
            DensityFunctionRepr::Constant { value } => "Constant",
            DensityFunctionRepr::Wrapper { input, wrapper } => "Wrapper",
            DensityFunctionRepr::Unary { input, data } => "Unary",
            DensityFunctionRepr::RangeChoice {
                input,
                when_in_range,
                when_out_range,
                data,
            } => "RangeChoice",
            DensityFunctionRepr::Clamp { input, data } => "Clamp",
            DensityFunctionRepr::Spline { spline, data } => "Spline",
            DensityFunctionRepr::WeirdScaled { input, data } => "WeirdScaled",
            DensityFunctionRepr::Binary {
                argument1,
                argument2,
                data,
            } => "Binary",
            DensityFunctionRepr::ShiftedNoise {
                shift_x,
                shift_y,
                shift_z,
                data,
            } => "ShiftedNoise",
            DensityFunctionRepr::BlendDensity { input } => "BlendDensity",
            DensityFunctionRepr::BlendOffset => "BlendOffset",
            DensityFunctionRepr::InterpolatedNoiseSampler { data } => "InterpolatedNoiseSampler",
            DensityFunctionRepr::Noise { data } => "Noise",
            DensityFunctionRepr::EndIslands => "EndIslands",
            DensityFunctionRepr::ShiftA { noise_id } => "ShiftA",
            DensityFunctionRepr::ShiftB { noise_id } => "ShiftB",
        }
    }
}
*/
