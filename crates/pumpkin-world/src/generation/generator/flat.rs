use super::{FlatDecoration, FlatLayer};
use crate::chunk::ChunkHeightmapType;
use crate::chunk_system::StagedChunkEnum;
use crate::generation::Seed;
use crate::generation::feature::features::fill_layer::FillLayerFeature;
use crate::generation::positions::chunk_pos::{start_block_x, start_block_z};
use crate::generation::proto_chunk::ProtoChunk;
use pumpkin_data::chunk::Biome;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::placed_feature::PlacedFeature;
use pumpkin_data::structures::GenerationStep;
use pumpkin_data::{Block, BlockState};

/// A decoration of a flat world, in the order vanilla places it within its step.
pub(crate) enum FlatFeature {
    Placed(PlacedFeature),
    /// A layer that does not block motion, placed after the terrain like vanilla does.
    FillLayer(FillLayerFeature),
}

pub struct FlatGenerator {
    pub seed: u64,
    pub dimension: Dimension,
    pub layers: Vec<FlatLayer>,
    pub biome: String,
    /// The terrain, bottom up; `None` for the layers placed as [`FlatFeature::FillLayer`].
    terrain: Vec<Option<&'static BlockState>>,
    feature_steps: Vec<Vec<FlatFeature>>,
}

impl FlatGenerator {
    #[must_use]
    pub fn new(
        seed: Seed,
        dimension: Dimension,
        layers: Vec<FlatLayer>,
        biome: String,
        decoration: FlatDecoration,
    ) -> Self {
        let states: Vec<&'static BlockState> = layers
            .iter()
            .flat_map(|layer| {
                let state = Block::from_name(&layer.block)
                    .map_or(Block::AIR.default_state, |block| block.default_state);
                std::iter::repeat_n(state, layer.height.max(0) as usize)
            })
            .take(dimension.height as usize)
            .collect();
        let feature_steps = feature_steps(&states, biome_by_name(&biome), decoration);
        let terrain = states
            .into_iter()
            .map(|state| {
                ChunkHeightmapType::MotionBlocking
                    .is_opaque(state)
                    .then_some(state)
            })
            .collect();
        Self {
            seed: seed.0,
            dimension,
            layers,
            biome,
            terrain,
            feature_steps,
        }
    }

    /// Decorations per generation step, indexed like vanilla's decoration seeds.
    #[must_use]
    pub(crate) fn feature_steps(&self) -> &[Vec<FlatFeature>] {
        &self.feature_steps
    }

    pub fn step_to_biomes(&self, chunk: &mut ProtoChunk) {
        chunk.flat_biome_map.fill(biome_by_name(&self.biome).id);
        chunk.stage = StagedChunkEnum::Biomes;
    }

    pub fn step_to_noise(&self, chunk: &mut ProtoChunk) {
        let start_x = start_block_x(chunk.x);
        let start_z = start_block_z(chunk.z);
        let bottom_y = chunk.bottom_y() as i32;
        for (y, state) in (bottom_y..).zip(&self.terrain) {
            let Some(state) = state else { continue };
            for x in 0..16 {
                for z in 0..16 {
                    chunk.set_block_state(start_x + x, y, start_z + z, state);
                }
            }
        }
        chunk.stage = StagedChunkEnum::Noise;
    }

    pub const fn step_to_surface(&self, chunk: &mut ProtoChunk) {
        chunk.stage = StagedChunkEnum::Surface;
    }

    pub const fn step_to_carvers(&self, chunk: &mut ProtoChunk) {
        chunk.stage = StagedChunkEnum::Carvers;
    }
}

fn biome_by_name(name: &str) -> &'static Biome {
    Biome::from_name(name.strip_prefix("minecraft:").unwrap_or(name)).unwrap_or(&Biome::PLAINS)
}

/// Vanilla `FlatLevelGeneratorSettings.adjustGenerationSettings` for the world's own biome.
fn feature_steps(
    layers: &[&'static BlockState],
    biome: &'static Biome,
    FlatDecoration { features, lakes }: FlatDecoration,
) -> Vec<Vec<FlatFeature>> {
    const STEPS: usize = GenerationStep::TopLayerModification.ordinal() + 1;
    let mut steps: Vec<Vec<FlatFeature>> = (0..STEPS).map(|_| Vec::new()).collect();

    if lakes {
        steps[GenerationStep::Lakes.ordinal()].extend([
            FlatFeature::Placed(PlacedFeature::LakeLavaUnderground),
            FlatFeature::Placed(PlacedFeature::LakeLavaSurface),
        ]);
    }

    // Vanilla `voidGen`: a world made only of air is decorated only in the void biome.
    let void_gen = layers.iter().all(|state| state.is_air());
    if features && (!void_gen || biome.id == Biome::THE_VOID.id) {
        for (step, placed) in biome.features.iter().enumerate().take(STEPS) {
            let skipped = step == GenerationStep::UndergroundStructures.ordinal()
                || step == GenerationStep::SurfaceStructures.ordinal()
                || (lakes && step == GenerationStep::Lakes.ordinal());
            if !skipped {
                steps[step].extend(placed.iter().copied().map(FlatFeature::Placed));
            }
        }
    }

    for (height, &state) in layers.iter().enumerate() {
        if !ChunkHeightmapType::MotionBlocking.is_opaque(state) {
            steps[GenerationStep::TopLayerModification.ordinal()].push(FlatFeature::FillLayer(
                FillLayerFeature {
                    height: height as i32,
                    state,
                },
            ));
        }
    }
    steps
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer(block: &str, height: i32) -> FlatLayer {
        FlatLayer {
            block: block.to_string(),
            height,
        }
    }

    fn placed(generator: &FlatGenerator, step: GenerationStep) -> Vec<PlacedFeature> {
        generator.feature_steps()[step.ordinal()]
            .iter()
            .filter_map(|feature| match feature {
                FlatFeature::Placed(placed) => Some(*placed),
                FlatFeature::FillLayer(_) => None,
            })
            .collect()
    }

    fn flat(layers: Vec<FlatLayer>, biome: &str, features: bool, lakes: bool) -> FlatGenerator {
        FlatGenerator::new(
            Seed(0),
            Dimension::OVERWORLD,
            layers,
            biome.to_string(),
            FlatDecoration { features, lakes },
        )
    }

    #[test]
    fn the_void_preset_places_the_start_platform() {
        let the_void = flat(
            vec![layer("minecraft:air", 1)],
            "minecraft:the_void",
            true,
            false,
        );

        assert_eq!(
            placed(&the_void, GenerationStep::TopLayerModification),
            vec![PlacedFeature::VoidStartPlatform]
        );
        // The air layer turns into a fill-layer decoration, as in vanilla.
        assert!(the_void.terrain.iter().all(Option::is_none));
    }

    #[test]
    fn a_world_of_air_only_decorates_the_void_biome() {
        let air_plains = flat(
            vec![layer("minecraft:air", 1)],
            "minecraft:plains",
            true,
            false,
        );

        assert!(placed(&air_plains, GenerationStep::VegetalDecoration).is_empty());
    }

    #[test]
    fn decorations_skip_structures_and_add_lakes_first() {
        let plains = flat(
            vec![layer("minecraft:stone", 3)],
            "minecraft:plains",
            true,
            true,
        );

        assert_eq!(
            placed(&plains, GenerationStep::Lakes),
            vec![
                PlacedFeature::LakeLavaUnderground,
                PlacedFeature::LakeLavaSurface
            ]
        );
        assert!(placed(&plains, GenerationStep::SurfaceStructures).is_empty());
        assert_eq!(
            placed(&plains, GenerationStep::VegetalDecoration),
            Biome::PLAINS.features[GenerationStep::VegetalDecoration.ordinal()]
        );
    }

    #[test]
    fn without_features_only_non_solid_layers_are_decorations() {
        let snowy = flat(
            vec![layer("minecraft:stone", 1), layer("minecraft:snow", 1)],
            "minecraft:snowy_plains",
            false,
            false,
        );
        let top = &snowy.feature_steps()[GenerationStep::TopLayerModification.ordinal()];

        assert!(matches!(
            top.as_slice(),
            [FlatFeature::FillLayer(FillLayerFeature { height: 1, .. })]
        ));
        assert!(snowy.terrain[0].is_some() && snowy.terrain[1].is_none());
    }
}
