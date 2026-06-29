/* This file is generated. Do not edit manually. */
use pumpkin_util::math::floor_div;
use pumpkin_util::random::{
    RandomGenerator, RandomImpl, get_carver_seed, get_region_seed, legacy_rand::LegacyRand,
    xoroshiro128::Xoroshiro,
};
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StructureKeys {
    PillagerOutpost,
    Mineshaft,
    MineshaftMesa,
    Mansion,
    JunglePyramid,
    DesertPyramid,
    Igloo,
    Shipwreck,
    ShipwreckBeached,
    SwampHut,
    Stronghold,
    Monument,
    OceanRuinCold,
    OceanRuinWarm,
    Fortress,
    NetherFossil,
    EndCity,
    BuriedTreasure,
    BastionRemnant,
    VillagePlains,
    VillageDesert,
    VillageSavanna,
    VillageSnowy,
    VillageTaiga,
    RuinedPortal,
    RuinedPortalDesert,
    RuinedPortalJungle,
    RuinedPortalSwamp,
    RuinedPortalMountain,
    RuinedPortalOcean,
    RuinedPortalNether,
    AncientCity,
    TrailRuins,
    TrialChambers,
}
pub struct StructureSet {
    pub placement: StructurePlacement,
    pub structures: &'static [WeightedEntry],
}
#[derive(Clone)]
pub struct WeightedEntry {
    pub structure: StructureKeys,
    pub weight: u32,
}
pub struct ExclusionZone {
    pub other_set: &'static str,
    pub chunk_count: i32,
}
pub struct StructurePlacement {
    pub frequency_reduction_method: Option<FrequencyReductionMethod>,
    pub frequency: Option<f32>,
    pub salt: u32,
    pub exclusion_zone: Option<ExclusionZone>,
    pub placement_type: StructurePlacementType,
}
#[derive(Clone, Copy)]
pub enum FrequencyReductionMethod {
    Default,
    LegacyType1,
    LegacyType2,
    LegacyType3,
}
pub enum StructurePlacementType {
    RandomSpread(RandomSpreadStructurePlacement),
    ConcentricRings(ConcentricRingsStructurePlacement),
}
pub struct RandomSpreadStructurePlacement {
    pub spacing: i32,
    pub separation: i32,
    pub spread_type: Option<SpreadType>,
}
pub struct ConcentricRingsStructurePlacement {
    pub spread: i32,
    pub distance: i32,
    pub count: i32,
    pub preferred_biomes: &'static str,
}
#[derive(Clone, Copy)]
pub enum SpreadType {
    Linear,
    Triangular,
}
impl SpreadType {
    pub fn get(&self, random: &mut RandomGenerator, bound: i32) -> i32 {
        match self {
            Self::Linear => random.next_bounded_i32(bound),
            Self::Triangular => i32::midpoint(
                random.next_bounded_i32(bound),
                random.next_bounded_i32(bound),
            ),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerrainAdaptation {
    None,
    BeardThin,
    BeardBox,
    Bury,
    Encapsulate,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StructureType {
    Jigsaw,
    BuriedTreasure,
    DesertPyramid,
    EndCity,
    Fortress,
    Igloo,
    JungleTemple,
    WoodlandMansion,
    Mineshaft,
    OceanMonument,
    NetherFossil,
    OceanRuin,
    RuinedPortal,
    Shipwreck,
    Stronghold,
    SwampHut,
    Unknown,
}
pub struct Structure {
    pub biomes: &'static str,
    pub step: GenerationStep,
    pub start_pool: Option<&'static str>,
    pub size: Option<i32>,
    pub terrain_adaptation: TerrainAdaptation,
    pub start_height: Option<i16>,
    pub project_start_to_heightmap: Option<&'static str>,
    pub max_distance_from_center: Option<i32>,
    pub liquid_settings: Option<&'static str>,
    pub dimension_padding: Option<i32>,
    pub use_expansion_hack: Option<bool>,
    pub structure_type: StructureType,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenerationStep {
    RawGeneration,
    Lakes,
    LocalModifications,
    UndergroundStructures,
    SurfaceStructures,
    Strongholds,
    UndergroundOres,
    UndergroundDecoration,
    FluidSprings,
    VegetalDecoration,
    TopLayerModification,
}
impl GenerationStep {
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        *self as usize
    }
}
pub struct StructurePlacementCalculator {
    pub seed: i64,
}
impl StructurePlacementCalculator {
    #[must_use]
    pub const fn new(seed: i64) -> Self {
        Self { seed }
    }
}
impl Structure {
    pub const ANCIENT_CITY: Self = Structure {
        biomes: "#minecraft:has_structure/ancient_city",
        step: GenerationStep::UndergroundDecoration,
        start_pool: Some("minecraft:ancient_city/city_center"),
        size: Some(7i32),
        terrain_adaptation: TerrainAdaptation::BeardBox,
        start_height: Some(-27i16),
        project_start_to_heightmap: None,
        max_distance_from_center: Some(116i32),
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: Some(false),
        structure_type: StructureType::Jigsaw,
    };
    pub const BASTION_REMNANT: Self = Structure {
        biomes: "#minecraft:has_structure/bastion_remnant",
        step: GenerationStep::SurfaceStructures,
        start_pool: Some("minecraft:bastion/starts"),
        size: Some(6i32),
        terrain_adaptation: TerrainAdaptation::None,
        start_height: Some(33i16),
        project_start_to_heightmap: None,
        max_distance_from_center: Some(80i32),
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: Some(false),
        structure_type: StructureType::Jigsaw,
    };
    pub const BURIED_TREASURE: Self = Structure {
        biomes: "#minecraft:has_structure/buried_treasure",
        step: GenerationStep::UndergroundStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::BuriedTreasure,
    };
    pub const DESERT_PYRAMID: Self = Structure {
        biomes: "#minecraft:has_structure/desert_pyramid",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::DesertPyramid,
    };
    pub const END_CITY: Self = Structure {
        biomes: "#minecraft:has_structure/end_city",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::EndCity,
    };
    pub const FORTRESS: Self = Structure {
        biomes: "#minecraft:has_structure/nether_fortress",
        step: GenerationStep::UndergroundDecoration,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::Fortress,
    };
    pub const IGLOO: Self = Structure {
        biomes: "#minecraft:has_structure/igloo",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::Igloo,
    };
    pub const JUNGLE_PYRAMID: Self = Structure {
        biomes: "#minecraft:has_structure/jungle_temple",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::JungleTemple,
    };
    pub const MANSION: Self = Structure {
        biomes: "#minecraft:has_structure/woodland_mansion",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::WoodlandMansion,
    };
    pub const MINESHAFT: Self = Structure {
        biomes: "#minecraft:has_structure/mineshaft",
        step: GenerationStep::UndergroundStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::Mineshaft,
    };
    pub const MINESHAFT_MESA: Self = Structure {
        biomes: "#minecraft:has_structure/mineshaft_mesa",
        step: GenerationStep::UndergroundStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::Mineshaft,
    };
    pub const MONUMENT: Self = Structure {
        biomes: "#minecraft:has_structure/ocean_monument",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::OceanMonument,
    };
    pub const NETHER_FOSSIL: Self = Structure {
        biomes: "#minecraft:has_structure/nether_fossil",
        step: GenerationStep::UndergroundDecoration,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::BeardThin,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::NetherFossil,
    };
    pub const OCEAN_RUIN_COLD: Self = Structure {
        biomes: "#minecraft:has_structure/ocean_ruin_cold",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::OceanRuin,
    };
    pub const OCEAN_RUIN_WARM: Self = Structure {
        biomes: "#minecraft:has_structure/ocean_ruin_warm",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::OceanRuin,
    };
    pub const PILLAGER_OUTPOST: Self = Structure {
        biomes: "#minecraft:has_structure/pillager_outpost",
        step: GenerationStep::SurfaceStructures,
        start_pool: Some("minecraft:pillager_outpost/base_plates"),
        size: Some(7i32),
        terrain_adaptation: TerrainAdaptation::BeardThin,
        start_height: Some(0i16),
        project_start_to_heightmap: Some("WORLD_SURFACE_WG"),
        max_distance_from_center: Some(80i32),
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: Some(true),
        structure_type: StructureType::Jigsaw,
    };
    pub const RUINED_PORTAL: Self = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_standard",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::RuinedPortal,
    };
    pub const RUINED_PORTAL_DESERT: Self = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_desert",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::RuinedPortal,
    };
    pub const RUINED_PORTAL_JUNGLE: Self = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_jungle",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::RuinedPortal,
    };
    pub const RUINED_PORTAL_MOUNTAIN: Self = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_mountain",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::RuinedPortal,
    };
    pub const RUINED_PORTAL_NETHER: Self = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_nether",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::RuinedPortal,
    };
    pub const RUINED_PORTAL_OCEAN: Self = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_ocean",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::RuinedPortal,
    };
    pub const RUINED_PORTAL_SWAMP: Self = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_swamp",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::RuinedPortal,
    };
    pub const SHIPWRECK: Self = Structure {
        biomes: "#minecraft:has_structure/shipwreck",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::Shipwreck,
    };
    pub const SHIPWRECK_BEACHED: Self = Structure {
        biomes: "#minecraft:has_structure/shipwreck_beached",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::Shipwreck,
    };
    pub const STRONGHOLD: Self = Structure {
        biomes: "#minecraft:has_structure/stronghold",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::Bury,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::Stronghold,
    };
    pub const SWAMP_HUT: Self = Structure {
        biomes: "#minecraft:has_structure/swamp_hut",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        structure_type: StructureType::SwampHut,
    };
    pub const TRAIL_RUINS: Self = Structure {
        biomes: "#minecraft:has_structure/trail_ruins",
        step: GenerationStep::UndergroundStructures,
        start_pool: Some("minecraft:trail_ruins/tower"),
        size: Some(7i32),
        terrain_adaptation: TerrainAdaptation::Bury,
        start_height: Some(-15i16),
        project_start_to_heightmap: Some("WORLD_SURFACE_WG"),
        max_distance_from_center: Some(80i32),
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: Some(false),
        structure_type: StructureType::Jigsaw,
    };
    pub const TRIAL_CHAMBERS: Self = Structure {
        biomes: "#minecraft:has_structure/trial_chambers",
        step: GenerationStep::UndergroundStructures,
        start_pool: Some("minecraft:trial_chambers/chamber/end"),
        size: Some(20i32),
        terrain_adaptation: TerrainAdaptation::Encapsulate,
        start_height: Some(-40i16),
        project_start_to_heightmap: None,
        max_distance_from_center: Some(116i32),
        liquid_settings: Some("ignore_waterlogging"),
        dimension_padding: Some(10i32),
        use_expansion_hack: Some(false),
        structure_type: StructureType::Jigsaw,
    };
    pub const VILLAGE_DESERT: Self = Structure {
        biomes: "#minecraft:has_structure/village_desert",
        step: GenerationStep::SurfaceStructures,
        start_pool: Some("minecraft:village/desert/town_centers"),
        size: Some(6i32),
        terrain_adaptation: TerrainAdaptation::BeardThin,
        start_height: Some(0i16),
        project_start_to_heightmap: Some("WORLD_SURFACE_WG"),
        max_distance_from_center: Some(80i32),
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: Some(true),
        structure_type: StructureType::Jigsaw,
    };
    pub const VILLAGE_PLAINS: Self = Structure {
        biomes: "#minecraft:has_structure/village_plains",
        step: GenerationStep::SurfaceStructures,
        start_pool: Some("minecraft:village/plains/town_centers"),
        size: Some(6i32),
        terrain_adaptation: TerrainAdaptation::BeardThin,
        start_height: Some(0i16),
        project_start_to_heightmap: Some("WORLD_SURFACE_WG"),
        max_distance_from_center: Some(80i32),
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: Some(true),
        structure_type: StructureType::Jigsaw,
    };
    pub const VILLAGE_SAVANNA: Self = Structure {
        biomes: "#minecraft:has_structure/village_savanna",
        step: GenerationStep::SurfaceStructures,
        start_pool: Some("minecraft:village/savanna/town_centers"),
        size: Some(6i32),
        terrain_adaptation: TerrainAdaptation::BeardThin,
        start_height: Some(0i16),
        project_start_to_heightmap: Some("WORLD_SURFACE_WG"),
        max_distance_from_center: Some(80i32),
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: Some(true),
        structure_type: StructureType::Jigsaw,
    };
    pub const VILLAGE_SNOWY: Self = Structure {
        biomes: "#minecraft:has_structure/village_snowy",
        step: GenerationStep::SurfaceStructures,
        start_pool: Some("minecraft:village/snowy/town_centers"),
        size: Some(6i32),
        terrain_adaptation: TerrainAdaptation::BeardThin,
        start_height: Some(0i16),
        project_start_to_heightmap: Some("WORLD_SURFACE_WG"),
        max_distance_from_center: Some(80i32),
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: Some(true),
        structure_type: StructureType::Jigsaw,
    };
    pub const VILLAGE_TAIGA: Self = Structure {
        biomes: "#minecraft:has_structure/village_taiga",
        step: GenerationStep::SurfaceStructures,
        start_pool: Some("minecraft:village/taiga/town_centers"),
        size: Some(6i32),
        terrain_adaptation: TerrainAdaptation::BeardThin,
        start_height: Some(0i16),
        project_start_to_heightmap: Some("WORLD_SURFACE_WG"),
        max_distance_from_center: Some(80i32),
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: Some(true),
        structure_type: StructureType::Jigsaw,
    };
    #[must_use]
    pub const fn get(key: &StructureKeys) -> &'static Self {
        match *key {
            StructureKeys::AncientCity => &Self::ANCIENT_CITY,
            StructureKeys::BastionRemnant => &Self::BASTION_REMNANT,
            StructureKeys::BuriedTreasure => &Self::BURIED_TREASURE,
            StructureKeys::DesertPyramid => &Self::DESERT_PYRAMID,
            StructureKeys::EndCity => &Self::END_CITY,
            StructureKeys::Fortress => &Self::FORTRESS,
            StructureKeys::Igloo => &Self::IGLOO,
            StructureKeys::JunglePyramid => &Self::JUNGLE_PYRAMID,
            StructureKeys::Mansion => &Self::MANSION,
            StructureKeys::Mineshaft => &Self::MINESHAFT,
            StructureKeys::MineshaftMesa => &Self::MINESHAFT_MESA,
            StructureKeys::Monument => &Self::MONUMENT,
            StructureKeys::NetherFossil => &Self::NETHER_FOSSIL,
            StructureKeys::OceanRuinCold => &Self::OCEAN_RUIN_COLD,
            StructureKeys::OceanRuinWarm => &Self::OCEAN_RUIN_WARM,
            StructureKeys::PillagerOutpost => &Self::PILLAGER_OUTPOST,
            StructureKeys::RuinedPortal => &Self::RUINED_PORTAL,
            StructureKeys::RuinedPortalDesert => &Self::RUINED_PORTAL_DESERT,
            StructureKeys::RuinedPortalJungle => &Self::RUINED_PORTAL_JUNGLE,
            StructureKeys::RuinedPortalMountain => &Self::RUINED_PORTAL_MOUNTAIN,
            StructureKeys::RuinedPortalNether => &Self::RUINED_PORTAL_NETHER,
            StructureKeys::RuinedPortalOcean => &Self::RUINED_PORTAL_OCEAN,
            StructureKeys::RuinedPortalSwamp => &Self::RUINED_PORTAL_SWAMP,
            StructureKeys::Shipwreck => &Self::SHIPWRECK,
            StructureKeys::ShipwreckBeached => &Self::SHIPWRECK_BEACHED,
            StructureKeys::Stronghold => &Self::STRONGHOLD,
            StructureKeys::SwampHut => &Self::SWAMP_HUT,
            StructureKeys::TrailRuins => &Self::TRAIL_RUINS,
            StructureKeys::TrialChambers => &Self::TRIAL_CHAMBERS,
            StructureKeys::VillageDesert => &Self::VILLAGE_DESERT,
            StructureKeys::VillagePlains => &Self::VILLAGE_PLAINS,
            StructureKeys::VillageSavanna => &Self::VILLAGE_SAVANNA,
            StructureKeys::VillageSnowy => &Self::VILLAGE_SNOWY,
            StructureKeys::VillageTaiga => &Self::VILLAGE_TAIGA,
        }
    }
}
impl StructureSet {
    pub const ANCIENT_CITIES: Self = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 20083232u32,
            exclusion_zone: None,
            placement_type: StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                spacing: 24i32,
                separation: 8i32,
                spread_type: None,
            }),
        },
        structures: &[WeightedEntry {
            structure: StructureKeys::AncientCity,
            weight: 1u32,
        }],
    };
    pub const BURIED_TREASURES: Self = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: Some(FrequencyReductionMethod::LegacyType2),
            frequency: Some(0.01f32),
            salt: 0u32,
            exclusion_zone: None,
            placement_type: StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                spacing: 1i32,
                separation: 0i32,
                spread_type: None,
            }),
        },
        structures: &[WeightedEntry {
            structure: StructureKeys::BuriedTreasure,
            weight: 1u32,
        }],
    };
    pub const DESERT_PYRAMIDS: Self = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 14357617u32,
            exclusion_zone: None,
            placement_type: StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                spacing: 32i32,
                separation: 8i32,
                spread_type: None,
            }),
        },
        structures: &[WeightedEntry {
            structure: StructureKeys::DesertPyramid,
            weight: 1u32,
        }],
    };
    pub const END_CITIES: Self = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 10387313u32,
            exclusion_zone: None,
            placement_type: StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                spacing: 20i32,
                separation: 11i32,
                spread_type: Some(SpreadType::Triangular),
            }),
        },
        structures: &[WeightedEntry {
            structure: StructureKeys::EndCity,
            weight: 1u32,
        }],
    };
    pub const IGLOOS: Self = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 14357618u32,
            exclusion_zone: None,
            placement_type: StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                spacing: 32i32,
                separation: 8i32,
                spread_type: None,
            }),
        },
        structures: &[WeightedEntry {
            structure: StructureKeys::Igloo,
            weight: 1u32,
        }],
    };
    pub const JUNGLE_TEMPLES: Self = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 14357619u32,
            exclusion_zone: None,
            placement_type: StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                spacing: 32i32,
                separation: 8i32,
                spread_type: None,
            }),
        },
        structures: &[WeightedEntry {
            structure: StructureKeys::JunglePyramid,
            weight: 1u32,
        }],
    };
    pub const MINESHAFTS: Self = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: Some(FrequencyReductionMethod::LegacyType3),
            frequency: Some(0.004f32),
            salt: 0u32,
            exclusion_zone: None,
            placement_type: StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                spacing: 1i32,
                separation: 0i32,
                spread_type: None,
            }),
        },
        structures: &[
            WeightedEntry {
                structure: StructureKeys::Mineshaft,
                weight: 1u32,
            },
            WeightedEntry {
                structure: StructureKeys::MineshaftMesa,
                weight: 1u32,
            },
        ],
    };
    pub const NETHER_COMPLEXES: Self = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 30084232u32,
            exclusion_zone: None,
            placement_type: StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                spacing: 27i32,
                separation: 4i32,
                spread_type: None,
            }),
        },
        structures: &[
            WeightedEntry {
                structure: StructureKeys::Fortress,
                weight: 2u32,
            },
            WeightedEntry {
                structure: StructureKeys::BastionRemnant,
                weight: 3u32,
            },
        ],
    };
    pub const NETHER_FOSSILS: Self = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 14357921u32,
            exclusion_zone: None,
            placement_type: StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                spacing: 2i32,
                separation: 1i32,
                spread_type: None,
            }),
        },
        structures: &[WeightedEntry {
            structure: StructureKeys::NetherFossil,
            weight: 1u32,
        }],
    };
    pub const OCEAN_MONUMENTS: Self = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 10387313u32,
            exclusion_zone: None,
            placement_type: StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                spacing: 32i32,
                separation: 5i32,
                spread_type: Some(SpreadType::Triangular),
            }),
        },
        structures: &[WeightedEntry {
            structure: StructureKeys::Monument,
            weight: 1u32,
        }],
    };
    pub const OCEAN_RUINS: Self = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 14357621u32,
            exclusion_zone: None,
            placement_type: StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                spacing: 20i32,
                separation: 8i32,
                spread_type: None,
            }),
        },
        structures: &[
            WeightedEntry {
                structure: StructureKeys::OceanRuinCold,
                weight: 1u32,
            },
            WeightedEntry {
                structure: StructureKeys::OceanRuinWarm,
                weight: 1u32,
            },
        ],
    };
    pub const PILLAGER_OUTPOSTS: Self = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: Some(FrequencyReductionMethod::LegacyType1),
            frequency: Some(0.2f32),
            salt: 165745296u32,
            exclusion_zone: Some(ExclusionZone {
                other_set: "minecraft:villages",
                chunk_count: 10i32,
            }),
            placement_type: StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                spacing: 32i32,
                separation: 8i32,
                spread_type: None,
            }),
        },
        structures: &[WeightedEntry {
            structure: StructureKeys::PillagerOutpost,
            weight: 1u32,
        }],
    };
    pub const RUINED_PORTALS: Self = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 34222645u32,
            exclusion_zone: None,
            placement_type: StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                spacing: 40i32,
                separation: 15i32,
                spread_type: None,
            }),
        },
        structures: &[
            WeightedEntry {
                structure: StructureKeys::RuinedPortal,
                weight: 1u32,
            },
            WeightedEntry {
                structure: StructureKeys::RuinedPortalDesert,
                weight: 1u32,
            },
            WeightedEntry {
                structure: StructureKeys::RuinedPortalJungle,
                weight: 1u32,
            },
            WeightedEntry {
                structure: StructureKeys::RuinedPortalSwamp,
                weight: 1u32,
            },
            WeightedEntry {
                structure: StructureKeys::RuinedPortalMountain,
                weight: 1u32,
            },
            WeightedEntry {
                structure: StructureKeys::RuinedPortalOcean,
                weight: 1u32,
            },
            WeightedEntry {
                structure: StructureKeys::RuinedPortalNether,
                weight: 1u32,
            },
        ],
    };
    pub const SHIPWRECKS: Self = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 165745295u32,
            exclusion_zone: None,
            placement_type: StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                spacing: 24i32,
                separation: 4i32,
                spread_type: None,
            }),
        },
        structures: &[
            WeightedEntry {
                structure: StructureKeys::Shipwreck,
                weight: 1u32,
            },
            WeightedEntry {
                structure: StructureKeys::ShipwreckBeached,
                weight: 1u32,
            },
        ],
    };
    pub const STRONGHOLDS: Self = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 0u32,
            exclusion_zone: None,
            placement_type: StructurePlacementType::ConcentricRings(
                ConcentricRingsStructurePlacement {
                    spread: 3i32,
                    distance: 32i32,
                    count: 128i32,
                    preferred_biomes: "#minecraft:stronghold_biased_to",
                },
            ),
        },
        structures: &[WeightedEntry {
            structure: StructureKeys::Stronghold,
            weight: 1u32,
        }],
    };
    pub const SWAMP_HUTS: Self = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 14357620u32,
            exclusion_zone: None,
            placement_type: StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                spacing: 32i32,
                separation: 8i32,
                spread_type: None,
            }),
        },
        structures: &[WeightedEntry {
            structure: StructureKeys::SwampHut,
            weight: 1u32,
        }],
    };
    pub const TRAIL_RUINS: Self = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 83469867u32,
            exclusion_zone: None,
            placement_type: StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                spacing: 34i32,
                separation: 8i32,
                spread_type: None,
            }),
        },
        structures: &[WeightedEntry {
            structure: StructureKeys::TrailRuins,
            weight: 1u32,
        }],
    };
    pub const TRIAL_CHAMBERS: Self = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 94251327u32,
            exclusion_zone: None,
            placement_type: StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                spacing: 34i32,
                separation: 12i32,
                spread_type: None,
            }),
        },
        structures: &[WeightedEntry {
            structure: StructureKeys::TrialChambers,
            weight: 1u32,
        }],
    };
    pub const VILLAGES: Self = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 10387312u32,
            exclusion_zone: None,
            placement_type: StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                spacing: 34i32,
                separation: 8i32,
                spread_type: None,
            }),
        },
        structures: &[
            WeightedEntry {
                structure: StructureKeys::VillagePlains,
                weight: 1u32,
            },
            WeightedEntry {
                structure: StructureKeys::VillageDesert,
                weight: 1u32,
            },
            WeightedEntry {
                structure: StructureKeys::VillageSavanna,
                weight: 1u32,
            },
            WeightedEntry {
                structure: StructureKeys::VillageSnowy,
                weight: 1u32,
            },
            WeightedEntry {
                structure: StructureKeys::VillageTaiga,
                weight: 1u32,
            },
        ],
    };
    pub const WOODLAND_MANSIONS: Self = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 10387319u32,
            exclusion_zone: None,
            placement_type: StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                spacing: 80i32,
                separation: 20i32,
                spread_type: Some(SpreadType::Triangular),
            }),
        },
        structures: &[WeightedEntry {
            structure: StructureKeys::Mansion,
            weight: 1u32,
        }],
    };
    pub const ALL: &'static [StructureSet] = &[
        Self::ANCIENT_CITIES,
        Self::BURIED_TREASURES,
        Self::DESERT_PYRAMIDS,
        Self::END_CITIES,
        Self::IGLOOS,
        Self::JUNGLE_TEMPLES,
        Self::MINESHAFTS,
        Self::NETHER_COMPLEXES,
        Self::NETHER_FOSSILS,
        Self::OCEAN_MONUMENTS,
        Self::OCEAN_RUINS,
        Self::PILLAGER_OUTPOSTS,
        Self::RUINED_PORTALS,
        Self::SHIPWRECKS,
        Self::STRONGHOLDS,
        Self::SWAMP_HUTS,
        Self::TRAIL_RUINS,
        Self::TRIAL_CHAMBERS,
        Self::VILLAGES,
        Self::WOODLAND_MANSIONS,
    ];
    #[must_use]
    pub fn get(name: &str) -> Option<&'static Self> {
        match name {
            "ancient_cities" => Some(&Self::ANCIENT_CITIES),
            "buried_treasures" => Some(&Self::BURIED_TREASURES),
            "desert_pyramids" => Some(&Self::DESERT_PYRAMIDS),
            "end_cities" => Some(&Self::END_CITIES),
            "igloos" => Some(&Self::IGLOOS),
            "jungle_temples" => Some(&Self::JUNGLE_TEMPLES),
            "mineshafts" => Some(&Self::MINESHAFTS),
            "nether_complexes" => Some(&Self::NETHER_COMPLEXES),
            "nether_fossils" => Some(&Self::NETHER_FOSSILS),
            "ocean_monuments" => Some(&Self::OCEAN_MONUMENTS),
            "ocean_ruins" => Some(&Self::OCEAN_RUINS),
            "pillager_outposts" => Some(&Self::PILLAGER_OUTPOSTS),
            "ruined_portals" => Some(&Self::RUINED_PORTALS),
            "shipwrecks" => Some(&Self::SHIPWRECKS),
            "strongholds" => Some(&Self::STRONGHOLDS),
            "swamp_huts" => Some(&Self::SWAMP_HUTS),
            "trail_ruins" => Some(&Self::TRAIL_RUINS),
            "trial_chambers" => Some(&Self::TRIAL_CHAMBERS),
            "villages" => Some(&Self::VILLAGES),
            "woodland_mansions" => Some(&Self::WOODLAND_MANSIONS),
            _ => None,
        }
    }
}
