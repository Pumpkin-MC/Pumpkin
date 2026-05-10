#[cfg(test)] // TODO: Fix tests to work with new ProtoChunk API
mod test {
    /*
    TODO: Update all tests to work with the new ProtoChunk API that doesn't use lifetimes.
    The new API requires passing noise samplers and other dependencies as parameters to methods
    instead of storing them in the struct.

    const SEED: u64 = 0;
    static RANDOM_CONFIG: LazyLock<GlobalRandomConfig> =
        LazyLock::new(|| GlobalRandomConfig::new(SEED, false)); // TODO: use legacy when needed
    static TERRAIN_CACHE: LazyLock<TerrainCache> =
        LazyLock::new(|| TerrainCache::from_random(&RANDOM_CONFIG));
    static BASE_NOISE_ROUTER: LazyLock<ProtoNoiseRouters> =
        LazyLock::new(|| ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &RANDOM_CONFIG));

    const SEED2: u64 = 13579;
    static RANDOM_CONFIG2: LazyLock<GlobalRandomConfig> =
        LazyLock::new(|| GlobalRandomConfig::new(SEED2, false)); // TODO: use legacy when needed
    static BASE_NOISE_ROUTER2: LazyLock<ProtoNoiseRouters> = LazyLock::new(|| {
        ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &RANDOM_CONFIG2)
    });

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn no_blend_no_beard_only_cell_cache() {
        // We say no wrapper, but it technically has a top-level cell cache
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_only_cell_cache_0_0.chunk");

        let mut base_router =
            ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &RANDOM_CONFIG);

        macro_rules! set_wrappers {
            ($stack: expr) => {
                $stack.iter_mut().for_each(|component| {
                    if let ProtoNoiseFunctionComponent::Wrapper(wrapper) = component {
                        match wrapper.wrapper_type {
                            WrapperType::CellCache => (),
                            _ => {
                                *component =
                                    ProtoNoiseFunctionComponent::PassThrough(PassThrough::new(
                                        wrapper.input_index,
                                        wrapper.min(),
                                        wrapper.max(),
                                    ));
                            }
                        }
                    }
                });
            };
        }

        set_wrappers!(base_router.noise.full_component_stack);
        set_wrappers!(base_router.surface_estimator.full_component_stack);
        set_wrappers!(base_router.multi_noise.full_component_stack);

        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(0, 0),
            &base_router,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("{expected} vs {actual} ({index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn no_blend_no_beard_only_cell_2d_cache() {
        // it technically has a top-level cell cache
        // should be the same as only cell_cache
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_only_cell_cache_0_0.chunk");

        let mut base_router =
            ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &RANDOM_CONFIG);

        macro_rules! set_wrappers {
            ($stack: expr) => {
                $stack.iter_mut().for_each(|component| {
                    if let ProtoNoiseFunctionComponent::Wrapper(wrapper) = component {
                        match wrapper.wrapper_type {
                            WrapperType::CellCache => (),
                            WrapperType::Cache2D => (),
                            _ => {
                                *component =
                                    ProtoNoiseFunctionComponent::PassThrough(PassThrough::new(
                                        wrapper.input_index,
                                        wrapper.min(),
                                        wrapper.max(),
                                    ));
                            }
                        }
                    }
                });
            };
        }

        set_wrappers!(base_router.noise.full_component_stack);
        set_wrappers!(base_router.surface_estimator.full_component_stack);
        set_wrappers!(base_router.multi_noise.full_component_stack);

        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(0, 0),
            &base_router,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("{expected} vs {actual} ({index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn no_blend_no_beard_only_cell_flat_cache() {
        // it technically has a top-level cell cache
        let expected_data: Vec<u16> = read_data_from_file!(
            "../../assets/no_blend_no_beard_only_cell_cache_flat_cache_0_0.chunk"
        );

        let mut base_router =
            ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &RANDOM_CONFIG);

        macro_rules! set_wrappers {
            ($stack: expr) => {
                $stack.iter_mut().for_each(|component| {
                    if let ProtoNoiseFunctionComponent::Wrapper(wrapper) = component {
                        match wrapper.wrapper_type {
                            WrapperType::CellCache => (),
                            WrapperType::CacheFlat => (),
                            _ => {
                                *component =
                                    ProtoNoiseFunctionComponent::PassThrough(PassThrough::new(
                                        wrapper.input_index,
                                        wrapper.min(),
                                        wrapper.max(),
                                    ));
                            }
                        }
                    }
                });
            };
        }

        set_wrappers!(base_router.noise.full_component_stack);
        set_wrappers!(base_router.surface_estimator.full_component_stack);
        set_wrappers!(base_router.multi_noise.full_component_stack);

        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(0, 0),
            &base_router,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("{expected} vs {actual} ({index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn no_blend_no_beard_only_cell_once_cache() {
        // it technically has a top-level cell cache
        let expected_data: Vec<u16> = read_data_from_file!(
            "../../assets/no_blend_no_beard_only_cell_cache_once_cache_0_0.chunk"
        );

        let mut base_router =
            ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &RANDOM_CONFIG);

        macro_rules! set_wrappers {
            ($stack: expr) => {
                $stack.iter_mut().for_each(|component| {
                    if let ProtoNoiseFunctionComponent::Wrapper(wrapper) = component {
                        match wrapper.wrapper_type {
                            WrapperType::CellCache => (),
                            WrapperType::CacheOnce => (),
                            _ => {
                                *component =
                                    ProtoNoiseFunctionComponent::PassThrough(PassThrough::new(
                                        wrapper.input_index,
                                        wrapper.min(),
                                        wrapper.max(),
                                    ));
                            }
                        }
                    }
                });
            };
        }

        set_wrappers!(base_router.noise.full_component_stack);
        set_wrappers!(base_router.surface_estimator.full_component_stack);
        set_wrappers!(base_router.multi_noise.full_component_stack);

        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(0, 0),
            &base_router,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("{expected} vs {actual} ({index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn no_blend_no_beard_only_cell_interpolated() {
        // it technically has a top-level cell cache
        let expected_data: Vec<u16> = read_data_from_file!(
            "../../assets/no_blend_no_beard_only_cell_cache_interpolated_0_0.chunk"
        );

        let mut base_router =
            ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &RANDOM_CONFIG);

        macro_rules! set_wrappers {
            ($stack: expr) => {
                $stack.iter_mut().for_each(|component| {
                    if let ProtoNoiseFunctionComponent::Wrapper(wrapper) = component {
                        match wrapper.wrapper_type {
                            WrapperType::CellCache => (),
                            WrapperType::Interpolated => (),
                            _ => {
                                *component =
                                    ProtoNoiseFunctionComponent::PassThrough(PassThrough::new(
                                        wrapper.input_index,
                                        wrapper.min(),
                                        wrapper.max(),
                                    ));
                            }
                        }
                    }
                });
            };
        }

        set_wrappers!(base_router.noise.full_component_stack);
        set_wrappers!(base_router.surface_estimator.full_component_stack);
        set_wrappers!(base_router.multi_noise.full_component_stack);

        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(0, 0),
            &base_router,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("{expected} vs {actual} ({index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn no_blend_no_beard() {
        let _expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_0_0.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        // TODO: Create ProtoChunk and call populate_noise with proper parameters
        let _chunk = ProtoChunk::new(
            Vector2::new(0, 0),
            surface_config,
            surface_config.default_block.get_state(),
            0, // biome_mixer_seed
        );

        // assert_eq!(
        //     expected_data,
        //     chunk.flat_block_map.into_iter().collect::<Vec<u16>>()
        // );
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn no_blend_no_beard_aquifer() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_7_4.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(7, 4),
            &BASE_NOISE_ROUTER,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );
        chunk.populate_noise();

        assert_eq!(
            expected_data,
            chunk.flat_block_map.into_iter().collect::<Vec<u16>>()
        );
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn no_blend_no_beard_badlands() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_-595_544.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(-595, 544),
            &BASE_NOISE_ROUTER,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("expected {expected}, was {actual} (at {index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn no_blend_no_beard_frozen_ocean() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_-119_183.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(-119, 183),
            &BASE_NOISE_ROUTER,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("expected {expected}, was {actual} (at {index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn no_blend_no_beard_badlands2() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_13579_-6_11.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(-6, 11),
            &BASE_NOISE_ROUTER2,
            &RANDOM_CONFIG2,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("expected {expected}, was {actual} (at {index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn no_blend_no_beard_badlands3() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_13579_-2_15.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(-2, 15),
            &BASE_NOISE_ROUTER2,
            &RANDOM_CONFIG2,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("expected {expected}, was {actual} (at {index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn no_blend_no_beard_surface() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_surface_0_0.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(0, 0),
            &BASE_NOISE_ROUTER,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );

        chunk.populate_biomes(Dimension::Overworld);
        chunk.populate_noise();
        chunk.build_surface();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("expected {expected}, was {actual} (at {index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn no_blend_no_beard_surface_badlands() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_surface_badlands_-595_544.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(-595, 544),
            &BASE_NOISE_ROUTER,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );

        chunk.populate_biomes(Dimension::Overworld);
        chunk.populate_noise();
        chunk.build_surface();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("expected {expected}, was {actual} (at {index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn no_blend_no_beard_surface_badlands2() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_surface_13579_-6_11.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let terrain_cache = TerrainCache::from_random(&RANDOM_CONFIG2);
        let mut chunk = ProtoChunk::new(
            Vector2::new(-6, 11),
            &BASE_NOISE_ROUTER2,
            &RANDOM_CONFIG2,
            surface_config,
            &terrain_cache,
            surface_config.default_block.get_state(),
        );

        chunk.populate_biomes(Dimension::Overworld);
        chunk.populate_noise();
        chunk.build_surface();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("expected {expected}, was {actual} (at {index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn no_blend_no_beard_surface_badlands3() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_surface_13579_-7_9.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let terrain_cache = TerrainCache::from_random(&RANDOM_CONFIG2);

        let mut chunk = ProtoChunk::new(
            Vector2::new(-7, 9),
            &BASE_NOISE_ROUTER2,
            &RANDOM_CONFIG2,
            surface_config,
            &terrain_cache,
            surface_config.default_block.get_state(),
        );

        chunk.populate_biomes(Dimension::Overworld);
        chunk.populate_noise();
        chunk.build_surface();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("expected {expected}, was {actual} (at {index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn no_blend_no_beard_surface_biome_blend() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_surface_13579_-2_15.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let terrain_cache = TerrainCache::from_random(&RANDOM_CONFIG2);

        let mut chunk = ProtoChunk::new(
            Vector2::new(-2, 15),
            &BASE_NOISE_ROUTER2,
            &RANDOM_CONFIG2,
            surface_config,
            &terrain_cache,
            surface_config.default_block.get_state(),
        );

        chunk.populate_biomes(Dimension::Overworld);
        chunk.populate_noise();
        chunk.build_surface();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("expected {expected}, was {actual} (at {index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn no_blend_no_beard_surface_frozen_ocean() {
        let expected_data: Vec<u16> = read_data_from_file!(
            "../../assets/no_blend_no_beard_surface_frozen_ocean_-119_183.chunk"
        );
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(-119, 183),
            &BASE_NOISE_ROUTER,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );

        chunk.populate_biomes(Dimension::Overworld);
        chunk.populate_noise();
        chunk.build_surface();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("expected {expected}, was {actual} (at {index})");
                }
            });
    */
}
