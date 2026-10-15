use pumpkin_data::noise_router::WrapperType;
use pumpkin_util::math::vector3::Vector3;

use super::{
    chunk_density_function::{
        Cache, ChunkNoiseFunctionBuilderOptions, ChunkSpecificNoiseFunctionComponent, Interpolated,
    },
    density_function::{
        NoiseFunctionComponentRange, PassThrough, StaticIndependentChunkNoiseFunctionComponentImpl,
        beardifier::Beardifier,
    },
    density_volume::DensityVolume,
    proto_noise_router::{
        DependentProtoNoiseFunctionComponent, IndependentProtoNoiseFunctionComponent,
        ProtoNoiseFunctionComponent, ProtoNoiseRouter,
    },
};

pub trait StaticChunkNoiseFunctionComponentImpl {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
    ) -> f32;

    fn sample_volume(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        buffer: &mut [f32],
        volume: &DensityVolume,
    ) {
        volume.fill_with(buffer, |pos| self.sample(component_stack, pos));
    }
}

pub trait MutableChunkNoiseFunctionComponentImpl {
    fn sample(
        &mut self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
    ) -> f32;

    fn sample_volume(
        &mut self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        buffer: &mut [f32],
        volume: &DensityVolume,
    ) {
        volume.fill_with(buffer, |pos| self.sample(component_stack, pos));
    }
}

pub enum ChunkNoiseFunctionComponent<'a> {
    Independent(&'a IndependentProtoNoiseFunctionComponent),
    Dependent(&'a DependentProtoNoiseFunctionComponent),
    Chunk(ChunkSpecificNoiseFunctionComponent),
    PassThrough(PassThrough),
}

impl NoiseFunctionComponentRange for ChunkNoiseFunctionComponent<'_> {
    #[inline]
    fn min(&self) -> f32 {
        match self {
            Self::Independent(independent) => independent.min(),
            Self::Dependent(dependent) => dependent.min(),
            Self::Chunk(chunk) => chunk.min(),
            Self::PassThrough(pass_through) => pass_through.min(),
        }
    }

    #[inline]
    fn max(&self) -> f32 {
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
        pos: &Vector3<i32>,
    ) -> f32 {
        match self {
            Self::Independent(independent) => independent.sample(pos),
            Self::Dependent(dependent) => dependent.sample(component_stack, pos),
            Self::Chunk(chunk) => chunk.sample(component_stack, pos),
            Self::PassThrough(pass_through) => ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=pass_through.input_index()],
                pos,
            ),
        }
    }

    #[inline]
    fn sample_volume(
        &mut self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        buffer: &mut [f32],
        volume: &DensityVolume,
    ) {
        match self {
            Self::Independent(independent) => independent.sample_volume(buffer, volume),
            Self::Dependent(dependent) => dependent.sample_volume(component_stack, buffer, volume),
            Self::Chunk(chunk) => chunk.sample_volume(component_stack, buffer, volume),
            Self::PassThrough(pass_through) => {
                ChunkNoiseFunctionComponent::sample_volume_from_stack(
                    &mut component_stack[..=pass_through.input_index()],
                    buffer,
                    volume,
                );
            }
        }
    }
}

impl ChunkNoiseFunctionComponent<'_> {
    #[inline]
    pub fn sample_from_stack(
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
    ) -> f32 {
        let Some((top_component, component_stack)) = component_stack.split_last_mut() else {
            return 0.0;
        };
        top_component.sample(component_stack, pos)
    }

    pub fn sample_volume_from_stack(
        component_stack: &mut [ChunkNoiseFunctionComponent],
        buffer: &mut [f32],
        volume: &DensityVolume,
    ) {
        if let Some((top_component, component_stack)) = component_stack.split_last_mut() {
            top_component.sample_volume(component_stack, buffer, volume);
        }
    }
}

pub struct ChunkNoiseDensityFunction<'a> {
    pub(crate) component_stack: &'a mut [ChunkNoiseFunctionComponent<'a>],
}

impl ChunkNoiseDensityFunction<'_> {
    #[inline]
    pub fn sample(&mut self, pos: &Vector3<i32>) -> f32 {
        ChunkNoiseFunctionComponent::sample_from_stack(self.component_stack, pos)
    }

    #[inline]
    pub fn sample_volume(&mut self, buffer: &mut [f32], volume: &DensityVolume) {
        ChunkNoiseFunctionComponent::sample_volume_from_stack(self.component_stack, buffer, volume);
    }
}

macro_rules! sample_function {
    ($name:ident, $volume_name:ident) => {
        #[inline]
        pub fn $name(&mut self, pos: &Vector3<i32>) -> f32 {
            ChunkNoiseFunctionComponent::sample_from_stack(
                &mut self.component_stack[..=self.$name],
                pos,
            )
        }

        #[inline]
        pub fn $volume_name(&mut self, buffer: &mut [f32], volume: &DensityVolume) {
            ChunkNoiseFunctionComponent::sample_volume_from_stack(
                &mut self.component_stack[..=self.$name],
                buffer,
                volume,
            );
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
    final_density: usize,
    vein_toggle: usize,
    vein_ridged: usize,
    vein_gap: usize,
    component_stack: Box<[ChunkNoiseFunctionComponent<'a>]>,
}

impl ChunkNoiseRouter<'_> {
    sample_function!(barrier_noise, barrier_noise_volume);
    sample_function!(
        fluid_level_floodedness_noise,
        fluid_level_floodedness_noise_volume
    );
    sample_function!(fluid_level_spread_noise, fluid_level_spread_noise_volume);
    sample_function!(lava_noise, lava_noise_volume);
    sample_function!(erosion, erosion_volume);
    sample_function!(depth, depth_volume);
    sample_function!(final_density, final_density_volume);
    sample_function!(vein_toggle, vein_toggle_volume);
    sample_function!(vein_ridged, vein_ridged_volume);
    sample_function!(vein_gap, vein_gap_volume);
}

impl<'a> ChunkNoiseRouter<'a> {
    #[must_use]
    pub fn component_stack_mut(&mut self) -> &mut [ChunkNoiseFunctionComponent<'a>] {
        &mut self.component_stack
    }

    #[must_use]
    pub fn generate(
        base: &'a ProtoNoiseRouter,
        build_options: &ChunkNoiseFunctionBuilderOptions,
    ) -> Self {
        let mut component_stack =
            Vec::<ChunkNoiseFunctionComponent>::with_capacity(base.full_component_stack.len());

        for base_component in &base.full_component_stack {
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
                ProtoNoiseFunctionComponent::Beardifier(_) => ChunkNoiseFunctionComponent::Chunk(
                    ChunkSpecificNoiseFunctionComponent::Beardifier(Beardifier::new(
                        build_options.beardifier_structures.clone(),
                        build_options.beardifier_junctions.clone(),
                        build_options.affected_box,
                    )),
                ),
                ProtoNoiseFunctionComponent::Wrapper(wrapper) => {
                    let min_value = component_stack[wrapper.input_index].min();
                    let max_value = component_stack[wrapper.input_index].max();

                    match wrapper.wrapper_type {
                        WrapperType::Interpolated {
                            cell_size_xz,
                            cell_size_y,
                        } => ChunkNoiseFunctionComponent::Chunk(
                            ChunkSpecificNoiseFunctionComponent::Interpolated(Interpolated::new(
                                wrapper.input_index,
                                cell_size_xz,
                                cell_size_y,
                                min_value,
                                max_value,
                            )),
                        ),
                        WrapperType::Cache => ChunkNoiseFunctionComponent::Chunk(
                            ChunkSpecificNoiseFunctionComponent::Cache(Cache::new(
                                wrapper.input_index,
                                min_value,
                                max_value,
                            )),
                        ),
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
            final_density: base.final_density,
            vein_toggle: base.vein_toggle,
            vein_ridged: base.vein_ridged,
            vein_gap: base.vein_gap,
            component_stack: component_stack.into_boxed_slice(),
        }
    }
}
