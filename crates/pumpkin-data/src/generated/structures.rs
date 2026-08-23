/* This file is generated. Do not edit manually. */
use pumpkin_util::math::floor_div;
use pumpkin_util::random::{
    RandomGenerator, RandomImpl, get_carver_seed, get_region_seed, legacy_rand::LegacyRand,
    xoroshiro128::Xoroshiro,
};
use pumpkin_util::y_offset::{AboveBottom, Absolute, BelowTop, YOffset};
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
impl StructureKeys {
    pub fn from_name(name: &str) -> Option<Self> {
        let name = name.strip_prefix("minecraft:").unwrap_or(name);
        match name {
            "ancient_city" => Some(Self::AncientCity),
            "bastion_remnant" => Some(Self::BastionRemnant),
            "buried_treasure" => Some(Self::BuriedTreasure),
            "desert_pyramid" => Some(Self::DesertPyramid),
            "end_city" => Some(Self::EndCity),
            "fortress" => Some(Self::Fortress),
            "igloo" => Some(Self::Igloo),
            "jungle_pyramid" => Some(Self::JunglePyramid),
            "mansion" => Some(Self::Mansion),
            "mineshaft" => Some(Self::Mineshaft),
            "mineshaft_mesa" => Some(Self::MineshaftMesa),
            "monument" => Some(Self::Monument),
            "nether_fossil" => Some(Self::NetherFossil),
            "ocean_ruin_cold" => Some(Self::OceanRuinCold),
            "ocean_ruin_warm" => Some(Self::OceanRuinWarm),
            "pillager_outpost" => Some(Self::PillagerOutpost),
            "ruined_portal" => Some(Self::RuinedPortal),
            "ruined_portal_desert" => Some(Self::RuinedPortalDesert),
            "ruined_portal_jungle" => Some(Self::RuinedPortalJungle),
            "ruined_portal_mountain" => Some(Self::RuinedPortalMountain),
            "ruined_portal_nether" => Some(Self::RuinedPortalNether),
            "ruined_portal_ocean" => Some(Self::RuinedPortalOcean),
            "ruined_portal_swamp" => Some(Self::RuinedPortalSwamp),
            "shipwreck" => Some(Self::Shipwreck),
            "shipwreck_beached" => Some(Self::ShipwreckBeached),
            "stronghold" => Some(Self::Stronghold),
            "swamp_hut" => Some(Self::SwampHut),
            "trail_ruins" => Some(Self::TrailRuins),
            "trial_chambers" => Some(Self::TrialChambers),
            "village_desert" => Some(Self::VillageDesert),
            "village_plains" => Some(Self::VillagePlains),
            "village_savanna" => Some(Self::VillageSavanna),
            "village_snowy" => Some(Self::VillageSnowy),
            "village_taiga" => Some(Self::VillageTaiga),
            _ => None,
        }
    }
    #[must_use]
    pub const fn to_name(&self) -> &'static str {
        match self {
            Self::AncientCity => "ancient_city",
            Self::BastionRemnant => "bastion_remnant",
            Self::BuriedTreasure => "buried_treasure",
            Self::DesertPyramid => "desert_pyramid",
            Self::EndCity => "end_city",
            Self::Fortress => "fortress",
            Self::Igloo => "igloo",
            Self::JunglePyramid => "jungle_pyramid",
            Self::Mansion => "mansion",
            Self::Mineshaft => "mineshaft",
            Self::MineshaftMesa => "mineshaft_mesa",
            Self::Monument => "monument",
            Self::NetherFossil => "nether_fossil",
            Self::OceanRuinCold => "ocean_ruin_cold",
            Self::OceanRuinWarm => "ocean_ruin_warm",
            Self::PillagerOutpost => "pillager_outpost",
            Self::RuinedPortal => "ruined_portal",
            Self::RuinedPortalDesert => "ruined_portal_desert",
            Self::RuinedPortalJungle => "ruined_portal_jungle",
            Self::RuinedPortalMountain => "ruined_portal_mountain",
            Self::RuinedPortalNether => "ruined_portal_nether",
            Self::RuinedPortalOcean => "ruined_portal_ocean",
            Self::RuinedPortalSwamp => "ruined_portal_swamp",
            Self::Shipwreck => "shipwreck",
            Self::ShipwreckBeached => "shipwreck_beached",
            Self::Stronghold => "stronghold",
            Self::SwampHut => "swamp_hut",
            Self::TrailRuins => "trail_ruins",
            Self::TrialChambers => "trial_chambers",
            Self::VillageDesert => "village_desert",
            Self::VillagePlains => "village_plains",
            Self::VillageSavanna => "village_savanna",
            Self::VillageSnowy => "village_snowy",
            Self::VillageTaiga => "village_taiga",
        }
    }
    #[must_use]
    pub const fn all_names() -> &'static [&'static str] {
        &[
            "minecraft:ancient_city",
            "minecraft:bastion_remnant",
            "minecraft:buried_treasure",
            "minecraft:desert_pyramid",
            "minecraft:end_city",
            "minecraft:fortress",
            "minecraft:igloo",
            "minecraft:jungle_pyramid",
            "minecraft:mansion",
            "minecraft:mineshaft",
            "minecraft:mineshaft_mesa",
            "minecraft:monument",
            "minecraft:nether_fossil",
            "minecraft:ocean_ruin_cold",
            "minecraft:ocean_ruin_warm",
            "minecraft:pillager_outpost",
            "minecraft:ruined_portal",
            "minecraft:ruined_portal_desert",
            "minecraft:ruined_portal_jungle",
            "minecraft:ruined_portal_mountain",
            "minecraft:ruined_portal_nether",
            "minecraft:ruined_portal_ocean",
            "minecraft:ruined_portal_swamp",
            "minecraft:shipwreck",
            "minecraft:shipwreck_beached",
            "minecraft:stronghold",
            "minecraft:swamp_hut",
            "minecraft:trail_ruins",
            "minecraft:trial_chambers",
            "minecraft:village_desert",
            "minecraft:village_plains",
            "minecraft:village_savanna",
            "minecraft:village_snowy",
            "minecraft:village_taiga",
        ]
    }
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundingBoxType {
    Piece,
    Full,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnEntry {
    pub entity_type: &'static str,
    pub min_count: u32,
    pub max_count: u32,
    pub weight: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnOverride {
    pub category: &'static str,
    pub bounding_box: BoundingBoxType,
    pub spawns: &'static [SpawnEntry],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeightProvider {
    Uniform(UniformHeightProvider),
    Trapezoid(TrapezoidHeightProvider),
    VeryBiasedToBottom(VeryBiasedToBottomHeightProvider),
}
impl HeightProvider {
    #[must_use]
    pub fn get(&self, random: &mut RandomGenerator, min_y: i8, height: u16) -> i32 {
        match self {
            Self::Uniform(provider) => provider.get(random, min_y, height),
            Self::Trapezoid(provider) => provider.get(random, min_y, height),
            Self::VeryBiasedToBottom(provider) => provider.get(random, min_y, height),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UniformHeightProvider {
    pub min_inclusive: YOffset,
    pub max_inclusive: YOffset,
}
impl UniformHeightProvider {
    #[must_use]
    pub fn get(&self, random: &mut RandomGenerator, min_y: i8, height: u16) -> i32 {
        let min = self.min_inclusive.get_y(min_y as i16, height);
        let max = self.max_inclusive.get_y(min_y as i16, height);
        if min >= max {
            min
        } else {
            random.next_inbetween_i32(min, max)
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrapezoidHeightProvider {
    pub min_inclusive: YOffset,
    pub max_inclusive: YOffset,
    pub plateau: Option<i32>,
}
impl TrapezoidHeightProvider {
    #[must_use]
    pub fn get(&self, random: &mut RandomGenerator, min_y: i8, height: u16) -> i32 {
        let plateau = self.plateau.unwrap_or(0);
        let i = self.min_inclusive.get_y(min_y as i16, height);
        let j = self.max_inclusive.get_y(min_y as i16, height);
        if i >= j {
            return i;
        }
        let k = j - i;
        if plateau >= k {
            return random.next_inbetween_i32(i, j);
        }
        let l = (k - plateau) / 2;
        let m = k - l;
        i + random.next_inbetween_i32(0, m) + random.next_inbetween_i32(0, l)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VeryBiasedToBottomHeightProvider {
    pub min_inclusive: YOffset,
    pub max_inclusive: YOffset,
    pub inner: Option<std::num::NonZero<u32>>,
}
impl VeryBiasedToBottomHeightProvider {
    #[must_use]
    pub fn get(&self, random: &mut RandomGenerator, min_y: i8, height: u16) -> i32 {
        let min = self.min_inclusive.get_y(min_y as i16, height);
        let max = self.max_inclusive.get_y(min_y as i16, height);
        let inner = self.inner.map_or(1, std::num::NonZero::get) as i32;
        if min >= max {
            return min;
        }
        let min_rnd = random.next_inbetween_i32(min + inner, max);
        let max_rnd = random.next_inbetween_i32(min, min_rnd - 1);
        random.next_inbetween_i32(min, max_rnd - 1 + inner)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoolAliasBinding {
    Direct {
        alias: &'static str,
        target: &'static str,
    },
    Random {
        alias: &'static str,
        targets: &'static [WeightedPoolAliasTarget],
    },
    RandomGroup {
        groups: &'static [WeightedPoolAliasGroup],
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeightedPoolAliasTarget {
    pub target: &'static str,
    pub weight: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeightedPoolAliasGroup {
    pub bindings: &'static [PoolAliasBinding],
    pub weight: u32,
}
pub struct Structure {
    pub biomes: &'static str,
    pub step: GenerationStep,
    pub start_pool: Option<&'static str>,
    pub start_jigsaw_name: Option<&'static str>,
    pub size: Option<i32>,
    pub terrain_adaptation: TerrainAdaptation,
    pub start_height: Option<HeightProvider>,
    pub project_start_to_heightmap: Option<&'static str>,
    pub max_distance_from_center: Option<i32>,
    pub liquid_settings: Option<&'static str>,
    pub dimension_padding: Option<i32>,
    pub use_expansion_hack: Option<bool>,
    pub pool_aliases: &'static [PoolAliasBinding],
    pub spawn_overrides: &'static [SpawnOverride],
    pub is_beached: Option<bool>,
    pub mineshaft_type: Option<&'static str>,
    pub biome_temp: Option<&'static str>,
    pub cluster_probability: Option<f32>,
    pub large_probability: Option<f32>,
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
        start_jigsaw_name: Some("minecraft:city_anchor"),
        size: Some(7i32),
        terrain_adaptation: TerrainAdaptation::BeardBox,
        start_height: Some(HeightProvider::Uniform(UniformHeightProvider {
            min_inclusive: YOffset::Absolute(Absolute { absolute: -27i16 }),
            max_inclusive: YOffset::Absolute(Absolute { absolute: -27i16 }),
        })),
        project_start_to_heightmap: None,
        max_distance_from_center: Some(116i32),
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: Some(false),
        pool_aliases: &[],
        spawn_overrides: &[
            SpawnOverride {
                category: "ambient",
                bounding_box: BoundingBoxType::Full,
                spawns: &[],
            },
            SpawnOverride {
                category: "axolotls",
                bounding_box: BoundingBoxType::Full,
                spawns: &[],
            },
            SpawnOverride {
                category: "creature",
                bounding_box: BoundingBoxType::Full,
                spawns: &[],
            },
            SpawnOverride {
                category: "misc",
                bounding_box: BoundingBoxType::Full,
                spawns: &[],
            },
            SpawnOverride {
                category: "monster",
                bounding_box: BoundingBoxType::Full,
                spawns: &[],
            },
            SpawnOverride {
                category: "underground_water_creature",
                bounding_box: BoundingBoxType::Full,
                spawns: &[],
            },
            SpawnOverride {
                category: "water_ambient",
                bounding_box: BoundingBoxType::Full,
                spawns: &[],
            },
            SpawnOverride {
                category: "water_creature",
                bounding_box: BoundingBoxType::Full,
                spawns: &[],
            },
        ],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::Jigsaw,
    };
    pub const BASTION_REMNANT: Self = Structure {
        biomes: "#minecraft:has_structure/bastion_remnant",
        step: GenerationStep::SurfaceStructures,
        start_pool: Some("minecraft:bastion/starts"),
        start_jigsaw_name: None,
        size: Some(6i32),
        terrain_adaptation: TerrainAdaptation::None,
        start_height: Some(HeightProvider::Uniform(UniformHeightProvider {
            min_inclusive: YOffset::Absolute(Absolute { absolute: 33i16 }),
            max_inclusive: YOffset::Absolute(Absolute { absolute: 33i16 }),
        })),
        project_start_to_heightmap: None,
        max_distance_from_center: Some(80i32),
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: Some(false),
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::Jigsaw,
    };
    pub const BURIED_TREASURE: Self = Structure {
        biomes: "#minecraft:has_structure/buried_treasure",
        step: GenerationStep::UndergroundStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::BuriedTreasure,
    };
    pub const DESERT_PYRAMID: Self = Structure {
        biomes: "#minecraft:has_structure/desert_pyramid",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::DesertPyramid,
    };
    pub const END_CITY: Self = Structure {
        biomes: "#minecraft:has_structure/end_city",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::EndCity,
    };
    pub const FORTRESS: Self = Structure {
        biomes: "#minecraft:has_structure/nether_fortress",
        step: GenerationStep::UndergroundDecoration,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[SpawnOverride {
            category: "monster",
            bounding_box: BoundingBoxType::Piece,
            spawns: &[
                SpawnEntry {
                    entity_type: "minecraft:blaze",
                    min_count: 2u32,
                    max_count: 3u32,
                    weight: 10u32,
                },
                SpawnEntry {
                    entity_type: "minecraft:zombified_piglin",
                    min_count: 4u32,
                    max_count: 4u32,
                    weight: 5u32,
                },
                SpawnEntry {
                    entity_type: "minecraft:wither_skeleton",
                    min_count: 5u32,
                    max_count: 5u32,
                    weight: 8u32,
                },
                SpawnEntry {
                    entity_type: "minecraft:skeleton",
                    min_count: 5u32,
                    max_count: 5u32,
                    weight: 2u32,
                },
                SpawnEntry {
                    entity_type: "minecraft:magma_cube",
                    min_count: 4u32,
                    max_count: 4u32,
                    weight: 3u32,
                },
            ],
        }],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::Fortress,
    };
    pub const IGLOO: Self = Structure {
        biomes: "#minecraft:has_structure/igloo",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::Igloo,
    };
    pub const JUNGLE_PYRAMID: Self = Structure {
        biomes: "#minecraft:has_structure/jungle_temple",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::JungleTemple,
    };
    pub const MANSION: Self = Structure {
        biomes: "#minecraft:has_structure/woodland_mansion",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::WoodlandMansion,
    };
    pub const MINESHAFT: Self = Structure {
        biomes: "#minecraft:has_structure/mineshaft",
        step: GenerationStep::UndergroundStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: Some("normal"),
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::Mineshaft,
    };
    pub const MINESHAFT_MESA: Self = Structure {
        biomes: "#minecraft:has_structure/mineshaft_mesa",
        step: GenerationStep::UndergroundStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: Some("mesa"),
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::Mineshaft,
    };
    pub const MONUMENT: Self = Structure {
        biomes: "#minecraft:has_structure/ocean_monument",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[
            SpawnOverride {
                category: "axolotls",
                bounding_box: BoundingBoxType::Full,
                spawns: &[],
            },
            SpawnOverride {
                category: "monster",
                bounding_box: BoundingBoxType::Full,
                spawns: &[SpawnEntry {
                    entity_type: "minecraft:guardian",
                    min_count: 2u32,
                    max_count: 4u32,
                    weight: 1u32,
                }],
            },
            SpawnOverride {
                category: "underground_water_creature",
                bounding_box: BoundingBoxType::Full,
                spawns: &[],
            },
        ],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::OceanMonument,
    };
    pub const NETHER_FOSSIL: Self = Structure {
        biomes: "#minecraft:has_structure/nether_fossil",
        step: GenerationStep::UndergroundDecoration,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::BeardThin,
        start_height: Some(HeightProvider::Uniform(UniformHeightProvider {
            min_inclusive: YOffset::Absolute(Absolute { absolute: 32i16 }),
            max_inclusive: YOffset::BelowTop(BelowTop { below_top: 2i8 }),
        })),
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::NetherFossil,
    };
    pub const OCEAN_RUIN_COLD: Self = Structure {
        biomes: "#minecraft:has_structure/ocean_ruin_cold",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: Some("cold"),
        cluster_probability: Some(0.9f32),
        large_probability: Some(0.3f32),
        structure_type: StructureType::OceanRuin,
    };
    pub const OCEAN_RUIN_WARM: Self = Structure {
        biomes: "#minecraft:has_structure/ocean_ruin_warm",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: Some("warm"),
        cluster_probability: Some(0.9f32),
        large_probability: Some(0.3f32),
        structure_type: StructureType::OceanRuin,
    };
    pub const PILLAGER_OUTPOST: Self = Structure {
        biomes: "#minecraft:has_structure/pillager_outpost",
        step: GenerationStep::SurfaceStructures,
        start_pool: Some("minecraft:pillager_outpost/base_plates"),
        start_jigsaw_name: None,
        size: Some(7i32),
        terrain_adaptation: TerrainAdaptation::BeardThin,
        start_height: Some(HeightProvider::Uniform(UniformHeightProvider {
            min_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
            max_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
        })),
        project_start_to_heightmap: Some("WORLD_SURFACE_WG"),
        max_distance_from_center: Some(80i32),
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: Some(true),
        pool_aliases: &[],
        spawn_overrides: &[SpawnOverride {
            category: "monster",
            bounding_box: BoundingBoxType::Full,
            spawns: &[SpawnEntry {
                entity_type: "minecraft:pillager",
                min_count: 1u32,
                max_count: 1u32,
                weight: 1u32,
            }],
        }],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::Jigsaw,
    };
    pub const RUINED_PORTAL: Self = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_standard",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::RuinedPortal,
    };
    pub const RUINED_PORTAL_DESERT: Self = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_desert",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::RuinedPortal,
    };
    pub const RUINED_PORTAL_JUNGLE: Self = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_jungle",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::RuinedPortal,
    };
    pub const RUINED_PORTAL_MOUNTAIN: Self = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_mountain",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::RuinedPortal,
    };
    pub const RUINED_PORTAL_NETHER: Self = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_nether",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::RuinedPortal,
    };
    pub const RUINED_PORTAL_OCEAN: Self = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_ocean",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::RuinedPortal,
    };
    pub const RUINED_PORTAL_SWAMP: Self = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_swamp",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::RuinedPortal,
    };
    pub const SHIPWRECK: Self = Structure {
        biomes: "#minecraft:has_structure/shipwreck",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: Some(false),
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::Shipwreck,
    };
    pub const SHIPWRECK_BEACHED: Self = Structure {
        biomes: "#minecraft:has_structure/shipwreck_beached",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: Some(true),
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::Shipwreck,
    };
    pub const STRONGHOLD: Self = Structure {
        biomes: "#minecraft:has_structure/stronghold",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::Bury,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::Stronghold,
    };
    pub const SWAMP_HUT: Self = Structure {
        biomes: "#minecraft:has_structure/swamp_hut",
        step: GenerationStep::SurfaceStructures,
        start_pool: None,
        start_jigsaw_name: None,
        size: None,
        terrain_adaptation: TerrainAdaptation::None,
        start_height: None,
        project_start_to_heightmap: None,
        max_distance_from_center: None,
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: None,
        pool_aliases: &[],
        spawn_overrides: &[
            SpawnOverride {
                category: "creature",
                bounding_box: BoundingBoxType::Piece,
                spawns: &[SpawnEntry {
                    entity_type: "minecraft:cat",
                    min_count: 1u32,
                    max_count: 1u32,
                    weight: 1u32,
                }],
            },
            SpawnOverride {
                category: "monster",
                bounding_box: BoundingBoxType::Piece,
                spawns: &[SpawnEntry {
                    entity_type: "minecraft:witch",
                    min_count: 1u32,
                    max_count: 1u32,
                    weight: 1u32,
                }],
            },
        ],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::SwampHut,
    };
    pub const TRAIL_RUINS: Self = Structure {
        biomes: "#minecraft:has_structure/trail_ruins",
        step: GenerationStep::UndergroundStructures,
        start_pool: Some("minecraft:trail_ruins/tower"),
        start_jigsaw_name: None,
        size: Some(7i32),
        terrain_adaptation: TerrainAdaptation::Bury,
        start_height: Some(HeightProvider::Uniform(UniformHeightProvider {
            min_inclusive: YOffset::Absolute(Absolute { absolute: -15i16 }),
            max_inclusive: YOffset::Absolute(Absolute { absolute: -15i16 }),
        })),
        project_start_to_heightmap: Some("WORLD_SURFACE_WG"),
        max_distance_from_center: Some(80i32),
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: Some(false),
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::Jigsaw,
    };
    pub const TRIAL_CHAMBERS: Self = Structure {
        biomes: "#minecraft:has_structure/trial_chambers",
        step: GenerationStep::UndergroundStructures,
        start_pool: Some("minecraft:trial_chambers/chamber/end"),
        start_jigsaw_name: None,
        size: Some(20i32),
        terrain_adaptation: TerrainAdaptation::Encapsulate,
        start_height: Some(HeightProvider::Uniform(UniformHeightProvider {
            min_inclusive: YOffset::Absolute(Absolute { absolute: -40i16 }),
            max_inclusive: YOffset::Absolute(Absolute { absolute: -20i16 }),
        })),
        project_start_to_heightmap: None,
        max_distance_from_center: Some(116i32),
        liquid_settings: Some("ignore_waterlogging"),
        dimension_padding: Some(10i32),
        use_expansion_hack: Some(false),
        pool_aliases: &[
            PoolAliasBinding::RandomGroup {
                groups: &[
                    WeightedPoolAliasGroup {
                        bindings: &[
                            PoolAliasBinding::Direct {
                                alias: "minecraft:trial_chambers/spawner/contents/ranged",
                                target: "minecraft:trial_chambers/spawner/ranged/skeleton",
                            },
                            PoolAliasBinding::Direct {
                                alias: "minecraft:trial_chambers/spawner/contents/slow_ranged",
                                target: "minecraft:trial_chambers/spawner/slow_ranged/skeleton",
                            },
                        ],
                        weight: 1u32,
                    },
                    WeightedPoolAliasGroup {
                        bindings: &[
                            PoolAliasBinding::Direct {
                                alias: "minecraft:trial_chambers/spawner/contents/ranged",
                                target: "minecraft:trial_chambers/spawner/ranged/stray",
                            },
                            PoolAliasBinding::Direct {
                                alias: "minecraft:trial_chambers/spawner/contents/slow_ranged",
                                target: "minecraft:trial_chambers/spawner/slow_ranged/stray",
                            },
                        ],
                        weight: 1u32,
                    },
                    WeightedPoolAliasGroup {
                        bindings: &[
                            PoolAliasBinding::Direct {
                                alias: "minecraft:trial_chambers/spawner/contents/ranged",
                                target: "minecraft:trial_chambers/spawner/ranged/poison_skeleton",
                            },
                            PoolAliasBinding::Direct {
                                alias: "minecraft:trial_chambers/spawner/contents/slow_ranged",
                                target: "minecraft:trial_chambers/spawner/slow_ranged/poison_skeleton",
                            },
                        ],
                        weight: 1u32,
                    },
                ],
            },
            PoolAliasBinding::Random {
                alias: "minecraft:trial_chambers/spawner/contents/melee",
                targets: &[
                    WeightedPoolAliasTarget {
                        target: "minecraft:trial_chambers/spawner/melee/zombie",
                        weight: 1u32,
                    },
                    WeightedPoolAliasTarget {
                        target: "minecraft:trial_chambers/spawner/melee/husk",
                        weight: 1u32,
                    },
                    WeightedPoolAliasTarget {
                        target: "minecraft:trial_chambers/spawner/melee/spider",
                        weight: 1u32,
                    },
                ],
            },
            PoolAliasBinding::Random {
                alias: "minecraft:trial_chambers/spawner/contents/small_melee",
                targets: &[
                    WeightedPoolAliasTarget {
                        target: "minecraft:trial_chambers/spawner/small_melee/slime",
                        weight: 1u32,
                    },
                    WeightedPoolAliasTarget {
                        target: "minecraft:trial_chambers/spawner/small_melee/cave_spider",
                        weight: 1u32,
                    },
                    WeightedPoolAliasTarget {
                        target: "minecraft:trial_chambers/spawner/small_melee/silverfish",
                        weight: 1u32,
                    },
                    WeightedPoolAliasTarget {
                        target: "minecraft:trial_chambers/spawner/small_melee/baby_zombie",
                        weight: 1u32,
                    },
                ],
            },
        ],
        spawn_overrides: &[
            SpawnOverride {
                category: "ambient",
                bounding_box: BoundingBoxType::Piece,
                spawns: &[],
            },
            SpawnOverride {
                category: "axolotls",
                bounding_box: BoundingBoxType::Piece,
                spawns: &[],
            },
            SpawnOverride {
                category: "creature",
                bounding_box: BoundingBoxType::Piece,
                spawns: &[],
            },
            SpawnOverride {
                category: "misc",
                bounding_box: BoundingBoxType::Piece,
                spawns: &[],
            },
            SpawnOverride {
                category: "monster",
                bounding_box: BoundingBoxType::Piece,
                spawns: &[],
            },
            SpawnOverride {
                category: "underground_water_creature",
                bounding_box: BoundingBoxType::Piece,
                spawns: &[],
            },
            SpawnOverride {
                category: "water_ambient",
                bounding_box: BoundingBoxType::Piece,
                spawns: &[],
            },
            SpawnOverride {
                category: "water_creature",
                bounding_box: BoundingBoxType::Piece,
                spawns: &[],
            },
        ],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::Jigsaw,
    };
    pub const VILLAGE_DESERT: Self = Structure {
        biomes: "#minecraft:has_structure/village_desert",
        step: GenerationStep::SurfaceStructures,
        start_pool: Some("minecraft:village/desert/town_centers"),
        start_jigsaw_name: None,
        size: Some(6i32),
        terrain_adaptation: TerrainAdaptation::BeardThin,
        start_height: Some(HeightProvider::Uniform(UniformHeightProvider {
            min_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
            max_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
        })),
        project_start_to_heightmap: Some("WORLD_SURFACE_WG"),
        max_distance_from_center: Some(80i32),
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: Some(true),
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::Jigsaw,
    };
    pub const VILLAGE_PLAINS: Self = Structure {
        biomes: "#minecraft:has_structure/village_plains",
        step: GenerationStep::SurfaceStructures,
        start_pool: Some("minecraft:village/plains/town_centers"),
        start_jigsaw_name: None,
        size: Some(6i32),
        terrain_adaptation: TerrainAdaptation::BeardThin,
        start_height: Some(HeightProvider::Uniform(UniformHeightProvider {
            min_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
            max_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
        })),
        project_start_to_heightmap: Some("WORLD_SURFACE_WG"),
        max_distance_from_center: Some(80i32),
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: Some(true),
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::Jigsaw,
    };
    pub const VILLAGE_SAVANNA: Self = Structure {
        biomes: "#minecraft:has_structure/village_savanna",
        step: GenerationStep::SurfaceStructures,
        start_pool: Some("minecraft:village/savanna/town_centers"),
        start_jigsaw_name: None,
        size: Some(6i32),
        terrain_adaptation: TerrainAdaptation::BeardThin,
        start_height: Some(HeightProvider::Uniform(UniformHeightProvider {
            min_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
            max_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
        })),
        project_start_to_heightmap: Some("WORLD_SURFACE_WG"),
        max_distance_from_center: Some(80i32),
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: Some(true),
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::Jigsaw,
    };
    pub const VILLAGE_SNOWY: Self = Structure {
        biomes: "#minecraft:has_structure/village_snowy",
        step: GenerationStep::SurfaceStructures,
        start_pool: Some("minecraft:village/snowy/town_centers"),
        start_jigsaw_name: None,
        size: Some(6i32),
        terrain_adaptation: TerrainAdaptation::BeardThin,
        start_height: Some(HeightProvider::Uniform(UniformHeightProvider {
            min_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
            max_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
        })),
        project_start_to_heightmap: Some("WORLD_SURFACE_WG"),
        max_distance_from_center: Some(80i32),
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: Some(true),
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
        structure_type: StructureType::Jigsaw,
    };
    pub const VILLAGE_TAIGA: Self = Structure {
        biomes: "#minecraft:has_structure/village_taiga",
        step: GenerationStep::SurfaceStructures,
        start_pool: Some("minecraft:village/taiga/town_centers"),
        start_jigsaw_name: None,
        size: Some(6i32),
        terrain_adaptation: TerrainAdaptation::BeardThin,
        start_height: Some(HeightProvider::Uniform(UniformHeightProvider {
            min_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
            max_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
        })),
        project_start_to_heightmap: Some("WORLD_SURFACE_WG"),
        max_distance_from_center: Some(80i32),
        liquid_settings: None,
        dimension_padding: None,
        use_expansion_hack: Some(true),
        pool_aliases: &[],
        spawn_overrides: &[],
        is_beached: None,
        mineshaft_type: None,
        biome_temp: None,
        cluster_probability: None,
        large_probability: None,
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
    #[doc = r" The registry names of all structure sets, in the same order as [`Self::ALL`]."]
    pub const NAMES: &'static [&'static str] = &[
        "ancient_cities",
        "buried_treasures",
        "desert_pyramids",
        "end_cities",
        "igloos",
        "jungle_temples",
        "mineshafts",
        "nether_complexes",
        "nether_fossils",
        "ocean_monuments",
        "ocean_ruins",
        "pillager_outposts",
        "ruined_portals",
        "shipwrecks",
        "strongholds",
        "swamp_huts",
        "trail_ruins",
        "trial_chambers",
        "villages",
        "woodland_mansions",
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
