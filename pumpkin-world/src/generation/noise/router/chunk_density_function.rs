use std::cell::RefCell;
use std::mem;

use super::{
    chunk_noise_router::{ChunkNoiseFunctionComponent, MutableChunkNoiseFunctionComponentImpl},
    density_function::{IndexToNoisePos, NoiseFunctionComponentRange, NoisePos},
};
use enum_dispatch::enum_dispatch;
use pumpkin_util::math::{lerp, lerp3, vector2::Vector2};

use crate::generation::{biome_coords, positions::chunk_pos};

thread_local! {
    static F64_BUFFER_POOL: RefCell<Vec<Vec<f64>>> = const {
        RefCell::new(Vec::new())
    };
}

#[inline]
fn get_buffer(len: usize) -> Box<[f64]> {
    F64_BUFFER_POOL.with(|pool| {
        let mut buffers = pool.borrow_mut();
        if let Some(mut buf) = buffers.pop() {
            if buf.len() != len {
                buf.resize(len, 0.0);
            } else {
                buf.fill(0.0);
            }
            buf.into_boxed_slice()
        } else {
            vec![0.0; len].into_boxed_slice()
        }
    })
}

#[inline]
fn recycle_buffer(buf: Box<[f64]>) {
    F64_BUFFER_POOL.with(|pool| {
        pool.borrow_mut().push(Vec::from(buf));
    });
}

pub struct WrapperData {
    // Our relative position within the cell
    pub(crate) cell_x_block_position: usize,
    pub(crate) cell_y_block_position: usize,
    pub(crate) cell_z_block_position: usize,

    // Number of blocks per cell per axis
    pub(crate) horizontal_cell_block_count: usize,
    pub(crate) vertical_cell_block_count: usize,
}

pub enum SampleAction {
    SkipCellCaches,
    CellCaches(WrapperData),
}

pub struct ChunkNoiseFunctionSampleOptions {
    populating_caches: bool,
    pub(crate) action: SampleAction,

    // Global IDs for the `CacheOnce` wrapper
    pub(crate) cache_result_unique_id: u64,
    pub(crate) cache_fill_unique_id: u64,

    // The current index of a slice being filled by the `fill` function
    pub(crate) fill_index: usize,
}

impl ChunkNoiseFunctionSampleOptions {
    pub const fn new(
        populating_caches: bool,
        action: SampleAction,
        cache_result_unique_id: u64,
        cache_fill_unique_id: u64,
        fill_index: usize,
    ) -> Self {
        Self {
            populating_caches,
            action,
            cache_result_unique_id,
            cache_fill_unique_id,
            fill_index,
        }
    }
}

pub struct ChunkNoiseFunctionBuilderOptions {
    // Number of blocks per cell per axis
    horizontal_cell_block_count: usize,
    vertical_cell_block_count: usize,

    // Number of cells per chunk per axis
    vertical_cell_count: usize,
    horizontal_cell_count: usize,

    // The biome coords of this chunk
    pub start_biome_x: i32,
    pub start_biome_z: i32,

    // Number of biome regions per chunk per axis
    pub horizontal_biome_end: usize,
}

impl ChunkNoiseFunctionBuilderOptions {
    pub const fn new(
        horizontal_cell_block_count: usize,
        vertical_cell_block_count: usize,
        vertical_cell_count: usize,
        horizontal_cell_count: usize,
        start_biome_x: i32,
        start_biome_z: i32,
        horizontal_biome_end: usize,
    ) -> Self {
        Self {
            horizontal_cell_block_count,
            vertical_cell_block_count,
            vertical_cell_count,
            horizontal_cell_count,
            start_biome_x,
            start_biome_z,
            horizontal_biome_end,
        }
    }
}

// These are chunk specific function components that are picked based on the wrapper type
pub struct DensityInterpolator {
    // What we are interpolating
    pub(crate) input_index: usize,

    // y-z plane buffers to be interpolated together, each of these values is that of the cell, not
    // the block
    pub(crate) start_buffer: Box<[f64]>,
    pub(crate) end_buffer: Box<[f64]>,

    first_pass: [f64; 8],
    second_pass: [f64; 4],
    third_pass: [f64; 2],
    result: f64,

    pub(crate) vertical_cell_count: usize,
    min_value: f64,
    max_value: f64,
}

impl NoiseFunctionComponentRange for DensityInterpolator {
    #[inline]
    fn min(&self) -> f64 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f64 {
        self.max_value
    }
}

impl DensityInterpolator {
    pub fn new(
        input_index: usize,
        min_value: f64,
        max_value: f64,
        builder_options: &ChunkNoiseFunctionBuilderOptions,
    ) -> Self {
        // These are all dummy values to be populated when sampling values
        Self {
            input_index,
            start_buffer: get_buffer(
                (builder_options.vertical_cell_count + 1)
                    * (builder_options.horizontal_cell_count + 1),
            ),
            end_buffer: get_buffer(
                (builder_options.vertical_cell_count + 1)
                    * (builder_options.horizontal_cell_count + 1),
            ),
            first_pass: Default::default(),
            second_pass: Default::default(),
            third_pass: Default::default(),
            result: Default::default(),
            vertical_cell_count: builder_options.vertical_cell_count,
            min_value,
            max_value,
        }
    }

    #[inline]
    pub(crate) fn yz_to_buf_index(&self, cell_y_position: usize, cell_z_position: usize) -> usize {
        cell_z_position * (self.vertical_cell_count + 1) + cell_y_position
    }

    pub(crate) fn on_sampled_cell_corners(
        &mut self,
        cell_y_position: usize,
        cell_z_position: usize,
    ) {
        self.first_pass[0] =
            self.start_buffer[self.yz_to_buf_index(cell_y_position, cell_z_position)];
        self.first_pass[1] =
            self.start_buffer[self.yz_to_buf_index(cell_y_position, cell_z_position + 1)];
        self.first_pass[4] =
            self.end_buffer[self.yz_to_buf_index(cell_y_position, cell_z_position)];
        self.first_pass[5] =
            self.end_buffer[self.yz_to_buf_index(cell_y_position, cell_z_position + 1)];
        self.first_pass[2] =
            self.start_buffer[self.yz_to_buf_index(cell_y_position + 1, cell_z_position)];
        self.first_pass[3] =
            self.start_buffer[self.yz_to_buf_index(cell_y_position + 1, cell_z_position + 1)];
        self.first_pass[6] =
            self.end_buffer[self.yz_to_buf_index(cell_y_position + 1, cell_z_position)];
        self.first_pass[7] =
            self.end_buffer[self.yz_to_buf_index(cell_y_position + 1, cell_z_position + 1)];
    }

    pub(crate) fn interpolate_y(&mut self, delta: f64) {
        self.second_pass[0] = lerp(delta, self.first_pass[0], self.first_pass[2]);
        self.second_pass[2] = lerp(delta, self.first_pass[4], self.first_pass[6]);
        self.second_pass[1] = lerp(delta, self.first_pass[1], self.first_pass[3]);
        self.second_pass[3] = lerp(delta, self.first_pass[5], self.first_pass[7]);
    }

    #[inline]
    pub(crate) fn interpolate_x(&mut self, delta: f64) {
        self.third_pass[0] = lerp(delta, self.second_pass[0], self.second_pass[2]);
        self.third_pass[1] = lerp(delta, self.second_pass[1], self.second_pass[3]);
    }

    #[inline]
    pub(crate) fn interpolate_z(&mut self, delta: f64) {
        self.result = lerp(delta, self.third_pass[0], self.third_pass[1]);
    }

    #[inline]
    pub(crate) fn swap_buffers(&mut self) {
        #[cfg(debug_assertions)]
        let test = self.start_buffer[0];
        mem::swap(&mut self.start_buffer, &mut self.end_buffer);
        #[cfg(debug_assertions)]
        assert_eq!(test, self.end_buffer[0]);
    }
}

impl Drop for DensityInterpolator {
    fn drop(&mut self) {
        recycle_buffer(std::mem::replace(
            &mut self.start_buffer,
            Vec::new().into_boxed_slice(),
        ));
        recycle_buffer(std::mem::replace(
            &mut self.end_buffer,
            Vec::new().into_boxed_slice(),
        ));
    }
}

impl MutableChunkNoiseFunctionComponentImpl for DensityInterpolator {
    fn sample(
        &mut self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &impl NoisePos,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        match sample_options.action {
            SampleAction::CellCaches(WrapperData {
                cell_x_block_position,
                cell_y_block_position,
                cell_z_block_position,
                horizontal_cell_block_count,
                vertical_cell_block_count,
            }) => {
                if sample_options.populating_caches {
                    lerp3(
                        cell_x_block_position as f64 / horizontal_cell_block_count as f64,
                        cell_y_block_position as f64 / vertical_cell_block_count as f64,
                        cell_z_block_position as f64 / horizontal_cell_block_count as f64,
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
            }
            SampleAction::SkipCellCaches => ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=self.input_index],
                pos,
                sample_options,
            ),
        }
    }

    fn fill(
        &mut self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        array: &mut [f64],
        mapper: &impl IndexToNoisePos,
        sample_options: &mut ChunkNoiseFunctionSampleOptions,
    ) {
        if sample_options.populating_caches {
            array.iter_mut().enumerate().for_each(|(index, value)| {
                let pos = mapper.at(index, Some(sample_options));
                let result = self.sample(component_stack, &pos, sample_options);
                *value = result;
            });
        } else {
            ChunkNoiseFunctionComponent::fill_from_stack(
                &mut component_stack[..=self.input_index],
                array,
                mapper,
                sample_options,
            );
        }
    }
}

pub struct FlatCache {
    pub(crate) input_index: usize,

    pub(crate) cache: Box<[f64]>,
    start_biome_x: i32,
    start_biome_z: i32,
    horizontal_biome_end: usize,

    min_value: f64,
    max_value: f64,
}

impl NoiseFunctionComponentRange for FlatCache {
    #[inline]
    fn min(&self) -> f64 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f64 {
        self.max_value
    }
}

impl MutableChunkNoiseFunctionComponentImpl for FlatCache {
    fn sample(
        &mut self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &impl NoisePos,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        let absolute_biome_x_position = biome_coords::from_block(pos.x());
        let absolute_biome_z_position = biome_coords::from_block(pos.z());

        let relative_biome_x_position = absolute_biome_x_position - self.start_biome_x;
        let relative_biome_z_position = absolute_biome_z_position - self.start_biome_z;

        if relative_biome_x_position >= 0
            && relative_biome_z_position >= 0
            && relative_biome_x_position <= self.horizontal_biome_end as i32
            && relative_biome_z_position <= self.horizontal_biome_end as i32
        {
            let cache_index = self.xz_to_index_const(
                relative_biome_x_position as usize,
                relative_biome_z_position as usize,
            );
            self.cache[cache_index]
        } else {
            ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=self.input_index],
                pos,
                sample_options,
            )
        }
    }
}

impl Drop for FlatCache {
    fn drop(&mut self) {
        recycle_buffer(std::mem::replace(
            &mut self.cache,
            Vec::new().into_boxed_slice(),
        ));
    }
}

impl FlatCache {
    pub fn new(
        input_index: usize,
        min_value: f64,
        max_value: f64,
        start_biome_x: i32,
        start_biome_z: i32,
        horizontal_biome_end: usize,
    ) -> Self {
        Self {
            input_index,
            cache: get_buffer((horizontal_biome_end + 1) * (horizontal_biome_end + 1)),
            start_biome_x,
            start_biome_z,
            horizontal_biome_end,
            min_value,
            max_value,
        }
    }

    #[inline]
    pub fn xz_to_index_const(&self, biome_x_position: usize, biome_z_position: usize) -> usize {
        biome_x_position * (self.horizontal_biome_end + 1) + biome_z_position
    }
}

#[derive(Clone)]
pub struct Cache2D {
    pub(crate) input_index: usize,
    last_sample_column: u64,
    last_sample_result: f64,

    min_value: f64,
    max_value: f64,
}

impl NoiseFunctionComponentRange for Cache2D {
    #[inline]
    fn min(&self) -> f64 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f64 {
        self.max_value
    }
}

impl MutableChunkNoiseFunctionComponentImpl for Cache2D {
    fn sample(
        &mut self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &impl NoisePos,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        let packed_column = chunk_pos::packed(&Vector2::new(pos.x(), pos.z()));
        if packed_column == self.last_sample_column {
            self.last_sample_result
        } else {
            let result = ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=self.input_index],
                pos,
                sample_options,
            );
            self.last_sample_column = packed_column;
            self.last_sample_result = result;

            result
        }
    }
}

impl Cache2D {
    pub fn new(input_index: usize, min_value: f64, max_value: f64) -> Self {
        Self {
            input_index,
            // I know this is because there's is definitely world coords that are this marker, but this
            // is how vanilla does it, so I'm going to for pairity
            last_sample_column: chunk_pos::MARKER,
            last_sample_result: Default::default(),
            min_value,
            max_value,
        }
    }
}

pub struct CacheOnce {
    pub(crate) input_index: usize,
    cache_result_unique_id: u64,
    cache_fill_unique_id: u64,
    last_sample_result: f64,

    cache: Box<[f64]>,

    min_value: f64,
    max_value: f64,
}

impl NoiseFunctionComponentRange for CacheOnce {
    #[inline]
    fn min(&self) -> f64 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f64 {
        self.max_value
    }
}

impl MutableChunkNoiseFunctionComponentImpl for CacheOnce {
    fn sample(
        &mut self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &impl NoisePos,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        match sample_options.action {
            SampleAction::CellCaches(_) => {
                if self.cache_fill_unique_id == sample_options.cache_fill_unique_id
                    && !self.cache.is_empty()
                {
                    self.cache[sample_options.fill_index]
                } else if self.cache_result_unique_id == sample_options.cache_result_unique_id {
                    self.last_sample_result
                } else {
                    let result = ChunkNoiseFunctionComponent::sample_from_stack(
                        &mut component_stack[..=self.input_index],
                        pos,
                        sample_options,
                    );
                    self.cache_result_unique_id = sample_options.cache_result_unique_id;
                    self.last_sample_result = result;

                    result
                }
            }
            SampleAction::SkipCellCaches => ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=self.input_index],
                pos,
                sample_options,
            ),
        }
    }

    fn fill(
        &mut self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        array: &mut [f64],
        mapper: &impl IndexToNoisePos,
        sample_options: &mut ChunkNoiseFunctionSampleOptions,
    ) {
        if self.cache_fill_unique_id == sample_options.cache_fill_unique_id
            && !self.cache.is_empty()
        {
            array.copy_from_slice(&self.cache);
            return;
        }

        ChunkNoiseFunctionComponent::fill_from_stack(
            &mut component_stack[..=self.input_index],
            array,
            mapper,
            sample_options,
        );

        // We need to make a new cache
        if self.cache.len() != array.len() {
            self.cache = vec![0.0; array.len()].into_boxed_slice();
        }

        self.cache.copy_from_slice(array);
        self.cache_fill_unique_id = sample_options.cache_fill_unique_id;
    }
}

impl CacheOnce {
    pub fn new(input_index: usize, min_value: f64, max_value: f64) -> Self {
        Self {
            input_index,
            // Make these max, just to be different from the overall default of 0
            cache_result_unique_id: 0,
            cache_fill_unique_id: 0,
            last_sample_result: Default::default(),
            cache: Box::new([]),
            min_value,
            max_value,
        }
    }
}

pub struct CellCache {
    pub(crate) input_index: usize,
    pub(crate) cache: Box<[f64]>,

    min_value: f64,
    max_value: f64,
}

impl NoiseFunctionComponentRange for CellCache {
    #[inline]
    fn min(&self) -> f64 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f64 {
        self.max_value
    }
}

impl MutableChunkNoiseFunctionComponentImpl for CellCache {
    fn sample(
        &mut self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &impl NoisePos,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        match sample_options.action {
            SampleAction::CellCaches(WrapperData {
                cell_x_block_position,
                cell_y_block_position,
                cell_z_block_position,
                horizontal_cell_block_count,
                vertical_cell_block_count,
            }) => {
                let cache_index = ((vertical_cell_block_count - 1 - cell_y_block_position)
                    * horizontal_cell_block_count
                    + cell_x_block_position)
                    * horizontal_cell_block_count
                    + cell_z_block_position;

                self.cache[cache_index]
            }
            SampleAction::SkipCellCaches => ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=self.input_index],
                pos,
                sample_options,
            ),
        }
    }
}

impl CellCache {
    pub fn new(
        input_index: usize,
        min_value: f64,
        max_value: f64,
        build_options: &ChunkNoiseFunctionBuilderOptions,
    ) -> Self {
        Self {
            input_index,
            cache: get_buffer(
                build_options.horizontal_cell_block_count
                    * build_options.horizontal_cell_block_count
                    * build_options.vertical_cell_block_count,
            ),
            min_value,
            max_value,
        }
    }

    /// Clones this instance, creating a new struct taking ownership of the cache and replacing the
    /// original with a dummy
    fn take_cache_clone(&mut self) -> Self {
        let mut cache: Box<[f64]> = Box::new([]);
        mem::swap(&mut cache, &mut self.cache);
        Self {
            input_index: self.input_index,
            cache,
            min_value: self.min_value,
            max_value: self.max_value,
        }
    }
}

#[enum_dispatch(MutableChunkNoiseFunctionComponentImpl, NoiseFunctionComponentRange)]
pub enum ChunkSpecificNoiseFunctionComponent {
    DensityInterpolator(DensityInterpolator),
    FlatCache(FlatCache),
    Cache2D(Cache2D),
    CacheOnce(CacheOnce),
    CellCache(CellCache),
}
