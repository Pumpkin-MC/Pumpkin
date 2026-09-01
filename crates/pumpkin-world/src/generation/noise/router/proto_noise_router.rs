use pumpkin_data::{
    chunk::DoublePerlinNoiseParameters,
    noise_router::{
        BaseNoiseFunctionComponent, BaseNoiseRouters, BinaryOperation, LinearOperation, SplineRepr,
        UnaryOperation,
    },
};
use pumpkin_util::random::{legacy_rand::LegacyRand, xoroshiro128::XoroshiroSplitter};

use crate::{
    GlobalRandomConfig,
    generation::noise::{
        perlin::DoublePerlinNoiseSampler, router::find_top_surface::FindTopSurface,
    },
};

use super::{
    chunk_density_function::ChunkNoiseFunctionSampleOptions,
    chunk_noise_router::{ChunkNoiseFunctionComponent, StaticChunkNoiseFunctionComponentImpl},
    density_function::{
        IndexToNoisePos, NoiseFunctionComponentRange, PassThrough,
        StaticIndependentChunkNoiseFunctionComponentImpl, Wrapper,
        beardifier::Beardifier,
        math::{Binary, Clamp, Constant, Lerp, Linear, Rounding, Unary},
        misc::{
            ClampedYGradient, DistanceToPoint, EndIsland, Gradient, IntervalSelect, RangeChoice,
            Slice,
        },
        noise::{InterpolatedNoiseSampler, Noise, ShiftA, ShiftB, ShiftedNoise},
        spline::{Spline, SplineFunction, SplinePoint, SplineValue},
    },
};
use pumpkin_util::math::vector3::Vector3;

pub static BEARDIFIER_ZERO_CONSTANT: IndependentProtoNoiseFunctionComponent =
    IndependentProtoNoiseFunctionComponent::Constant(Constant::new(0.0));

pub enum IndependentProtoNoiseFunctionComponent {
    Constant(Constant),
    EndIsland(EndIsland),
    Noise(Noise),
    ShiftA(ShiftA),
    ShiftB(ShiftB),
    InterpolatedNoise(InterpolatedNoiseSampler),
    ClampedYGradient(ClampedYGradient),
    Gradient(Gradient),
    DistanceToPoint(DistanceToPoint),
}

impl NoiseFunctionComponentRange for IndependentProtoNoiseFunctionComponent {
    #[inline]
    fn min(&self) -> f32 {
        match self {
            Self::Constant(c) => c.min(),
            Self::EndIsland(e) => e.min(),
            Self::Noise(n) => n.min(),
            Self::ShiftA(s) => s.min(),
            Self::ShiftB(s) => s.min(),
            Self::InterpolatedNoise(i) => i.min(),
            Self::ClampedYGradient(c) => c.min(),
            Self::Gradient(g) => g.min(),
            Self::DistanceToPoint(d) => d.min(),
        }
    }

    #[inline]
    fn max(&self) -> f32 {
        match self {
            Self::Constant(c) => c.max(),
            Self::EndIsland(e) => e.max(),
            Self::Noise(n) => n.max(),
            Self::ShiftA(s) => s.max(),
            Self::ShiftB(s) => s.max(),
            Self::InterpolatedNoise(i) => i.max(),
            Self::ClampedYGradient(c) => c.max(),
            Self::Gradient(g) => g.max(),
            Self::DistanceToPoint(d) => d.max(),
        }
    }
}

impl StaticIndependentChunkNoiseFunctionComponentImpl for IndependentProtoNoiseFunctionComponent {
    #[inline]
    fn sample(&self, pos: &Vector3<i32>) -> f32 {
        match self {
            Self::Constant(c) => c.sample(pos),
            Self::EndIsland(e) => e.sample(pos),
            Self::Noise(n) => n.sample(pos),
            Self::ShiftA(s) => s.sample(pos),
            Self::ShiftB(s) => s.sample(pos),
            Self::InterpolatedNoise(i) => i.sample(pos),
            Self::ClampedYGradient(c) => c.sample(pos),
            Self::Gradient(g) => g.sample(pos),
            Self::DistanceToPoint(d) => d.sample(pos),
        }
    }

    #[inline]
    fn fill(&self, array: &mut [f32], mapper: &impl IndexToNoisePos) {
        match self {
            Self::Constant(c) => c.fill(array, mapper),
            Self::EndIsland(e) => e.fill(array, mapper),
            Self::Noise(n) => n.fill(array, mapper),
            Self::ShiftA(s) => s.fill(array, mapper),
            Self::ShiftB(s) => s.fill(array, mapper),
            Self::InterpolatedNoise(i) => i.fill(array, mapper),
            Self::ClampedYGradient(c) => c.fill(array, mapper),
            Self::Gradient(g) => g.fill(array, mapper),
            Self::DistanceToPoint(d) => d.fill(array, mapper),
        }
    }
}

pub enum DependentProtoNoiseFunctionComponent {
    Linear(Linear),
    Unary(Unary),
    Binary(Binary),
    ShiftedNoise(ShiftedNoise),
    IntervalSelect(IntervalSelect),
    FindTopSurface(FindTopSurface),
    Clamp(Clamp),
    RangeChoice(RangeChoice),
    Spline(SplineFunction),
    Lerp(Lerp),
    Rounding(Rounding),
    Slice(Slice),
}

impl NoiseFunctionComponentRange for DependentProtoNoiseFunctionComponent {
    #[inline]
    fn min(&self) -> f32 {
        match self {
            Self::Linear(l) => l.min(),
            Self::Unary(u) => u.min(),
            Self::Binary(b) => b.min(),
            Self::ShiftedNoise(s) => s.min(),
            Self::IntervalSelect(i) => i.min(),
            Self::FindTopSurface(f) => f.min(),
            Self::Clamp(c) => c.min(),
            Self::RangeChoice(r) => r.min(),
            Self::Spline(s) => s.min(),
            Self::Lerp(l) => l.min(),
            Self::Rounding(r) => r.min(),
            Self::Slice(s) => s.min(),
        }
    }

    #[inline]
    fn max(&self) -> f32 {
        match self {
            Self::Linear(l) => l.max(),
            Self::Unary(u) => u.max(),
            Self::Binary(b) => b.max(),
            Self::ShiftedNoise(s) => s.max(),
            Self::IntervalSelect(i) => i.max(),
            Self::FindTopSurface(f) => f.max(),
            Self::Clamp(c) => c.max(),
            Self::RangeChoice(r) => r.max(),
            Self::Spline(s) => s.max(),
            Self::Lerp(l) => l.max(),
            Self::Rounding(r) => r.max(),
            Self::Slice(s) => s.max(),
        }
    }
}

impl StaticChunkNoiseFunctionComponentImpl for DependentProtoNoiseFunctionComponent {
    #[inline]
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f32 {
        match self {
            Self::Linear(l) => l.sample(component_stack, pos, sample_options),
            Self::Unary(u) => u.sample(component_stack, pos, sample_options),
            Self::Binary(b) => b.sample(component_stack, pos, sample_options),
            Self::ShiftedNoise(s) => s.sample(component_stack, pos, sample_options),
            Self::IntervalSelect(i) => i.sample(component_stack, pos, sample_options),
            Self::FindTopSurface(f) => f.sample(component_stack, pos, sample_options),
            Self::Clamp(c) => c.sample(component_stack, pos, sample_options),
            Self::RangeChoice(r) => r.sample(component_stack, pos, sample_options),
            Self::Spline(s) => s.sample(component_stack, pos, sample_options),
            Self::Lerp(l) => l.sample(component_stack, pos, sample_options),
            Self::Rounding(r) => r.sample(component_stack, pos, sample_options),
            Self::Slice(s) => s.sample(component_stack, pos, sample_options),
        }
    }

    #[inline]
    fn fill(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        array: &mut [f32],
        mapper: &impl IndexToNoisePos,
        sample_options: &mut ChunkNoiseFunctionSampleOptions,
    ) {
        match self {
            Self::Linear(l) => l.fill(component_stack, array, mapper, sample_options),
            Self::Unary(u) => u.fill(component_stack, array, mapper, sample_options),
            Self::Binary(b) => b.fill(component_stack, array, mapper, sample_options),
            Self::ShiftedNoise(s) => s.fill(component_stack, array, mapper, sample_options),
            Self::IntervalSelect(i) => i.fill(component_stack, array, mapper, sample_options),
            Self::FindTopSurface(f) => f.fill(component_stack, array, mapper, sample_options),
            Self::Clamp(c) => c.fill(component_stack, array, mapper, sample_options),
            Self::RangeChoice(r) => r.fill(component_stack, array, mapper, sample_options),
            Self::Spline(s) => s.fill(component_stack, array, mapper, sample_options),
            Self::Lerp(l) => l.fill(component_stack, array, mapper, sample_options),
            Self::Rounding(r) => r.fill(component_stack, array, mapper, sample_options),
            Self::Slice(s) => s.fill(component_stack, array, mapper, sample_options),
        }
    }
}

pub enum ProtoNoiseFunctionComponent {
    Independent(IndependentProtoNoiseFunctionComponent),
    Dependent(DependentProtoNoiseFunctionComponent),
    Wrapper(Wrapper),
    Beardifier(Beardifier),
    PassThrough(PassThrough),
}

impl NoiseFunctionComponentRange for ProtoNoiseFunctionComponent {
    #[inline]
    fn min(&self) -> f32 {
        match self {
            Self::Independent(independent) => independent.min(),
            Self::Dependent(dependent) => dependent.min(),
            Self::Wrapper(wrapper) => wrapper.min(),
            Self::Beardifier(beardifier) => beardifier.min(),
            Self::PassThrough(pass_through) => pass_through.min(),
        }
    }

    #[inline]
    fn max(&self) -> f32 {
        match self {
            Self::Independent(independent) => independent.max(),
            Self::Dependent(dependent) => dependent.max(),
            Self::Wrapper(wrapper) => wrapper.max(),
            Self::Beardifier(beardifier) => beardifier.max(),
            Self::PassThrough(pass_through) => pass_through.max(),
        }
    }
}

pub struct ProtoNoiseRouter {
    pub full_component_stack: Box<[ProtoNoiseFunctionComponent]>,
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

pub struct ProtoSurfaceEstimator {
    pub full_component_stack: Box<[ProtoNoiseFunctionComponent]>,
}

pub struct ProtoMultiNoiseRouter {
    pub full_component_stack: Box<[ProtoNoiseFunctionComponent]>,
    pub temperature: usize,
    pub vegetation: usize,
    pub continents: usize,
    pub erosion: usize,
    pub depth: usize,
    pub ridges: usize,
}

pub struct ProtoNoiseRouters {
    pub noise: ProtoNoiseRouter,
    pub surface_estimator: ProtoSurfaceEstimator,
    pub multi_noise: ProtoMultiNoiseRouter,
}

pub struct DoublePerlinNoiseBuilder;

impl DoublePerlinNoiseBuilder {
    #[must_use]
    pub fn get_noise_sampler_for_id(
        base_random_deriver: &XoroshiroSplitter,
        parameters: &DoublePerlinNoiseParameters,
    ) -> DoublePerlinNoiseSampler {
        let mut random = base_random_deriver.from_lo_and_hi(parameters.lo, parameters.hi);
        DoublePerlinNoiseSampler::from_params(&mut random, parameters, false)
    }
}

fn build_spline_recursive(spline: &SplineRepr) -> SplineValue {
    match spline {
        SplineRepr::Fixed { value } => SplineValue::Fixed(*value),
        SplineRepr::Standard {
            location_function_index,
            points,
        } => {
            let mut points_vec = Vec::with_capacity(points.len());

            for point in *points {
                let value = build_spline_recursive(point.value);
                points_vec.push(SplinePoint::new(point.location, value, point.derivative));
            }

            SplineValue::Spline(Spline::new(*location_function_index, points_vec.into()))
        }
    }
}

impl ProtoNoiseRouters {
    #[must_use]
    #[expect(clippy::unreachable, clippy::too_many_lines)]
    pub fn generate_proto_stack(
        base_stack: &[BaseNoiseFunctionComponent],
        random_config: &GlobalRandomConfig,
    ) -> Box<[ProtoNoiseFunctionComponent]> {
        let base_random_deriver = &random_config.base_random_deriver;

        // Contiguous memory for our function components
        let mut stack = Vec::<ProtoNoiseFunctionComponent>::with_capacity(base_stack.len());

        for component in base_stack {
            let converted = match component {
                BaseNoiseFunctionComponent::Spline { spline } => {
                    let spline = match build_spline_recursive(spline) {
                        SplineValue::Spline(spline) => spline,
                        SplineValue::Fixed(_) => unreachable!(),
                    };

                    ProtoNoiseFunctionComponent::Dependent(
                        DependentProtoNoiseFunctionComponent::Spline(SplineFunction::new(
                            spline, &stack,
                        )),
                    )
                }
                BaseNoiseFunctionComponent::FindTopSurface {
                    density_index,
                    upper_bound_index,
                    data,
                } => {
                    let min_value = data.lower_bound as f32;
                    let max_value = stack[*upper_bound_index].max().max(min_value);

                    ProtoNoiseFunctionComponent::Dependent(
                        DependentProtoNoiseFunctionComponent::FindTopSurface(FindTopSurface::new(
                            *density_index,
                            *upper_bound_index,
                            min_value,
                            max_value,
                            data,
                        )),
                    )
                }
                BaseNoiseFunctionComponent::EndIslands => ProtoNoiseFunctionComponent::Independent(
                    IndependentProtoNoiseFunctionComponent::EndIsland(EndIsland::new(
                        random_config.seed,
                    )),
                ),
                BaseNoiseFunctionComponent::Noise { data } => {
                    let sampler = DoublePerlinNoiseBuilder::get_noise_sampler_for_id(
                        base_random_deriver,
                        &data.noise_id,
                    );
                    ProtoNoiseFunctionComponent::Independent(
                        IndependentProtoNoiseFunctionComponent::Noise(Noise::new(sampler, data)),
                    )
                }
                BaseNoiseFunctionComponent::ShiftA { noise_id } => {
                    let sampler = DoublePerlinNoiseBuilder::get_noise_sampler_for_id(
                        base_random_deriver,
                        noise_id,
                    );
                    ProtoNoiseFunctionComponent::Independent(
                        IndependentProtoNoiseFunctionComponent::ShiftA(ShiftA::new(sampler)),
                    )
                }
                BaseNoiseFunctionComponent::ShiftB { noise_id } => {
                    let sampler = DoublePerlinNoiseBuilder::get_noise_sampler_for_id(
                        base_random_deriver,
                        noise_id,
                    );
                    ProtoNoiseFunctionComponent::Independent(
                        IndependentProtoNoiseFunctionComponent::ShiftB(ShiftB::new(sampler)),
                    )
                }
                BaseNoiseFunctionComponent::BlendDensity { input_index } => {
                    let min_value = stack[*input_index].min();
                    let max_value = stack[*input_index].max();

                    ProtoNoiseFunctionComponent::PassThrough(PassThrough::new(
                        *input_index,
                        min_value,
                        max_value,
                    ))
                }
                BaseNoiseFunctionComponent::BlendAlpha => ProtoNoiseFunctionComponent::Independent(
                    IndependentProtoNoiseFunctionComponent::Constant(Constant::new(1.0)),
                ),
                BaseNoiseFunctionComponent::BlendOffset => {
                    ProtoNoiseFunctionComponent::Independent(
                        IndependentProtoNoiseFunctionComponent::Constant(Constant::new(0.0)),
                    )
                }
                BaseNoiseFunctionComponent::Beardifier => ProtoNoiseFunctionComponent::Beardifier(
                    Beardifier::new(Vec::new(), Vec::new(), None),
                ),
                BaseNoiseFunctionComponent::ShiftedNoise {
                    shift_x_index,
                    shift_y_index,
                    shift_z_index,
                    data,
                } => {
                    let sampler = match data.noise_id.id {
                        id if id == DoublePerlinNoiseParameters::NETHER_TEMPERATURE.id => {
                            let mut legacy_rand =
                                LegacyRand::from_seed(random_config.seed.wrapping_add(0));
                            DoublePerlinNoiseSampler::from_params(
                                &mut legacy_rand,
                                &data.noise_id,
                                true,
                            )
                        }
                        id if id == DoublePerlinNoiseParameters::NETHER_VEGETATION.id => {
                            let mut legacy_rand =
                                LegacyRand::from_seed(random_config.seed.wrapping_add(1));
                            DoublePerlinNoiseSampler::from_params(
                                &mut legacy_rand,
                                &data.noise_id,
                                true,
                            )
                        }
                        _ => DoublePerlinNoiseBuilder::get_noise_sampler_for_id(
                            base_random_deriver,
                            &data.noise_id,
                        ),
                    };
                    ProtoNoiseFunctionComponent::Dependent(
                        DependentProtoNoiseFunctionComponent::ShiftedNoise(ShiftedNoise::new(
                            *shift_x_index,
                            *shift_y_index,
                            *shift_z_index,
                            sampler,
                            data,
                        )),
                    )
                }
                BaseNoiseFunctionComponent::RangeChoice {
                    input_index,
                    when_in_range_index,
                    when_out_range_index,
                    data,
                } => {
                    let min_value = stack[*when_in_range_index]
                        .min()
                        .min(stack[*when_out_range_index].min());
                    let max_value = stack[*when_in_range_index]
                        .max()
                        .max(stack[*when_out_range_index].max());

                    ProtoNoiseFunctionComponent::Dependent(
                        DependentProtoNoiseFunctionComponent::RangeChoice(RangeChoice::new(
                            *input_index,
                            *when_in_range_index,
                            *when_out_range_index,
                            min_value,
                            max_value,
                            data,
                        )),
                    )
                }
                BaseNoiseFunctionComponent::Binary {
                    argument1_index,
                    argument2_index,
                    data,
                } => {
                    let arg1_min = stack[*argument1_index].min();
                    let arg1_max = stack[*argument1_index].max();

                    let arg2_min = stack[*argument2_index].min();
                    let arg2_max = stack[*argument2_index].max();

                    let (min, max) = match data.operation {
                        BinaryOperation::Add => (arg1_min + arg2_min, arg1_max + arg2_max),
                        BinaryOperation::Mul => {
                            let min = if arg1_min > 0.0 && arg2_min > 0.0 {
                                arg1_min * arg2_min
                            } else if arg1_max < 0.0 && arg2_max < 0.0 {
                                arg1_max * arg2_max
                            } else {
                                (arg1_min * arg2_max).min(arg1_max * arg2_min)
                            };

                            let max = if arg1_min > 0.0 && arg2_min > 0.0 {
                                arg1_max * arg2_max
                            } else if arg1_max < 0.0 && arg2_max < 0.0 {
                                arg1_min * arg2_min
                            } else {
                                (arg1_min * arg2_min).max(arg1_max * arg2_max)
                            };

                            (min, max)
                        }
                        BinaryOperation::Min => (arg1_min.min(arg2_min), arg1_max.min(arg2_max)),
                        BinaryOperation::Max => (arg1_min.max(arg2_min), arg1_max.max(arg2_max)),
                        BinaryOperation::Sub => (arg1_min - arg2_max, arg1_max - arg2_min),
                        BinaryOperation::Div => (f32::NEG_INFINITY, f32::INFINITY),
                        BinaryOperation::Pow => (0.0, f32::INFINITY),
                    };

                    ProtoNoiseFunctionComponent::Dependent(
                        DependentProtoNoiseFunctionComponent::Binary(Binary::new(
                            *argument1_index,
                            *argument2_index,
                            min,
                            max,
                            data,
                        )),
                    )
                }
                BaseNoiseFunctionComponent::ClampedYGradient { data } => {
                    ProtoNoiseFunctionComponent::Independent(
                        IndependentProtoNoiseFunctionComponent::ClampedYGradient(
                            ClampedYGradient::new(data),
                        ),
                    )
                }
                BaseNoiseFunctionComponent::Gradient { data } => {
                    ProtoNoiseFunctionComponent::Independent(
                        IndependentProtoNoiseFunctionComponent::Gradient(Gradient::new(data)),
                    )
                }
                BaseNoiseFunctionComponent::DistanceToPoint { data } => {
                    ProtoNoiseFunctionComponent::Independent(
                        IndependentProtoNoiseFunctionComponent::DistanceToPoint(
                            DistanceToPoint::new(data),
                        ),
                    )
                }
                BaseNoiseFunctionComponent::Lerp {
                    alpha_index,
                    first_index,
                    second_index,
                } => {
                    let min_value = stack[*first_index].min().min(stack[*second_index].min());
                    let max_value = stack[*first_index].max().max(stack[*second_index].max());

                    ProtoNoiseFunctionComponent::Dependent(
                        DependentProtoNoiseFunctionComponent::Lerp(Lerp::new(
                            *alpha_index,
                            *first_index,
                            *second_index,
                            min_value,
                            max_value,
                        )),
                    )
                }
                BaseNoiseFunctionComponent::Rounding {
                    input_index,
                    multiple_index,
                    data,
                } => {
                    let min_value = stack[*input_index].min();
                    let max_value = stack[*input_index].max();

                    ProtoNoiseFunctionComponent::Dependent(
                        DependentProtoNoiseFunctionComponent::Rounding(Rounding::new(
                            *input_index,
                            *multiple_index,
                            min_value,
                            max_value,
                            data,
                        )),
                    )
                }
                BaseNoiseFunctionComponent::Slice {
                    input_index,
                    axis,
                    coordinate,
                } => {
                    let min_value = stack[*input_index].min();
                    let max_value = stack[*input_index].max();

                    ProtoNoiseFunctionComponent::Dependent(
                        DependentProtoNoiseFunctionComponent::Slice(Slice::new(
                            *input_index,
                            *axis,
                            *coordinate,
                            min_value,
                            max_value,
                        )),
                    )
                }
                BaseNoiseFunctionComponent::Constant { value } => {
                    ProtoNoiseFunctionComponent::Independent(
                        IndependentProtoNoiseFunctionComponent::Constant(Constant::new(*value)),
                    )
                }
                BaseNoiseFunctionComponent::Wrapper {
                    input_index,
                    wrapper,
                } => {
                    let min_value = stack[*input_index].min();
                    let max_value = stack[*input_index].max();

                    ProtoNoiseFunctionComponent::Wrapper(Wrapper::new(
                        *input_index,
                        *wrapper,
                        min_value,
                        max_value,
                    ))
                }
                BaseNoiseFunctionComponent::Linear { input_index, data } => {
                    let arg1_min = stack[*input_index].min();
                    let arg1_max = stack[*input_index].max();

                    let (min, max) = match data.operation {
                        LinearOperation::Add => {
                            (arg1_min + data.argument, arg1_max + data.argument)
                        }
                        LinearOperation::Mul => {
                            let min = if arg1_min > 0.0 && data.argument > 0.0 {
                                arg1_min * data.argument
                            } else if arg1_max < 0.0 && data.argument < 0.0 {
                                arg1_max * data.argument
                            } else {
                                (arg1_min * data.argument).min(arg1_max * data.argument)
                            };

                            let max = if arg1_min > 0.0 && data.argument > 0.0 {
                                arg1_max * data.argument
                            } else if arg1_max < 0.0 && data.argument < 0.0 {
                                arg1_min * data.argument
                            } else {
                                (arg1_min * data.argument).max(arg1_max * data.argument)
                            };

                            (min, max)
                        }
                    };

                    ProtoNoiseFunctionComponent::Dependent(
                        DependentProtoNoiseFunctionComponent::Linear(Linear::new(
                            *input_index,
                            min,
                            max,
                            data,
                        )),
                    )
                }
                BaseNoiseFunctionComponent::Clamp { input_index, data } => {
                    ProtoNoiseFunctionComponent::Dependent(
                        DependentProtoNoiseFunctionComponent::Clamp(Clamp::new(*input_index, data)),
                    )
                }
                BaseNoiseFunctionComponent::Unary { input_index, data } => {
                    let arg1_min = stack[*input_index].min();
                    let arg1_max = stack[*input_index].max();

                    let applied_min_value = data.apply_density(arg1_min);
                    let applied_max_value = data.apply_density(arg1_max);

                    let (min_value, max_value) = match data.operation {
                        UnaryOperation::Abs | UnaryOperation::Square => {
                            (arg1_min.max(0.0), applied_min_value.max(applied_max_value))
                        }
                        UnaryOperation::Squeeze
                        | UnaryOperation::Cube
                        | UnaryOperation::QuarterNegative
                        | UnaryOperation::HalfNegative => (applied_min_value, applied_max_value),
                        UnaryOperation::Invert => {
                            if arg1_min < 0.0 && arg1_max > 0.0 {
                                (f32::NEG_INFINITY, f32::INFINITY)
                            } else {
                                (applied_max_value, applied_min_value)
                            }
                        }
                        UnaryOperation::Negate => (-arg1_max, -arg1_min),
                        UnaryOperation::Sqrt => (0.0, applied_max_value.max(0.0)),
                        UnaryOperation::Log => (f32::NEG_INFINITY, applied_max_value),
                        UnaryOperation::Sign => (-1.0, 1.0),
                    };

                    ProtoNoiseFunctionComponent::Dependent(
                        DependentProtoNoiseFunctionComponent::Unary(Unary::new(
                            *input_index,
                            min_value,
                            max_value,
                            data,
                        )),
                    )
                }
                BaseNoiseFunctionComponent::IntervalSelect {
                    input_index,
                    thresholds,
                    functions_indices,
                } => {
                    let mut min_value = f32::INFINITY;
                    let mut max_value = f32::NEG_INFINITY;
                    for &idx in *functions_indices {
                        let min = stack[idx].min();
                        let max = stack[idx].max();
                        if min < min_value {
                            min_value = min;
                        }
                        if max > max_value {
                            max_value = max;
                        }
                    }

                    ProtoNoiseFunctionComponent::Dependent(
                        DependentProtoNoiseFunctionComponent::IntervalSelect(IntervalSelect::new(
                            *input_index,
                            thresholds,
                            functions_indices,
                            min_value,
                            max_value,
                        )),
                    )
                }
                BaseNoiseFunctionComponent::InterpolatedNoiseSampler { data } => {
                    ProtoNoiseFunctionComponent::Independent(
                        IndependentProtoNoiseFunctionComponent::InterpolatedNoise(
                            if random_config.legacy_random_source {
                                let mut legacy_rand =
                                    LegacyRand::from_seed(random_config.seed.wrapping_add(0));
                                InterpolatedNoiseSampler::new(data, &mut legacy_rand)
                            } else {
                                let mut random_generator = random_config
                                    .base_random_deriver
                                    .split_string("minecraft:terrain");
                                InterpolatedNoiseSampler::new(data, &mut random_generator)
                            },
                        ),
                    )
                }
            };

            stack.push(converted);
        }

        stack.into()
    }

    #[must_use]
    pub fn generate(base: &BaseNoiseRouters, random_config: &GlobalRandomConfig) -> Self {
        let noise_stack =
            Self::generate_proto_stack(base.noise.full_component_stack, random_config);
        let surface_stack =
            Self::generate_proto_stack(base.surface_estimator.full_component_stack, random_config);
        let multi_noise_stack =
            Self::generate_proto_stack(base.multi_noise.full_component_stack, random_config);

        Self {
            noise: ProtoNoiseRouter {
                full_component_stack: noise_stack,
                barrier_noise: base.noise.barrier_noise,
                fluid_level_floodedness_noise: base.noise.fluid_level_floodedness_noise,
                fluid_level_spread_noise: base.noise.fluid_level_spread_noise,
                lava_noise: base.noise.lava_noise,
                erosion: base.noise.erosion,
                depth: base.noise.depth,
                final_density: base.noise.final_density,
                vein_toggle: base.noise.vein_toggle,
                vein_ridged: base.noise.vein_ridged,
                vein_gap: base.noise.vein_gap,
            },
            surface_estimator: ProtoSurfaceEstimator {
                full_component_stack: surface_stack,
            },
            multi_noise: ProtoMultiNoiseRouter {
                full_component_stack: multi_noise_stack,
                temperature: base.multi_noise.temperature,
                vegetation: base.multi_noise.vegetation,
                continents: base.multi_noise.continents,
                erosion: base.multi_noise.erosion,
                depth: base.multi_noise.depth,
                ridges: base.multi_noise.ridges,
            },
        }
    }
}
