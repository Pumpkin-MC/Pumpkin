use std::collections::HashMap;

use pumpkin_data::noise_router::WrapperType;
use pumpkin_util::math::vector2::Vector2;

use crate::generation::{biome_coords, positions::chunk_pos};

use super::{
    chunk_density_function::{
        Cache2D, ChunkNoiseFunctionSampleOptions, ChunkSpecificNoiseFunctionComponent, FlatCache,
        SampleAction,
    },
    chunk_noise_router::ChunkNoiseFunctionComponent,
    density_function::{NoiseFunctionComponentRange, PassThrough, UnblendedNoisePos},
    proto_noise_router::{ProtoNoiseFunctionComponent, ProtoSurfaceEstimator},
};

pub struct SurfaceHeightSamplerBuilderOptions {
    // The biome coords of this chunk
    start_biome_x: i32,
    start_biome_z: i32,

    // Number of biome regions per chunk per axis
    horizontal_biome_end: usize,

    // Minimum y level to check
    minimum_y: i32,
    // Maximum y level to check
    maximum_y: i32,
    y_level_step_count: usize,
}

impl SurfaceHeightSamplerBuilderOptions {
    pub fn new(
        start_biome_x: i32,
        start_biome_z: i32,
        horizontal_biome_end: usize,
        minimum_y: i32,
        maximum_y: i32,
        y_level_step_count: usize,
    ) -> Self {
        Self {
            start_biome_x,
            start_biome_z,
            horizontal_biome_end,
            minimum_y,
            maximum_y,
            y_level_step_count,
        }
    }
}

pub struct SurfaceHeightEstimateSampler<'a> {
    minimum_y: i32,
    maximum_y: i32,
    y_level_step_count: usize,

    component_stack: Box<[ChunkNoiseFunctionComponent<'a>]>,

    // TODO: Can this be a flat map? I think the aquifer sampler samples outside of the chunk
    cache: HashMap<u64, i32>,
}

impl<'a> SurfaceHeightEstimateSampler<'a> {
    const NOTCHIAN_SAMPLE_CUTOFF: f64 = 0.390625;

    pub fn estimate_height(&mut self, block_x: i32, block_z: i32) -> i32 {
        let biome_aligned_x = biome_coords::to_block(biome_coords::from_block(block_x));
        let biome_aligned_z = biome_coords::to_block(biome_coords::from_block(block_z));

        let packed_column = chunk_pos::packed(&Vector2::new(biome_aligned_x, biome_aligned_z));
        if let Some(estimate) = self.cache.get(&packed_column) {
            *estimate
        } else {
            let estimate = self.calculate_height_estimate(biome_aligned_x, biome_aligned_z);
            self.cache.insert(packed_column, estimate);
            estimate
        }
    }

    fn calculate_height_estimate(&mut self, aligned_x: i32, aligned_z: i32) -> i32 {
        for y in (self.minimum_y..=self.maximum_y)
            .rev()
            .step_by(self.y_level_step_count)
        {
            let pos = UnblendedNoisePos::new(aligned_x, y, aligned_z);
            let density_sample = ChunkNoiseFunctionComponent::sample_from_stack(
                &mut self.component_stack,
                &pos,
                &ChunkNoiseFunctionSampleOptions::new(false, SampleAction::SkipCellCaches, 0, 0, 0),
            );

            if density_sample > Self::NOTCHIAN_SAMPLE_CUTOFF {
                return y;
            }
        }

        i32::MAX
    }

    pub fn generate(
        base: &'a ProtoSurfaceEstimator,
        build_options: &SurfaceHeightSamplerBuilderOptions,
    ) -> Self {
        // TODO: It seems kind of wasteful to iter over all components (even those we dont need
        // because they're for chunk population), but this is the best I've got for now.
        // (Should we traverse the functions and update the indices?)
        let mut component_stack =
            Vec::<ChunkNoiseFunctionComponent>::with_capacity(base.full_component_stack.len());
        for base_component in base.full_component_stack.iter() {
            let chunk_component = match base_component {
                ProtoNoiseFunctionComponent::Dependent(dependent) => {
                    ChunkNoiseFunctionComponent::Dependent(dependent)
                }
                ProtoNoiseFunctionComponent::Independent(independent) => {
                    ChunkNoiseFunctionComponent::Independent(independent)
                }
                ProtoNoiseFunctionComponent::PassThrough(pass_through) => {
                    ChunkNoiseFunctionComponent::PassThrough(pass_through.clone())
                }
                ProtoNoiseFunctionComponent::Wrapper(wrapper) => {
                    //NOTE: Due to our previous invariant with the proto-function, it is guaranteed
                    // that the wrapped function is already on the stack
                    let min_value = component_stack[wrapper.input_index()].min();
                    let max_value = component_stack[wrapper.input_index()].max();

                    match wrapper.wrapper_type() {
                        WrapperType::Cache2D => ChunkNoiseFunctionComponent::Chunk(Box::new(
                            ChunkSpecificNoiseFunctionComponent::Cache2D(Cache2D::new(
                                wrapper.input_index(),
                                min_value,
                                max_value,
                            )),
                        )),
                        WrapperType::CacheFlat => {
                            let mut flat_cache = FlatCache::new(
                                wrapper.input_index(),
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
                                        &mut component_stack[..=wrapper.input_index()],
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
                        // Java passes thru if the noise pos is not the chunk itself, which it is
                        // never for the Height estimator
                        _ => ChunkNoiseFunctionComponent::PassThrough(PassThrough::new(
                            wrapper.input_index(),
                            min_value,
                            max_value,
                        )),
                    }
                }
            };
            component_stack.push(chunk_component);
        }

        Self {
            component_stack: component_stack.into_boxed_slice(),

            maximum_y: build_options.maximum_y,
            minimum_y: build_options.minimum_y,
            y_level_step_count: build_options.y_level_step_count,

            cache: HashMap::new(),
        }
    }
}
