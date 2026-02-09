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
pub struct StructurePlacement {
    pub frequency_reduction_method: Option<FrequencyReductionMethod>,
    pub frequency: Option<f32>,
    pub salt: u32,
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
pub struct Structure {
    pub biomes: &'static str,
    pub step: GenerationStep,
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
    pub const ANCIENT_CITY: Structure = Structure {
        biomes: "#minecraft:has_structure/ancient_city",
        step: GenerationStep::UndergroundDecoration,
    };
    pub const BASTION_REMNANT: Structure = Structure {
        biomes: "#minecraft:has_structure/bastion_remnant",
        step: GenerationStep::SurfaceStructures,
    };
    pub const BURIED_TREASURE: Structure = Structure {
        biomes: "#minecraft:has_structure/buried_treasure",
        step: GenerationStep::UndergroundStructures,
    };
    pub const DESERT_PYRAMID: Structure = Structure {
        biomes: "#minecraft:has_structure/desert_pyramid",
        step: GenerationStep::SurfaceStructures,
    };
    pub const END_CITY: Structure = Structure {
        biomes: "#minecraft:has_structure/end_city",
        step: GenerationStep::SurfaceStructures,
    };
    pub const FORTRESS: Structure = Structure {
        biomes: "#minecraft:has_structure/nether_fortress",
        step: GenerationStep::UndergroundDecoration,
    };
    pub const IGLOO: Structure = Structure {
        biomes: "#minecraft:has_structure/igloo",
        step: GenerationStep::SurfaceStructures,
    };
    pub const JUNGLE_PYRAMID: Structure = Structure {
        biomes: "#minecraft:has_structure/jungle_temple",
        step: GenerationStep::SurfaceStructures,
    };
    pub const MANSION: Structure = Structure {
        biomes: "#minecraft:has_structure/woodland_mansion",
        step: GenerationStep::SurfaceStructures,
    };
    pub const MINESHAFT: Structure = Structure {
        biomes: "#minecraft:has_structure/mineshaft",
        step: GenerationStep::UndergroundStructures,
    };
    pub const MINESHAFT_MESA: Structure = Structure {
        biomes: "#minecraft:has_structure/mineshaft_mesa",
        step: GenerationStep::UndergroundStructures,
    };
    pub const MONUMENT: Structure = Structure {
        biomes: "#minecraft:has_structure/ocean_monument",
        step: GenerationStep::SurfaceStructures,
    };
    pub const NETHER_FOSSIL: Structure = Structure {
        biomes: "#minecraft:has_structure/nether_fossil",
        step: GenerationStep::UndergroundDecoration,
    };
    pub const OCEAN_RUIN_COLD: Structure = Structure {
        biomes: "#minecraft:has_structure/ocean_ruin_cold",
        step: GenerationStep::SurfaceStructures,
    };
    pub const OCEAN_RUIN_WARM: Structure = Structure {
        biomes: "#minecraft:has_structure/ocean_ruin_warm",
        step: GenerationStep::SurfaceStructures,
    };
    pub const PILLAGER_OUTPOST: Structure = Structure {
        biomes: "#minecraft:has_structure/pillager_outpost",
        step: GenerationStep::SurfaceStructures,
    };
    pub const RUINED_PORTAL: Structure = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_standard",
        step: GenerationStep::SurfaceStructures,
    };
    pub const RUINED_PORTAL_DESERT: Structure = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_desert",
        step: GenerationStep::SurfaceStructures,
    };
    pub const RUINED_PORTAL_JUNGLE: Structure = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_jungle",
        step: GenerationStep::SurfaceStructures,
    };
    pub const RUINED_PORTAL_MOUNTAIN: Structure = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_mountain",
        step: GenerationStep::SurfaceStructures,
    };
    pub const RUINED_PORTAL_NETHER: Structure = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_nether",
        step: GenerationStep::SurfaceStructures,
    };
    pub const RUINED_PORTAL_OCEAN: Structure = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_ocean",
        step: GenerationStep::SurfaceStructures,
    };
    pub const RUINED_PORTAL_SWAMP: Structure = Structure {
        biomes: "#minecraft:has_structure/ruined_portal_swamp",
        step: GenerationStep::SurfaceStructures,
    };
    pub const SHIPWRECK: Structure = Structure {
        biomes: "#minecraft:has_structure/shipwreck",
        step: GenerationStep::SurfaceStructures,
    };
    pub const SHIPWRECK_BEACHED: Structure = Structure {
        biomes: "#minecraft:has_structure/shipwreck_beached",
        step: GenerationStep::SurfaceStructures,
    };
    pub const STRONGHOLD: Structure = Structure {
        biomes: "#minecraft:has_structure/stronghold",
        step: GenerationStep::SurfaceStructures,
    };
    pub const SWAMP_HUT: Structure = Structure {
        biomes: "#minecraft:has_structure/swamp_hut",
        step: GenerationStep::SurfaceStructures,
    };
    pub const TRAIL_RUINS: Structure = Structure {
        biomes: "#minecraft:has_structure/trail_ruins",
        step: GenerationStep::UndergroundStructures,
    };
    pub const TRIAL_CHAMBERS: Structure = Structure {
        biomes: "#minecraft:has_structure/trial_chambers",
        step: GenerationStep::UndergroundStructures,
    };
    pub const VILLAGE_DESERT: Structure = Structure {
        biomes: "#minecraft:has_structure/village_desert",
        step: GenerationStep::SurfaceStructures,
    };
    pub const VILLAGE_PLAINS: Structure = Structure {
        biomes: "#minecraft:has_structure/village_plains",
        step: GenerationStep::SurfaceStructures,
    };
    pub const VILLAGE_SAVANNA: Structure = Structure {
        biomes: "#minecraft:has_structure/village_savanna",
        step: GenerationStep::SurfaceStructures,
    };
    pub const VILLAGE_SNOWY: Structure = Structure {
        biomes: "#minecraft:has_structure/village_snowy",
        step: GenerationStep::SurfaceStructures,
    };
    pub const VILLAGE_TAIGA: Structure = Structure {
        biomes: "#minecraft:has_structure/village_taiga",
        step: GenerationStep::SurfaceStructures,
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
    pub const ANCIENT_CITIES: StructureSet = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 20083232u32,
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
    pub const BURIED_TREASURES: StructureSet = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: Some(FrequencyReductionMethod::LegacyType2),
            frequency: Some(0.01f32),
            salt: 0u32,
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
    pub const DESERT_PYRAMIDS: StructureSet = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 14357617u32,
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
    pub const END_CITIES: StructureSet = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 10387313u32,
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
    pub const IGLOOS: StructureSet = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 14357618u32,
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
    pub const JUNGLE_TEMPLES: StructureSet = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 14357619u32,
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
    pub const MINESHAFTS: StructureSet = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: Some(FrequencyReductionMethod::LegacyType3),
            frequency: Some(0.004f32),
            salt: 0u32,
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
    pub const NETHER_COMPLEXES: StructureSet = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 30084232u32,
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
    pub const NETHER_FOSSILS: StructureSet = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 14357921u32,
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
    pub const OCEAN_MONUMENTS: StructureSet = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 10387313u32,
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
    pub const OCEAN_RUINS: StructureSet = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 14357621u32,
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
    pub const PILLAGER_OUTPOSTS: StructureSet = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: Some(FrequencyReductionMethod::LegacyType1),
            frequency: Some(0.2f32),
            salt: 165745296u32,
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
    pub const RUINED_PORTALS: StructureSet = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 34222645u32,
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
    pub const SHIPWRECKS: StructureSet = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 165745295u32,
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
    pub const STRONGHOLDS: StructureSet = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 0u32,
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
    pub const SWAMP_HUTS: StructureSet = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 14357620u32,
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
    pub const TRAIL_RUINS: StructureSet = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 83469867u32,
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
    pub const TRIAL_CHAMBERS: StructureSet = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 94251327u32,
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
    pub const VILLAGES: StructureSet = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 10387312u32,
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
    pub const WOODLAND_MANSIONS: StructureSet = StructureSet {
        placement: StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 10387319u32,
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
