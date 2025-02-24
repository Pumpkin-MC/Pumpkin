use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use enum_dispatch::enum_dispatch;
use pumpkin_data::chunk::DoublePerlinNoiseParameters;

use crate::{
    GlobalRandomConfig,
    generation::noise::perlin::DoublePerlinNoiseSampler,
    noise_router::{
        density_function_ast::{DensityFunctionRepr, SplineRepr},
        noise_router_ast::NoiseRouterRepr,
    },
};

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
#[derive(Clone)]
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
#[derive(Clone)]
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
#[derive(Clone)]
pub enum ProtoNoiseFunctionComponent {
    Independent(IndependentProtoNoiseFunctionComponent),
    Dependent(DependentProtoNoiseFunctionComponent),
    Wrapper(Wrapper),
    PassThrough(PassThrough),
}

pub(crate) struct DoublePerlinNoiseBuilder<'a> {
    random_config: &'a GlobalRandomConfig,
}

impl<'a> DoublePerlinNoiseBuilder<'a> {
    pub fn new(rand: &'a GlobalRandomConfig) -> Self {
        Self {
            random_config: rand,
        }
    }

    fn get_noise_sampler_for_id(&mut self, id: &str) -> DoublePerlinNoiseSampler {
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

// NOTE: Invariant: all index references point to components that have a lower index than the
// component referencing it

/// Returns the index of component the AST represents on the stack
pub(crate) fn recursive_build_proto_stack<'a>(
    ast: &'a DensityFunctionRepr,
    random_config: &GlobalRandomConfig,
    stack: &mut Vec<ProtoNoiseFunctionComponent>,
    map: &mut HashMap<u64, usize>,
    perlin_noise_builder: &mut DoublePerlinNoiseBuilder<'a>,
) -> usize {
    let mut hasher = DefaultHasher::new();
    ast.hash(&mut hasher);
    let ast_hash = hasher.finish();

    map.get(&ast_hash).copied().unwrap_or_else(|| {
        let component = match ast {
            DensityFunctionRepr::Spline { spline, data } => {
                let spline = match recursive_build_spline(
                    spline,
                    random_config,
                    stack,
                    map,
                    perlin_noise_builder,
                ) {
                    SplineValue::Spline(spline) => spline,
                    _ => unreachable!(),
                };

                ProtoNoiseFunctionComponent::Dependent(
                    DependentProtoNoiseFunctionComponent::Spline(SplineFunction::new(spline, data)),
                )
            }
            DensityFunctionRepr::EndIslands => ProtoNoiseFunctionComponent::Independent(
                IndependentProtoNoiseFunctionComponent::EndIsland(EndIsland::new(
                    random_config.seed,
                )),
            ),
            DensityFunctionRepr::Noise { data } => {
                let sampler = perlin_noise_builder.get_noise_sampler_for_id(&data.noise_id);
                ProtoNoiseFunctionComponent::Independent(
                    IndependentProtoNoiseFunctionComponent::Noise(Noise::new(sampler, data)),
                )
            }
            DensityFunctionRepr::ShiftA { noise_id } => {
                let sampler = perlin_noise_builder.get_noise_sampler_for_id(noise_id);
                ProtoNoiseFunctionComponent::Independent(
                    IndependentProtoNoiseFunctionComponent::ShiftA(ShiftA::new(sampler)),
                )
            }
            DensityFunctionRepr::ShiftB { noise_id } => {
                let sampler = perlin_noise_builder.get_noise_sampler_for_id(noise_id);
                ProtoNoiseFunctionComponent::Independent(
                    IndependentProtoNoiseFunctionComponent::ShiftB(ShiftB::new(sampler)),
                )
            }
            DensityFunctionRepr::BlendDensity { input } => {
                // TODO: Replace this when the blender is implemented
                return recursive_build_proto_stack(
                    input,
                    random_config,
                    stack,
                    map,
                    perlin_noise_builder,
                );
            }
            DensityFunctionRepr::BlendAlpha => {
                // TODO: Replace this with the cache when the blender is implemented
                ProtoNoiseFunctionComponent::Independent(
                    IndependentProtoNoiseFunctionComponent::Constant(Constant::new(1.0)),
                )
            }
            DensityFunctionRepr::BlendOffset => {
                // TODO: Replace this with the cache when the blender is implemented
                ProtoNoiseFunctionComponent::Independent(
                    IndependentProtoNoiseFunctionComponent::Constant(Constant::new(0.0)),
                )
            }
            DensityFunctionRepr::Beardifier => {
                // TODO: Replace this when world structures are implemented
                ProtoNoiseFunctionComponent::Independent(
                    IndependentProtoNoiseFunctionComponent::Constant(Constant::new(0.0)),
                )
            }
            DensityFunctionRepr::ShiftedNoise {
                shift_x,
                shift_y,
                shift_z,
                data,
            } => {
                let input_x_index = recursive_build_proto_stack(
                    shift_x,
                    random_config,
                    stack,
                    map,
                    perlin_noise_builder,
                );

                let input_y_index = recursive_build_proto_stack(
                    shift_y,
                    random_config,
                    stack,
                    map,
                    perlin_noise_builder,
                );

                let input_z_index = recursive_build_proto_stack(
                    shift_z,
                    random_config,
                    stack,
                    map,
                    perlin_noise_builder,
                );

                let sampler = perlin_noise_builder.get_noise_sampler_for_id(&data.noise_id);
                ProtoNoiseFunctionComponent::Dependent(
                    DependentProtoNoiseFunctionComponent::ShiftedNoise(ShiftedNoise::new(
                        input_x_index,
                        input_y_index,
                        input_z_index,
                        sampler,
                        data,
                    )),
                )
            }
            DensityFunctionRepr::RangeChoice {
                input,
                when_in_range,
                when_out_range,
                data,
            } => {
                let input_index = recursive_build_proto_stack(
                    input,
                    random_config,
                    stack,
                    map,
                    perlin_noise_builder,
                );

                let in_range_index = recursive_build_proto_stack(
                    when_in_range,
                    random_config,
                    stack,
                    map,
                    perlin_noise_builder,
                );

                let out_range_index = recursive_build_proto_stack(
                    when_out_range,
                    random_config,
                    stack,
                    map,
                    perlin_noise_builder,
                );

                let min_value = stack[in_range_index]
                    .min()
                    .min(stack[out_range_index].min());
                let max_value = stack[in_range_index]
                    .max()
                    .max(stack[out_range_index].max());

                ProtoNoiseFunctionComponent::Dependent(
                    DependentProtoNoiseFunctionComponent::RangeChoice(RangeChoice::new(
                        input_index,
                        in_range_index,
                        out_range_index,
                        min_value,
                        max_value,
                        data,
                    )),
                )
            }
            DensityFunctionRepr::Binary {
                argument1,
                argument2,
                data,
            } => {
                let input1_index = recursive_build_proto_stack(
                    argument1,
                    random_config,
                    stack,
                    map,
                    perlin_noise_builder,
                );

                let input2_index = recursive_build_proto_stack(
                    argument2,
                    random_config,
                    stack,
                    map,
                    perlin_noise_builder,
                );

                ProtoNoiseFunctionComponent::Dependent(
                    DependentProtoNoiseFunctionComponent::Binary(Binary::new(
                        input1_index,
                        input2_index,
                        data,
                    )),
                )
            }
            DensityFunctionRepr::ClampedYGradient { data } => {
                ProtoNoiseFunctionComponent::Independent(
                    IndependentProtoNoiseFunctionComponent::ClampedYGradient(
                        ClampedYGradient::new(data),
                    ),
                )
            }
            DensityFunctionRepr::Constant { value } => ProtoNoiseFunctionComponent::Independent(
                IndependentProtoNoiseFunctionComponent::Constant(Constant::new(value.0)),
            ),
            DensityFunctionRepr::Wrapper { input, wrapper } => {
                let input_index = recursive_build_proto_stack(
                    input,
                    random_config,
                    stack,
                    map,
                    perlin_noise_builder,
                );
                let min_value = stack[input_index].min();
                let max_value = stack[input_index].max();

                ProtoNoiseFunctionComponent::Wrapper(Wrapper::new(
                    input_index,
                    *wrapper,
                    min_value,
                    max_value,
                ))
            }
            DensityFunctionRepr::Linear { input, data } => {
                let input_index = recursive_build_proto_stack(
                    input,
                    random_config,
                    stack,
                    map,
                    perlin_noise_builder,
                );

                ProtoNoiseFunctionComponent::Dependent(
                    DependentProtoNoiseFunctionComponent::Linear(Linear::new(input_index, data)),
                )
            }
            DensityFunctionRepr::Clamp { input, data } => {
                let input_index = recursive_build_proto_stack(
                    input,
                    random_config,
                    stack,
                    map,
                    perlin_noise_builder,
                );

                ProtoNoiseFunctionComponent::Dependent(DependentProtoNoiseFunctionComponent::Clamp(
                    Clamp::new(input_index, data),
                ))
            }
            DensityFunctionRepr::Unary { input, data } => {
                let input_index = recursive_build_proto_stack(
                    input,
                    random_config,
                    stack,
                    map,
                    perlin_noise_builder,
                );

                ProtoNoiseFunctionComponent::Dependent(DependentProtoNoiseFunctionComponent::Unary(
                    Unary::new(input_index, data),
                ))
            }
            DensityFunctionRepr::WeirdScaled { input, data } => {
                let input_index = recursive_build_proto_stack(
                    input,
                    random_config,
                    stack,
                    map,
                    perlin_noise_builder,
                );

                let sampler = perlin_noise_builder.get_noise_sampler_for_id(&data.noise_id);
                ProtoNoiseFunctionComponent::Dependent(
                    DependentProtoNoiseFunctionComponent::WeirdScaled(WeirdScaled::new(
                        input_index,
                        sampler,
                        data,
                    )),
                )
            }
            DensityFunctionRepr::InterpolatedNoiseSampler { data } => {
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

        //NOTE: Invariant: the current component is at the top of the stack
        let component_index = stack.len();
        stack.push(component);
        map.insert(ast_hash, component_index);
        component_index
    })
}

fn recursive_build_spline<'a>(
    spline_ast: &'a SplineRepr,
    random_config: &GlobalRandomConfig,
    stack: &mut Vec<ProtoNoiseFunctionComponent>,
    map: &mut HashMap<u64, usize>,
    perlin_noise_builder: &mut DoublePerlinNoiseBuilder<'a>,
) -> SplineValue {
    match spline_ast {
        SplineRepr::Standard {
            location_function,
            locations,
            values,
            derivatives,
        } => {
            let input_index = recursive_build_proto_stack(
                location_function,
                random_config,
                stack,
                map,
                perlin_noise_builder,
            );

            let points: Vec<_> = locations
                .iter()
                .zip(values)
                .zip(derivatives)
                .map(|((location, v), derivative)| {
                    let value =
                        recursive_build_spline(v, random_config, stack, map, perlin_noise_builder);
                    SplinePoint::new(location.0, value, derivative.0)
                })
                .collect();

            SplineValue::Spline(Spline::new(input_index, points.into_boxed_slice()))
        }
        SplineRepr::Fixed { value } => SplineValue::Fixed(value.0),
    }
}

#[derive(Clone)]
pub struct GlobalProtoNoiseRouter {
    pub barrier_noise: usize,
    pub fluid_level_floodedness_noise: usize,
    pub fluid_level_spread_noise: usize,
    pub lava_noise: usize,
    pub erosion: usize,
    pub depth: usize,
    pub initial_density_without_jaggedness: usize,
    pub final_density: usize,
    pub vein_toggle: usize,
    pub vein_ridged: usize,
    pub vein_gap: usize,
    pub ridges: usize,
    pub temperature: usize,
    pub continents: usize,
    pub vegetation: usize,
    pub component_stack: Box<[ProtoNoiseFunctionComponent]>,
}

impl GlobalProtoNoiseRouter {
    pub fn generate(ast: &NoiseRouterRepr, random_config: &GlobalRandomConfig) -> Self {
        // Contiguous memory for our function components
        let mut stack = Vec::<ProtoNoiseFunctionComponent>::new();
        // Map of AST hash to index in the stack
        let mut map = HashMap::<u64, usize>::new();
        let mut perlin_noise_builder = DoublePerlinNoiseBuilder::new(random_config);

        // Keep the functions that are called most frequently closer together in memory to try to
        // keep in it mem cache more. Functions that are added to the stack first are the most dense
        // with functions being added later the least dense due to only adding one component to the
        // stack based on the AST hash.
        //
        // This was determined visually and should probably be more programmatically tested.
        // E.g. everything with a flat cached gets cached on init so we dont care about where it
        // lives in memory

        macro_rules! push_ast {
            ($name:expr) => {
                recursive_build_proto_stack(
                    $name,
                    random_config,
                    &mut stack,
                    &mut map,
                    &mut perlin_noise_builder,
                )
            };
        }

        // The height estimator is called multiple times per aquifer call
        let initial_density_without_jaggedness =
            push_ast!(ast.initial_density_without_jaggedness());

        // The aquifer sampler is called most often
        let final_density = push_ast!(ast.final_density());
        let barrier_noise = push_ast!(ast.barrier_noise());
        let fluid_level_floodedness_noise = push_ast!(ast.fluid_level_floodedness_noise());
        let fluid_level_spread_noise = push_ast!(ast.fluid_level_spread_noise());
        let lava_noise = push_ast!(ast.lava_noise());

        // Ore sampler is called fewer times than aquifer sampler
        let vein_toggle = push_ast!(ast.vein_toggle());
        let vein_ridged = push_ast!(ast.vein_ridged());
        let vein_gap = push_ast!(ast.vein_gap());

        // These should all be cached so it doesnt matter where their components are
        let erosion = push_ast!(ast.erosion());
        let depth = push_ast!(ast.depth());

        //NOTE: Invariant: MultiNoiseSampler functions are pushed after the populate noise functions with
        let ridges = push_ast!(ast.ridges());
        let temperature = push_ast!(ast.temperature());
        let vegetation = push_ast!(ast.vegetation());
        let continents = push_ast!(ast.continents());

        Self {
            barrier_noise,
            fluid_level_floodedness_noise,
            fluid_level_spread_noise,
            final_density,
            lava_noise,
            erosion,
            depth,
            vein_toggle,
            vein_ridged,
            vein_gap,
            ridges,
            temperature,
            vegetation,
            continents,
            initial_density_without_jaggedness,
            component_stack: stack.into_boxed_slice(),
        }
    }
}
