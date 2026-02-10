/* This file is generated. Do not edit manually. */
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
    Fixed {
        value: f32,
    },
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
    Beardifier,
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
    Wrapper {
        input_index: usize,
        wrapper: WrapperType,
    },
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
pub const OVERWORLD_BASE_NOISE_ROUTER: BaseNoiseRouters = BaseNoiseRouters {
    noise: BaseNoiseRouter {
        full_component_stack: &[
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: -64f64,
                    to_y: -40f64,
                    from_value: 0f64,
                    to_value: 1f64,
                },
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: 240f64,
                    to_y: 256f64,
                    from_value: 1f64,
                    to_value: 0f64,
                },
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: -64f64,
                    to_y: 320f64,
                    from_value: 1.5f64,
                    to_value: -1.5f64,
                },
            },
            BaseNoiseFunctionComponent::BlendOffset,
            BaseNoiseFunctionComponent::BlendAlpha,
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 4usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 5usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 6usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 3usize,
                argument2_index: 7usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::ShiftA { noise_id: "offset" },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 9usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 10usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Constant { value: 0f64 },
            BaseNoiseFunctionComponent::ShiftB { noise_id: "offset" },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 13usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 14usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 11usize,
                shift_y_index: 12usize,
                shift_z_index: 15usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: "continentalness",
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 16usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 11usize,
                shift_y_index: 12usize,
                shift_z_index: 15usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: "erosion",
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 18usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 11usize,
                shift_y_index: 12usize,
                shift_z_index: 15usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: "ridge",
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 20usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 21usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 22usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.6666666666666666f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 23usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 24usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.3333333333333333f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 25usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -3f64,
                },
            },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 17usize,
                    points: &[
                        SplinePoint {
                            location: -1.1f32,
                            value: &SplineRepr::Fixed { value: 0.044f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -1.02f32,
                            value: &SplineRepr::Fixed { value: -0.2222f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.51f32,
                            value: &SplineRepr::Fixed { value: -0.2222f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.44f32,
                            value: &SplineRepr::Fixed { value: -0.12f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.18f32,
                            value: &SplineRepr::Fixed { value: -0.12f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.16f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.08880186f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.69000006f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.115760356f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.6400001f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.75f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.65f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.5954547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.6054547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.100000024f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.3f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0.1f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0.06f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.15f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.08880186f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.69000006f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.115760356f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.6400001f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.75f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.65f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.5954547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.6054547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.100000024f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.3f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0.1f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0.06f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.1f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.08880186f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.69000006f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.115760356f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.6400001f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.75f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.65f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.5954547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.6054547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.100000024f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.25f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.001f32 },
                                                    derivative: 0.01f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.003f32 },
                                                    derivative: 0.01f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0.094000004f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.12f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 0.25f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.20235021f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.7161751f32,
                                                    },
                                                    derivative: 0.5138249f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1.23f32 },
                                                    derivative: 0.5138249f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.44682026f32,
                                                    },
                                                    derivative: 0.43317974f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.88f32 },
                                                    derivative: 0.43317974f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.30829495f32,
                                                    },
                                                    derivative: 0.3917051f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.70000005f32,
                                                    },
                                                    derivative: 0.3917051f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.25f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.42000002f32,
                                                    },
                                                    derivative: 0.049000014f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.0069999998f32,
                                                    },
                                                    derivative: 0.07f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.021f32 },
                                                    derivative: 0.07f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0.658f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.42000002f32,
                                                    },
                                                    derivative: 0.049000014f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.1f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.1f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.58f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.12f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 1f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.34792626f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.9239631f32,
                                                    },
                                                    derivative: 0.5760369f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1.5f32 },
                                                    derivative: 0.5760369f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.5391705f32,
                                                    },
                                                    derivative: 0.4608295f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1f32 },
                                                    derivative: 0.4608295f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.5391705f32,
                                                    },
                                                    derivative: 0.4608295f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1f32 },
                                                    derivative: 0.4608295f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.6f32 },
                                                    derivative: 0.070000015f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0.099999994f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.099999994f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0.94f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.6f32 },
                                                    derivative: 0.070000015f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.05f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.05f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.58f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0.015f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                    ],
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 27usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.5037500262260437f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 28usize,
                argument2_index: 5usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 8usize,
                argument2_index: 29usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 30usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 31usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 2usize,
                argument2_index: 32usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 17usize,
                    points: &[
                        SplinePoint {
                            location: -0.11f32,
                            value: &SplineRepr::Fixed { value: 0f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 0.03f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.19999999f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.44999996f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.63f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.78f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.19999999f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.44999996f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.315f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.15f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.5775f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.19999999f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.44999996f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.315f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.15f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.375f32,
                                        value: &SplineRepr::Fixed { value: 0f32 },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 0.65f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.19999999f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.44999996f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.63f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.63f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.78f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.19999999f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.44999996f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.63f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.5775f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.19999999f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.44999996f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.63f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.375f32,
                                        value: &SplineRepr::Fixed { value: 0f32 },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                    ],
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 34usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 4usize,
                argument2_index: 35usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 36usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 37usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 38usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "jagged",
                    xz_scale: 1500f64,
                    y_scale: 0f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 40usize,
                data: &UnaryData {
                    operation: UnaryOperation::HalfNegative,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 39usize,
                argument2_index: 41usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 33usize,
                argument2_index: 42usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 17usize,
                    points: &[
                        SplinePoint {
                            location: -0.19f32,
                            value: &SplineRepr::Fixed { value: 3.95f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.15f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.5f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.25f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.03f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.35f32,
                                        value: &SplineRepr::Fixed { value: 6.25f32 },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.25f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.25f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.62f32,
                                        value: &SplineRepr::Fixed { value: 6.25f32 },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.1f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.5f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.25f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.03f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.35f32,
                                        value: &SplineRepr::Fixed { value: 5.47f32 },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 5.47f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 5.47f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.62f32,
                                        value: &SplineRepr::Fixed { value: 5.47f32 },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 0.03f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.5f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.25f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.03f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.35f32,
                                        value: &SplineRepr::Fixed { value: 5.08f32 },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 5.08f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 5.08f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.62f32,
                                        value: &SplineRepr::Fixed { value: 5.08f32 },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 0.06f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 4.69f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.5f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 4.69f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.25f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 4.69f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.03f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 4.69f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.05f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.45f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 4.69f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.7f32,
                                                    value: &SplineRepr::Fixed { value: 1.56f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.45f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 4.69f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.7f32,
                                                    value: &SplineRepr::Fixed { value: 1.56f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.7f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 4.69f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.15f32,
                                                    value: &SplineRepr::Fixed { value: 1.37f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.7f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 4.69f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.15f32,
                                                    value: &SplineRepr::Fixed { value: 1.37f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.58f32,
                                        value: &SplineRepr::Fixed { value: 4.69f32 },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                    ],
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 44usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -10f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 4usize,
                argument2_index: 45usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 46usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 10f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 47usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 48usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 43usize,
                argument2_index: 49usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 50usize,
                data: &UnaryData {
                    operation: UnaryOperation::QuarterNegative,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 51usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 4f64,
                },
            },
            BaseNoiseFunctionComponent::InterpolatedNoiseSampler {
                data: &InterpolatedNoiseSamplerData {
                    scaled_xz_scale: 171.103f64,
                    scaled_y_scale: 85.5515f64,
                    xz_factor: 80f64,
                    y_factor: 160f64,
                    smear_scale_multiplier: 8f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 52usize,
                argument2_index: 53usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "cave_entrance",
                    xz_scale: 0.75f64,
                    y_scale: 0.5f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 55usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.37f64,
                },
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: -10f64,
                    to_y: 30f64,
                    from_value: 0.3f64,
                    to_value: 0f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 56usize,
                argument2_index: 57usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "spaghetti_roughness_modulator",
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 59usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.05f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 60usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.05f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "spaghetti_roughness",
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 62usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 63usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.4f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 61usize,
                argument2_index: 64usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 65usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "spaghetti_3d_rarity",
                    xz_scale: 2f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 67usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::WeirdScaled {
                input_index: 68usize,
                data: &WeirdScaledData {
                    noise_id: "spaghetti_3d_1",
                    mapper: WeirdScaledMapper::Tunnels,
                },
            },
            BaseNoiseFunctionComponent::WeirdScaled {
                input_index: 68usize,
                data: &WeirdScaledData {
                    noise_id: "spaghetti_3d_2",
                    mapper: WeirdScaledMapper::Tunnels,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 69usize,
                argument2_index: 70usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "spaghetti_3d_thickness",
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 72usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.011499999999999996f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 73usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.0765f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 71usize,
                argument2_index: 74usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 75usize,
                data: &ClampData {
                    min_value: -1f64,
                    max_value: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 66usize,
                argument2_index: 76usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 58usize,
                argument2_index: 77usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 78usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 79usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 5f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 54usize,
                argument2_index: 80usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "cave_layer",
                    xz_scale: 1f64,
                    y_scale: 8f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 82usize,
                data: &UnaryData {
                    operation: UnaryOperation::Square,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 83usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 4f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "cave_cheese",
                    xz_scale: 1f64,
                    y_scale: 0.6666666666666666f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 85usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.27f64,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 86usize,
                data: &ClampData {
                    min_value: -1f64,
                    max_value: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 54usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.64f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 88usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 1.5f64,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 89usize,
                data: &ClampData {
                    min_value: 0f64,
                    max_value: 0.5f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 87usize,
                argument2_index: 90usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 84usize,
                argument2_index: 91usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 92usize,
                argument2_index: 79usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "spaghetti_2d_modulator",
                    xz_scale: 2f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::WeirdScaled {
                input_index: 94usize,
                data: &WeirdScaledData {
                    noise_id: "spaghetti_2d",
                    mapper: WeirdScaledMapper::Caves,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "spaghetti_2d_thickness",
                    xz_scale: 2f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 96usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.35000000000000003f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 97usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.95f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 98usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 99usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.083f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 95usize,
                argument2_index: 100usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "spaghetti_2d_elevation",
                    xz_scale: 1f64,
                    y_scale: 0f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 102usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 8f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 103usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0f64,
                },
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: -64f64,
                    to_y: 320f64,
                    from_value: 8f64,
                    to_value: -40f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 104usize,
                argument2_index: 105usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 106usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 107usize,
                argument2_index: 99usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 108usize,
                data: &UnaryData {
                    operation: UnaryOperation::Cube,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 101usize,
                argument2_index: 109usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 110usize,
                data: &ClampData {
                    min_value: -1f64,
                    max_value: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 111usize,
                argument2_index: 66usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 93usize,
                argument2_index: 112usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "pillar",
                    xz_scale: 25f64,
                    y_scale: 0.3f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 114usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 2f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "pillar_rareness",
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 116usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 117usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -1f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 115usize,
                argument2_index: 118usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "pillar_thickness",
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 120usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.55f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 121usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.55f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 122usize,
                data: &UnaryData {
                    operation: UnaryOperation::Cube,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 119usize,
                argument2_index: 123usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 124usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Constant { value: -1000000f64 },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 125usize,
                when_in_range_index: 126usize,
                when_out_range_index: 125usize,
                data: &RangeChoiceData {
                    min_inclusive: -1000000f64,
                    max_exclusive: 0.03f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 113usize,
                argument2_index: 127usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 54usize,
                when_in_range_index: 81usize,
                when_out_range_index: 128usize,
                data: &RangeChoiceData {
                    min_inclusive: -1000000f64,
                    max_exclusive: 1.5625f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 129usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.078125f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 1usize,
                argument2_index: 130usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 131usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.078125f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 132usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.1171875f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 0usize,
                argument2_index: 133usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 134usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.1171875f64,
                },
            },
            BaseNoiseFunctionComponent::BlendDensity {
                input_index: 135usize,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 136usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 137usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.64f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 138usize,
                data: &UnaryData {
                    operation: UnaryOperation::Squeeze,
                },
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: -4064f64,
                    to_y: 4062f64,
                    from_value: -4064f64,
                    to_value: 4062f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "noodle",
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Constant { value: -1f64 },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 140usize,
                when_in_range_index: 141usize,
                when_out_range_index: 142usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 321f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 143usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Constant { value: 64f64 },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "noodle_thickness",
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 146usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.025f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 147usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.07500000000000001f64,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 140usize,
                when_in_range_index: 148usize,
                when_out_range_index: 12usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 321f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 149usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "noodle_ridge_a",
                    xz_scale: 2.6666666666666665f64,
                    y_scale: 2.6666666666666665f64,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 140usize,
                when_in_range_index: 151usize,
                when_out_range_index: 12usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 321f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 152usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 153usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "noodle_ridge_b",
                    xz_scale: 2.6666666666666665f64,
                    y_scale: 2.6666666666666665f64,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 140usize,
                when_in_range_index: 155usize,
                when_out_range_index: 12usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 321f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 156usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 157usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 154usize,
                argument2_index: 158usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 159usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1.5f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 150usize,
                argument2_index: 160usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 144usize,
                when_in_range_index: 145usize,
                when_out_range_index: 161usize,
                data: &RangeChoiceData {
                    min_inclusive: -1000000f64,
                    max_exclusive: 0f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 139usize,
                argument2_index: 162usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Beardifier,
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 163usize,
                argument2_index: 164usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 165usize,
                wrapper: WrapperType::CellCache,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "aquifer_barrier",
                    xz_scale: 1f64,
                    y_scale: 0.5f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "aquifer_fluid_level_floodedness",
                    xz_scale: 1f64,
                    y_scale: 0.67f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "aquifer_fluid_level_spread",
                    xz_scale: 1f64,
                    y_scale: 0.7142857142857143f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "aquifer_lava",
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "ore_veininess",
                    xz_scale: 1.5f64,
                    y_scale: 1.5f64,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 140usize,
                when_in_range_index: 171usize,
                when_out_range_index: 12usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 51f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 172usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "ore_vein_a",
                    xz_scale: 4f64,
                    y_scale: 4f64,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 140usize,
                when_in_range_index: 174usize,
                when_out_range_index: 12usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 51f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 175usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 176usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "ore_vein_b",
                    xz_scale: 4f64,
                    y_scale: 4f64,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 140usize,
                when_in_range_index: 178usize,
                when_out_range_index: 12usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 51f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 179usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 180usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 177usize,
                argument2_index: 181usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 182usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.07999999821186066f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: "ore_gap",
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
        ],
        barrier_noise: 167usize,
        fluid_level_floodedness_noise: 168usize,
        fluid_level_spread_noise: 169usize,
        lava_noise: 170usize,
        erosion: 19usize,
        depth: 33usize,
        final_density: 166usize,
        vein_toggle: 173usize,
        vein_ridged: 183usize,
        vein_gap: 184usize,
    },
    surface_estimator: BaseSurfaceEstimator {
        full_component_stack: &[
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: -64f64,
                    to_y: -40f64,
                    from_value: 0f64,
                    to_value: 1f64,
                },
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: 240f64,
                    to_y: 256f64,
                    from_value: 1f64,
                    to_value: 0f64,
                },
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: -64f64,
                    to_y: 320f64,
                    from_value: 1.5f64,
                    to_value: -1.5f64,
                },
            },
            BaseNoiseFunctionComponent::BlendOffset,
            BaseNoiseFunctionComponent::BlendAlpha,
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 4usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 5usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 6usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 3usize,
                argument2_index: 7usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::ShiftA { noise_id: "offset" },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 9usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 10usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Constant { value: 0f64 },
            BaseNoiseFunctionComponent::ShiftB { noise_id: "offset" },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 13usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 14usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 11usize,
                shift_y_index: 12usize,
                shift_z_index: 15usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: "continentalness",
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 16usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 11usize,
                shift_y_index: 12usize,
                shift_z_index: 15usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: "erosion",
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 18usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 11usize,
                shift_y_index: 12usize,
                shift_z_index: 15usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: "ridge",
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 20usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 21usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 22usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.6666666666666666f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 23usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 24usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.3333333333333333f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 25usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -3f64,
                },
            },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 17usize,
                    points: &[
                        SplinePoint {
                            location: -1.1f32,
                            value: &SplineRepr::Fixed { value: 0.044f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -1.02f32,
                            value: &SplineRepr::Fixed { value: -0.2222f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.51f32,
                            value: &SplineRepr::Fixed { value: -0.2222f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.44f32,
                            value: &SplineRepr::Fixed { value: -0.12f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.18f32,
                            value: &SplineRepr::Fixed { value: -0.12f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.16f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.08880186f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.69000006f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.115760356f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.6400001f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.75f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.65f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.5954547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.6054547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.100000024f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.3f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0.1f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0.06f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.15f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.08880186f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.69000006f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.115760356f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.6400001f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.75f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.65f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.5954547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.6054547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.100000024f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.3f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0.1f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0.06f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.1f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.08880186f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.69000006f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.115760356f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.6400001f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.75f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.65f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.5954547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.6054547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.100000024f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.25f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.001f32 },
                                                    derivative: 0.01f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.003f32 },
                                                    derivative: 0.01f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0.094000004f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.12f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 0.25f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.20235021f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.7161751f32,
                                                    },
                                                    derivative: 0.5138249f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1.23f32 },
                                                    derivative: 0.5138249f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.44682026f32,
                                                    },
                                                    derivative: 0.43317974f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.88f32 },
                                                    derivative: 0.43317974f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.30829495f32,
                                                    },
                                                    derivative: 0.3917051f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.70000005f32,
                                                    },
                                                    derivative: 0.3917051f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.25f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.42000002f32,
                                                    },
                                                    derivative: 0.049000014f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.0069999998f32,
                                                    },
                                                    derivative: 0.07f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.021f32 },
                                                    derivative: 0.07f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0.658f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.42000002f32,
                                                    },
                                                    derivative: 0.049000014f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.1f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.1f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.58f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.12f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 1f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.34792626f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.9239631f32,
                                                    },
                                                    derivative: 0.5760369f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1.5f32 },
                                                    derivative: 0.5760369f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.5391705f32,
                                                    },
                                                    derivative: 0.4608295f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1f32 },
                                                    derivative: 0.4608295f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.5391705f32,
                                                    },
                                                    derivative: 0.4608295f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1f32 },
                                                    derivative: 0.4608295f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.6f32 },
                                                    derivative: 0.070000015f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0.099999994f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.099999994f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0.94f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.6f32 },
                                                    derivative: 0.070000015f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.05f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.05f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.58f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0.015f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                    ],
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 27usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.5037500262260437f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 28usize,
                argument2_index: 5usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 8usize,
                argument2_index: 29usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 30usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 31usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 2usize,
                argument2_index: 32usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 17usize,
                    points: &[
                        SplinePoint {
                            location: -0.19f32,
                            value: &SplineRepr::Fixed { value: 3.95f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.15f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.5f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.25f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.03f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.35f32,
                                        value: &SplineRepr::Fixed { value: 6.25f32 },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.25f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.25f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.62f32,
                                        value: &SplineRepr::Fixed { value: 6.25f32 },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.1f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.5f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.25f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.03f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.35f32,
                                        value: &SplineRepr::Fixed { value: 5.47f32 },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 5.47f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 5.47f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.62f32,
                                        value: &SplineRepr::Fixed { value: 5.47f32 },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 0.03f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.5f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.25f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.03f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.35f32,
                                        value: &SplineRepr::Fixed { value: 5.08f32 },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 5.08f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 5.08f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.62f32,
                                        value: &SplineRepr::Fixed { value: 5.08f32 },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 0.06f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 4.69f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.5f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 4.69f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.25f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 4.69f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.03f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 4.69f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.05f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.45f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 4.69f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.7f32,
                                                    value: &SplineRepr::Fixed { value: 1.56f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.45f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 4.69f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.7f32,
                                                    value: &SplineRepr::Fixed { value: 1.56f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.7f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 4.69f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.15f32,
                                                    value: &SplineRepr::Fixed { value: 1.37f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.7f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 4.69f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.15f32,
                                                    value: &SplineRepr::Fixed { value: 1.37f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.58f32,
                                        value: &SplineRepr::Fixed { value: 4.69f32 },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                    ],
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 34usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -10f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 4usize,
                argument2_index: 35usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 36usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 10f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 37usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 38usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 39usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 33usize,
                argument2_index: 40usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 41usize,
                data: &UnaryData {
                    operation: UnaryOperation::QuarterNegative,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 42usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 4f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 43usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.703125f64,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 44usize,
                data: &ClampData {
                    min_value: -64f64,
                    max_value: 64f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 45usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.078125f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 1usize,
                argument2_index: 46usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 47usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.078125f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 48usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.1171875f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 0usize,
                argument2_index: 49usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 50usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.1171875f64,
                },
            },
        ],
    },
    multi_noise: BaseMultiNoiseRouter {
        full_component_stack: &[
            BaseNoiseFunctionComponent::ShiftA { noise_id: "offset" },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 0usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 1usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Constant { value: 0f64 },
            BaseNoiseFunctionComponent::ShiftB { noise_id: "offset" },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 4usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 5usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 2usize,
                shift_y_index: 3usize,
                shift_z_index: 6usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: "ridge",
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 7usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 2usize,
                shift_y_index: 3usize,
                shift_z_index: 6usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: "temperature",
                },
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 2usize,
                shift_y_index: 3usize,
                shift_z_index: 6usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: "vegetation",
                },
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 2usize,
                shift_y_index: 3usize,
                shift_z_index: 6usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: "continentalness",
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 11usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 2usize,
                shift_y_index: 3usize,
                shift_z_index: 6usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: "erosion",
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 13usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: -64f64,
                    to_y: 320f64,
                    from_value: 1.5f64,
                    to_value: -1.5f64,
                },
            },
            BaseNoiseFunctionComponent::BlendOffset,
            BaseNoiseFunctionComponent::BlendAlpha,
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 17usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 18usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 19usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 16usize,
                argument2_index: 20usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 8usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 22usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.6666666666666666f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 23usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 24usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.3333333333333333f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 25usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -3f64,
                },
            },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 12usize,
                    points: &[
                        SplinePoint {
                            location: -1.1f32,
                            value: &SplineRepr::Fixed { value: 0.044f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -1.02f32,
                            value: &SplineRepr::Fixed { value: -0.2222f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.51f32,
                            value: &SplineRepr::Fixed { value: -0.2222f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.44f32,
                            value: &SplineRepr::Fixed { value: -0.12f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.18f32,
                            value: &SplineRepr::Fixed { value: -0.12f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.16f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 14usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.08880186f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.69000006f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.115760356f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.6400001f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.75f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.65f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.5954547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.6054547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.100000024f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.3f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0.1f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0.06f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.15f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 14usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.08880186f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.69000006f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.115760356f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.6400001f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.75f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.65f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.5954547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.6054547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.100000024f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.3f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0.1f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0.06f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.1f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 14usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.08880186f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.69000006f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.115760356f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.6400001f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.75f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.65f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.5954547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.6054547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.100000024f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.25f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.001f32 },
                                                    derivative: 0.01f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.003f32 },
                                                    derivative: 0.01f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0.094000004f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.12f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 0.25f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 14usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.20235021f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.7161751f32,
                                                    },
                                                    derivative: 0.5138249f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1.23f32 },
                                                    derivative: 0.5138249f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.44682026f32,
                                                    },
                                                    derivative: 0.43317974f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.88f32 },
                                                    derivative: 0.43317974f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.30829495f32,
                                                    },
                                                    derivative: 0.3917051f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.70000005f32,
                                                    },
                                                    derivative: 0.3917051f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.25f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.42000002f32,
                                                    },
                                                    derivative: 0.049000014f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.0069999998f32,
                                                    },
                                                    derivative: 0.07f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.021f32 },
                                                    derivative: 0.07f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0.658f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.42000002f32,
                                                    },
                                                    derivative: 0.049000014f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.1f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.1f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.58f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.12f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 1f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 14usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.34792626f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.9239631f32,
                                                    },
                                                    derivative: 0.5760369f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1.5f32 },
                                                    derivative: 0.5760369f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.5391705f32,
                                                    },
                                                    derivative: 0.4608295f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1f32 },
                                                    derivative: 0.4608295f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.5391705f32,
                                                    },
                                                    derivative: 0.4608295f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1f32 },
                                                    derivative: 0.4608295f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.6f32 },
                                                    derivative: 0.070000015f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0.099999994f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.099999994f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0.94f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.6f32 },
                                                    derivative: 0.070000015f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.05f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.05f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.58f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0.015f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                    ],
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 27usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.5037500262260437f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 28usize,
                argument2_index: 18usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 21usize,
                argument2_index: 29usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 30usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 31usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 15usize,
                argument2_index: 32usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
        ],
        temperature: 9usize,
        vegetation: 10usize,
        continents: 12usize,
        erosion: 14usize,
        depth: 33usize,
        ridges: 8usize,
    },
};
pub const NETHER_BASE_NOISE_ROUTER: BaseNoiseRouters = BaseNoiseRouters {
    noise: BaseNoiseRouter {
        full_component_stack: &[
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: -8f64,
                    to_y: 24f64,
                    from_value: 0f64,
                    to_value: 1f64,
                },
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: 104f64,
                    to_y: 128f64,
                    from_value: 1f64,
                    to_value: 0f64,
                },
            },
            BaseNoiseFunctionComponent::InterpolatedNoiseSampler {
                data: &InterpolatedNoiseSamplerData {
                    scaled_xz_scale: 171.103f64,
                    scaled_y_scale: 256.6545f64,
                    xz_factor: 80f64,
                    y_factor: 60f64,
                    smear_scale_multiplier: 8f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 2usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.9375f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 1usize,
                argument2_index: 3usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 4usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.9375f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 5usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -2.5f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 0usize,
                argument2_index: 6usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 7usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 2.5f64,
                },
            },
            BaseNoiseFunctionComponent::BlendDensity {
                input_index: 8usize,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 9usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 10usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.64f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 11usize,
                data: &UnaryData {
                    operation: UnaryOperation::Squeeze,
                },
            },
            BaseNoiseFunctionComponent::Beardifier,
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 12usize,
                argument2_index: 13usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 14usize,
                wrapper: WrapperType::CellCache,
            },
            BaseNoiseFunctionComponent::Constant { value: 0f64 },
        ],
        barrier_noise: 16usize,
        fluid_level_floodedness_noise: 16usize,
        fluid_level_spread_noise: 16usize,
        lava_noise: 16usize,
        erosion: 16usize,
        depth: 16usize,
        final_density: 15usize,
        vein_toggle: 16usize,
        vein_ridged: 16usize,
        vein_gap: 16usize,
    },
    surface_estimator: BaseSurfaceEstimator {
        full_component_stack: &[BaseNoiseFunctionComponent::Constant { value: 0f64 }],
    },
    multi_noise: BaseMultiNoiseRouter {
        full_component_stack: &[
            BaseNoiseFunctionComponent::Constant { value: 0f64 },
            BaseNoiseFunctionComponent::ShiftA { noise_id: "offset" },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 1usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 2usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ShiftB { noise_id: "offset" },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 4usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 5usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 3usize,
                shift_y_index: 0usize,
                shift_z_index: 6usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: "temperature",
                },
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 3usize,
                shift_y_index: 0usize,
                shift_z_index: 6usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: "vegetation",
                },
            },
        ],
        temperature: 7usize,
        vegetation: 8usize,
        continents: 0usize,
        erosion: 0usize,
        depth: 0usize,
        ridges: 0usize,
    },
};
pub const END_BASE_NOISE_ROUTER: BaseNoiseRouters = BaseNoiseRouters {
    noise: BaseNoiseRouter {
        full_component_stack: &[
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: 4f64,
                    to_y: 32f64,
                    from_value: 0f64,
                    to_value: 1f64,
                },
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: 56f64,
                    to_y: 312f64,
                    from_value: 1f64,
                    to_value: 0f64,
                },
            },
            BaseNoiseFunctionComponent::EndIslands,
            BaseNoiseFunctionComponent::InterpolatedNoiseSampler {
                data: &InterpolatedNoiseSamplerData {
                    scaled_xz_scale: 171.103f64,
                    scaled_y_scale: 171.103f64,
                    xz_factor: 80f64,
                    y_factor: 160f64,
                    smear_scale_multiplier: 4f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 2usize,
                argument2_index: 3usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 4usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 23.4375f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 1usize,
                argument2_index: 5usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 6usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -23.4375f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 7usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.234375f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 0usize,
                argument2_index: 8usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 9usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.234375f64,
                },
            },
            BaseNoiseFunctionComponent::BlendDensity {
                input_index: 10usize,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 11usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 12usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.64f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 13usize,
                data: &UnaryData {
                    operation: UnaryOperation::Squeeze,
                },
            },
            BaseNoiseFunctionComponent::Constant { value: 0f64 },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 2usize,
                wrapper: WrapperType::Cache2D,
            },
        ],
        barrier_noise: 15usize,
        fluid_level_floodedness_noise: 15usize,
        fluid_level_spread_noise: 15usize,
        lava_noise: 15usize,
        erosion: 16usize,
        depth: 15usize,
        final_density: 14usize,
        vein_toggle: 15usize,
        vein_ridged: 15usize,
        vein_gap: 15usize,
    },
    surface_estimator: BaseSurfaceEstimator {
        full_component_stack: &[
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: 4f64,
                    to_y: 32f64,
                    from_value: 0f64,
                    to_value: 1f64,
                },
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: 56f64,
                    to_y: 312f64,
                    from_value: 1f64,
                    to_value: 0f64,
                },
            },
            BaseNoiseFunctionComponent::EndIslands,
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 2usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 3usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.703125f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 4usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 23.4375f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 1usize,
                argument2_index: 5usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 6usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -23.4375f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 7usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.234375f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 0usize,
                argument2_index: 8usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 9usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.234375f64,
                },
            },
        ],
    },
    multi_noise: BaseMultiNoiseRouter {
        full_component_stack: &[
            BaseNoiseFunctionComponent::Constant { value: 0f64 },
            BaseNoiseFunctionComponent::EndIslands,
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 1usize,
                wrapper: WrapperType::Cache2D,
            },
        ],
        temperature: 0usize,
        vegetation: 0usize,
        continents: 0usize,
        erosion: 2usize,
        depth: 0usize,
        ridges: 0usize,
    },
};
