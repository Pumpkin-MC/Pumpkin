use std::{collections::HashMap, hash::Hash, mem, num::Wrapping, ops::AddAssign, sync::Arc};

use lazy_static::lazy_static;
use num_traits::Zero;
use parking_lot::Mutex;
use pumpkin_core::math::{floor_div, vector2::Vector2, vector3::Vector3};

use crate::{
    block::BlockState,
    match_ref_implementations,
    world_gen::{
        noise::{density::basic::WrapperType, lerp3},
        section_coords,
    },
};

use super::{
    biome_coords,
    blender::Blender,
    generation_shapes::GenerationShape,
    noise::{
        config::NoiseConfig,
        density::{
            basic::{BeardifierFunction, WrapperFunction},
            component_functions::{
                ApplierImpl, ComponentFunctionImpl, ComponentReference,
                ComponentReferenceImplementation, ConversionResultPre, ConverterEnvironment,
                ConverterImpl, DensityFunctionEnvironment, EnvironmentApplierImpl,
                ImmutableComponentFunctionImpl, MutableComponentFunctionImpl,
                MutableComponentReference, OwnedConverterEnvironment, SharedComponentReference,
            },
            noise::InternalNoise,
            NoisePos, NoisePosImpl, UnblendedNoisePos,
        },
        lerp,
    },
    positions::chunk_pos,
    sampler::{
        AquiferSampler, BlockStateSampler, ChainedBlockStateSampler, FluidLevelSampler,
        OreVeinSampler, SeaLevelAquiferSampler, WorldAquiferSampler,
    },
};

lazy_static! {
    pub static ref STONE_BLOCK: BlockState = BlockState::new("minecraft:stone").unwrap();
    pub static ref LAVA_BLOCK: BlockState = BlockState::new("minecraft:lava").unwrap();
    pub static ref WATER_BLOCK: BlockState = BlockState::new("minecraft:water").unwrap();
}

pub struct ChunkCacheOnceFunction<R: ComponentReference<ChunkNoiseState>> {
    delegate: R,
    sample_unique_index: u64,
    cache_once_unique_index: u64,
    last_sample_result: f64,
    cache: Option<Box<[f64]>>,
}

impl<R: ComponentReference<ChunkNoiseState>> ChunkCacheOnceFunction<R> {
    fn new(delegate: R) -> Self {
        Self {
            delegate,
            sample_unique_index: 0,
            cache_once_unique_index: 0,
            last_sample_result: 0f64,
            cache: None,
        }
    }
}

impl<R: ComponentReference<ChunkNoiseState>> ComponentFunctionImpl for ChunkCacheOnceFunction<R> {}

impl<R: ComponentReference<ChunkNoiseState>> MutableComponentFunctionImpl<ChunkNoiseState>
    for ChunkCacheOnceFunction<R>
{
    fn sample_mut(&mut self, pos: &NoisePos, env: &ChunkNoiseState) -> f64 {
        if let NoisePos::Chunk(_chunk_pos) = pos {
            if let Some(cache) = &mut self.cache {
                if self.cache_once_unique_index == env.cache_once_unique_index.0 {
                    return cache[env.index];
                }
            }

            if self.sample_unique_index == env.sample_unique_index.0 {
                return self.last_sample_result;
            }

            self.sample_unique_index = env.sample_unique_index.0;
            self.last_sample_result = self.delegate.sample_mut(pos, env);
            return self.last_sample_result;
        }
        self.delegate.sample_mut(pos, env)
    }

    fn fill_mut(
        &mut self,
        densities: &mut [f64],
        applier: &mut dyn EnvironmentApplierImpl<Env = ChunkNoiseState>,
    ) {
        if let Some(cache) = &mut self.cache {
            let env = applier.env();
            if self.cache_once_unique_index == env.cache_once_unique_index.0 {
                densities.iter_mut().enumerate().for_each(|(index, val)| {
                    *val = cache[index];
                });
                return;
            }
        }

        self.delegate.fill_mut(densities, applier);

        let env = applier.env();
        self.cache_once_unique_index = env.cache_once_unique_index.0;

        if let Some(cache) = &mut self.cache {
            if densities.len() == cache.len() {
                cache.iter_mut().enumerate().for_each(|(index, val)| {
                    *val = densities[index];
                });
                return;
            }
        }

        self.cache = Some(densities.to_vec().into());
    }

    fn environment(&self) -> ConverterEnvironment<ChunkNoiseState> {
        ConverterEnvironment::ChunkNoise
    }

    fn into_environment(self: Box<Self>) -> OwnedConverterEnvironment<ChunkNoiseState> {
        unreachable!()
    }

    fn convert(
        self: Box<Self>,
        _converter: &mut dyn ConverterImpl<ChunkNoiseState>,
    ) -> ComponentReferenceImplementation<ChunkNoiseState> {
        unreachable!()
    }

    fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<ChunkNoiseState> {
        match_ref_implementations!(
            (delegate, self.delegate.clone_to_new_ref());
            {
                MutableComponentReference(Box::new(
                    ChunkCacheOnceFunction {
                        delegate,
                        sample_unique_index: self.sample_unique_index,
                        cache_once_unique_index: self.cache_once_unique_index,
                        last_sample_result: self.last_sample_result,
                        cache: self.cache.clone(),
                    }
                )).into()
            }
        )
    }
}

pub struct ChunkFlatCacheFunction<R: ComponentReference<ChunkNoiseState>> {
    delegate: R,
    cache: Box<[f64]>,
}

impl<R: ComponentReference<ChunkNoiseState>> ChunkFlatCacheFunction<R> {
    fn new(delegate: R, sample: bool, state: &ChunkNoiseState) -> Self {
        let mut cache = vec![
            0f64;
            (state.horizontal_biome_end as usize + 1)
                * (state.horizontal_biome_end as usize + 1)
        ];
        let mut delegate = delegate;

        if sample {
            for biome_x in 0..=state.horizontal_biome_end {
                let true_biome_x = state.start_biome_pos.x + biome_x as i32;
                let block_x = biome_coords::to_block(true_biome_x);

                for biome_z in 0..=state.horizontal_biome_end {
                    let true_biome_z = state.start_biome_pos.z + biome_z as i32;
                    let block_z = biome_coords::to_block(true_biome_z);

                    let index =
                        Self::xz_to_index_const(state.horizontal_biome_end, biome_x, biome_z);

                    cache[index] = delegate.sample_mut(
                        &NoisePos::Unblended(UnblendedNoisePos::new(block_x, 0, block_z)),
                        state,
                    );
                }
            }
        }

        Self {
            delegate,
            cache: cache.into(),
        }
    }

    const fn xz_to_index_const(horizontal_biome_end: u8, x: u8, z: u8) -> usize {
        x as usize * (horizontal_biome_end as usize + 1) + z as usize
    }
}

impl<R: ComponentReference<ChunkNoiseState>> ComponentFunctionImpl for ChunkFlatCacheFunction<R> {}

impl<R: ComponentReference<ChunkNoiseState>> MutableComponentFunctionImpl<ChunkNoiseState>
    for ChunkFlatCacheFunction<R>
{
    fn sample_mut(&mut self, pos: &NoisePos, env: &ChunkNoiseState) -> f64 {
        let biome_x = biome_coords::from_block(pos.x());
        let biome_z = biome_coords::from_block(pos.z());

        let rel_biome_x = biome_x - env.start_biome_pos.x;
        let rel_biome_z = biome_z - env.start_biome_pos.z;

        if rel_biome_x >= 0
            && rel_biome_z >= 0
            && rel_biome_x <= env.horizontal_biome_end as i32
            && rel_biome_z <= env.horizontal_biome_end as i32
        {
            let index = Self::xz_to_index_const(
                env.horizontal_biome_end,
                rel_biome_x as u8,
                rel_biome_z as u8,
            );
            self.cache[index]
        } else {
            self.delegate.sample_mut(pos, env)
        }
    }

    fn fill_mut(
        &mut self,
        arr: &mut [f64],
        applier: &mut dyn EnvironmentApplierImpl<Env = ChunkNoiseState>,
    ) {
        applier.fill_mut(arr, self);
    }

    fn environment(&self) -> ConverterEnvironment<ChunkNoiseState> {
        ConverterEnvironment::ChunkNoise
    }

    fn into_environment(self: Box<Self>) -> OwnedConverterEnvironment<ChunkNoiseState> {
        unreachable!()
    }

    fn convert(
        self: Box<Self>,
        _converter: &mut dyn ConverterImpl<ChunkNoiseState>,
    ) -> ComponentReferenceImplementation<ChunkNoiseState> {
        unreachable!()
    }

    fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<ChunkNoiseState> {
        match_ref_implementations!(
            (delegate, self.delegate.clone_to_new_ref());
            {
                MutableComponentReference(Box::new(
                   ChunkFlatCacheFunction {
                    delegate,
                    cache: self.cache.clone(),
                }
                )).into()
            }
        )
    }
}

pub struct ChunkCellCacheFunction {
    delegate: Box<dyn ComponentReference<ChunkNoiseState>>,
    cache: Box<[f64]>,
}

impl ChunkCellCacheFunction {
    fn new(
        delegate: Box<dyn ComponentReference<ChunkNoiseState>>,
        state: &ChunkNoiseState,
    ) -> Self {
        Self {
            delegate,
            cache: Self::create_cache(state).into(),
        }
    }

    fn create_cache(state: &ChunkNoiseState) -> Vec<f64> {
        vec![
            0f64;
            state.horizontal_cell_block_count as usize
                * state.horizontal_cell_block_count as usize
                * state.vertical_cell_block_count as usize
        ]
    }
}

impl ComponentFunctionImpl for ChunkCellCacheFunction {}

impl MutableComponentFunctionImpl<ChunkNoiseState> for ChunkCellCacheFunction {
    fn sample_mut(&mut self, pos: &NoisePos, env: &ChunkNoiseState) -> f64 {
        if let NoisePos::Chunk(_chunk_pos) = pos {
            #[cfg(debug_assertions)]
            assert!(env.is_interpolating);

            let cell_block_x = env.cell_block_pos.x;
            let cell_block_y = env.cell_block_pos.y;
            let cell_block_z = env.cell_block_pos.z;

            if cell_block_x < env.horizontal_cell_block_count
                && cell_block_y < env.vertical_cell_block_count
                && cell_block_z < env.horizontal_cell_block_count
            {
                return self.cache[((env.vertical_cell_block_count as usize
                    - 1
                    - cell_block_y as usize)
                    * env.horizontal_cell_block_count as usize
                    + cell_block_x as usize)
                    * env.horizontal_cell_block_count as usize
                    + cell_block_z as usize];
            }
        }
        self.delegate.sample_mut(pos, env)
    }

    fn fill_mut(
        &mut self,
        arr: &mut [f64],
        applier: &mut dyn EnvironmentApplierImpl<Env = ChunkNoiseState>,
    ) {
        applier.fill_mut(arr, self);
    }

    fn environment(&self) -> ConverterEnvironment<ChunkNoiseState> {
        ConverterEnvironment::ChunkNoise
    }

    fn into_environment(self: Box<Self>) -> OwnedConverterEnvironment<ChunkNoiseState> {
        unreachable!()
    }

    fn convert(
        self: Box<Self>,
        _converter: &mut dyn ConverterImpl<ChunkNoiseState>,
    ) -> ComponentReferenceImplementation<ChunkNoiseState> {
        unreachable!()
    }

    fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<ChunkNoiseState> {
        unreachable!()
    }
}

pub struct Chunk2DCacheFunction<R: ComponentReference<ChunkNoiseState>> {
    delegate: R,
    last_sampled_column: u64,
    last_result: f64,
}

impl<R: ComponentReference<ChunkNoiseState>> Chunk2DCacheFunction<R> {
    fn new(delegate: R) -> Self {
        Self {
            delegate,
            last_sampled_column: chunk_pos::MARKER,
            last_result: 0f64,
        }
    }
}

impl<R: ComponentReference<ChunkNoiseState>> ComponentFunctionImpl for Chunk2DCacheFunction<R> {}

impl<R: ComponentReference<ChunkNoiseState>> MutableComponentFunctionImpl<ChunkNoiseState>
    for Chunk2DCacheFunction<R>
{
    fn sample_mut(&mut self, pos: &NoisePos, env: &ChunkNoiseState) -> f64 {
        let block_x = pos.x();
        let block_z = pos.z();

        // This is the chunk packing function, but we use it for block positions here
        let hash = chunk_pos::packed(&Vector2::new(block_x, block_z));

        if hash == self.last_sampled_column {
            self.last_result
        } else {
            self.last_sampled_column = hash;
            self.last_result = self.delegate.sample_mut(pos, env);
            self.last_result
        }
    }

    fn fill_mut(
        &mut self,
        arr: &mut [f64],
        applier: &mut dyn EnvironmentApplierImpl<Env = ChunkNoiseState>,
    ) {
        self.delegate.fill_mut(arr, applier);
    }

    fn environment(&self) -> ConverterEnvironment<ChunkNoiseState> {
        ConverterEnvironment::ChunkNoise
    }
    fn into_environment(self: Box<Self>) -> OwnedConverterEnvironment<ChunkNoiseState> {
        unreachable!()
    }

    fn convert(
        self: Box<Self>,
        _converter: &mut dyn ConverterImpl<ChunkNoiseState>,
    ) -> ComponentReferenceImplementation<ChunkNoiseState> {
        unreachable!()
    }

    fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<ChunkNoiseState> {
        match_ref_implementations!(
            (delegate, self.delegate.clone_to_new_ref());
            {
                MutableComponentReference(Box::new(
                    Chunk2DCacheFunction {
                        delegate,
                        last_sampled_column: self.last_sampled_column,
                        last_result: self.last_result,
                    }
                )).into()
            }
        )
    }
}

pub struct ChunkInterpolatorFunction {
    delegate: Box<dyn ComponentReference<ChunkNoiseState>>,

    start_buf: Box<[f64]>,
    end_buf: Box<[f64]>,

    first_pass: [f64; 8],
    second_pass: [f64; 4],
    third_pass: [f64; 2],
    result: f64,
}

impl ChunkInterpolatorFunction {
    fn new(
        delegate: Box<dyn ComponentReference<ChunkNoiseState>>,
        state: &ChunkNoiseState,
    ) -> Self {
        Self {
            delegate,
            start_buf: vec![
                0f64;
                (state.vertical_cell_count as usize + 1)
                    * (state.horizontal_cell_count as usize + 1)
            ]
            .into(),

            end_buf: vec![
                0f64;
                (state.vertical_cell_count as usize + 1)
                    * (state.horizontal_cell_count as usize + 1)
            ]
            .into(),

            first_pass: [0f64; 8],
            second_pass: [0f64; 4],
            third_pass: [0f64; 2],
            result: 0f64,
        }
    }

    fn yz_to_buf_index(cell_y: u16, cell_z: u8, state: &ChunkNoiseState) -> usize {
        cell_z as usize * (state.vertical_cell_count as usize + 1) + cell_y as usize
    }

    fn on_sampled_cell_corners(&mut self, cell_y: u16, cell_z: u8, state: &ChunkNoiseState) {
        self.first_pass[0] = self.start_buf[Self::yz_to_buf_index(cell_y, cell_z, state)];
        self.first_pass[1] = self.start_buf[Self::yz_to_buf_index(cell_y, cell_z + 1, state)];
        self.first_pass[4] = self.end_buf[Self::yz_to_buf_index(cell_y, cell_z, state)];
        self.first_pass[5] = self.end_buf[Self::yz_to_buf_index(cell_y, cell_z + 1, state)];
        self.first_pass[2] = self.start_buf[Self::yz_to_buf_index(cell_y + 1, cell_z, state)];
        self.first_pass[3] = self.start_buf[Self::yz_to_buf_index(cell_y + 1, cell_z + 1, state)];
        self.first_pass[6] = self.end_buf[Self::yz_to_buf_index(cell_y + 1, cell_z, state)];
        self.first_pass[7] = self.end_buf[Self::yz_to_buf_index(cell_y + 1, cell_z + 1, state)];

        //log::debug!("{} First pass: {:?}", self.shared.unique_id, first_pass);
    }

    fn interpolate_y(&mut self, delta: f64) {
        self.second_pass[0] = lerp(delta, self.first_pass[0], self.first_pass[2]);
        self.second_pass[2] = lerp(delta, self.first_pass[4], self.first_pass[6]);
        self.second_pass[1] = lerp(delta, self.first_pass[1], self.first_pass[3]);
        self.second_pass[3] = lerp(delta, self.first_pass[5], self.first_pass[7]);

        //log::debug!("{} Second pass: {:?}", self.shared.unique_id, second_pass);
    }

    fn interpolate_x(&mut self, delta: f64) {
        self.third_pass[0] = lerp(delta, self.second_pass[0], self.second_pass[2]);
        self.third_pass[1] = lerp(delta, self.second_pass[1], self.second_pass[3]);

        //log::debug!("{} Third pass: {:?}", self.shared.unique_id, third_pass);
    }

    fn interpolate_z(&mut self, delta: f64) {
        self.result = lerp(delta, self.third_pass[0], self.third_pass[1]);

        //log::debug!("{} Result: {:?}", self.shared.unique_id, *result);
    }

    fn swap_buffers(&mut self) {
        #[cfg(debug_assertions)]
        let temp1 = self.start_buf.clone();
        #[cfg(debug_assertions)]
        let temp2 = self.end_buf.clone();

        mem::swap(&mut self.start_buf, &mut self.end_buf);

        #[cfg(debug_assertions)]
        assert!(temp1.iter().eq(self.end_buf.iter()));
        #[cfg(debug_assertions)]
        assert!(temp2.iter().eq(self.start_buf.iter()));
    }
}

impl ComponentFunctionImpl for ChunkInterpolatorFunction {}

impl MutableComponentFunctionImpl<ChunkNoiseState> for ChunkInterpolatorFunction {
    fn sample_mut(&mut self, pos: &NoisePos, env: &ChunkNoiseState) -> f64 {
        if let NoisePos::Chunk(_chunk_pos) = pos {
            #[cfg(debug_assertions)]
            assert!(env.is_interpolating);

            if env.is_sampling_for_caches {
                lerp3(
                    env.cell_block_pos.x as f64 / env.horizontal_cell_block_count as f64,
                    env.cell_block_pos.y as f64 / env.vertical_cell_block_count as f64,
                    env.cell_block_pos.z as f64 / env.horizontal_cell_block_count as f64,
                    self.first_pass[0],
                    self.first_pass[4],
                    self.first_pass[2],
                    self.first_pass[6],
                    self.first_pass[1],
                    self.first_pass[5],
                    self.first_pass[3],
                    self.first_pass[7],
                )
            } else {
                self.result
            }
        } else {
            self.delegate.sample_mut(pos, env)
        }
    }

    fn fill_mut(
        &mut self,
        arr: &mut [f64],
        applier: &mut dyn EnvironmentApplierImpl<Env = ChunkNoiseState>,
    ) {
        let env = applier.env();
        if env.is_sampling_for_caches {
            applier.fill_mut(arr, self);
        } else {
            self.delegate.fill_mut(arr, applier);
        }
    }

    fn environment(&self) -> ConverterEnvironment<ChunkNoiseState> {
        ConverterEnvironment::ChunkNoise
    }

    fn into_environment(self: Box<Self>) -> OwnedConverterEnvironment<ChunkNoiseState> {
        unreachable!()
    }

    fn convert(
        self: Box<Self>,
        _converter: &mut dyn ConverterImpl<ChunkNoiseState>,
    ) -> ComponentReferenceImplementation<ChunkNoiseState> {
        unreachable!()
    }

    fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<ChunkNoiseState> {
        unreachable!()
    }
}

// TODO: Can we do this without mutexes?
struct CachedFunctions {
    interpolators: Vec<Arc<Mutex<ChunkInterpolatorFunction>>>,
    caches: Vec<Arc<Mutex<ChunkCellCacheFunction>>>,
    cached_results:
        HashMap<SharedComponentReference, ComponentReferenceImplementation<ChunkNoiseState>>,
}

pub struct ChunkNoiseWrappedFunctionConverter<'a> {
    shared_data: &'a ChunkNoiseState,
    functions: CachedFunctions,
}

impl<'a> ChunkNoiseWrappedFunctionConverter<'a> {
    fn new(shared_data: &'a ChunkNoiseState, functions: CachedFunctions) -> Self {
        Self {
            shared_data,
            functions,
        }
    }

    fn consume(self) -> CachedFunctions {
        self.functions
    }
}

struct WrappedMutexFunction {
    wrapped: Arc<Mutex<dyn MutableComponentFunctionImpl<ChunkNoiseState>>>,
}

impl ComponentFunctionImpl for WrappedMutexFunction {}

impl MutableComponentFunctionImpl<ChunkNoiseState> for WrappedMutexFunction {
    fn sample_mut(&mut self, pos: &NoisePos, env: &ChunkNoiseState) -> f64 {
        let mut wrapped = self.wrapped.lock();

        wrapped.sample_mut(pos, env)
    }

    fn fill_mut(
        &mut self,
        arr: &mut [f64],
        applier: &mut dyn EnvironmentApplierImpl<Env = ChunkNoiseState>,
    ) {
        let mut wrapped = self.wrapped.lock();

        wrapped.fill_mut(arr, applier);
    }

    fn convert(
        self: Box<Self>,
        _converter: &mut dyn ConverterImpl<ChunkNoiseState>,
    ) -> ComponentReferenceImplementation<ChunkNoiseState> {
        unreachable!()
    }

    fn environment(&self) -> ConverterEnvironment<ChunkNoiseState> {
        ConverterEnvironment::ChunkNoise
    }

    fn into_environment(self: Box<Self>) -> OwnedConverterEnvironment<ChunkNoiseState> {
        unreachable!()
    }

    fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<ChunkNoiseState> {
        MutableComponentReference(Box::new(WrappedMutexFunction {
            wrapped: self.wrapped.clone(),
        }))
        .into()
    }
}

impl<'a> ConverterImpl<ChunkNoiseState> for ChunkNoiseWrappedFunctionConverter<'a> {
    fn convert_noise(&mut self, _noise: &Arc<InternalNoise>) -> Option<Arc<InternalNoise>> {
        None
    }

    fn convert_env_pre_internal(
        &mut self,
        component: ConverterEnvironment<ChunkNoiseState>,
    ) -> ConversionResultPre<ChunkNoiseState> {
        match component {
            ConverterEnvironment::ChunkNoise => ConversionResultPre::NoChange,
            _ => ConversionResultPre::Default,
        }
    }

    fn converts_post_internal(&mut self, component: ConverterEnvironment<ChunkNoiseState>) -> bool {
        matches!(component, ConverterEnvironment::Wrapper(_, _))
    }

    fn convert_env_post_internal(
        &mut self,
        component: OwnedConverterEnvironment<ChunkNoiseState>,
    ) -> ComponentReferenceImplementation<ChunkNoiseState> {
        match component {
            OwnedConverterEnvironment::Wrapper(wrapped, action) => match action {
                WrapperType::Cache2D => {
                    let new_ref = match_ref_implementations!(
                        (wrapped, wrapped);
                        {
                            MutableComponentReference(Box::new(Chunk2DCacheFunction::new(
                                wrapped
                            )))
                        }
                    );
                    new_ref.into()
                }
                WrapperType::FlatCache => {
                    let new_ref = match_ref_implementations!(
                        (wrapped, wrapped);
                        {
                            MutableComponentReference(Box::new(ChunkFlatCacheFunction::new(
                                wrapped,
                                true,
                                self.shared_data,
                            )))
                        }
                    );
                    new_ref.into()
                }
                WrapperType::OnceCache => {
                    let new_ref = match_ref_implementations!(
                        (wrapped, wrapped);
                        {
                            MutableComponentReference(Box::new(ChunkCacheOnceFunction::new(
                                wrapped
                            )))
                        }
                    );
                    new_ref.into()
                }
                WrapperType::CellCache => {
                    let function = Arc::new(Mutex::new(ChunkCellCacheFunction::new(
                        wrapped.boxed(),
                        self.shared_data,
                    )));
                    self.functions.caches.push(function.clone());

                    MutableComponentReference(Box::new(WrappedMutexFunction { wrapped: function }))
                        .into()
                }
                WrapperType::Interpolated => {
                    let function = Arc::new(Mutex::new(ChunkInterpolatorFunction::new(
                        wrapped.boxed(),
                        self.shared_data,
                    )));
                    self.functions.interpolators.push(function.clone());
                    MutableComponentReference(Box::new(WrappedMutexFunction { wrapped: function }))
                        .into()
                }
            },
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct ChunkNoisePos {
    x: i32,
    y: i32,
    z: i32,
    //unique_id: UniqueChunkNoiseId,
}

impl NoisePosImpl for ChunkNoisePos {
    fn x(&self) -> i32 {
        self.x
    }

    fn y(&self) -> i32 {
        self.y
    }

    fn z(&self) -> i32 {
        self.z
    }

    // TODO: implement blender
    fn get_blender(&self) -> Blender {
        Blender::NO_BLEND
    }
}

pub struct ChunkNoiseInterpolationApplier<'a> {
    shared: &'a mut ChunkNoiseState,
}

impl<'a> ChunkNoiseInterpolationApplier<'a> {
    fn new(shared: &'a mut ChunkNoiseState) -> Self {
        Self { shared }
    }
}

impl<'a> EnvironmentApplierImpl for ChunkNoiseInterpolationApplier<'a> {
    type Env = ChunkNoiseState;

    fn fill_mut(
        &mut self,
        densities: &mut [f64],
        function: &mut dyn MutableComponentFunctionImpl<Self::Env>,
    ) {
        for cell_y in 0..=self.shared.vertical_cell_count {
            self.shared.start_block_pos.y = (cell_y as i32 + self.shared.minimum_cell_y as i32)
                * self.shared.vertical_cell_block_count as i32;
            self.shared.sample_unique_index.add_assign(1);
            self.shared.cell_block_pos.y = 0;
            self.shared.index = cell_y as usize;
            let pos = self.shared.create_noise_pos();
            let sample = function.sample_mut(&pos, self.env());

            //log::debug!("{} for {:?}", sample, pos);
            densities[cell_y as usize] = sample;
        }
    }

    fn env(&mut self) -> &Self::Env {
        self.shared
    }

    fn cast_up(&mut self) -> &mut dyn ApplierImpl {
        self
    }
}

impl<'a> ApplierImpl for ChunkNoiseInterpolationApplier<'a> {
    fn at(&mut self, index: usize) -> NoisePos {
        self.shared.start_block_pos.y = (index as i32 + self.shared.minimum_cell_y as i32)
            * self.shared.vertical_cell_block_count as i32;
        self.shared.sample_unique_index.add_assign(1);
        self.shared.cell_block_pos.y = 0;
        self.shared.index = index;
        self.shared.create_noise_pos()
    }

    fn fill(&mut self, densities: &mut [f64], function: &dyn ImmutableComponentFunctionImpl) {
        for cell_y in 0..=self.shared.vertical_cell_count {
            self.shared.start_block_pos.y = (cell_y as i32 + self.shared.minimum_cell_y as i32)
                * self.shared.vertical_cell_block_count as i32;
            self.shared.sample_unique_index.add_assign(1);
            self.shared.cell_block_pos.y = 0;
            self.shared.index = cell_y as usize;
            let pos = self.shared.create_noise_pos();
            let sample = function.sample(&pos);

            //log::debug!("{} for {:?}", sample, pos);
            densities[cell_y as usize] = sample;
        }
    }
}

pub struct ChunkNoiseApplier<'a> {
    shared: &'a mut ChunkNoiseState,
}

impl<'a> ChunkNoiseApplier<'a> {
    fn new(shared: &'a mut ChunkNoiseState) -> Self {
        Self { shared }
    }
}

impl<'a> EnvironmentApplierImpl for ChunkNoiseApplier<'a> {
    type Env = ChunkNoiseState;

    fn fill_mut(
        &mut self,
        densities: &mut [f64],
        function: &mut dyn MutableComponentFunctionImpl<Self::Env>,
    ) {
        self.shared.index = 0;
        for cell_y in (0..self.shared.vertical_cell_block_count).rev() {
            self.shared.cell_block_pos.y = cell_y;

            for cell_x in 0..self.shared.horizontal_cell_block_count {
                self.shared.cell_block_pos.x = cell_x;

                for cell_z in 0..self.shared.horizontal_cell_block_count {
                    self.shared.cell_block_pos.z = cell_z;

                    densities[self.shared.index] =
                        function.sample_mut(&self.shared.create_noise_pos(), self.env());
                    self.shared.index.add_assign(1);
                }
            }
        }
    }

    fn env(&mut self) -> &Self::Env {
        self.shared
    }

    fn cast_up(&mut self) -> &mut dyn ApplierImpl {
        self
    }
}

impl<'a> ApplierImpl for ChunkNoiseApplier<'a> {
    fn at(&mut self, index: usize) -> NoisePos {
        let cell_block_z = index % self.shared.horizontal_cell_block_count as usize;
        let xy_chunk = index / self.shared.horizontal_cell_block_count as usize;
        let cell_block_x = xy_chunk % self.shared.horizontal_cell_block_count as usize;
        let cell_block_y = self.shared.vertical_cell_block_count as usize
            - 1
            - (xy_chunk / self.shared.horizontal_cell_block_count as usize);

        self.shared.cell_block_pos.x = cell_block_x as u8;
        self.shared.cell_block_pos.y = cell_block_y as u8;
        self.shared.cell_block_pos.z = cell_block_z as u8;
        self.shared.index = index;
        self.shared.create_noise_pos()
    }

    fn fill(&mut self, densities: &mut [f64], function: &dyn ImmutableComponentFunctionImpl) {
        self.shared.index = 0;
        for cell_y in (0..self.shared.vertical_cell_block_count).rev() {
            self.shared.cell_block_pos.y = cell_y;

            for cell_x in 0..self.shared.horizontal_cell_block_count {
                self.shared.cell_block_pos.x = cell_x;

                for cell_z in 0..self.shared.horizontal_cell_block_count {
                    self.shared.cell_block_pos.z = cell_z;

                    densities[self.shared.index] = function.sample(&self.shared.create_noise_pos());
                    self.shared.index.add_assign(1);
                }
            }
        }
    }
}

pub const CHUNK_DIM: u8 = 16;

#[derive(PartialEq, Eq, Clone, Hash, Default)]
pub struct ChunkNoiseState {
    cell_block_pos: Vector3<u8>,
    start_cell_pos: Vector2<i32>,
    start_block_pos: Vector3<i32>,
    start_biome_pos: Vector2<i32>,

    height: u16,
    vertical_cell_count: u16,
    horizontal_cell_count: u8,
    horizontal_cell_block_count: u8,
    vertical_cell_block_count: u8,
    horizontal_biome_end: u8,
    minimum_cell_y: i8,
    minimum_y: i8,

    is_interpolating: bool,
    is_sampling_for_caches: bool,

    index: usize,
    cache_once_unique_index: Wrapping<u64>,
    sample_unique_index: Wrapping<u64>,
}

impl DensityFunctionEnvironment for ChunkNoiseState {}

impl ChunkNoiseState {
    pub fn create_noise_pos(&self) -> NoisePos {
        let cell_block_x = self.cell_block_pos.x;
        let cell_block_y = self.cell_block_pos.y;
        let cell_block_z = self.cell_block_pos.z;

        let start_block_x = self.start_block_pos.x;
        let start_block_y = self.start_block_pos.y;
        let start_block_z = self.start_block_pos.z;

        let x = start_block_x + cell_block_x as i32;
        let y = start_block_y + cell_block_y as i32;
        let z = start_block_z + cell_block_z as i32;

        //log::debug!("Sampling pos {} {} {}", x, y, z);
        // TODO: Add blender
        NoisePos::Chunk(ChunkNoisePos {
            x,
            y,
            z,
            //unique_id: self.unique_id,
        })
    }
}

pub struct ChunkNoiseDensityFunctions {
    initial_density: Box<dyn ComponentReference<ChunkNoiseState>>,
    surface_height_estimate: HashMap<u64, i32>,
}

impl ChunkNoiseDensityFunctions {
    pub fn estimate_surface_height(
        &mut self,
        shared: &ChunkNoiseState,
        block_x: i32,
        block_z: i32,
    ) -> i32 {
        let biome_aligned_x = biome_coords::to_block(biome_coords::from_block(block_x));
        let biome_aligned_z = biome_coords::to_block(biome_coords::from_block(block_z));
        let packed = chunk_pos::packed(&Vector2::new(biome_aligned_x, biome_aligned_z));

        if let Some(estimate) = self.surface_height_estimate.get(&packed) {
            *estimate
        } else {
            let estimate = self.calculate_height_estimate(shared, packed);
            self.surface_height_estimate.insert(packed, estimate);
            estimate
        }
    }

    fn calculate_height_estimate(&mut self, shared: &ChunkNoiseState, packed_pos: u64) -> i32 {
        let x = chunk_pos::unpack_x(packed_pos);
        let z = chunk_pos::unpack_z(packed_pos);

        for y in ((shared.minimum_y as i32)..=(shared.minimum_y as i32 + shared.height as i32))
            .rev()
            .step_by(shared.vertical_cell_block_count as usize)
        {
            if self.initial_density.sample_mut(
                &NoisePos::Unblended(UnblendedNoisePos::new(x, y, z)),
                shared,
            ) > 0.390625f64
            {
                return y;
            }
        }

        i32::MAX
    }
}

pub struct ChunkNoiseGenerator {
    state_sampler: BlockStateSampler,
    generation_shape: GenerationShape,

    shared: ChunkNoiseState,
    //TODO: Handle without mutexs somehow
    cell_caches: Box<[Arc<Mutex<ChunkCellCacheFunction>>]>,
    interpolators: Box<[Arc<Mutex<ChunkInterpolatorFunction>>]>,
    density_functions: ChunkNoiseDensityFunctions,
}

impl ChunkNoiseGenerator {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        horizontal_cell_count: u8,
        start_block_x: i32,
        start_block_z: i32,
        generation_shape: GenerationShape,
        config: &NoiseConfig,
        level_sampler: FluidLevelSampler,
        aquifers: bool,
        ore_veins: bool,
    ) -> Self {
        let start_cell_pos = Vector2::new(
            floor_div(
                start_block_x,
                generation_shape.horizontal_cell_block_count() as i32,
            ),
            floor_div(
                start_block_z,
                generation_shape.horizontal_cell_block_count() as i32,
            ),
        );

        let start_block_pos = Vector3::new(0, 0, 0);
        let cell_block_pos = Vector3::new(0, 0, 0);
        let biome_pos = Vector2::new(
            biome_coords::from_block(start_block_x),
            biome_coords::from_block(start_block_z),
        );
        let is_interpolating = false;
        let is_sampling = false;
        let cache_once_unique_index = Wrapping(0);
        let sample_unique_index = Wrapping(0);
        let index = 0;
        let horizontal_biome_end = biome_coords::from_block(
            horizontal_cell_count * generation_shape.horizontal_cell_block_count(),
        );

        let vertical_cell_count =
            generation_shape.height() / generation_shape.vertical_cell_block_count() as u16;
        let minimum_cell_y = floor_div(
            generation_shape.min_y(),
            generation_shape.vertical_cell_block_count() as i8,
        );
        let vertical_cell_block_count = generation_shape.vertical_cell_block_count();
        let horizontal_cell_block_count = generation_shape.horizontal_cell_block_count();

        let shared = ChunkNoiseState {
            cell_block_pos,
            start_cell_pos,
            start_block_pos,
            start_biome_pos: biome_pos,
            height: generation_shape.height(),
            vertical_cell_block_count,
            horizontal_cell_block_count,
            vertical_cell_count,
            horizontal_cell_count,
            horizontal_biome_end,
            minimum_cell_y,
            minimum_y: generation_shape.min_y(),
            is_interpolating,
            is_sampling_for_caches: is_sampling,
            index,
            cache_once_unique_index,
            sample_unique_index,
        };

        let functions = CachedFunctions {
            caches: Vec::new(),
            interpolators: Vec::new(),
            cached_results: HashMap::new(),
        };

        // Convert router for this chunk
        let original_router = config.router();

        let mut converter = ChunkNoiseWrappedFunctionConverter::new(&shared, functions);
        let router = original_router.convert(&mut converter);

        let final_density = router.final_density;
        let vein_toggle = router.vein_toggle;
        let vein_ridged = router.vein_ridged;
        let vein_gap = router.vein_gap;

        // Create and convert aquifer density function
        let converted_aquifer_density = match final_density
            .wrapped_ref_from_box()
            .add(BeardifierFunction::INSTANCE.into())
        {
            ComponentReferenceImplementation::Mutable(owned) => MutableComponentReference(
                Box::new(WrapperFunction::new(owned, WrapperType::CellCache)),
            )
            .convert(&mut converter),
            ComponentReferenceImplementation::Shared(shared) => {
                SharedComponentReference(Arc::new(WrapperFunction::<
                    ChunkNoiseState,
                    SharedComponentReference,
                >::new(
                    shared, WrapperType::CellCache
                )))
                .convert(&mut converter)
            }
        };

        let functions = converter.consume();

        let aquifer_sampler = if aquifers {
            let section_x = section_coords::block_to_section(start_block_x);
            let section_z = section_coords::block_to_section(start_block_z);
            AquiferSampler::Aquifier(WorldAquiferSampler::new(
                Vector2::new(section_x, section_z),
                router.barrier,
                router.fluid_level_floodedness,
                router.fluid_level_spread,
                router.lava,
                router.erosion,
                router.depth,
                converted_aquifer_density.boxed(),
                config.aquifer_deriver(),
                generation_shape.min_y(),
                generation_shape.height(),
                level_sampler,
            ))
        } else {
            AquiferSampler::SeaLevel(SeaLevelAquiferSampler::new(
                level_sampler,
                converted_aquifer_density.boxed(),
            ))
        };

        let mut samplers = vec![BlockStateSampler::Aquifer(aquifer_sampler)];

        if ore_veins {
            let ore_sampler =
                OreVeinSampler::new(vein_toggle, vein_ridged, vein_gap, config.ore_deriver());
            samplers.push(BlockStateSampler::Ore(ore_sampler));
        };

        let state_sampler = BlockStateSampler::Chained(ChainedBlockStateSampler::new(samplers));

        let density_functions = ChunkNoiseDensityFunctions {
            initial_density: router.internal_density,
            surface_height_estimate: HashMap::new(),
        };

        Self {
            state_sampler,
            generation_shape,
            shared,
            cell_caches: functions.caches.into(),
            interpolators: functions.interpolators.into(),
            density_functions,
        }
    }

    pub fn stop_interpolation(&mut self) {
        assert!(self.shared.is_interpolating);
        self.shared.is_interpolating = false;
    }

    pub fn sample_start_density(&mut self) {
        assert!(!self.shared.is_interpolating);
        self.shared.is_interpolating = true;
        self.shared.sample_unique_index.set_zero();
        self.sample_density(true, self.shared.start_cell_pos.x);
    }

    pub fn sample_end_density(&mut self, cell_x: u8) {
        self.sample_density(false, self.shared.start_cell_pos.x + cell_x as i32 + 1);
        self.shared.start_block_pos.x = (self.shared.start_cell_pos.x + cell_x as i32)
            * self.horizontal_cell_block_count() as i32;
    }

    fn sample_density(&mut self, start: bool, current_x: i32) {
        self.shared.start_block_pos.x = current_x * self.horizontal_cell_block_count() as i32;
        self.shared.cell_block_pos.x = 0;

        for cell_z in 0..=self.horizontal_cell_block_count() {
            let current_z = self.shared.start_cell_pos.z + cell_z as i32;
            self.shared.start_block_pos.z = current_z * self.horizontal_cell_block_count() as i32;
            self.shared.cell_block_pos.z = 0;

            self.shared.cache_once_unique_index.add_assign(1);
            for interpolator in &self.interpolators {
                let mut interpolator = interpolator.lock();

                let mut y_buf = vec![0f64; self.shared.vertical_cell_count as usize + 1];

                let mut applier = ChunkNoiseInterpolationApplier::new(&mut self.shared);
                interpolator.fill_mut(&mut y_buf, &mut applier);

                let interp_buf = if start {
                    let start_index =
                        ChunkInterpolatorFunction::yz_to_buf_index(0, cell_z, &self.shared);
                    &mut interpolator.start_buf
                        [start_index..=start_index + self.shared.vertical_cell_count as usize]
                } else {
                    let start_index =
                        ChunkInterpolatorFunction::yz_to_buf_index(0, cell_z, &self.shared);
                    &mut interpolator.end_buf
                        [start_index..=start_index + self.shared.vertical_cell_count as usize]
                };

                interp_buf.copy_from_slice(&y_buf);
            }
        }
        self.shared.cache_once_unique_index.add_assign(1);
    }

    pub fn interpolate_y(&mut self, block_y: i32, delta: f64) {
        self.shared.cell_block_pos.y = (block_y - self.shared.start_block_pos.y) as u8;

        for interpolator in &self.interpolators {
            let mut interpolator = interpolator.lock();
            interpolator.interpolate_y(delta);
        }
    }

    pub fn interpolate_x(&mut self, block_x: i32, delta: f64) {
        self.shared.cell_block_pos.x = (block_x - self.shared.start_block_pos.x) as u8;

        for interpolator in &self.interpolators {
            let mut interpolator = interpolator.lock();

            interpolator.interpolate_x(delta);
        }
    }

    pub fn interpolate_z(&mut self, block_z: i32, delta: f64) {
        self.shared.cell_block_pos.z = (block_z - self.shared.start_block_pos.z) as u8;
        self.shared.sample_unique_index.add_assign(1);

        for interpolator in &self.interpolators {
            let mut interpolator = interpolator.lock();

            interpolator.interpolate_z(delta);
        }
    }

    pub fn swap_buffers(&self) {
        for interpolator in &self.interpolators {
            let mut interpolator = interpolator.lock();

            interpolator.swap_buffers();
        }
    }

    pub fn on_sampled_cell_corners(&mut self, cell_y: u16, cell_z: u8) {
        for interpolator in &self.interpolators {
            let mut interpolator = interpolator.lock();

            interpolator.on_sampled_cell_corners(cell_y, cell_z, &self.shared);
        }

        self.shared.is_sampling_for_caches = true;
        self.shared.start_block_pos.y = (cell_y as i32 + self.minimum_cell_y() as i32)
            * self.vertical_cell_block_count() as i32;
        self.shared.start_block_pos.z = (cell_z as i32 + self.shared.start_cell_pos.z)
            * self.horizontal_cell_block_count() as i32;
        self.shared.cache_once_unique_index.add_assign(1);

        for cell_cache in &self.cell_caches {
            let mut cell_cache = cell_cache.lock();

            let mut new_cache = ChunkCellCacheFunction::create_cache(&self.shared);
            let mut applier = ChunkNoiseApplier::new(&mut self.shared);
            cell_cache.delegate.fill_mut(&mut new_cache, &mut applier);
            cell_cache.cache = new_cache.into();
        }

        self.shared.cache_once_unique_index.add_assign(1);
        self.shared.is_sampling_for_caches = false;
    }

    pub fn sample_block_state(&mut self) -> Option<BlockState> {
        self.state_sampler.sample(
            &self.shared.create_noise_pos(),
            &self.shared,
            &mut self.density_functions,
        )
    }

    pub fn horizontal_cell_block_count(&self) -> u8 {
        self.generation_shape.horizontal_cell_block_count()
    }

    pub fn vertical_cell_block_count(&self) -> u8 {
        self.generation_shape.vertical_cell_block_count()
    }

    pub fn min_y(&self) -> i8 {
        self.generation_shape.min_y()
    }

    pub fn minimum_cell_y(&self) -> i8 {
        self.generation_shape.min_y() / self.generation_shape.vertical_cell_block_count() as i8
    }

    pub fn height(&self) -> u16 {
        self.generation_shape.height()
    }
}