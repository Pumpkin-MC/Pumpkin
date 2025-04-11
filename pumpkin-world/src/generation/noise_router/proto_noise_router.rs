use enum_dispatch::enum_dispatch;
use pumpkin_data::{
    chunk::DoublePerlinNoiseParameters,
    noise_router::{
        BaseNoiseFunctionComponent, BaseNoiseRouters, BinaryOperation, LinearOperation, SplineRepr,
        UnaryOperation,
    },
};
use pumpkin_util::random::RandomDeriverImpl;

use crate::{GlobalRandomConfig, generation::noise::perlin::DoublePerlinNoiseSampler};

use super::{
    chunk_density_function::ChunkNoiseFunctionSampleOptions,
    chunk_noise_router::{ChunkNoiseFunctionComponent, StaticChunkNoiseFunctionComponentImpl},
    density_function::{
        IndexToNoisePos, NoiseFunctionComponentRange, NoisePos, PassThrough,
        StaticIndependentChunkNoiseFunctionComponentImpl, Wrapper,
        math::{Binary, Clamp, Constant, Linear, Unary},
        misc::{ClampedYGradient, EndIsland, RangeChoice, WeirdScaled},
        noise::{InterpolatedNoiseSampler, Noise, ShiftA, ShiftB, ShiftedNoise},
        spline::{Spline, SplineFunction, SplinePoint, SplineValue},
    },
};

#[enum_dispatch(
    StaticIndependentChunkNoiseFunctionComponentImpl,
    NoiseFunctionComponentRange
)]
pub enum IndependentProtoNoiseFunctionComponent {
    Constant(Constant),
    EndIsland(EndIsland),
    Noise(Noise),
    ShiftA(ShiftA),
    ShiftB(ShiftB),
    InterpolatedNoise(InterpolatedNoiseSampler),
    ClampedYGradient(ClampedYGradient),
}

#[enum_dispatch(StaticChunkNoiseFunctionComponentImpl, NoiseFunctionComponentRange)]
pub enum DependentProtoNoiseFunctionComponent {
    Linear(Linear),
    Unary(Unary),
    Binary(Binary),
    ShiftedNoise(ShiftedNoise),
    WeirdScaled(WeirdScaled),
    Clamp(Clamp),
    RangeChoice(RangeChoice),
    Spline(SplineFunction),
}

#[enum_dispatch(NoiseFunctionComponentRange)]
pub enum ProtoNoiseFunctionComponent {
    Independent(IndependentProtoNoiseFunctionComponent),
    Dependent(DependentProtoNoiseFunctionComponent),
    Wrapper(Wrapper),
    PassThrough(PassThrough),
}

pub struct DoublePerlinNoiseBuilder<'a> {
    random_config: &'a GlobalRandomConfig,
}

impl<'a> DoublePerlinNoiseBuilder<'a> {
    pub fn new(rand: &'a GlobalRandomConfig) -> Self {
        Self {
            random_config: rand,
        }
    }

    pub fn get_noise_sampler_for_id(&mut self, id: &str) -> DoublePerlinNoiseSampler {
        let parameters = DoublePerlinNoiseParameters::id_to_parameters(id)
            .unwrap_or_else(|| panic!("Unknown noise id: {}", id));

        // Note that the parameters' id is differenent than `id`
        let mut random = self
            .random_config
            .base_random_deriver
            .split_string(parameters.id());
        DoublePerlinNoiseSampler::new(&mut random, parameters, false)
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

fn build_spline_recursive(spline_repr: &SplineRepr) -> SplineValue {
    match spline_repr {
        SplineRepr::Standard {
            location_function_index,
            points,
        } => {
            let points = points
                .iter()
                .map(|point| {
                    let value = build_spline_recursive(point.value);
                    SplinePoint::new(point.location, value, point.derivative)
                })
                .collect();
            SplineValue::Spline(Spline::new(*location_function_index, points))
        }
        // Top level splines always take a density function as input
        SplineRepr::Fixed { value } => SplineValue::Fixed(*value),
    }
}

impl ProtoNoiseRouters {
    pub fn generate_proto_stack(
        base_stack: &[BaseNoiseFunctionComponent],
        random_config: &GlobalRandomConfig,
    ) -> Box<[ProtoNoiseFunctionComponent]> {
        let mut perlin_noise_builder = DoublePerlinNoiseBuilder::new(random_config);

        // Contiguous memory for our function components
        let mut stack = Vec::<ProtoNoiseFunctionComponent>::with_capacity(base_stack.len());

        for component in base_stack {
            let converted = match component {
                BaseNoiseFunctionComponent::Spline { spline } => {
                    let spline = match build_spline_recursive(spline) {
                        SplineValue::Spline(spline) => spline,
                        // Top level splines always take in a density function
                        SplineValue::Fixed(_) => unreachable!(),
                    };

                    ProtoNoiseFunctionComponent::Dependent(
                        DependentProtoNoiseFunctionComponent::Spline(SplineFunction::new(
                            spline, &stack,
                        )),
                    )
                }
                BaseNoiseFunctionComponent::EndIslands => ProtoNoiseFunctionComponent::Independent(
                    IndependentProtoNoiseFunctionComponent::EndIsland(EndIsland::new(
                        random_config.seed,
                    )),
                ),
                BaseNoiseFunctionComponent::Noise { data } => {
                    let sampler = perlin_noise_builder.get_noise_sampler_for_id(data.noise_id);
                    ProtoNoiseFunctionComponent::Independent(
                        IndependentProtoNoiseFunctionComponent::Noise(Noise::new(sampler, data)),
                    )
                }
                BaseNoiseFunctionComponent::ShiftA { noise_id } => {
                    let sampler = perlin_noise_builder.get_noise_sampler_for_id(noise_id);
                    ProtoNoiseFunctionComponent::Independent(
                        IndependentProtoNoiseFunctionComponent::ShiftA(ShiftA::new(sampler)),
                    )
                }
                BaseNoiseFunctionComponent::ShiftB { noise_id } => {
                    let sampler = perlin_noise_builder.get_noise_sampler_for_id(noise_id);
                    ProtoNoiseFunctionComponent::Independent(
                        IndependentProtoNoiseFunctionComponent::ShiftB(ShiftB::new(sampler)),
                    )
                }
                BaseNoiseFunctionComponent::BlendDensity { input_index } => {
                    // TODO: Replace this when the blender is implemented
                    let min_value = stack[*input_index].min();
                    let max_value = stack[*input_index].max();

                    ProtoNoiseFunctionComponent::PassThrough(PassThrough::new(
                        *input_index,
                        min_value,
                        max_value,
                    ))
                }
                BaseNoiseFunctionComponent::BlendAlpha => {
                    // TODO: Replace this with the cache when the blender is implemented
                    ProtoNoiseFunctionComponent::Independent(
                        IndependentProtoNoiseFunctionComponent::Constant(Constant::new(1.0)),
                    )
                }
                BaseNoiseFunctionComponent::BlendOffset => {
                    // TODO: Replace this with the cache when the blender is implemented
                    ProtoNoiseFunctionComponent::Independent(
                        IndependentProtoNoiseFunctionComponent::Constant(Constant::new(0.0)),
                    )
                }
                BaseNoiseFunctionComponent::Beardifier => {
                    // TODO: Replace this when world structures are implemented
                    ProtoNoiseFunctionComponent::Independent(
                        IndependentProtoNoiseFunctionComponent::Constant(Constant::new(0.0)),
                    )
                }
                BaseNoiseFunctionComponent::ShiftedNoise {
                    shift_x_index,
                    shift_y_index,
                    shift_z_index,
                    data,
                } => {
                    let sampler = perlin_noise_builder.get_noise_sampler_for_id(data.noise_id);
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
                        // TODO: I'm pretty sure this can be more restrictive
                        UnaryOperation::Abs | UnaryOperation::Square => {
                            (arg1_min.max(0.0), applied_min_value.max(applied_max_value))
                        }
                        UnaryOperation::Squeeze
                        | UnaryOperation::Cube
                        | UnaryOperation::QuarterNegative
                        | UnaryOperation::HalfNegative => (applied_min_value, applied_max_value),
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
                BaseNoiseFunctionComponent::WeirdScaled { input_index, data } => {
                    let sampler = perlin_noise_builder.get_noise_sampler_for_id(data.noise_id);
                    ProtoNoiseFunctionComponent::Dependent(
                        DependentProtoNoiseFunctionComponent::WeirdScaled(WeirdScaled::new(
                            *input_index,
                            sampler,
                            data,
                        )),
                    )
                }
                BaseNoiseFunctionComponent::InterpolatedNoiseSampler { data } => {
                    let mut random_generator = random_config
                        .base_random_deriver
                        .split_string("minecraft:terrain");

                    ProtoNoiseFunctionComponent::Independent(
                        IndependentProtoNoiseFunctionComponent::InterpolatedNoise(
                            InterpolatedNoiseSampler::new(data, &mut random_generator),
                        ),
                    )
                }
            };

            stack.push(converted);
        }

        stack.into()
    }

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
