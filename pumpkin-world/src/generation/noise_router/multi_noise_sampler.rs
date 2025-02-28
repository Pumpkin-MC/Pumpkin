use crate::{
    GlobalProtoNoiseRouter, generation::biome_coords,
    noise_router::density_function_ast::WrapperType,
};

use super::{
    chunk_density_function::{
        Cache2D, ChunkNoiseFunctionSampleOptions, ChunkSpecificNoiseFunctionComponent, FlatCache,
        SampleAction,
    },
    chunk_noise_router::ChunkNoiseFunctionComponent,
    density_function::{NoiseFunctionComponentRange, PassThrough, UnblendedNoisePos},
    proto_noise_router::ProtoNoiseFunctionComponent,
};

pub struct MultiNoiseSamplerBuilderOptions {
    // The biome coords of this chunk
    start_biome_x: i32,
    start_biome_z: i32,

    // Number of biome regions per chunk per axis
    horizontal_biome_end: usize,
}

impl MultiNoiseSamplerBuilderOptions {
    pub fn new(start_biome_x: i32, start_biome_z: i32, horizontal_biome_end: usize) -> Self {
        Self {
            start_biome_x,
            start_biome_z,
            horizontal_biome_end,
        }
    }
}

pub struct MultiNoiseSampler<'a> {
    temperature: usize,
    // AKA: Humidity
    vegetation: usize,
    continents: usize,
    erosion: usize,
    depth: usize,
    // AKA: Weirdness
    ridges: usize,
    component_stack: Box<[ChunkNoiseFunctionComponent<'a>]>,
}

impl<'a> MultiNoiseSampler<'a> {
    pub fn sample(&mut self, biome_x: i32, biome_y: i32, biome_z: i32) {
        let block_x = biome_coords::to_block(biome_x);
        let block_y = biome_coords::to_block(biome_y);
        let block_z = biome_coords::to_block(biome_z);

        let pos = UnblendedNoisePos::new(block_x, block_y, block_z);
        let sample_options =
            ChunkNoiseFunctionSampleOptions::new(false, SampleAction::SkipCellCaches, 0, 0, 0);

        let _temperature = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut self.component_stack[..=self.temperature],
            &pos,
            &sample_options,
        ) as f32;

        let _humidity = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut self.component_stack[..=self.vegetation],
            &pos,
            &sample_options,
        ) as f32;

        let _continentalness = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut self.component_stack[..=self.continents],
            &pos,
            &sample_options,
        ) as f32;

        let _erosion = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut self.component_stack[..=self.erosion],
            &pos,
            &sample_options,
        ) as f32;

        let _depth = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut self.component_stack[..=self.depth],
            &pos,
            &sample_options,
        ) as f32;

        let _weirdness = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut self.component_stack[..=self.ridges],
            &pos,
            &sample_options,
        ) as f32;

        // TODO: Multi noise value here
    }

    pub fn generate(
        base: &'a GlobalProtoNoiseRouter,
        build_options: &MultiNoiseSamplerBuilderOptions,
    ) -> Self {
        // TODO: It seems kind of wasteful to iter over all components (even those we dont need
        // because they're for chunk population), but this is the best I've got for now.
        // (Should we traverse the functions and update the indices?)
        let mut component_stack =
            Vec::<ChunkNoiseFunctionComponent>::with_capacity(base.component_stack.len());
        for base_component in base.component_stack.iter() {
            let chunk_component = match base_component {
                ProtoNoiseFunctionComponent::Dependent(dependent) => {
                    ChunkNoiseFunctionComponent::Dependent(dependent)
                }
                ProtoNoiseFunctionComponent::Independent(independent) => {
                    ChunkNoiseFunctionComponent::Independent(independent)
                }
                ProtoNoiseFunctionComponent::PassThrough(pass_through) => {
                    let min_value = component_stack[pass_through.input_index].min();
                    let max_value = component_stack[pass_through.input_index].max();
                    ChunkNoiseFunctionComponent::PassThrough(PassThrough {
                        input_index: pass_through.input_index,
                        max_value,
                        min_value,
                    })
                }
                ProtoNoiseFunctionComponent::Wrapper(wrapper) => {
                    //NOTE: Due to our previous invariant with the proto-function, it is guaranteed
                    // that the wrapped function is already on the stack
                    let min_value = component_stack[wrapper.input_index].min();
                    let max_value = component_stack[wrapper.input_index].max();

                    match wrapper.wrapper_type {
                        WrapperType::Cache2D => ChunkNoiseFunctionComponent::Chunk(Box::new(
                            ChunkSpecificNoiseFunctionComponent::Cache2D(Cache2D::new(
                                wrapper.input_index,
                                min_value,
                                max_value,
                            )),
                        )),
                        WrapperType::CacheFlat => {
                            let mut flat_cache = FlatCache::new(
                                wrapper.input_index,
                                min_value,
                                max_value,
                                build_options.start_biome_x,
                                build_options.start_biome_z,
                                build_options.horizontal_biome_end,
                            );
                            let sample_options = ChunkNoiseFunctionSampleOptions::new(
                                false,
                                SampleAction::SkipCellCaches,
                                0,
                                0,
                                0,
                            );

                            for biome_x_position in 0..=build_options.horizontal_biome_end {
                                let absolute_biome_x_position =
                                    build_options.start_biome_x + biome_x_position as i32;
                                let block_x_position =
                                    biome_coords::to_block(absolute_biome_x_position);

                                for biome_z_position in 0..=build_options.horizontal_biome_end {
                                    let absolute_biome_z_position =
                                        build_options.start_biome_z + biome_z_position as i32;
                                    let block_z_position =
                                        biome_coords::to_block(absolute_biome_z_position);

                                    let pos = UnblendedNoisePos::new(
                                        block_x_position,
                                        0,
                                        block_z_position,
                                    );

                                    //NOTE: Due to our stack invariant, what is on the stack is a
                                    // valid density function
                                    let sample = ChunkNoiseFunctionComponent::sample_from_stack(
                                        &mut component_stack[..=wrapper.input_index],
                                        &pos,
                                        &sample_options,
                                    );

                                    let cache_index = flat_cache
                                        .xz_to_index_const(biome_x_position, biome_z_position);
                                    flat_cache.cache[cache_index] = sample;
                                }
                            }

                            ChunkNoiseFunctionComponent::Chunk(Box::new(
                                ChunkSpecificNoiseFunctionComponent::FlatCache(flat_cache),
                            ))
                        }
                        _ => {
                            panic!(
                                "These density functions should not be a part of the MultiNoiseSampler! We probably need to re-write code"
                            );
                        }
                    }
                }
            };
            component_stack.push(chunk_component);
        }

        Self {
            temperature: base.temperature,
            vegetation: base.vegetation,
            continents: base.continents,
            depth: base.depth,
            erosion: base.erosion,
            ridges: base.ridges,
            component_stack: component_stack.into_boxed_slice(),
        }
    }
}
