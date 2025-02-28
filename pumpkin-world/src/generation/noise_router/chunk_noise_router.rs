use enum_dispatch::enum_dispatch;

use crate::{generation::biome_coords, noise_router::density_function_ast::WrapperType};

use super::{
    chunk_density_function::{
        Cache2D, CacheOnce, CellCache, ChunkNoiseFunctionBuilderOptions,
        ChunkNoiseFunctionSampleOptions, ChunkSpecificNoiseFunctionComponent, DensityInterpolator,
        FlatCache, SampleAction,
    },
    density_function::{
        IndexToNoisePos, NoiseFunctionComponentRange, NoisePos, PassThrough,
        StaticIndependentChunkNoiseFunctionComponentImpl, UnblendedNoisePos,
    },
    proto_noise_router::{
        DependentProtoNoiseFunctionComponent, GlobalProtoNoiseRouter,
        IndependentProtoNoiseFunctionComponent, ProtoNoiseFunctionComponent,
    },
};

#[enum_dispatch]
pub trait StaticChunkNoiseFunctionComponentImpl {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &impl NoisePos,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64;

    fn fill(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        array: &mut [f64],
        mapper: &impl IndexToNoisePos,
        sample_options: &mut ChunkNoiseFunctionSampleOptions,
    ) {
        array.iter_mut().enumerate().for_each(|(index, value)| {
            let pos = mapper.at(index, Some(sample_options));
            *value = self.sample(component_stack, &pos, sample_options);
        });
    }
}

#[enum_dispatch]
pub trait MutableChunkNoiseFunctionComponentImpl {
    fn sample(
        &mut self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &impl NoisePos,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64;

    fn fill(
        &mut self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        array: &mut [f64],
        mapper: &impl IndexToNoisePos,
        sample_options: &mut ChunkNoiseFunctionSampleOptions,
    ) {
        array.iter_mut().enumerate().for_each(|(index, value)| {
            let pos = mapper.at(index, Some(sample_options));
            *value = self.sample(component_stack, &pos, sample_options);
        });
    }
}

pub enum ChunkNoiseFunctionComponent<'a> {
    Independent(&'a IndependentProtoNoiseFunctionComponent),
    Dependent(&'a DependentProtoNoiseFunctionComponent),
    Chunk(Box<ChunkSpecificNoiseFunctionComponent>),
    PassThrough(PassThrough),
}

impl NoiseFunctionComponentRange for ChunkNoiseFunctionComponent<'_> {
    #[inline]
    fn min(&self) -> f64 {
        match self {
            Self::Independent(independent) => independent.min(),
            Self::Dependent(dependent) => dependent.min(),
            Self::Chunk(chunk) => chunk.min(),
            Self::PassThrough(pass_through) => pass_through.min(),
        }
    }

    #[inline]
    fn max(&self) -> f64 {
        match self {
            Self::Independent(independent) => independent.max(),
            Self::Dependent(dependent) => dependent.max(),
            Self::Chunk(chunk) => chunk.max(),
            Self::PassThrough(pass_through) => pass_through.max(),
        }
    }
}

impl MutableChunkNoiseFunctionComponentImpl for ChunkNoiseFunctionComponent<'_> {
    #[inline]
    fn sample(
        &mut self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &impl NoisePos,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        match self {
            Self::Independent(independent) => independent.sample(pos),
            Self::Dependent(dependent) => dependent.sample(component_stack, pos, sample_options),
            Self::Chunk(chunk) => chunk.sample(component_stack, pos, sample_options),
            Self::PassThrough(pass_through) => ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=pass_through.input_index],
                pos,
                sample_options,
            ),
        }
    }

    #[inline]
    fn fill(
        &mut self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        array: &mut [f64],
        mapper: &impl IndexToNoisePos,
        sample_options: &mut ChunkNoiseFunctionSampleOptions,
    ) {
        match self {
            Self::Independent(independent) => independent.fill(array, mapper),
            Self::Dependent(dependent) => {
                dependent.fill(component_stack, array, mapper, sample_options)
            }
            Self::Chunk(chunk) => chunk.fill(component_stack, array, mapper, sample_options),
            Self::PassThrough(pass_through) => ChunkNoiseFunctionComponent::fill_from_stack(
                &mut component_stack[..=pass_through.input_index],
                array,
                mapper,
                sample_options,
            ),
        }
    }
}

impl ChunkNoiseFunctionComponent<'_> {
    pub fn sample_from_stack(
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &impl NoisePos,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        let (top_component, component_stack) = component_stack.split_last_mut().unwrap();
        top_component.sample(component_stack, pos, sample_options)
    }

    pub fn fill_from_stack(
        component_stack: &mut [ChunkNoiseFunctionComponent],
        array: &mut [f64],
        mapper: &impl IndexToNoisePos,
        sample_options: &mut ChunkNoiseFunctionSampleOptions,
    ) {
        let (top_component, component_stack) = component_stack.split_last_mut().unwrap();
        top_component.fill(component_stack, array, mapper, sample_options);
    }
}

pub struct ChunkNoiseDensityFunction<'a> {
    pub(crate) component_stack: &'a mut [ChunkNoiseFunctionComponent<'a>],
}

impl ChunkNoiseDensityFunction<'_> {
    #[inline]
    pub fn sample(
        &mut self,
        pos: &impl NoisePos,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        ChunkNoiseFunctionComponent::sample_from_stack(self.component_stack, pos, sample_options)
    }

    #[inline]
    fn fill(
        &mut self,
        array: &mut [f64],
        mapper: &impl IndexToNoisePos,
        sample_options: &mut ChunkNoiseFunctionSampleOptions,
    ) {
        ChunkNoiseFunctionComponent::fill_from_stack(
            self.component_stack,
            array,
            mapper,
            sample_options,
        );
    }
}

macro_rules! sample_function {
    ($name:ident) => {
        #[inline]
        pub fn $name(
            &mut self,
            pos: &impl NoisePos,
            sample_options: &ChunkNoiseFunctionSampleOptions,
        ) -> f64 {
            ChunkNoiseFunctionComponent::sample_from_stack(
                &mut self.component_stack[..=self.$name],
                pos,
                sample_options,
            )
        }
    };
}

pub struct ChunkNoiseRouter<'a> {
    barrier_noise: usize,
    fluid_level_floodedness_noise: usize,
    fluid_level_spread_noise: usize,
    lava_noise: usize,
    erosion: usize,
    depth: usize,
    initial_density_without_jaggedness: usize,
    final_density: usize,
    vein_toggle: usize,
    vein_ridged: usize,
    vein_gap: usize,
    component_stack: Box<[ChunkNoiseFunctionComponent<'a>]>,
    interpolator_indices: Box<[usize]>,
    cell_indices: Box<[usize]>,
}

impl ChunkNoiseRouter<'_> {
    sample_function!(barrier_noise);
    sample_function!(fluid_level_floodedness_noise);
    sample_function!(fluid_level_spread_noise);
    sample_function!(lava_noise);
    sample_function!(erosion);
    sample_function!(depth);
    sample_function!(initial_density_without_jaggedness);
    sample_function!(final_density);
    sample_function!(vein_toggle);
    sample_function!(vein_ridged);
    sample_function!(vein_gap);
}

impl<'a> ChunkNoiseRouter<'a> {
    pub fn generate(
        base: &'a GlobalProtoNoiseRouter,
        build_options: &ChunkNoiseFunctionBuilderOptions,
    ) -> Self {
        let mut component_stack =
            Vec::<ChunkNoiseFunctionComponent>::with_capacity(base.component_stack.len());
        let mut cell_cache_indices = Vec::new();
        let mut interpolator_indices = Vec::new();

        // NOTE: Only iter what we need; we dont care about the MultiNoiseSampler functions that are
        // pushed after due to our invariant
        let max_index = [
            base.barrier_noise,
            base.fluid_level_floodedness_noise,
            base.fluid_level_spread_noise,
            base.lava_noise,
            base.erosion,
            base.depth,
            base.initial_density_without_jaggedness,
            base.final_density,
            base.vein_toggle,
            base.vein_gap,
            base.vein_ridged,
        ]
        .into_iter()
        .max()
        .unwrap();

        for (component_index, base_component) in
            base.component_stack[..=max_index].iter().enumerate()
        {
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
                        WrapperType::Interpolated => {
                            interpolator_indices.push(component_index);
                            ChunkNoiseFunctionComponent::Chunk(Box::new(
                                ChunkSpecificNoiseFunctionComponent::DensityInterpolator(
                                    DensityInterpolator::new(
                                        wrapper.input_index,
                                        min_value,
                                        max_value,
                                        build_options,
                                    ),
                                ),
                            ))
                        }
                        WrapperType::CellCache => {
                            cell_cache_indices.push(component_index);
                            ChunkNoiseFunctionComponent::Chunk(Box::new(
                                ChunkSpecificNoiseFunctionComponent::CellCache(CellCache::new(
                                    wrapper.input_index,
                                    min_value,
                                    max_value,
                                    build_options,
                                )),
                            ))
                        }
                        WrapperType::CacheOnce => ChunkNoiseFunctionComponent::Chunk(Box::new(
                            ChunkSpecificNoiseFunctionComponent::CacheOnce(CacheOnce::new(
                                wrapper.input_index,
                                min_value,
                                max_value,
                            )),
                        )),
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
                    }
                }
            };
            component_stack.push(chunk_component);
        }

        Self {
            barrier_noise: base.barrier_noise,
            fluid_level_floodedness_noise: base.fluid_level_floodedness_noise,
            fluid_level_spread_noise: base.fluid_level_spread_noise,
            lava_noise: base.lava_noise,
            erosion: base.erosion,
            depth: base.depth,
            initial_density_without_jaggedness: base.initial_density_without_jaggedness,
            final_density: base.final_density,
            vein_toggle: base.vein_toggle,
            vein_ridged: base.vein_ridged,
            vein_gap: base.vein_gap,
            component_stack: component_stack.into_boxed_slice(),
            interpolator_indices: interpolator_indices.into_boxed_slice(),
            cell_indices: cell_cache_indices.into_boxed_slice(),
        }
    }

    pub fn fill_cell_caches(
        &mut self,
        mapper: &impl IndexToNoisePos,
        sample_options: &mut ChunkNoiseFunctionSampleOptions,
    ) {
        let indices = &self.cell_indices;
        let components = &mut self.component_stack;
        for cell_cache_index in indices {
            let (component_stack, component) = components.split_at_mut(*cell_cache_index);

            let ChunkNoiseFunctionComponent::Chunk(chunk) = component.first_mut().unwrap() else {
                unreachable!();
            };
            let ChunkSpecificNoiseFunctionComponent::CellCache(cell_cache) = chunk.as_mut() else {
                unreachable!();
            };

            ChunkNoiseFunctionComponent::fill_from_stack(
                &mut component_stack[..=cell_cache.input_index],
                &mut cell_cache.cache,
                mapper,
                sample_options,
            );
        }
    }

    pub fn fill_interpolator_buffers(
        &mut self,
        start: bool,
        cell_z: usize,
        mapper: &impl IndexToNoisePos,
        sample_options: &mut ChunkNoiseFunctionSampleOptions,
    ) {
        let indices = &self.interpolator_indices;
        let components = &mut self.component_stack;
        for interpolator_index in indices {
            let (component_stack, component) = components.split_at_mut(*interpolator_index);

            let ChunkNoiseFunctionComponent::Chunk(chunk) = component.first_mut().unwrap() else {
                unreachable!();
            };
            let ChunkSpecificNoiseFunctionComponent::DensityInterpolator(density_interpolator) =
                chunk.as_mut()
            else {
                unreachable!();
            };

            let start_index = density_interpolator.yz_to_buf_index(0, cell_z);
            let buf = if start {
                &mut density_interpolator.start_buffer
                    [start_index..=start_index + density_interpolator.vertical_cell_count]
            } else {
                &mut density_interpolator.end_buffer
                    [start_index..=start_index + density_interpolator.vertical_cell_count]
            };

            ChunkNoiseFunctionComponent::fill_from_stack(
                &mut component_stack[..=density_interpolator.input_index],
                buf,
                mapper,
                sample_options,
            );
        }
    }

    pub fn interpolate_x(&mut self, delta: f64) {
        let indices = &self.interpolator_indices;
        let components = &mut self.component_stack;
        for interpolator_index in indices {
            let ChunkNoiseFunctionComponent::Chunk(chunk) = &mut components[*interpolator_index]
            else {
                unreachable!();
            };
            let ChunkSpecificNoiseFunctionComponent::DensityInterpolator(density_interpolator) =
                chunk.as_mut()
            else {
                unreachable!();
            };

            density_interpolator.interpolate_x(delta);
        }
    }

    pub fn interpolate_y(&mut self, delta: f64) {
        let indices = &self.interpolator_indices;
        let components = &mut self.component_stack;
        for interpolator_index in indices {
            let ChunkNoiseFunctionComponent::Chunk(chunk) = &mut components[*interpolator_index]
            else {
                unreachable!();
            };
            let ChunkSpecificNoiseFunctionComponent::DensityInterpolator(density_interpolator) =
                chunk.as_mut()
            else {
                unreachable!();
            };

            density_interpolator.interpolate_y(delta);
        }
    }

    pub fn interpolate_z(&mut self, delta: f64) {
        let indices = &self.interpolator_indices;
        let components = &mut self.component_stack;
        for interpolator_index in indices {
            let ChunkNoiseFunctionComponent::Chunk(chunk) = &mut components[*interpolator_index]
            else {
                unreachable!();
            };
            let ChunkSpecificNoiseFunctionComponent::DensityInterpolator(density_interpolator) =
                chunk.as_mut()
            else {
                unreachable!();
            };

            density_interpolator.interpolate_z(delta);
        }
    }

    pub fn on_sampled_cell_corners(&mut self, cell_y_position: usize, cell_z_position: usize) {
        let indices = &self.interpolator_indices;
        let components = &mut self.component_stack;
        for interpolator_index in indices {
            let ChunkNoiseFunctionComponent::Chunk(chunk) = &mut components[*interpolator_index]
            else {
                unreachable!();
            };
            let ChunkSpecificNoiseFunctionComponent::DensityInterpolator(density_interpolator) =
                chunk.as_mut()
            else {
                unreachable!();
            };

            density_interpolator.on_sampled_cell_corners(cell_y_position, cell_z_position);
        }
    }

    pub fn swap_buffers(&mut self) {
        let indices = &self.interpolator_indices;
        let components = &mut self.component_stack;
        for interpolator_index in indices {
            let ChunkNoiseFunctionComponent::Chunk(chunk) = &mut components[*interpolator_index]
            else {
                unreachable!();
            };
            let ChunkSpecificNoiseFunctionComponent::DensityInterpolator(density_interpolator) =
                chunk.as_mut()
            else {
                unreachable!();
            };

            density_interpolator.swap_buffers();
        }
    }
}
