use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};

use crate::{
    block::BlockState,
    generation::{
        chunk_noise::CHUNK_DIM, generation_shapes::GenerationShape, positions::chunk_pos,
    },
};

use super::{
    GlobalRandomConfig,
    aquifer_sampler::{FluidLevel, FluidLevelSampler, FluidLevelSamplerImpl},
    chunk_noise::{ChunkNoiseGenerator, LAVA_BLOCK, STONE_BLOCK, WATER_BLOCK},
    noise_router::proto_noise_router::GlobalProtoNoiseRouter,
    positions::chunk_pos::{start_block_x, start_block_z},
};

pub struct StandardChunkFluidLevelSampler {
    top_fluid: FluidLevel,
    bottom_fluid: FluidLevel,
    bottom_y: i32,
}

impl StandardChunkFluidLevelSampler {
    pub fn new(top_fluid: FluidLevel, bottom_fluid: FluidLevel) -> Self {
        let bottom_y = top_fluid
            .max_y_exclusive()
            .min(bottom_fluid.max_y_exclusive());
        Self {
            top_fluid,
            bottom_fluid,
            bottom_y,
        }
    }
}

impl FluidLevelSamplerImpl for StandardChunkFluidLevelSampler {
    fn get_fluid_level(&self, _x: i32, y: i32, _z: i32) -> FluidLevel {
        if y < self.bottom_y {
            self.bottom_fluid.clone()
        } else {
            self.top_fluid.clone()
        }
    }
}

pub struct ProtoChunk<'a> {
    chunk_pos: Vector2<i32>,
    sampler: ChunkNoiseGenerator<'a>,
    // These are local positions
    flat_block_map: Vec<BlockState>,
    // may want to use chunk status
}

impl<'a> ProtoChunk<'a> {
    pub fn new(
        chunk_pos: Vector2<i32>,
        base_router: &'a GlobalProtoNoiseRouter,
        random_config: &'a GlobalRandomConfig,
    ) -> Self {
        let generation_shape = GenerationShape::SURFACE;

        let horizontal_cell_count = CHUNK_DIM / generation_shape.horizontal_cell_block_count();

        // TODO: Customize these
        let sampler = FluidLevelSampler::Chunk(StandardChunkFluidLevelSampler::new(
            FluidLevel::new(63, WATER_BLOCK),
            FluidLevel::new(-54, LAVA_BLOCK),
        ));

        let height = generation_shape.height() as usize;
        let sampler = ChunkNoiseGenerator::new(
            base_router,
            random_config,
            horizontal_cell_count,
            chunk_pos::start_block_x(&chunk_pos),
            chunk_pos::start_block_z(&chunk_pos),
            generation_shape,
            sampler,
            true,
            true,
        );

        Self {
            chunk_pos,
            sampler,
            flat_block_map: vec![BlockState::AIR; CHUNK_DIM as usize * CHUNK_DIM as usize * height],
        }
    }

    #[inline]
    fn local_pos_to_index(&self, local_pos: &Vector3<i32>) -> usize {
        #[cfg(debug_assertions)]
        {
            assert!(local_pos.x >= 0 && local_pos.x <= 15);
            assert!(local_pos.y < self.sampler.height() as i32 && local_pos.y >= 0);
            assert!(local_pos.z >= 0 && local_pos.z <= 15);
        }
        self.sampler.height() as usize * CHUNK_DIM as usize * local_pos.x as usize
            + CHUNK_DIM as usize * local_pos.y as usize
            + local_pos.z as usize
    }

    #[inline]
    pub fn get_block_state(&self, local_pos: &Vector3<i32>) -> BlockState {
        let local_pos = Vector3::new(
            local_pos.x & 15,
            local_pos.y - self.sampler.min_y() as i32,
            local_pos.z & 15,
        );
        if local_pos.y < 0 || local_pos.y >= self.sampler.height() as i32 {
            BlockState::AIR
        } else {
            self.flat_block_map[self.local_pos_to_index(&local_pos)]
        }
    }

    pub fn populate_noise(&mut self) {
        let horizontal_cell_block_count = self.sampler.horizontal_cell_block_count();
        let vertical_cell_block_count = self.sampler.vertical_cell_block_count();

        let horizontal_cells = CHUNK_DIM / horizontal_cell_block_count;

        let min_y = self.sampler.min_y();
        let minimum_cell_y = min_y / vertical_cell_block_count as i8;
        let cell_height = self.sampler.height() / vertical_cell_block_count as u16;

        // TODO: Block state updates when we implement those
        self.sampler.sample_start_density();
        for cell_x in 0..horizontal_cells {
            self.sampler.sample_end_density(cell_x);
            let sample_start_x =
                (self.start_cell_x() + cell_x as i32) * horizontal_cell_block_count as i32;

            for cell_z in 0..horizontal_cells {
                for cell_y in (0..cell_height).rev() {
                    self.sampler.on_sampled_cell_corners(cell_x, cell_y, cell_z);
                    let sample_start_y =
                        (minimum_cell_y as i32 + cell_y as i32) * vertical_cell_block_count as i32;
                    let sample_start_z =
                        (self.start_cell_z() + cell_z as i32) * horizontal_cell_block_count as i32;
                    for local_y in (0..vertical_cell_block_count).rev() {
                        let block_y = (minimum_cell_y as i32 + cell_y as i32)
                            * vertical_cell_block_count as i32
                            + local_y as i32;
                        let delta_y = local_y as f64 / vertical_cell_block_count as f64;
                        self.sampler.interpolate_y(delta_y);

                        for local_x in 0..horizontal_cell_block_count {
                            let block_x = self.start_block_x()
                                + cell_x as i32 * horizontal_cell_block_count as i32
                                + local_x as i32;
                            let delta_x = local_x as f64 / horizontal_cell_block_count as f64;
                            self.sampler.interpolate_x(delta_x);

                            for local_z in 0..horizontal_cell_block_count {
                                let block_z = self.start_block_z()
                                    + cell_z as i32 * horizontal_cell_block_count as i32
                                    + local_z as i32;
                                let delta_z = local_z as f64 / horizontal_cell_block_count as f64;
                                self.sampler.interpolate_z(delta_z);

                                // TODO: Can the math here be simplified? Do the above values come
                                // to the same results?
                                let cell_offset_x = block_x - sample_start_x;
                                let cell_offset_y = block_y - sample_start_y;
                                let cell_offset_z = block_z - sample_start_z;

                                #[cfg(debug_assertions)]
                                {
                                    assert!(cell_offset_x >= 0);
                                    assert!(cell_offset_y >= 0);
                                    assert!(cell_offset_z >= 0);
                                }

                                // TODO: Change default block
                                let block_state = self
                                    .sampler
                                    .sample_block_state(
                                        sample_start_x,
                                        sample_start_y,
                                        sample_start_z,
                                        cell_offset_x as usize,
                                        cell_offset_y as usize,
                                        cell_offset_z as usize,
                                    )
                                    .unwrap_or(STONE_BLOCK);
                                //log::debug!("Sampled block state in {:?}", inst.elapsed());

                                let local_pos = Vector3 {
                                    x: block_x & 15,
                                    y: block_y - min_y as i32,
                                    z: block_z & 15,
                                };

                                #[cfg(debug_assertions)]
                                {
                                    assert!(local_pos.x < 16 && local_pos.x >= 0);
                                    assert!(
                                        local_pos.y < self.sampler.height() as i32
                                            && local_pos.y >= 0
                                    );
                                    assert!(local_pos.z < 16 && local_pos.z >= 0);
                                }
                                let index = self.local_pos_to_index(&local_pos);
                                self.flat_block_map[index] = block_state;
                            }
                        }
                    }
                }
            }

            self.sampler.swap_buffers();
        }
    }

    fn start_cell_x(&self) -> i32 {
        self.start_block_x() / self.sampler.horizontal_cell_block_count() as i32
    }

    fn start_cell_z(&self) -> i32 {
        self.start_block_z() / self.sampler.horizontal_cell_block_count() as i32
    }

    fn start_block_x(&self) -> i32 {
        start_block_x(&self.chunk_pos)
    }

    fn start_block_z(&self) -> i32 {
        start_block_z(&self.chunk_pos)
    }
}

#[cfg(test)]
mod test {
    use std::sync::LazyLock;

    use pumpkin_util::math::vector2::Vector2;

    use crate::{
        generation::{
            GlobalRandomConfig,
            noise_router::{
                density_function::{NoiseFunctionComponentRange, PassThrough},
                proto_noise_router::{GlobalProtoNoiseRouter, ProtoNoiseFunctionComponent},
            },
        },
        noise_router::{NOISE_ROUTER_ASTS, density_function_ast::WrapperType},
        read_data_from_file,
    };

    use super::ProtoChunk;

    const SEED: u64 = 0;
    static RANDOM_CONFIG: LazyLock<GlobalRandomConfig> =
        LazyLock::new(|| GlobalRandomConfig::new(SEED));
    static BASE_NOISE_ROUTER: LazyLock<GlobalProtoNoiseRouter> = LazyLock::new(|| {
        GlobalProtoNoiseRouter::generate(&NOISE_ROUTER_ASTS.overworld, &RANDOM_CONFIG)
    });

    #[test]
    fn test_no_blend_no_beard_only_cell_cache() {
        // We say no wrapper, but it technically has a top-level cell cache
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_only_cell_cache_0_0.chunk");

        let mut base_router = BASE_NOISE_ROUTER.clone();
        base_router
            .component_stack
            .iter_mut()
            .for_each(|component| {
                if let ProtoNoiseFunctionComponent::Wrapper(wrapper) = component {
                    match wrapper.wrapper_type() {
                        WrapperType::CellCache => (),
                        _ => {
                            *component = ProtoNoiseFunctionComponent::PassThrough(PassThrough {
                                input_index: wrapper.input_index(),
                                min_value: wrapper.min(),
                                max_value: wrapper.max(),
                            });
                        }
                    }
                }
            });

        let mut chunk = ProtoChunk::new(Vector2::new(0, 0), &base_router, &RANDOM_CONFIG);
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual.state_id {
                    panic!("{} vs {} ({})", expected, actual.state_id, index);
                }
            });
    }

    #[test]
    fn test_no_blend_no_beard_only_cell_2d_cache() {
        // it technically has a top-level cell cache
        // should be the same as only cell_cache
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_only_cell_cache_0_0.chunk");

        let mut base_router = BASE_NOISE_ROUTER.clone();
        base_router
            .component_stack
            .iter_mut()
            .for_each(|component| {
                if let ProtoNoiseFunctionComponent::Wrapper(wrapper) = component {
                    match wrapper.wrapper_type() {
                        WrapperType::CellCache => (),
                        WrapperType::Cache2D => (),
                        _ => {
                            *component = ProtoNoiseFunctionComponent::PassThrough(PassThrough {
                                input_index: wrapper.input_index(),
                                min_value: wrapper.min(),
                                max_value: wrapper.max(),
                            });
                        }
                    }
                }
            });

        let mut chunk = ProtoChunk::new(Vector2::new(0, 0), &base_router, &RANDOM_CONFIG);
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual.state_id {
                    panic!("{} vs {} ({})", expected, actual.state_id, index);
                }
            });
    }

    #[test]
    fn test_no_blend_no_beard_only_cell_flat_cache() {
        // it technically has a top-level cell cache
        let expected_data: Vec<u16> = read_data_from_file!(
            "../../assets/no_blend_no_beard_only_cell_cache_flat_cache_0_0.chunk"
        );

        let mut base_router = BASE_NOISE_ROUTER.clone();
        base_router
            .component_stack
            .iter_mut()
            .for_each(|component| {
                if let ProtoNoiseFunctionComponent::Wrapper(wrapper) = component {
                    match wrapper.wrapper_type() {
                        WrapperType::CellCache => (),
                        WrapperType::CacheFlat => (),
                        _ => {
                            *component = ProtoNoiseFunctionComponent::PassThrough(PassThrough {
                                input_index: wrapper.input_index(),
                                min_value: wrapper.min(),
                                max_value: wrapper.max(),
                            });
                        }
                    }
                }
            });

        let mut chunk = ProtoChunk::new(Vector2::new(0, 0), &base_router, &RANDOM_CONFIG);
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual.state_id {
                    panic!("{} vs {} ({})", expected, actual.state_id, index);
                }
            });
    }

    #[test]
    fn test_no_blend_no_beard_only_cell_once_cache() {
        // it technically has a top-level cell cache
        let expected_data: Vec<u16> = read_data_from_file!(
            "../../assets/no_blend_no_beard_only_cell_cache_once_cache_0_0.chunk"
        );

        let mut base_router = BASE_NOISE_ROUTER.clone();
        base_router
            .component_stack
            .iter_mut()
            .for_each(|component| {
                if let ProtoNoiseFunctionComponent::Wrapper(wrapper) = component {
                    match wrapper.wrapper_type() {
                        WrapperType::CellCache => (),
                        WrapperType::CacheOnce => (),
                        _ => {
                            *component = ProtoNoiseFunctionComponent::PassThrough(PassThrough {
                                input_index: wrapper.input_index(),
                                min_value: wrapper.min(),
                                max_value: wrapper.max(),
                            });
                        }
                    }
                }
            });

        let mut chunk = ProtoChunk::new(Vector2::new(0, 0), &base_router, &RANDOM_CONFIG);
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual.state_id {
                    panic!("{} vs {} ({})", expected, actual.state_id, index);
                }
            });
    }

    #[test]
    fn test_no_blend_no_beard_only_cell_interpolated() {
        // it technically has a top-level cell cache
        let expected_data: Vec<u16> = read_data_from_file!(
            "../../assets/no_blend_no_beard_only_cell_cache_interpolated_0_0.chunk"
        );

        let mut base_router = BASE_NOISE_ROUTER.clone();
        base_router
            .component_stack
            .iter_mut()
            .for_each(|component| {
                if let ProtoNoiseFunctionComponent::Wrapper(wrapper) = component {
                    match wrapper.wrapper_type() {
                        WrapperType::CellCache => (),
                        WrapperType::Interpolated => (),
                        _ => {
                            *component = ProtoNoiseFunctionComponent::PassThrough(PassThrough {
                                input_index: wrapper.input_index(),
                                min_value: wrapper.min(),
                                max_value: wrapper.max(),
                            });
                        }
                    }
                }
            });

        let mut chunk = ProtoChunk::new(Vector2::new(0, 0), &base_router, &RANDOM_CONFIG);
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual.state_id {
                    panic!("{} vs {} ({})", expected, actual.state_id, index);
                }
            });
    }

    #[test]
    fn test_no_blend_no_beard() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_0_0.chunk");
        let mut chunk = ProtoChunk::new(Vector2::new(0, 0), &BASE_NOISE_ROUTER, &RANDOM_CONFIG);
        chunk.populate_noise();

        assert_eq!(
            expected_data,
            chunk
                .flat_block_map
                .into_iter()
                .map(|state| state.state_id)
                .collect::<Vec<u16>>()
        );
    }

    #[test]
    fn test_no_blend_no_beard_aquifer() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_7_4.chunk");
        let mut chunk = ProtoChunk::new(Vector2::new(7, 4), &BASE_NOISE_ROUTER, &RANDOM_CONFIG);
        chunk.populate_noise();

        assert_eq!(
            expected_data,
            chunk
                .flat_block_map
                .into_iter()
                .map(|state| state.state_id)
                .collect::<Vec<u16>>()
        );
    }
}
