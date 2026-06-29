/* This file is generated. Do not edit manually. */
use crate::chunk::DoublePerlinNoiseParameters;
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
    Invert,
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
                if density == 0.0 {
                    f64::INFINITY
                } else {
                    1.0 / density
                }
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
            BaseNoiseFunctionComponent::ShiftA {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 9usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 10usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Constant { value: 0f64 },
            BaseNoiseFunctionComponent::ShiftB {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
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
                    noise_id: DoublePerlinNoiseParameters::CONTINENTALNESS,
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
                    noise_id: DoublePerlinNoiseParameters::EROSION,
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
                    noise_id: DoublePerlinNoiseParameters::RIDGE,
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
                    noise_id: DoublePerlinNoiseParameters::JAGGED,
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
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 42usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 33usize,
                argument2_index: 43usize,
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
                input_index: 45usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -10f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 4usize,
                argument2_index: 46usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 47usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 10f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 48usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 49usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 44usize,
                argument2_index: 50usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 51usize,
                data: &UnaryData {
                    operation: UnaryOperation::QuarterNegative,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 52usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 4f64,
                },
            },
            BaseNoiseFunctionComponent::InterpolatedNoiseSampler {
                data: &InterpolatedNoiseSamplerData {
                    scaled_xz_scale: 0.25f64,
                    scaled_y_scale: 0.25f64,
                    xz_factor: 80f64,
                    y_factor: 160f64,
                    smear_scale_multiplier: 8f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 53usize,
                argument2_index: 54usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 55usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::CAVE_ENTRANCE,
                    xz_scale: 0.75f64,
                    y_scale: 0.5f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 57usize,
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
                argument1_index: 58usize,
                argument2_index: 59usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_ROUGHNESS_MODULATOR,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 61usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.05f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 62usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.05f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_ROUGHNESS,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 64usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 65usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.4f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 63usize,
                argument2_index: 66usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 67usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_RARITY,
                    xz_scale: 2f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 69usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
                    xz_scale: 1.3333333333333333f64,
                    y_scale: 1.3333333333333333f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 71usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.75f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 73usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
                    xz_scale: 0.6666666666666666f64,
                    y_scale: 0.6666666666666666f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 75usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1.5f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
                    xz_scale: 0.5f64,
                    y_scale: 0.5f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 77usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 2f64,
                },
            },
            BaseNoiseFunctionComponent::IntervalSelect {
                input_index: 70usize,
                thresholds: &[-0.5f64, 0f64, 0.5f64],
                functions_indices: &[72usize, 74usize, 76usize, 78usize],
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 79usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
                    xz_scale: 1.3333333333333333f64,
                    y_scale: 1.3333333333333333f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 81usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.75f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 83usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
                    xz_scale: 0.6666666666666666f64,
                    y_scale: 0.6666666666666666f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 85usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1.5f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
                    xz_scale: 0.5f64,
                    y_scale: 0.5f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 87usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 2f64,
                },
            },
            BaseNoiseFunctionComponent::IntervalSelect {
                input_index: 70usize,
                thresholds: &[-0.5f64, 0f64, 0.5f64],
                functions_indices: &[82usize, 84usize, 86usize, 88usize],
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 89usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 80usize,
                argument2_index: 90usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_THICKNESS,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 92usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.011499999999999996f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 93usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.0765f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 91usize,
                argument2_index: 94usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 95usize,
                data: &ClampData {
                    min_value: -1f64,
                    max_value: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 68usize,
                argument2_index: 96usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 60usize,
                argument2_index: 97usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
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
                    argument: 5f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 56usize,
                argument2_index: 100usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::CAVE_LAYER,
                    xz_scale: 1f64,
                    y_scale: 8f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 102usize,
                data: &UnaryData {
                    operation: UnaryOperation::Square,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 103usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 4f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::CAVE_CHEESE,
                    xz_scale: 1f64,
                    y_scale: 0.6666666666666666f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 105usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.27f64,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 106usize,
                data: &ClampData {
                    min_value: -1f64,
                    max_value: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 56usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.64f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 108usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 1.5f64,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 109usize,
                data: &ClampData {
                    min_value: 0f64,
                    max_value: 0.5f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 107usize,
                argument2_index: 110usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 104usize,
                argument2_index: 111usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 112usize,
                argument2_index: 99usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D_MODULATOR,
                    xz_scale: 2f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D,
                    xz_scale: 2f64,
                    y_scale: 2f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 115usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.5f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D,
                    xz_scale: 1.3333333333333333f64,
                    y_scale: 1.3333333333333333f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 117usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.75f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 119usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D,
                    xz_scale: 0.5f64,
                    y_scale: 0.5f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 121usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 2f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D,
                    xz_scale: 0.3333333333333333f64,
                    y_scale: 0.3333333333333333f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 123usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 3f64,
                },
            },
            BaseNoiseFunctionComponent::IntervalSelect {
                input_index: 114usize,
                thresholds: &[-0.75f64, -0.5f64, 0.5f64, 0.75f64],
                functions_indices: &[116usize, 118usize, 120usize, 122usize, 124usize],
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 125usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D_THICKNESS,
                    xz_scale: 2f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 127usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.35000000000000003f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 128usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.95f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 129usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 130usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.083f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 126usize,
                argument2_index: 131usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D_ELEVATION,
                    xz_scale: 1f64,
                    y_scale: 0f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 133usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 8f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 134usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 135usize,
                wrapper: WrapperType::CacheFlat,
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
                argument1_index: 136usize,
                argument2_index: 137usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 138usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 139usize,
                argument2_index: 130usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 140usize,
                data: &UnaryData {
                    operation: UnaryOperation::Cube,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 132usize,
                argument2_index: 141usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 142usize,
                data: &ClampData {
                    min_value: -1f64,
                    max_value: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 143usize,
                argument2_index: 68usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 113usize,
                argument2_index: 144usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::PILLAR,
                    xz_scale: 25f64,
                    y_scale: 0.3f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 146usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 2f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::PILLAR_RARENESS,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 148usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 149usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -1f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 147usize,
                argument2_index: 150usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::PILLAR_THICKNESS,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 152usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.55f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 153usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.55f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 154usize,
                data: &UnaryData {
                    operation: UnaryOperation::Cube,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 151usize,
                argument2_index: 155usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 156usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Constant { value: -1000000f64 },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 157usize,
                when_in_range_index: 158usize,
                when_out_range_index: 157usize,
                data: &RangeChoiceData {
                    min_inclusive: -1000000f64,
                    max_exclusive: 0.03f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 145usize,
                argument2_index: 159usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 56usize,
                when_in_range_index: 101usize,
                when_out_range_index: 160usize,
                data: &RangeChoiceData {
                    min_inclusive: -1000000f64,
                    max_exclusive: 1.5625f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 161usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.078125f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 1usize,
                argument2_index: 162usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 163usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.078125f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 164usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.1171875f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 0usize,
                argument2_index: 165usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 166usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.1171875f64,
                },
            },
            BaseNoiseFunctionComponent::BlendDensity {
                input_index: 167usize,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 168usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.64f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 169usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 170usize,
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
                    noise_id: DoublePerlinNoiseParameters::NOODLE,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Constant { value: -1f64 },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 172usize,
                when_in_range_index: 173usize,
                when_out_range_index: 174usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 321f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 175usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Constant { value: 64f64 },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::NOODLE_THICKNESS,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 178usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.025f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 179usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.07500000000000001f64,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 172usize,
                when_in_range_index: 180usize,
                when_out_range_index: 12usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 321f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 181usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::NOODLE_RIDGE_A,
                    xz_scale: 2.6666666666666665f64,
                    y_scale: 2.6666666666666665f64,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 172usize,
                when_in_range_index: 183usize,
                when_out_range_index: 12usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 321f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 184usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 185usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::NOODLE_RIDGE_B,
                    xz_scale: 2.6666666666666665f64,
                    y_scale: 2.6666666666666665f64,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 172usize,
                when_in_range_index: 187usize,
                when_out_range_index: 12usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 321f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 188usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 189usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 186usize,
                argument2_index: 190usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 191usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1.5f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 182usize,
                argument2_index: 192usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 176usize,
                when_in_range_index: 177usize,
                when_out_range_index: 193usize,
                data: &RangeChoiceData {
                    min_inclusive: -1000000f64,
                    max_exclusive: 0f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 171usize,
                argument2_index: 194usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Beardifier,
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 195usize,
                argument2_index: 196usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 197usize,
                wrapper: WrapperType::CellCache,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::AQUIFER_BARRIER,
                    xz_scale: 1f64,
                    y_scale: 0.5f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::AQUIFER_FLUID_LEVEL_FLOODEDNESS,
                    xz_scale: 1f64,
                    y_scale: 0.67f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::AQUIFER_FLUID_LEVEL_SPREAD,
                    xz_scale: 1f64,
                    y_scale: 0.7142857142857143f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::AQUIFER_LAVA,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::ORE_VEININESS,
                    xz_scale: 1.5f64,
                    y_scale: 1.5f64,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 172usize,
                when_in_range_index: 203usize,
                when_out_range_index: 12usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 51f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 204usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::ORE_VEIN_A,
                    xz_scale: 4f64,
                    y_scale: 4f64,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 172usize,
                when_in_range_index: 206usize,
                when_out_range_index: 12usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 51f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 207usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 208usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::ORE_VEIN_B,
                    xz_scale: 4f64,
                    y_scale: 4f64,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 172usize,
                when_in_range_index: 210usize,
                when_out_range_index: 12usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 51f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 211usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 212usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 209usize,
                argument2_index: 213usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 214usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.07999999821186066f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::ORE_GAP,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
        ],
        barrier_noise: 199usize,
        fluid_level_floodedness_noise: 200usize,
        fluid_level_spread_noise: 201usize,
        lava_noise: 202usize,
        erosion: 19usize,
        depth: 33usize,
        final_density: 198usize,
        vein_toggle: 205usize,
        vein_ridged: 215usize,
        vein_gap: 216usize,
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
            BaseNoiseFunctionComponent::ShiftA {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 9usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 10usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Constant { value: 0f64 },
            BaseNoiseFunctionComponent::ShiftB {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
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
                    noise_id: DoublePerlinNoiseParameters::CONTINENTALNESS,
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
                    noise_id: DoublePerlinNoiseParameters::EROSION,
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
                    noise_id: DoublePerlinNoiseParameters::RIDGE,
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
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 32usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 2usize,
                argument2_index: 33usize,
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
                input_index: 35usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -10f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 4usize,
                argument2_index: 36usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 37usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 10f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 38usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 39usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 40usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 34usize,
                argument2_index: 41usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 42usize,
                data: &UnaryData {
                    operation: UnaryOperation::QuarterNegative,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 43usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 4f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 44usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.703125f64,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 45usize,
                data: &ClampData {
                    min_value: -64f64,
                    max_value: 64f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 46usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.078125f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 1usize,
                argument2_index: 47usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 48usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.078125f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 49usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.1171875f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 0usize,
                argument2_index: 50usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 51usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.1171875f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 52usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.390625f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 41usize,
                data: &UnaryData {
                    operation: UnaryOperation::Invert,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 54usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.2734375f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 33usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -1f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 55usize,
                argument2_index: 56usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 57usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -128f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 58usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 128f64,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 59usize,
                data: &ClampData {
                    min_value: -40f64,
                    max_value: 320f64,
                },
            },
            BaseNoiseFunctionComponent::FindTopSurface {
                density_index: 53usize,
                upper_bound_index: 60usize,
                data: &FindTopSurfaceData {
                    lower_bound: -64i32,
                    cell_height: 8i32,
                },
            },
        ],
    },
    multi_noise: BaseMultiNoiseRouter {
        full_component_stack: &[
            BaseNoiseFunctionComponent::ShiftA {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 0usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 1usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Constant { value: 0f64 },
            BaseNoiseFunctionComponent::ShiftB {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
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
                    noise_id: DoublePerlinNoiseParameters::RIDGE,
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
                    noise_id: DoublePerlinNoiseParameters::TEMPERATURE,
                },
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 2usize,
                shift_y_index: 3usize,
                shift_z_index: 6usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::VEGETATION,
                },
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 2usize,
                shift_y_index: 3usize,
                shift_z_index: 6usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::CONTINENTALNESS,
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
                    noise_id: DoublePerlinNoiseParameters::EROSION,
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
                    scaled_xz_scale: 0.25f64,
                    scaled_y_scale: 0.28125f64,
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
            BaseNoiseFunctionComponent::Linear {
                input_index: 9usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.64f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 10usize,
                wrapper: WrapperType::Interpolated,
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
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 0usize,
                shift_y_index: 0usize,
                shift_z_index: 0usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::NETHER_TEMPERATURE,
                },
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 0usize,
                shift_y_index: 0usize,
                shift_z_index: 0usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::NETHER_VEGETATION,
                },
            },
        ],
        temperature: 1usize,
        vegetation: 2usize,
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
                    scaled_xz_scale: 0.25f64,
                    scaled_y_scale: 0.5f64,
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
            BaseNoiseFunctionComponent::Linear {
                input_index: 11usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.64f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 12usize,
                wrapper: WrapperType::Interpolated,
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
        full_component_stack: &[BaseNoiseFunctionComponent::Constant { value: 0f64 }],
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
