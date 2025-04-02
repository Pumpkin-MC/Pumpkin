use pumpkin_data::chunk::Biome;
use serde::{Deserialize, Serialize};

pub fn to_long(float: f32) -> i64 {
    (float * 10000f32) as i64
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct NoiseValuePoint {
    pub temperature: i64,
    pub humidity: i64,
    pub continentalness: i64,
    pub erosion: i64,
    pub depth: i64,
    pub weirdness: i64,
}

impl NoiseValuePoint {
    pub fn convert_to_list(&self) -> [i64; 7] {
        [
            self.temperature,
            self.humidity,
            self.continentalness,
            self.erosion,
            self.depth,
            self.weirdness,
            0,
        ]
    }
}

#[derive(Deserialize, PartialEq)]
pub struct ParameterRange {
    min: i64,
    max: i64,
}

impl ParameterRange {
    fn calc_distance(&self, noise: i64) -> i64 {
        if noise > self.max {
            noise - self.max
        } else if noise < self.min {
            self.min - noise
        } else {
            0
        }
    }

    pub fn combine(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}

#[derive(Deserialize, PartialEq)]
#[serde(tag = "_type", rename_all = "lowercase")]
pub enum BiomeTree {
    Leaf {
        parameters: [ParameterRange; 7],
        biome: &'static Biome,
    },
    Branch {
        parameters: [ParameterRange; 7],
        #[serde(rename = "subTree")]
        nodes: Box<[BiomeTree]>,
    },
}

impl BiomeTree {
    pub fn get<'a>(
        &'a self,
        point: &NoiseValuePoint,
        previous_result_node: &mut Option<&'a BiomeTree>,
    ) -> &'static Biome {
        let point_as_list = point.convert_to_list();
        let result_node = self.get_resulting_node(&point_as_list, *previous_result_node);
        match result_node {
            BiomeTree::Leaf { biome, .. } => {
                *previous_result_node = Some(result_node);
                biome
            }
            _ => unreachable!(),
        }
    }

    fn get_resulting_node<'a>(
        &'a self,
        point_list: &[i64; 7],
        previous_result_node: Option<&'a BiomeTree>,
    ) -> &'a BiomeTree {
        match self {
            Self::Leaf { .. } => self,
            Self::Branch { nodes, .. } => {
                let mut distance = previous_result_node
                    .map(|node| node.get_squared_distance(point_list))
                    .unwrap_or(i64::MAX);
                let mut best_node = previous_result_node;

                for node in nodes {
                    let node_distance = node.get_squared_distance(point_list);
                    if distance > node_distance {
                        let node2 = node.get_resulting_node(point_list, best_node);
                        let node2_distance = if node == node2 {
                            node_distance
                        } else {
                            node2.get_squared_distance(point_list)
                        };

                        if distance > node2_distance {
                            distance = node2_distance;
                            best_node = Some(node2);
                        }
                    }
                }

                best_node.expect("This should be populated after traversing the tree")
            }
        }
    }

    fn get_squared_distance(&self, point_list: &[i64; 7]) -> i64 {
        let parameters = match self {
            Self::Leaf { parameters, .. } => parameters,
            Self::Branch { parameters, .. } => parameters,
        };

        parameters
            .iter()
            .zip(point_list)
            .map(|(bound, value)| {
                let distance = bound.calc_distance(*value);
                distance * distance
            })
            .sum()
    }
}

#[cfg(test)]
mod test {
    use pumpkin_data::chunk::Biome;
    use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};

    use crate::{
        GENERATION_SETTINGS, GeneratorSetting, GlobalProtoNoiseRouter, GlobalRandomConfig,
        NOISE_ROUTER_ASTS, ProtoChunk,
        biome::{BiomeSupplier, MultiNoiseBiomeSupplier},
        generation::noise_router::multi_noise_sampler::{
            MultiNoiseSampler, MultiNoiseSamplerBuilderOptions,
        },
        read_data_from_file,
    };

    #[test]
    fn test_sample_value() {
        type PosToPoint = (i32, i32, i32, i64, i64, i64, i64, i64, i64);
        let expected_data: Vec<PosToPoint> =
            read_data_from_file!("../../assets/multi_noise_sample_no_blend_no_beard_0_0_0.json");

        let seed = 0;
        let chunk_pos = Vector2::new(0, 0);
        let random_config = GlobalRandomConfig::new(seed, false);
        let noise_rounter =
            GlobalProtoNoiseRouter::generate(&NOISE_ROUTER_ASTS.overworld, &random_config);

        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();

        let mut chunk = ProtoChunk::new(chunk_pos, &noise_rounter, &random_config, surface_config);

        for (x, y, z, tem, hum, con, ero, dep, wei) in expected_data.into_iter() {
            let point = chunk.multi_noise_sampler.sample(x, y, z);
            assert_eq!(point.temperature, tem);
            assert_eq!(point.humidity, hum);
            assert_eq!(point.continentalness, con);
            assert_eq!(point.erosion, ero);
            assert_eq!(point.depth, dep);
            assert_eq!(point.weirdness, wei);
        }
    }

    #[test]
    fn test_sample_multinoise_biome() {
        let expected_data: Vec<(i32, i32, i32, u8)> =
            read_data_from_file!("../../assets/multi_noise_biome_source_test.json");

        let seed = 0;
        let random_config = GlobalRandomConfig::new(seed, false);
        let noise_router =
            GlobalProtoNoiseRouter::generate(&NOISE_ROUTER_ASTS.overworld, &random_config);

        let mut sampler = MultiNoiseSampler::generate(
            &noise_router,
            &MultiNoiseSamplerBuilderOptions::new(0, 0, 4),
        );

        for (x, y, z, biome_id) in expected_data {
            let global_biome_pos = Vector3::new(x, y, z);
            let calculated_biome = MultiNoiseBiomeSupplier::biome(&global_biome_pos, &mut sampler);

            assert_eq!(
                biome_id,
                calculated_biome.id,
                "Expected {:?} was {:?} at {},{},{}",
                Biome::from_id(biome_id),
                calculated_biome,
                x,
                y,
                z
            );
        }
    }
}
