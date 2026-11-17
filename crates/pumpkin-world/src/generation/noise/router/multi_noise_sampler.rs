use pumpkin_data::noise_router::{Axis, WrapperType};
use pumpkin_util::math::vector3::Vector3;

use crate::{
    biome::multi_noise::{NoiseValuePoint, to_long},
    generation::biome_coords,
};

use super::{
    chunk_density_function::{Cache, ChunkSpecificNoiseFunctionComponent},
    chunk_noise_router::ChunkNoiseFunctionComponent,
    density_function::{NoiseFunctionComponentRange, PassThrough},
    density_volume::{DensityBuffer, DensityVolume},
    proto_noise_router::{
        BEARDIFIER_ZERO_CONSTANT, DependentProtoNoiseFunctionComponent,
        IndependentProtoNoiseFunctionComponent, ProtoMultiNoiseRouter, ProtoNoiseFunctionComponent,
    },
};

enum SampledParameter {
    Column2D([i64; 16]),
    Volume3D(DensityBuffer),
}

impl SampledParameter {
    #[inline]
    fn get(&self, index: usize, col_index: usize) -> i64 {
        match self {
            Self::Column2D(cols) => cols[col_index],
            Self::Volume3D(buf) => to_long(buf[index]),
        }
    }
}

pub struct MultiNoiseSampler<'a> {
    temperature: usize,
    humidity: usize,
    continentalness: usize,
    erosion: usize,
    depth: usize,
    // AKA: Weirdness
    ridges: usize,
    base: &'a ProtoMultiNoiseRouter,
    component_stack: Option<Box<[ChunkNoiseFunctionComponent<'a>]>>,
    volume: Option<DensityVolume>,
    buffers: Option<[SampledParameter; 6]>,
}

impl<'a> MultiNoiseSampler<'a> {
    fn ensure_stack(&mut self) -> &mut [ChunkNoiseFunctionComponent<'a>] {
        let base = self.base;
        self.component_stack
            .get_or_insert_with(|| Self::build_component_stack(base))
    }

    pub fn fill_volume(&mut self, volume: DensityVolume) {
        let indices = [
            self.temperature,
            self.humidity,
            self.continentalness,
            self.erosion,
            self.depth,
            self.ridges,
        ];
        let cols_len = volume.size_x * volume.size_z;
        let component_stack = self.ensure_stack();
        let buffers = indices.map(|index| {
            let y_slice_coord = if cols_len <= 16 {
                match &component_stack[index] {
                    ChunkNoiseFunctionComponent::Dependent(
                        DependentProtoNoiseFunctionComponent::Slice(slice),
                    ) if matches!(slice.axis, Axis::Y) => Some(slice.coordinate),
                    ChunkNoiseFunctionComponent::Independent(
                        IndependentProtoNoiseFunctionComponent::Constant(_),
                    ) => Some(volume.min_block_y),
                    _ => None,
                }
            } else {
                None
            };

            if let Some(coord_y) = y_slice_coord {
                let volume_2d = DensityVolume::new(
                    volume.size_x,
                    1,
                    volume.size_z,
                    volume.min_block_x,
                    coord_y,
                    volume.min_block_z,
                    volume.step_block_x,
                    1,
                    volume.step_block_z,
                );
                let mut raw = [0.0f32; 16];
                ChunkNoiseFunctionComponent::sample_volume_from_stack(
                    &mut component_stack[..=index],
                    &mut raw[..cols_len],
                    &volume_2d,
                );
                SampledParameter::Column2D(std::array::from_fn(|i| to_long(raw[i])))
            } else {
                let mut buffer = DensityBuffer::acquire(&volume);
                ChunkNoiseFunctionComponent::sample_volume_from_stack(
                    &mut component_stack[..=index],
                    &mut buffer,
                    &volume,
                );
                SampledParameter::Volume3D(buffer)
            }
        });
        self.volume = Some(volume);
        self.buffers = Some(buffers);
    }

    pub fn sample(&mut self, biome_x: i32, biome_y: i32, biome_z: i32) -> NoiseValuePoint {
        let block_x = biome_coords::to_block(biome_x);
        let block_y = biome_coords::to_block(biome_y);
        let block_z = biome_coords::to_block(biome_z);

        if let (Some(volume), Some(buffers)) = (&self.volume, &self.buffers)
            && let Some(index) = volume.index_of_block(block_x, block_y, block_z)
        {
            let col_index = index / volume.size_y;
            return NoiseValuePoint {
                temperature: buffers[0].get(index, col_index),
                humidity: buffers[1].get(index, col_index),
                continentalness: buffers[2].get(index, col_index),
                erosion: buffers[3].get(index, col_index),
                depth: buffers[4].get(index, col_index),
                weirdness: buffers[5].get(index, col_index),
            };
        }

        let pos = Vector3::new(block_x, block_y, block_z);
        let temperature_idx = self.temperature;
        let humidity_idx = self.humidity;
        let continentalness_idx = self.continentalness;
        let erosion_idx = self.erosion;
        let depth_idx = self.depth;
        let ridges_idx = self.ridges;
        let component_stack = self.ensure_stack();

        let temperature = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=temperature_idx],
            &pos,
        );

        let humidity = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=humidity_idx],
            &pos,
        );

        let continentalness = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=continentalness_idx],
            &pos,
        );

        let erosion = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=erosion_idx],
            &pos,
        );

        let depth = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=depth_idx],
            &pos,
        );

        let weirdness = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=ridges_idx],
            &pos,
        );

        NoiseValuePoint {
            temperature: to_long(temperature),
            humidity: to_long(humidity),
            continentalness: to_long(continentalness),
            erosion: to_long(erosion),
            depth: to_long(depth),
            weirdness: to_long(weirdness),
        }
    }

    pub fn sample_erosion(&mut self, block_x: i32, block_y: i32, block_z: i32) -> f32 {
        let pos = Vector3::new(block_x, block_y, block_z);
        let erosion_idx = self.erosion;
        let component_stack = self.ensure_stack();

        ChunkNoiseFunctionComponent::sample_from_stack(&mut component_stack[..=erosion_idx], &pos)
    }

    #[must_use]
    pub const fn generate(base: &'a ProtoMultiNoiseRouter) -> Self {
        Self {
            temperature: base.temperature,
            humidity: base.vegetation,
            continentalness: base.continents,
            depth: base.depth,
            erosion: base.erosion,
            ridges: base.ridges,
            base,
            component_stack: None,
            volume: None,
            buffers: None,
        }
    }

    fn build_component_stack(
        base: &'a ProtoMultiNoiseRouter,
    ) -> Box<[ChunkNoiseFunctionComponent<'a>]> {
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
                ProtoNoiseFunctionComponent::Beardifier(_) => {
                    ChunkNoiseFunctionComponent::Independent(&BEARDIFIER_ZERO_CONSTANT)
                }
                ProtoNoiseFunctionComponent::Wrapper(wrapper) => {
                    //NOTE: Due to our previous invariant with the proto-function, it is guaranteed
                    // that the wrapped function is already on the stack
                    let min_value = component_stack[wrapper.input_index].min();
                    let max_value = component_stack[wrapper.input_index].max();

                    match wrapper.wrapper_type {
                        WrapperType::Cache => ChunkNoiseFunctionComponent::Chunk(
                            ChunkSpecificNoiseFunctionComponent::Cache(Cache::new(
                                wrapper.input_index,
                                min_value,
                                max_value,
                            )),
                        ),
                        WrapperType::Interpolated { .. } => {
                            ChunkNoiseFunctionComponent::PassThrough(PassThrough::new(
                                wrapper.input_index,
                                min_value,
                                max_value,
                            ))
                        }
                    }
                }
            };
            component_stack.push(chunk_component);
        }

        component_stack.into_boxed_slice()
    }
}

#[cfg(test)]
mod test {
    use pumpkin_data::noise_router::OVERWORLD_BASE_NOISE_ROUTER;

    use crate::{
        GlobalRandomConfig, biome::multi_noise::NoiseValuePoint,
        generation::noise::router::proto_noise_router::ProtoNoiseRouters,
    };

    use super::MultiNoiseSampler;

    #[test]
    fn sample() {
        let seed = 123;
        let random_config = GlobalRandomConfig::new(seed, false);
        let noise_router =
            ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &random_config);
        let mut sampler = MultiNoiseSampler::generate(&noise_router.multi_noise);
        let expected = NoiseValuePoint {
            temperature: -5727,
            humidity: 55,
            continentalness: 4996,
            erosion: 2371,
            depth: -19774,
            weirdness: 4421,
        };
        assert_eq!(sampler.sample(123, 123, 123), expected);
    }

    #[test]
    fn sample_2() {
        // we use a different seed
        let seed = 13579;
        let random_config = GlobalRandomConfig::new(seed, false);
        let noise_router =
            ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &random_config);
        let mut sampler = MultiNoiseSampler::generate(&noise_router.multi_noise);
        let expected = NoiseValuePoint {
            temperature: 7489,
            humidity: 3502,
            continentalness: -2168,
            erosion: -3511,
            depth: -21237,
            weirdness: -5222,
        };
        assert_eq!(sampler.sample(123, 123, 123), expected);
    }
}
