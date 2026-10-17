/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplatePoolProjection {
    Rigid,
    TerrainMatching,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StaticPoolElementKind {
    Empty,
    Single {
        location: &'static str,
        processors: &'static str,
        legacy: bool,
    },
    List(&'static [StaticPoolElementKind]),
    Feature(&'static str),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticPoolElement {
    pub weight: u32,
    pub projection: TemplatePoolProjection,
    pub kind: StaticPoolElementKind,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticTemplatePool {
    pub id: &'static str,
    pub fallback: &'static str,
    pub elements: &'static [StaticPoolElement],
}
impl StaticTemplatePool {
    pub const ABANDONED_CAMP_CAMP_BAMBOO_JUNGLE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/camp/bamboo_jungle",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/bamboo_jungle/campsite_bamboo_jungle_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/bamboo_jungle/campsite_bamboo_jungle_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/bamboo_jungle/campsite_bamboo_jungle_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/bamboo_jungle/campsite_bamboo_jungle_4",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_CAMP_BIRCH_FOREST: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/camp/birch_forest",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/birch_forest/campsite_birch_forest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/birch_forest/campsite_birch_forest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/birch_forest/campsite_birch_forest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/birch_forest/campsite_birch_forest_4",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_CAMP_CHERRY_GROVE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/camp/cherry_grove",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/cherry_grove/campsite_cherry_grove_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/cherry_grove/campsite_cherry_grove_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/cherry_grove/campsite_cherry_grove_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/cherry_grove/campsite_cherry_grove_4",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_CAMP_DAPPLED_FOREST: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/camp/dappled_forest",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/dappled_forest/campsite_dappled_forest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/dappled_forest/campsite_dappled_forest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/dappled_forest/campsite_dappled_forest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/dappled_forest/campsite_dappled_forest_4",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_CAMP_FLOWER_FOREST: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/camp/flower_forest",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/flower_forest/campsite_flower_forest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/flower_forest/campsite_flower_forest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/flower_forest/campsite_flower_forest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/flower_forest/campsite_flower_forest_4",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_CAMP_FOREST: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/camp/forest",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/forest/campsite_forest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/forest/campsite_forest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/forest/campsite_forest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/forest/campsite_forest_4",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_CAMP_MEADOW: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/camp/meadow",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/meadow/campsite_meadow_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/meadow/campsite_meadow_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/meadow/campsite_meadow_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/meadow/campsite_meadow_4",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_CAMP_OLD_GROWTH_BIRCH_FOREST: StaticTemplatePool =
        StaticTemplatePool {
            id: "minecraft:abandoned_camp/camp/old_growth_birch_forest",
            fallback: "minecraft:empty",
            elements: &[
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_1",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_2",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_3",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_4",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_5",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_6",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_7",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_8",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_9",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_10",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_11",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_12",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_13",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_14",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_15",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_1",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_2",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_3",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_4",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_5",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_6",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_7",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_8",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_9",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_10",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_11",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_12",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_13",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_14",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_15",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_1",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_2",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_3",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_4",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_5",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_6",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_7",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_8",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_9",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_10",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_11",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_12",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_13",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_14",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_15",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/old_growth_birch_forest/campsite_old_growth_birch_forest_1",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/old_growth_birch_forest/campsite_old_growth_birch_forest_2",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/old_growth_birch_forest/campsite_old_growth_birch_forest_3",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/old_growth_birch_forest/campsite_old_growth_birch_forest_4",
                        processors: "",
                        legacy: true,
                    },
                },
            ],
        };
    pub const ABANDONED_CAMP_CAMP_OLD_GROWTH_PINE_TAIGA: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/camp/old_growth_pine_taiga",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/old_growth_pine_taiga/campsite_old_growth_pine_taiga_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/old_growth_pine_taiga/campsite_old_growth_pine_taiga_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/old_growth_pine_taiga/campsite_old_growth_pine_taiga_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/old_growth_pine_taiga/campsite_old_growth_pine_taiga_4",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_CAMP_OLD_GROWTH_SPRUCE_TAIGA: StaticTemplatePool =
        StaticTemplatePool {
            id: "minecraft:abandoned_camp/camp/old_growth_spruce_taiga",
            fallback: "minecraft:empty",
            elements: &[
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_1",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_2",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_3",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_4",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_5",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_6",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_7",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_8",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_9",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_10",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_11",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_12",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_13",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_14",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_15",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_1",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_2",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_3",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_4",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_5",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_6",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_7",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_8",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_9",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_10",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_11",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_12",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_13",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_14",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_15",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_1",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_2",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_3",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_4",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_5",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_6",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_7",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_8",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_9",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_10",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_11",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_12",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_13",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_14",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/default/campsite_default_special_15",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/old_growth_spruce_taiga/campsite_old_growth_spruce_taiga_1",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/old_growth_spruce_taiga/campsite_old_growth_spruce_taiga_2",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/old_growth_spruce_taiga/campsite_old_growth_spruce_taiga_3",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/camp/old_growth_spruce_taiga/campsite_old_growth_spruce_taiga_4",
                        processors: "",
                        legacy: true,
                    },
                },
            ],
        };
    pub const ABANDONED_CAMP_CAMP_PALE_GARDEN: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/camp/pale_garden",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/pale_garden/campsite_pale_garden_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/pale_garden/campsite_pale_garden_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/pale_garden/campsite_pale_garden_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/pale_garden/campsite_pale_garden_4",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_CAMP_SAVANNA: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/camp/savanna",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/savanna/campsite_savanna_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/savanna/campsite_savanna_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/savanna/campsite_savanna_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/savanna/campsite_savanna_4",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_CAMP_SNOWY_TAIGA: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/camp/snowy_taiga",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/snowy_taiga/campsite_snowy_taiga_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/snowy_taiga/campsite_snowy_taiga_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/snowy_taiga/campsite_snowy_taiga_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/snowy_taiga/campsite_snowy_taiga_4",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_CAMP_SPARSE_JUNGLE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/camp/sparse_jungle",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/sparse_jungle/campsite_sparse_jungle_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/sparse_jungle/campsite_sparse_jungle_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/sparse_jungle/campsite_sparse_jungle_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/sparse_jungle/campsite_sparse_jungle_4",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_CAMP_SWAMP: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/camp/swamp",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/swamp/campsite_swamp_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/swamp/campsite_swamp_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/swamp/campsite_swamp_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/swamp/campsite_swamp_4",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_CAMP_TAIGA: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/camp/taiga",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/taiga/campsite_taiga_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/taiga/campsite_taiga_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/taiga/campsite_taiga_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/taiga/campsite_taiga_4",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_CAMP_WINDSWEPT_FOREST: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/camp/windswept_forest",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/windswept_forest/campsite_windswept_forest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/windswept_forest/campsite_windswept_forest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/windswept_forest/campsite_windswept_forest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/windswept_forest/campsite_windswept_forest_4",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_CAMP_WOODED_BADLANDS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/camp/wooded_badlands",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_chest_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_barrel_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_10",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_11",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_12",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_13",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_14",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/default/campsite_default_special_15",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/wooded_badlands/campsite_wooded_badlands_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/wooded_badlands/campsite_wooded_badlands_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/wooded_badlands/campsite_wooded_badlands_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/camp/wooded_badlands/campsite_wooded_badlands_4",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_TENT_BAMBOO_JUNGLE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/tent/bamboo_jungle",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/bamboo_jungle/tent_bamboo_jungle_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/bamboo_jungle/tent_bamboo_jungle_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/bamboo_jungle/tent_bamboo_jungle_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/bamboo_jungle/tent_bamboo_jungle_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/bamboo_jungle/tent_bamboo_jungle_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/bamboo_jungle/tent_bamboo_jungle_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/bamboo_jungle/tent_bamboo_jungle_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/bamboo_jungle/tent_bamboo_jungle_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/bamboo_jungle/tent_bamboo_jungle_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/bamboo_jungle/tent_bamboo_jungle_10",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_TENT_BIRCH_FOREST: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/tent/birch_forest",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/birch_forest/tent_birch_forest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/birch_forest/tent_birch_forest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/birch_forest/tent_birch_forest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/birch_forest/tent_birch_forest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/birch_forest/tent_birch_forest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/birch_forest/tent_birch_forest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/birch_forest/tent_birch_forest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/birch_forest/tent_birch_forest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/birch_forest/tent_birch_forest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/birch_forest/tent_birch_forest_10",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_TENT_CHERRY_GROVE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/tent/cherry_grove",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/cherry_grove/tent_cherry_grove_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/cherry_grove/tent_cherry_grove_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/cherry_grove/tent_cherry_grove_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/cherry_grove/tent_cherry_grove_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/cherry_grove/tent_cherry_grove_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/cherry_grove/tent_cherry_grove_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/cherry_grove/tent_cherry_grove_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/cherry_grove/tent_cherry_grove_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/cherry_grove/tent_cherry_grove_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/cherry_grove/tent_cherry_grove_10",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_TENT_DAPPLED_FOREST: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/tent/dappled_forest",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/dappled_forest/tent_dappled_forest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/dappled_forest/tent_dappled_forest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/dappled_forest/tent_dappled_forest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/dappled_forest/tent_dappled_forest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/dappled_forest/tent_dappled_forest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/dappled_forest/tent_dappled_forest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/dappled_forest/tent_dappled_forest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/dappled_forest/tent_dappled_forest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/dappled_forest/tent_dappled_forest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/dappled_forest/tent_dappled_forest_10",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_TENT_FLOWER_FOREST: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/tent/flower_forest",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/flower_forest/tent_flower_forest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/flower_forest/tent_flower_forest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/flower_forest/tent_flower_forest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/flower_forest/tent_flower_forest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/flower_forest/tent_flower_forest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/flower_forest/tent_flower_forest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/flower_forest/tent_flower_forest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/flower_forest/tent_flower_forest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/flower_forest/tent_flower_forest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/flower_forest/tent_flower_forest_10",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_TENT_FOREST: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/tent/forest",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/forest/tent_forest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/forest/tent_forest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/forest/tent_forest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/forest/tent_forest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/forest/tent_forest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/forest/tent_forest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/forest/tent_forest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/forest/tent_forest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/forest/tent_forest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/forest/tent_forest_10",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_TENT_MEADOW: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/tent/meadow",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/meadow/tent_meadow_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/meadow/tent_meadow_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/meadow/tent_meadow_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/meadow/tent_meadow_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/meadow/tent_meadow_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/meadow/tent_meadow_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/meadow/tent_meadow_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/meadow/tent_meadow_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/meadow/tent_meadow_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/meadow/tent_meadow_10",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_TENT_OLD_GROWTH_BIRCH_FOREST: StaticTemplatePool =
        StaticTemplatePool {
            id: "minecraft:abandoned_camp/tent/old_growth_birch_forest",
            fallback: "minecraft:empty",
            elements: &[
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/tent/old_growth_birch_forest/tent_old_growth_birch_forest_1",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/tent/old_growth_birch_forest/tent_old_growth_birch_forest_2",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/tent/old_growth_birch_forest/tent_old_growth_birch_forest_3",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/tent/old_growth_birch_forest/tent_old_growth_birch_forest_4",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/tent/old_growth_birch_forest/tent_old_growth_birch_forest_5",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/tent/old_growth_birch_forest/tent_old_growth_birch_forest_6",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/tent/old_growth_birch_forest/tent_old_growth_birch_forest_7",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/tent/old_growth_birch_forest/tent_old_growth_birch_forest_8",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/tent/old_growth_birch_forest/tent_old_growth_birch_forest_9",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/tent/old_growth_birch_forest/tent_old_growth_birch_forest_10",
                        processors: "",
                        legacy: true,
                    },
                },
            ],
        };
    pub const ABANDONED_CAMP_TENT_OLD_GROWTH_PINE_TAIGA: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/tent/old_growth_pine_taiga",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/old_growth_pine_taiga/tent_old_growth_pine_taiga_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/old_growth_pine_taiga/tent_old_growth_pine_taiga_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/old_growth_pine_taiga/tent_old_growth_pine_taiga_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/old_growth_pine_taiga/tent_old_growth_pine_taiga_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/old_growth_pine_taiga/tent_old_growth_pine_taiga_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/old_growth_pine_taiga/tent_old_growth_pine_taiga_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/old_growth_pine_taiga/tent_old_growth_pine_taiga_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/old_growth_pine_taiga/tent_old_growth_pine_taiga_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/old_growth_pine_taiga/tent_old_growth_pine_taiga_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/old_growth_pine_taiga/tent_old_growth_pine_taiga_10",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_TENT_OLD_GROWTH_SPRUCE_TAIGA: StaticTemplatePool =
        StaticTemplatePool {
            id: "minecraft:abandoned_camp/tent/old_growth_spruce_taiga",
            fallback: "minecraft:empty",
            elements: &[
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/tent/old_growth_spruce_taiga/tent_old_growth_spruce_taiga_1",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/tent/old_growth_spruce_taiga/tent_old_growth_spruce_taiga_2",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/tent/old_growth_spruce_taiga/tent_old_growth_spruce_taiga_3",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/tent/old_growth_spruce_taiga/tent_old_growth_spruce_taiga_4",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/tent/old_growth_spruce_taiga/tent_old_growth_spruce_taiga_5",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/tent/old_growth_spruce_taiga/tent_old_growth_spruce_taiga_6",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/tent/old_growth_spruce_taiga/tent_old_growth_spruce_taiga_7",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/tent/old_growth_spruce_taiga/tent_old_growth_spruce_taiga_8",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/tent/old_growth_spruce_taiga/tent_old_growth_spruce_taiga_9",
                        processors: "",
                        legacy: true,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:abandoned_camp/tent/old_growth_spruce_taiga/tent_old_growth_spruce_taiga_10",
                        processors: "",
                        legacy: true,
                    },
                },
            ],
        };
    pub const ABANDONED_CAMP_TENT_PALE_GARDEN: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/tent/pale_garden",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/pale_garden/tent_pale_garden_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/pale_garden/tent_pale_garden_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/pale_garden/tent_pale_garden_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/pale_garden/tent_pale_garden_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/pale_garden/tent_pale_garden_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/pale_garden/tent_pale_garden_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/pale_garden/tent_pale_garden_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/pale_garden/tent_pale_garden_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/pale_garden/tent_pale_garden_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/pale_garden/tent_pale_garden_10",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_TENT_SAVANNA: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/tent/savanna",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/savanna/tent_savanna_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/savanna/tent_savanna_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/savanna/tent_savanna_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/savanna/tent_savanna_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/savanna/tent_savanna_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/savanna/tent_savanna_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/savanna/tent_savanna_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/savanna/tent_savanna_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/savanna/tent_savanna_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/savanna/tent_savanna_10",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_TENT_SNOWY_TAIGA: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/tent/snowy_taiga",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/snowy_taiga/tent_snowy_taiga_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/snowy_taiga/tent_snowy_taiga_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/snowy_taiga/tent_snowy_taiga_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/snowy_taiga/tent_snowy_taiga_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/snowy_taiga/tent_snowy_taiga_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/snowy_taiga/tent_snowy_taiga_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/snowy_taiga/tent_snowy_taiga_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/snowy_taiga/tent_snowy_taiga_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/snowy_taiga/tent_snowy_taiga_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/snowy_taiga/tent_snowy_taiga_10",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_TENT_SPARSE_JUNGLE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/tent/sparse_jungle",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/sparse_jungle/tent_sparse_jungle_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/sparse_jungle/tent_sparse_jungle_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/sparse_jungle/tent_sparse_jungle_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/sparse_jungle/tent_sparse_jungle_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/sparse_jungle/tent_sparse_jungle_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/sparse_jungle/tent_sparse_jungle_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/sparse_jungle/tent_sparse_jungle_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/sparse_jungle/tent_sparse_jungle_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/sparse_jungle/tent_sparse_jungle_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/sparse_jungle/tent_sparse_jungle_10",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_TENT_SWAMP: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/tent/swamp",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/swamp/tent_swamp_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/swamp/tent_swamp_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/swamp/tent_swamp_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/swamp/tent_swamp_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/swamp/tent_swamp_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/swamp/tent_swamp_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/swamp/tent_swamp_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/swamp/tent_swamp_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/swamp/tent_swamp_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/swamp/tent_swamp_10",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_TENT_TAIGA: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/tent/taiga",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/taiga/tent_taiga_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/taiga/tent_taiga_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/taiga/tent_taiga_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/taiga/tent_taiga_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/taiga/tent_taiga_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/taiga/tent_taiga_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/taiga/tent_taiga_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/taiga/tent_taiga_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/taiga/tent_taiga_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/taiga/tent_taiga_10",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_TENT_WINDSWEPT_FOREST: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/tent/windswept_forest",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/windswept_forest/tent_windswept_forest_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/windswept_forest/tent_windswept_forest_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/windswept_forest/tent_windswept_forest_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/windswept_forest/tent_windswept_forest_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/windswept_forest/tent_windswept_forest_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/windswept_forest/tent_windswept_forest_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/windswept_forest/tent_windswept_forest_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/windswept_forest/tent_windswept_forest_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/windswept_forest/tent_windswept_forest_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/windswept_forest/tent_windswept_forest_10",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_TENT_WOODED_BADLANDS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/tent/wooded_badlands",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/wooded_badlands/tent_wooded_badlands_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/wooded_badlands/tent_wooded_badlands_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/wooded_badlands/tent_wooded_badlands_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/wooded_badlands/tent_wooded_badlands_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/wooded_badlands/tent_wooded_badlands_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/wooded_badlands/tent_wooded_badlands_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/wooded_badlands/tent_wooded_badlands_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/wooded_badlands/tent_wooded_badlands_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/wooded_badlands/tent_wooded_badlands_9",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:abandoned_camp/tent/wooded_badlands/tent_wooded_badlands_10",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const ABANDONED_CAMP_TREES_ACACIA: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/acacia",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:acacia_checked"),
        }],
    };
    pub const ABANDONED_CAMP_TREES_BAMBOO: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/bamboo",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:bamboo_in_structure"),
        }],
    };
    pub const ABANDONED_CAMP_TREES_BIRCH: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/birch",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:birch_checked"),
        }],
    };
    pub const ABANDONED_CAMP_TREES_BIRCH_BEES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/birch_bees",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:birch_bees_002"),
        }],
    };
    pub const ABANDONED_CAMP_TREES_CHERRY: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/cherry",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:cherry_checked"),
        }],
    };
    pub const ABANDONED_CAMP_TREES_CHERRY_BEES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/cherry_bees",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:cherry_bees_005"),
        }],
    };
    pub const ABANDONED_CAMP_TREES_FANCY_OAK: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/fancy_oak",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:fancy_oak_checked"),
        }],
    };
    pub const ABANDONED_CAMP_TREES_FANCY_OAK_BEES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/fancy_oak_bees",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:fancy_oak_bees_002"),
        }],
    };
    pub const ABANDONED_CAMP_TREES_JUNGLE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/jungle",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:jungle_tree"),
        }],
    };
    pub const ABANDONED_CAMP_TREES_MEGA_JUNGLE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/mega_jungle",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:mega_jungle_tree_checked"),
        }],
    };
    pub const ABANDONED_CAMP_TREES_MEGA_PINE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/mega_pine",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:mega_pine_checked"),
        }],
    };
    pub const ABANDONED_CAMP_TREES_OAK: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/oak",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:oak_checked"),
        }],
    };
    pub const ABANDONED_CAMP_TREES_ORANGE_POPLAR: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/orange_poplar",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:orange_poplar"),
        }],
    };
    pub const ABANDONED_CAMP_TREES_PALE_OAK: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/pale_oak",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:pale_oak_checked"),
        }],
    };
    pub const ABANDONED_CAMP_TREES_PINE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/pine",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:pine_checked"),
        }],
    };
    pub const ABANDONED_CAMP_TREES_RED_POPLAR: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/red_poplar",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:red_poplar"),
        }],
    };
    pub const ABANDONED_CAMP_TREES_SPRUCE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/spruce",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:spruce_checked"),
        }],
    };
    pub const ABANDONED_CAMP_TREES_SPRUCE_ON_SNOW: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/spruce_on_snow",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:spruce_on_snow"),
        }],
    };
    pub const ABANDONED_CAMP_TREES_SUPER_BIRCH_BEES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/super_birch_bees",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:super_birch_bees_0002"),
        }],
    };
    pub const ABANDONED_CAMP_TREES_THICK_SPRUCE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/thick_spruce",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:mega_spruce_checked"),
        }],
    };
    pub const ABANDONED_CAMP_TREES_YELLOW_POPLAR: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:abandoned_camp/trees/yellow_poplar",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:yellow_poplar"),
        }],
    };
    pub const ANCIENT_CITY_CITY_ENTRANCE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:ancient_city/city/entrance",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/city/entrance/entrance_connector",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/city/entrance/entrance_path_1",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/city/entrance/entrance_path_2",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/city/entrance/entrance_path_3",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/city/entrance/entrance_path_4",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/city/entrance/entrance_path_5",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const ANCIENT_CITY_CITY_CENTER: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:ancient_city/city_center",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/city_center/city_center_1",
                    processors: "minecraft:ancient_city_start_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/city_center/city_center_2",
                    processors: "minecraft:ancient_city_start_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/city_center/city_center_3",
                    processors: "minecraft:ancient_city_start_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const ANCIENT_CITY_CITY_CENTER_WALLS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:ancient_city/city_center/walls",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/city_center/walls/bottom_1",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/city_center/walls/bottom_2",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/city_center/walls/bottom_left_corner",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/city_center/walls/bottom_right_corner_1",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/city_center/walls/bottom_right_corner_2",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/city_center/walls/left",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/city_center/walls/right",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/city_center/walls/top",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/city_center/walls/top_right_corner",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/city_center/walls/top_left_corner",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const ANCIENT_CITY_SCULK: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:ancient_city/sculk",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 6u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:sculk_patch_ancient_city"),
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const ANCIENT_CITY_STRUCTURES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:ancient_city/structures",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 7u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/structures/barracks",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/structures/chamber_1",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/structures/chamber_2",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/structures/chamber_3",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/structures/sauna_1",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/structures/small_statue",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/structures/large_ruin_1",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/structures/tall_ruin_1",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/structures/tall_ruin_2",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/structures/tall_ruin_3",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/structures/tall_ruin_4",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::List(&[
                    StaticPoolElementKind::Single {
                        location: "minecraft:ancient_city/structures/camp_1",
                        processors: "minecraft:ancient_city_generic_degradation",
                        legacy: false,
                    },
                    StaticPoolElementKind::Single {
                        location: "minecraft:ancient_city/structures/camp_2",
                        processors: "minecraft:ancient_city_generic_degradation",
                        legacy: false,
                    },
                    StaticPoolElementKind::Single {
                        location: "minecraft:ancient_city/structures/camp_3",
                        processors: "minecraft:ancient_city_generic_degradation",
                        legacy: false,
                    },
                ]),
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/structures/medium_ruin_1",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/structures/medium_ruin_2",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/structures/small_ruin_1",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/structures/small_ruin_2",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/structures/large_pillar_1",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/structures/medium_pillar_1",
                    processors: "minecraft:ancient_city_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::List(&[StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/structures/ice_box_1",
                    processors: "",
                    legacy: false,
                }]),
            },
        ],
    };
    pub const ANCIENT_CITY_WALLS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:ancient_city/walls",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/intact_corner_wall_1",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/intact_intersection_wall_1",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/intact_lshape_wall_1",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/intact_horizontal_wall_1",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/intact_horizontal_wall_2",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/intact_horizontal_wall_stairs_1",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/intact_horizontal_wall_stairs_2",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/intact_horizontal_wall_stairs_3",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/intact_horizontal_wall_stairs_4",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/intact_horizontal_wall_passage_1",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/ruined_corner_wall_1",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/ruined_corner_wall_2",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/ruined_horizontal_wall_stairs_1",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/ruined_horizontal_wall_stairs_2",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/ruined_horizontal_wall_stairs_3",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/ruined_horizontal_wall_stairs_4",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const ANCIENT_CITY_WALLS_NO_CORNERS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:ancient_city/walls/no_corners",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/intact_horizontal_wall_1",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/intact_horizontal_wall_2",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/intact_horizontal_wall_stairs_1",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/intact_horizontal_wall_stairs_2",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/intact_horizontal_wall_stairs_3",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/intact_horizontal_wall_stairs_4",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/intact_horizontal_wall_stairs_5",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:ancient_city/walls/intact_horizontal_wall_bridge",
                    processors: "minecraft:ancient_city_walls_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_BLOCKS_GOLD: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/blocks/gold",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/blocks/air",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/blocks/gold",
                    processors: "",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_BRIDGE_BRIDGE_PIECES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/bridge/bridge_pieces",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:bastion/bridge/bridge_pieces/bridge",
                processors: "minecraft:bridge",
                legacy: false,
            },
        }],
    };
    pub const BASTION_BRIDGE_CONNECTORS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/bridge/connectors",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/bridge/connectors/back_bridge_top",
                    processors: "minecraft:bastion_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/bridge/connectors/back_bridge_bottom",
                    processors: "minecraft:bastion_generic_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_BRIDGE_LEGS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/bridge/legs",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/bridge/legs/leg_0",
                    processors: "minecraft:bastion_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/bridge/legs/leg_1",
                    processors: "minecraft:bastion_generic_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_BRIDGE_RAMPART_PLATES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/bridge/rampart_plates",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:bastion/bridge/rampart_plates/plate_0",
                processors: "minecraft:rampart_degradation",
                legacy: false,
            },
        }],
    };
    pub const BASTION_BRIDGE_RAMPARTS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/bridge/ramparts",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/bridge/ramparts/rampart_0",
                    processors: "minecraft:rampart_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/bridge/ramparts/rampart_1",
                    processors: "minecraft:rampart_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_BRIDGE_STARTING_PIECES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/bridge/starting_pieces",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/bridge/starting_pieces/entrance",
                    processors: "minecraft:entrance_replacement",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/bridge/starting_pieces/entrance_face",
                    processors: "minecraft:bastion_generic_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_BRIDGE_WALLS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/bridge/walls",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/bridge/walls/wall_base_0",
                    processors: "minecraft:rampart_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/bridge/walls/wall_base_1",
                    processors: "minecraft:rampart_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_HOGLIN_STABLE_CONNECTORS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/hoglin_stable/connectors",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:bastion/hoglin_stable/connectors/end_post_connector",
                processors: "minecraft:stable_degradation",
                legacy: false,
            },
        }],
    };
    pub const BASTION_HOGLIN_STABLE_LARGE_STABLES_INNER: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/hoglin_stable/large_stables/inner",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/large_stables/inner_0",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/large_stables/inner_1",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/large_stables/inner_2",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/large_stables/inner_3",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/large_stables/inner_4",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_HOGLIN_STABLE_LARGE_STABLES_OUTER: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/hoglin_stable/large_stables/outer",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/large_stables/outer_0",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/large_stables/outer_1",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/large_stables/outer_2",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/large_stables/outer_3",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/large_stables/outer_4",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_HOGLIN_STABLE_MIRRORED_STARTING_PIECES: StaticTemplatePool =
        StaticTemplatePool {
            id: "minecraft:bastion/hoglin_stable/mirrored_starting_pieces",
            fallback: "minecraft:empty",
            elements: &[
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:bastion/hoglin_stable/starting_pieces/stairs_0_mirrored",
                        processors: "minecraft:stable_degradation",
                        legacy: false,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:bastion/hoglin_stable/starting_pieces/stairs_1_mirrored",
                        processors: "minecraft:stable_degradation",
                        legacy: false,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:bastion/hoglin_stable/starting_pieces/stairs_2_mirrored",
                        processors: "minecraft:stable_degradation",
                        legacy: false,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:bastion/hoglin_stable/starting_pieces/stairs_3_mirrored",
                        processors: "minecraft:stable_degradation",
                        legacy: false,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:bastion/hoglin_stable/starting_pieces/stairs_4_mirrored",
                        processors: "minecraft:stable_degradation",
                        legacy: false,
                    },
                },
            ],
        };
    pub const BASTION_HOGLIN_STABLE_POSTS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/hoglin_stable/posts",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/posts/stair_post",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/posts/end_post",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_HOGLIN_STABLE_RAMPART_PLATES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/hoglin_stable/rampart_plates",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:bastion/hoglin_stable/rampart_plates/rampart_plate_1",
                processors: "minecraft:stable_degradation",
                legacy: false,
            },
        }],
    };
    pub const BASTION_HOGLIN_STABLE_RAMPARTS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/hoglin_stable/ramparts",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/ramparts/ramparts_1",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/ramparts/ramparts_2",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/ramparts/ramparts_3",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_HOGLIN_STABLE_SMALL_STABLES_INNER: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/hoglin_stable/small_stables/inner",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/small_stables/inner_0",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/small_stables/inner_1",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/small_stables/inner_2",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/small_stables/inner_3",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_HOGLIN_STABLE_SMALL_STABLES_OUTER: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/hoglin_stable/small_stables/outer",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/small_stables/outer_0",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/small_stables/outer_1",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/small_stables/outer_2",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/small_stables/outer_3",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_HOGLIN_STABLE_STAIRS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/hoglin_stable/stairs",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/stairs/stairs_1_0",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/stairs/stairs_1_1",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/stairs/stairs_1_2",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/stairs/stairs_1_3",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/stairs/stairs_1_4",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/stairs/stairs_2_0",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/stairs/stairs_2_1",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/stairs/stairs_2_2",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/stairs/stairs_2_3",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/stairs/stairs_2_4",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/stairs/stairs_3_0",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/stairs/stairs_3_1",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/stairs/stairs_3_2",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/stairs/stairs_3_3",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/stairs/stairs_3_4",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_HOGLIN_STABLE_STARTING_PIECES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/hoglin_stable/starting_pieces",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/starting_pieces/starting_stairs_0",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/starting_pieces/starting_stairs_1",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/starting_pieces/starting_stairs_2",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/starting_pieces/starting_stairs_3",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/starting_pieces/starting_stairs_4",
                    processors: "minecraft:stable_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_HOGLIN_STABLE_WALL_BASES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/hoglin_stable/wall_bases",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:bastion/hoglin_stable/walls/wall_base",
                processors: "minecraft:stable_degradation",
                legacy: false,
            },
        }],
    };
    pub const BASTION_HOGLIN_STABLE_WALLS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/hoglin_stable/walls",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/walls/side_wall_0",
                    processors: "minecraft:side_wall_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/walls/side_wall_1",
                    processors: "minecraft:side_wall_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_MOBS_HOGLIN: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/mobs/hoglin",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/mobs/hoglin",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/mobs/empty",
                    processors: "",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_MOBS_PIGLIN: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/mobs/piglin",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/mobs/melee_piglin",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/mobs/sword_piglin",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/mobs/crossbow_piglin",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/mobs/empty",
                    processors: "",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_MOBS_PIGLIN_MELEE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/mobs/piglin_melee",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/mobs/melee_piglin_always",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 5u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/mobs/melee_piglin",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/mobs/sword_piglin",
                    processors: "",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_STARTS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/starts",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/air_base",
                    processors: "minecraft:bastion_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/hoglin_stable/air_base",
                    processors: "minecraft:bastion_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/big_air_full",
                    processors: "minecraft:bastion_generic_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/bridge/starting_pieces/entrance_base",
                    processors: "minecraft:bastion_generic_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_TREASURE_BASES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/treasure/bases",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:bastion/treasure/bases/lava_basin",
                processors: "minecraft:treasure_rooms",
                legacy: false,
            },
        }],
    };
    pub const BASTION_TREASURE_BASES_CENTERS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/treasure/bases/centers",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/bases/centers/center_0",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/bases/centers/center_1",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/bases/centers/center_2",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/bases/centers/center_3",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_TREASURE_BRAINS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/treasure/brains",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:bastion/treasure/brains/center_brain",
                processors: "minecraft:treasure_rooms",
                legacy: false,
            },
        }],
    };
    pub const BASTION_TREASURE_CONNECTORS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/treasure/connectors",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/connectors/center_to_wall_middle",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/connectors/center_to_wall_top",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/connectors/center_to_wall_top_entrance",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_TREASURE_CORNERS_BOTTOM: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/treasure/corners/bottom",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/corners/bottom/corner_0",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/corners/bottom/corner_1",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_TREASURE_CORNERS_EDGES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/treasure/corners/edges",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/corners/edges/bottom",
                    processors: "minecraft:high_wall",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/corners/edges/middle",
                    processors: "minecraft:high_wall",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/corners/edges/top",
                    processors: "minecraft:high_wall",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_TREASURE_CORNERS_MIDDLE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/treasure/corners/middle",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/corners/middle/corner_0",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/corners/middle/corner_1",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_TREASURE_CORNERS_TOP: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/treasure/corners/top",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/corners/top/corner_0",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/corners/top/corner_1",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_TREASURE_ENTRANCES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/treasure/entrances",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:bastion/treasure/entrances/entrance_0",
                processors: "minecraft:treasure_rooms",
                legacy: false,
            },
        }],
    };
    pub const BASTION_TREASURE_EXTENSIONS_HOUSES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/treasure/extensions/houses",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/extensions/house_0",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/extensions/house_1",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_TREASURE_EXTENSIONS_LARGE_POOL: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/treasure/extensions/large_pool",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/extensions/empty",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/extensions/empty",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/extensions/fire_room",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/extensions/large_bridge_0",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/extensions/large_bridge_1",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/extensions/large_bridge_2",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/extensions/large_bridge_3",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/extensions/roofed_bridge",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/extensions/empty",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_TREASURE_EXTENSIONS_SMALL_POOL: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/treasure/extensions/small_pool",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/extensions/empty",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/extensions/fire_room",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/extensions/empty",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/extensions/small_bridge_0",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/extensions/small_bridge_1",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/extensions/small_bridge_2",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/extensions/small_bridge_3",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_TREASURE_RAMPARTS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/treasure/ramparts",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/ramparts/mid_wall_main",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/ramparts/mid_wall_side",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/ramparts/bottom_wall_0",
                    processors: "minecraft:bottom_rampart",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/ramparts/top_wall",
                    processors: "minecraft:high_rampart",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/ramparts/lava_basin_side",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/ramparts/lava_basin_main",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_TREASURE_ROOFS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/treasure/roofs",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/roofs/wall_roof",
                    processors: "minecraft:roof",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/roofs/corner_roof",
                    processors: "minecraft:roof",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/roofs/center_roof",
                    processors: "minecraft:roof",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_TREASURE_STAIRS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/treasure/stairs",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:bastion/treasure/stairs/lower_stairs",
                processors: "minecraft:treasure_rooms",
                legacy: false,
            },
        }],
    };
    pub const BASTION_TREASURE_WALLS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/treasure/walls",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/walls/lava_wall",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/walls/entrance_wall",
                    processors: "minecraft:high_wall",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_TREASURE_WALLS_BOTTOM: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/treasure/walls/bottom",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/walls/bottom/wall_0",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/walls/bottom/wall_1",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/walls/bottom/wall_2",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/walls/bottom/wall_3",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_TREASURE_WALLS_MID: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/treasure/walls/mid",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/walls/mid/wall_0",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/walls/mid/wall_1",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/walls/mid/wall_2",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_TREASURE_WALLS_OUTER: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/treasure/walls/outer",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/walls/outer/top_corner",
                    processors: "minecraft:high_wall",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/walls/outer/mid_corner",
                    processors: "minecraft:high_wall",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/walls/outer/bottom_corner",
                    processors: "minecraft:high_wall",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/walls/outer/outer_wall",
                    processors: "minecraft:high_wall",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/walls/outer/medium_outer_wall",
                    processors: "minecraft:high_wall",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/walls/outer/tall_outer_wall",
                    processors: "minecraft:high_wall",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_TREASURE_WALLS_TOP: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/treasure/walls/top",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/walls/top/main_entrance",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/walls/top/wall_0",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/treasure/walls/top/wall_1",
                    processors: "minecraft:treasure_rooms",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_UNITS_CENTER_PIECES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/units/center_pieces",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/center_pieces/center_0",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/center_pieces/center_1",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/center_pieces/center_2",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_UNITS_EDGE_WALL_UNITS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/units/edge_wall_units",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:bastion/units/wall_units/edge_0_large",
                processors: "minecraft:housing",
                legacy: false,
            },
        }],
    };
    pub const BASTION_UNITS_EDGES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/units/edges",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:bastion/units/edges/edge_0",
                processors: "minecraft:housing",
                legacy: false,
            },
        }],
    };
    pub const BASTION_UNITS_FILLERS_STAGE_0: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/units/fillers/stage_0",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:bastion/units/fillers/stage_0",
                processors: "minecraft:housing",
                legacy: false,
            },
        }],
    };
    pub const BASTION_UNITS_LARGE_RAMPARTS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/units/large_ramparts",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:bastion/units/ramparts/ramparts_0",
                processors: "minecraft:housing",
                legacy: false,
            },
        }],
    };
    pub const BASTION_UNITS_PATHWAYS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/units/pathways",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/pathways/pathway_0",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/pathways/pathway_wall_0",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_UNITS_RAMPART_PLATES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/units/rampart_plates",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:bastion/units/rampart_plates/plate_0",
                processors: "minecraft:housing",
                legacy: false,
            },
        }],
    };
    pub const BASTION_UNITS_RAMPARTS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/units/ramparts",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/ramparts/ramparts_0",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/ramparts/ramparts_1",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/ramparts/ramparts_2",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_UNITS_STAGES_ROT_STAGE_1: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/units/stages/rot/stage_1",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:bastion/units/stages/rot/stage_1_0",
                processors: "minecraft:housing",
                legacy: false,
            },
        }],
    };
    pub const BASTION_UNITS_STAGES_STAGE_0: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/units/stages/stage_0",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/stages/stage_0_0",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/stages/stage_0_1",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/stages/stage_0_2",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/stages/stage_0_3",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_UNITS_STAGES_STAGE_1: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/units/stages/stage_1",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/stages/stage_1_0",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/stages/stage_1_1",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/stages/stage_1_2",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/stages/stage_1_3",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_UNITS_STAGES_STAGE_2: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/units/stages/stage_2",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/stages/stage_2_0",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/stages/stage_2_1",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_UNITS_STAGES_STAGE_3: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/units/stages/stage_3",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/stages/stage_3_0",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/stages/stage_3_1",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/stages/stage_3_2",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/stages/stage_3_3",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
        ],
    };
    pub const BASTION_UNITS_WALL_UNITS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/units/wall_units",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:bastion/units/wall_units/unit_0",
                processors: "minecraft:housing",
                legacy: false,
            },
        }],
    };
    pub const BASTION_UNITS_WALLS_WALL_BASES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:bastion/units/walls/wall_bases",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/walls/wall_base",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:bastion/units/walls/connected_wall",
                    processors: "minecraft:housing",
                    legacy: false,
                },
            },
        ],
    };
    pub const EMPTY: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:empty",
        fallback: "minecraft:empty",
        elements: &[],
    };
    pub const PILLAGER_OUTPOST_BASE_PLATES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:pillager_outpost/base_plates",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:pillager_outpost/base_plate",
                processors: "",
                legacy: true,
            },
        }],
    };
    pub const PILLAGER_OUTPOST_FEATURE_PLATES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:pillager_outpost/feature_plates",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::TerrainMatching,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:pillager_outpost/feature_plate",
                processors: "",
                legacy: true,
            },
        }],
    };
    pub const PILLAGER_OUTPOST_FEATURES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:pillager_outpost/features",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:pillager_outpost/feature_cage1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:pillager_outpost/feature_cage2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:pillager_outpost/feature_cage_with_allays",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:pillager_outpost/feature_logs",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:pillager_outpost/feature_tent1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:pillager_outpost/feature_tent2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:pillager_outpost/feature_targets",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 6u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const PILLAGER_OUTPOST_TOWERS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:pillager_outpost/towers",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::List(&[
                StaticPoolElementKind::Single {
                    location: "minecraft:pillager_outpost/watchtower",
                    processors: "",
                    legacy: true,
                },
                StaticPoolElementKind::Single {
                    location: "minecraft:pillager_outpost/watchtower_overgrown",
                    processors: "minecraft:outpost_rot",
                    legacy: true,
                },
            ]),
        }],
    };
    pub const TRAIL_RUINS_BUILDINGS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trail_ruins/buildings",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_hall_1",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_hall_2",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_hall_3",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_hall_4",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_hall_5",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/large_room_1",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/large_room_2",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/large_room_3",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/large_room_4",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/large_room_5",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/one_room_1",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/one_room_2",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/one_room_3",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/one_room_4",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/one_room_5",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRAIL_RUINS_BUILDINGS_GROUPED: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trail_ruins/buildings/grouped",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_full_1",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_full_2",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_full_3",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_full_4",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_full_5",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_lower_1",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_lower_2",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_lower_3",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_lower_4",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_lower_5",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_upper_1",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_upper_2",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_upper_3",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_upper_4",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_upper_5",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_room_1",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_room_2",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_room_3",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_room_4",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/buildings/group_room_5",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRAIL_RUINS_DECOR: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trail_ruins/decor",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/decor/decor_1",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/decor/decor_2",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/decor/decor_3",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/decor/decor_4",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/decor/decor_5",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/decor/decor_6",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/decor/decor_7",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRAIL_RUINS_ROADS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trail_ruins/roads",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/roads/long_road_end",
                    processors: "minecraft:trail_ruins_roads_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/roads/road_end_1",
                    processors: "minecraft:trail_ruins_roads_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/roads/road_section_1",
                    processors: "minecraft:trail_ruins_roads_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/roads/road_section_2",
                    processors: "minecraft:trail_ruins_roads_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/roads/road_section_3",
                    processors: "minecraft:trail_ruins_roads_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/roads/road_section_4",
                    processors: "minecraft:trail_ruins_roads_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/roads/road_spacer_1",
                    processors: "minecraft:trail_ruins_roads_archaeology",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRAIL_RUINS_TOWER: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trail_ruins/tower",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/tower_1",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/tower_2",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/tower_3",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/tower_4",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/tower_5",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRAIL_RUINS_TOWER_ADDITIONS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trail_ruins/tower/additions",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/hall_1",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/hall_2",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/hall_3",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/hall_4",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/hall_5",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/large_hall_1",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/large_hall_2",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/large_hall_3",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/large_hall_4",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/large_hall_5",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/one_room_1",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/one_room_2",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/one_room_3",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/one_room_4",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/one_room_5",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/platform_1",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/platform_2",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/platform_3",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/platform_4",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/platform_5",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/stable_1",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/stable_2",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/stable_3",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/stable_4",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/stable_5",
                    processors: "minecraft:trail_ruins_houses_archaeology",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRAIL_RUINS_TOWER_TOWER_TOP: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trail_ruins/tower/tower_top",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/tower_top_1",
                    processors: "minecraft:trail_ruins_tower_top_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/tower_top_2",
                    processors: "minecraft:trail_ruins_tower_top_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/tower_top_3",
                    processors: "minecraft:trail_ruins_tower_top_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/tower_top_4",
                    processors: "minecraft:trail_ruins_tower_top_archaeology",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trail_ruins/tower/tower_top_5",
                    processors: "minecraft:trail_ruins_tower_top_archaeology",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_ATRIUM: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/atrium",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/atrium/bogged_relief",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/atrium/breeze_relief",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/atrium/spiral_relief",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/atrium/spider_relief",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/atrium/grand_staircase_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/atrium/grand_staircase_2",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/atrium/grand_staircase_3",
                    processors: "",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_CHAMBER_ADDON: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/chamber/addon",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/addon/full_stacked_walkway",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/addon/full_stacked_walkway_2",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/addon/full_corner_column",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/addon/grate_bridge",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/addon/hanging_platform",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/addon/short_grate_platform",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/addon/short_platform",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/addon/lower_staircase_down",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/addon/walkway_with_bridge_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/addon/c1_breeze",
                    processors: "",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_CHAMBER_ASSEMBLY: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/chamber/assembly",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/full_column",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/cover_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/cover_2",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/cover_3",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/cover_4",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/cover_5",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/cover_6",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 5u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/cover_7",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/platform_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/spawner_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/hanging_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/hanging_2",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/hanging_3",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/hanging_4",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/hanging_5",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/left_staircase_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/left_staircase_2",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/left_staircase_3",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/right_staircase_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/right_staircase_2",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly/right_staircase_3",
                    processors: "",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_CHAMBER_END: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/chamber/end",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/end_1",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/end_2",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_CHAMBER_ENTRANCE_CAP: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/chamber/entrance_cap",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:trial_chambers/chamber/entrance_cap",
                processors: "minecraft:trial_chambers_copper_bulb_degradation",
                legacy: false,
            },
        }],
    };
    pub const TRIAL_CHAMBERS_CHAMBER_ERUPTION: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/chamber/eruption",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/eruption/center_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/eruption/breeze_slice_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/eruption/slice_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/eruption/slice_2",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/eruption/slice_3",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/eruption/quadrant_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/eruption/quadrant_2",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/eruption/quadrant_3",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/eruption/quadrant_4",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/eruption/quadrant_5",
                    processors: "",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_CHAMBER_PEDESTAL: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/chamber/pedestal",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/pedestal/center_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/pedestal/slice_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/pedestal/slice_2",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/pedestal/slice_3",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/pedestal/slice_4",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/pedestal/slice_5",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/pedestal/ominous_slice_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/pedestal/quadrant_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/pedestal/quadrant_2",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/pedestal/quadrant_3",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/slanted/quadrant_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/slanted/quadrant_2",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/slanted/quadrant_3",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/slanted/quadrant_4",
                    processors: "",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_CHAMBER_SLANTED: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/chamber/slanted",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/slanted/center",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/slanted/hallway_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/slanted/hallway_2",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/slanted/hallway_3",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/slanted/quadrant_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/slanted/quadrant_2",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/slanted/quadrant_3",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/slanted/quadrant_4",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/slanted/ramp_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/slanted/ramp_2",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/slanted/ramp_3",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/slanted/ramp_4",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/slanted/ominous_upper_arm_1",
                    processors: "",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_CHAMBERS_END: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/chambers/end",
        fallback: "minecraft:trial_chambers/hallway/fallback",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/chamber_1",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/eruption",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/slanted",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_CHESTS_CONTENTS_SUPPLY: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/chests/contents/supply",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:trial_chambers/chests/supply",
                processors: "",
                legacy: false,
            },
        }],
    };
    pub const TRIAL_CHAMBERS_CHESTS_SUPPLY: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/chests/supply",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:trial_chambers/chests/connectors/supply",
                processors: "",
                legacy: false,
            },
        }],
    };
    pub const TRIAL_CHAMBERS_CORRIDOR: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/corridor",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/second_plate",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/intersection/intersection_1",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/intersection/intersection_2",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/intersection/intersection_3",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/first_plate",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/atrium_1",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/entrance_1",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/entrance_2",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/entrance_3",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_CORRIDOR_SLICES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/corridor/slices",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/straight_1",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/straight_2",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/straight_3",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/straight_4",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/straight_5",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/straight_6",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/straight_7",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/straight_8",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_CORRIDORS_ADDON_LOWER: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/corridors/addon/lower",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 8u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/addon/staircase",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/addon/wall",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/addon/ladder_to_middle",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/addon/arrow_dispenser",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/addon/bridge_lower",
                    processors: "",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_CORRIDORS_ADDON_MIDDLE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/corridors/addon/middle",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 8u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/addon/open_walkway",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/addon/walled_walkway",
                    processors: "",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_CORRIDORS_ADDON_MIDDLE_UPPER: StaticTemplatePool =
        StaticTemplatePool {
            id: "minecraft:trial_chambers/corridors/addon/middle_upper",
            fallback: "minecraft:empty",
            elements: &[
                StaticPoolElement {
                    weight: 6u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Empty,
                },
                StaticPoolElement {
                    weight: 2u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:trial_chambers/corridor/addon/open_walkway_upper",
                        processors: "",
                        legacy: false,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:trial_chambers/corridor/addon/chandelier_upper",
                        processors: "",
                        legacy: false,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:trial_chambers/corridor/addon/decoration_upper",
                        processors: "",
                        legacy: false,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:trial_chambers/corridor/addon/head_upper",
                        processors: "",
                        legacy: false,
                    },
                },
                StaticPoolElement {
                    weight: 1u32,
                    projection: TemplatePoolProjection::Rigid,
                    kind: StaticPoolElementKind::Single {
                        location: "minecraft:trial_chambers/corridor/addon/reward_upper",
                        processors: "",
                        legacy: false,
                    },
                },
            ],
        };
    pub const TRIAL_CHAMBERS_DECOR: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/decor",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 22u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/empty_pot",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/dead_bush_pot",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 10u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/undecorated_pot",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/flow_pot",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/guster_pot",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/scrape_pot",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/candle_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/candle_2",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/candle_3",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/candle_4",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/barrel",
                    processors: "",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_DECOR_BED: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/decor/bed",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/white_bed",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/light_gray_bed",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/gray_bed",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/black_bed",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/brown_bed",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/red_bed",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/orange_bed",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/yellow_bed",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/lime_bed",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/green_bed",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/cyan_bed",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/light_blue_bed",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/blue_bed",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/purple_bed",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/magenta_bed",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/pink_bed",
                    processors: "",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_DECOR_CHAMBER: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/decor/chamber",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/decor/undecorated_pot",
                    processors: "",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_DECOR_DISPOSAL: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/decor/disposal",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:trial_chambers/decor/disposal",
                processors: "",
                legacy: false,
            },
        }],
    };
    pub const TRIAL_CHAMBERS_DISPENSERS_CHAMBER: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/dispensers/chamber",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/dispensers/chamber",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/dispensers/wall_dispenser",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/dispensers/floor_dispenser",
                    processors: "",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_ENTRANCE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/entrance",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/addon/display_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/addon/display_2",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/corridor/addon/display_3",
                    processors: "",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_HALLWAY: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/hallway",
        fallback: "minecraft:trial_chambers/hallway/fallback",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/corridor_connector_1",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/upper_hallway_connector",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/lower_hallway_connector",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/rubble",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 150u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/chamber_1",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 150u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/chamber_2",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 150u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/chamber_4",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 150u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/chamber_8",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 150u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/assembly",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 150u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/eruption",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 150u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/slanted",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 150u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/chamber/pedestal",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 10u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/rubble_chamber",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/rubble_chamber_thin",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/cache_1",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/left_corner",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/right_corner",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/corner_staircase",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/corner_staircase_down",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/long_straight_staircase",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/long_straight_staircase_down",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/straight",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/straight_staircase",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/straight_staircase_down",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/trapped_staircase",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/encounter_1",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/encounter_2",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/encounter_3",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/encounter_4",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/encounter_5",
                    processors: "minecraft:trial_chambers_copper_bulb_degradation",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_HALLWAY_FALLBACK: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/hallway/fallback",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/rubble",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/rubble_chamber",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/rubble_thin",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/hallway/rubble_chamber_thin",
                    processors: "",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_REWARD_ALL: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/reward/all",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:trial_chambers/reward/vault",
                processors: "",
                legacy: false,
            },
        }],
    };
    pub const TRIAL_CHAMBERS_REWARD_CONTENTS_DEFAULT: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/reward/contents/default",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:trial_chambers/reward/vault",
                processors: "",
                legacy: false,
            },
        }],
    };
    pub const TRIAL_CHAMBERS_REWARD_OMINOUS_VAULT: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/reward/ominous_vault",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:trial_chambers/reward/ominous_vault",
                processors: "",
                legacy: false,
            },
        }],
    };
    pub const TRIAL_CHAMBERS_SPAWNER_ALL: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/spawner/all",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/spawner/connectors/ranged",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/spawner/connectors/melee",
                    processors: "",
                    legacy: false,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/spawner/connectors/small_melee",
                    processors: "",
                    legacy: false,
                },
            },
        ],
    };
    pub const TRIAL_CHAMBERS_SPAWNER_BREEZE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/spawner/breeze",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:trial_chambers/spawner/connectors/breeze",
                processors: "",
                legacy: false,
            },
        }],
    };
    pub const TRIAL_CHAMBERS_SPAWNER_CONTENTS_BREEZE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/spawner/contents/breeze",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:trial_chambers/spawner/breeze/breeze",
                processors: "",
                legacy: false,
            },
        }],
    };
    pub const TRIAL_CHAMBERS_SPAWNER_MELEE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/spawner/melee",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:trial_chambers/spawner/connectors/melee",
                processors: "",
                legacy: false,
            },
        }],
    };
    pub const TRIAL_CHAMBERS_SPAWNER_MELEE_HUSK: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/spawner/melee/husk",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:trial_chambers/spawner/melee/husk",
                processors: "",
                legacy: false,
            },
        }],
    };
    pub const TRIAL_CHAMBERS_SPAWNER_MELEE_SPIDER: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/spawner/melee/spider",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:trial_chambers/spawner/melee/spider",
                processors: "",
                legacy: false,
            },
        }],
    };
    pub const TRIAL_CHAMBERS_SPAWNER_MELEE_ZOMBIE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/spawner/melee/zombie",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:trial_chambers/spawner/melee/zombie",
                processors: "",
                legacy: false,
            },
        }],
    };
    pub const TRIAL_CHAMBERS_SPAWNER_RANGED: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/spawner/ranged",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:trial_chambers/spawner/connectors/ranged",
                processors: "",
                legacy: false,
            },
        }],
    };
    pub const TRIAL_CHAMBERS_SPAWNER_RANGED_POISON_SKELETON: StaticTemplatePool =
        StaticTemplatePool {
            id: "minecraft:trial_chambers/spawner/ranged/poison_skeleton",
            fallback: "minecraft:empty",
            elements: &[StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/spawner/ranged/poison_skeleton",
                    processors: "",
                    legacy: false,
                },
            }],
        };
    pub const TRIAL_CHAMBERS_SPAWNER_RANGED_SKELETON: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/spawner/ranged/skeleton",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:trial_chambers/spawner/ranged/skeleton",
                processors: "",
                legacy: false,
            },
        }],
    };
    pub const TRIAL_CHAMBERS_SPAWNER_RANGED_STRAY: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/spawner/ranged/stray",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:trial_chambers/spawner/ranged/stray",
                processors: "",
                legacy: false,
            },
        }],
    };
    pub const TRIAL_CHAMBERS_SPAWNER_SLOW_RANGED: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/spawner/slow_ranged",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:trial_chambers/spawner/connectors/slow_ranged",
                processors: "",
                legacy: false,
            },
        }],
    };
    pub const TRIAL_CHAMBERS_SPAWNER_SLOW_RANGED_POISON_SKELETON: StaticTemplatePool =
        StaticTemplatePool {
            id: "minecraft:trial_chambers/spawner/slow_ranged/poison_skeleton",
            fallback: "minecraft:empty",
            elements: &[StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/spawner/slow_ranged/poison_skeleton",
                    processors: "",
                    legacy: false,
                },
            }],
        };
    pub const TRIAL_CHAMBERS_SPAWNER_SLOW_RANGED_SKELETON: StaticTemplatePool =
        StaticTemplatePool {
            id: "minecraft:trial_chambers/spawner/slow_ranged/skeleton",
            fallback: "minecraft:empty",
            elements: &[StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/spawner/slow_ranged/skeleton",
                    processors: "",
                    legacy: false,
                },
            }],
        };
    pub const TRIAL_CHAMBERS_SPAWNER_SLOW_RANGED_STRAY: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/spawner/slow_ranged/stray",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:trial_chambers/spawner/slow_ranged/stray",
                processors: "",
                legacy: false,
            },
        }],
    };
    pub const TRIAL_CHAMBERS_SPAWNER_SMALL_MELEE: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/spawner/small_melee",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:trial_chambers/spawner/connectors/small_melee",
                processors: "",
                legacy: false,
            },
        }],
    };
    pub const TRIAL_CHAMBERS_SPAWNER_SMALL_MELEE_BABY_ZOMBIE: StaticTemplatePool =
        StaticTemplatePool {
            id: "minecraft:trial_chambers/spawner/small_melee/baby_zombie",
            fallback: "minecraft:empty",
            elements: &[StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/spawner/small_melee/baby_zombie",
                    processors: "",
                    legacy: false,
                },
            }],
        };
    pub const TRIAL_CHAMBERS_SPAWNER_SMALL_MELEE_CAVE_SPIDER: StaticTemplatePool =
        StaticTemplatePool {
            id: "minecraft:trial_chambers/spawner/small_melee/cave_spider",
            fallback: "minecraft:empty",
            elements: &[StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/spawner/small_melee/cave_spider",
                    processors: "",
                    legacy: false,
                },
            }],
        };
    pub const TRIAL_CHAMBERS_SPAWNER_SMALL_MELEE_SILVERFISH: StaticTemplatePool =
        StaticTemplatePool {
            id: "minecraft:trial_chambers/spawner/small_melee/silverfish",
            fallback: "minecraft:empty",
            elements: &[StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:trial_chambers/spawner/small_melee/silverfish",
                    processors: "",
                    legacy: false,
                },
            }],
        };
    pub const TRIAL_CHAMBERS_SPAWNER_SMALL_MELEE_SLIME: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:trial_chambers/spawner/small_melee/slime",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:trial_chambers/spawner/small_melee/slime",
                processors: "",
                legacy: false,
            },
        }],
    };
    pub const VILLAGE_COMMON_ANIMALS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/common/animals",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 7u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/cows_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 7u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/pigs_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/horses_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/horses_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/horses_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/horses_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/horses_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/sheep_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/sheep_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 5u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_COMMON_BUTCHER_ANIMALS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/common/butcher_animals",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/cows_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/pigs_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/sheep_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/sheep_2",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_COMMON_CATS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/common/cats",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/cat_black",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/cat_british",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/cat_calico",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/cat_persian",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/cat_ragdoll",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/cat_red",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/cat_siamese",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/cat_tabby",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/cat_white",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/cat_jellie",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_COMMON_IRON_GOLEM: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/common/iron_golem",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:village/common/iron_golem",
                processors: "",
                legacy: true,
            },
        }],
    };
    pub const VILLAGE_COMMON_SHEEP: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/common/sheep",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/sheep_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/common/animals/sheep_2",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_COMMON_WELL_BOTTOMS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/common/well_bottoms",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:village/common/well_bottom",
                processors: "",
                legacy: true,
            },
        }],
    };
    pub const VILLAGE_DESERT_CAMEL: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/desert/camel",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Single {
                location: "minecraft:village/desert/camel_spawn",
                processors: "",
                legacy: true,
            },
        }],
    };
    pub const VILLAGE_DESERT_DECOR: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/desert/decor",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 10u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/desert_lamp_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:patch_cactus"),
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:pile_hay"),
            },
            StaticPoolElement {
                weight: 10u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_DESERT_HOUSES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/desert/houses",
        fallback: "minecraft:village/desert/terminators",
        elements: &[
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_small_house_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_small_house_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_small_house_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_small_house_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_small_house_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_small_house_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_small_house_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_small_house_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_medium_house_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_medium_house_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_butcher_shop_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_tool_smith_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_fletcher_house_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_shepherd_house_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_armorer_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_fisher_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_tannery_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_cartographer_house_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_library_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_mason_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_weaponsmith_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_temple_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_temple_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 11u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_large_farm_1",
                    processors: "minecraft:farm_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_farm_1",
                    processors: "minecraft:farm_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_farm_2",
                    processors: "minecraft:farm_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_animal_pen_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_animal_pen_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 5u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_DESERT_STREETS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/desert/streets",
        fallback: "minecraft:village/desert/terminators",
        elements: &[
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/streets/corner_01",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/streets/corner_02",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/streets/straight_01",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/streets/straight_02",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/streets/straight_03",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/streets/crossroad_01",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/streets/crossroad_02",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/streets/crossroad_03",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/streets/square_01",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/streets/square_02",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/streets/turn_01",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_DESERT_TERMINATORS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/desert/terminators",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/terminators/terminator_01",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/terminators/terminator_02",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_DESERT_TOWN_CENTERS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/desert/town_centers",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 98u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/town_centers/desert_meeting_point_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 98u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/town_centers/desert_meeting_point_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 49u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/town_centers/desert_meeting_point_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/town_centers/desert_meeting_point_1",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/town_centers/desert_meeting_point_2",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/town_centers/desert_meeting_point_3",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_DESERT_VILLAGERS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/desert/villagers",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/villagers/nitwit",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/villagers/baby",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 10u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/villagers/unemployed",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_DESERT_ZOMBIE_DECOR: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/desert/zombie/decor",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 10u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/desert_lamp_1",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:patch_cactus"),
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:pile_hay"),
            },
            StaticPoolElement {
                weight: 10u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_DESERT_ZOMBIE_HOUSES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/desert/zombie/houses",
        fallback: "minecraft:village/desert/zombie/terminators",
        elements: &[
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/houses/desert_small_house_1",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/houses/desert_small_house_2",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/houses/desert_small_house_3",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/houses/desert_small_house_4",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/houses/desert_small_house_5",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/houses/desert_small_house_6",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/houses/desert_small_house_7",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/houses/desert_small_house_8",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/houses/desert_medium_house_1",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/houses/desert_medium_house_2",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_butcher_shop_1",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_tool_smith_1",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_fletcher_house_1",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_shepherd_house_1",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_armorer_1",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_fisher_1",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_tannery_1",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_cartographer_house_1",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_library_1",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_mason_1",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_weaponsmith_1",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_temple_1",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_temple_2",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 7u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_large_farm_1",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_farm_1",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_farm_2",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_animal_pen_1",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/houses/desert_animal_pen_2",
                    processors: "minecraft:zombie_desert",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 5u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_DESERT_ZOMBIE_STREETS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/desert/zombie/streets",
        fallback: "minecraft:village/desert/zombie/terminators",
        elements: &[
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/streets/corner_01",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/streets/corner_02",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/streets/straight_01",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/streets/straight_02",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/streets/straight_03",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/streets/crossroad_01",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/streets/crossroad_02",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/streets/crossroad_03",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/streets/square_01",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/streets/square_02",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/streets/turn_01",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_DESERT_ZOMBIE_TERMINATORS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/desert/zombie/terminators",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/terminators/terminator_01",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/terminators/terminator_02",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_DESERT_ZOMBIE_VILLAGERS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/desert/zombie/villagers",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/villagers/nitwit",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 10u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/desert/zombie/villagers/unemployed",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_PLAINS_DECOR: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/plains/decor",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/plains_lamp_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:oak"),
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:flower_plain"),
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:pile_hay"),
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_PLAINS_HOUSES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/plains/houses",
        fallback: "minecraft:village/plains/terminators",
        elements: &[
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_small_house_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_small_house_2",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_small_house_3",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_small_house_4",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_small_house_5",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_small_house_6",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_small_house_7",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_small_house_8",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_medium_house_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_medium_house_2",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_big_house_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_butcher_shop_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_butcher_shop_2",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_tool_smith_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_fletcher_house_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_shepherds_house_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_armorer_house_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_fisher_cottage_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_tannery_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_cartographer_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 5u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_library_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_library_2",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_masons_house_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_weaponsmith_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_temple_3",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_temple_4",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_stable_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_stable_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_large_farm_1",
                    processors: "minecraft:farm_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_small_farm_1",
                    processors: "minecraft:farm_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_animal_pen_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_animal_pen_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 5u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_animal_pen_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_accessory_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_meeting_point_4",
                    processors: "minecraft:mossify_70_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_meeting_point_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 10u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_PLAINS_STREETS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/plains/streets",
        fallback: "minecraft:village/plains/terminators",
        elements: &[
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/streets/corner_01",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/streets/corner_02",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/streets/corner_03",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/streets/straight_01",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/streets/straight_02",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 7u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/streets/straight_03",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 7u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/streets/straight_04",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/streets/straight_05",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/streets/straight_06",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/streets/crossroad_01",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/streets/crossroad_02",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/streets/crossroad_03",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/streets/crossroad_04",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/streets/crossroad_05",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/streets/crossroad_06",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/streets/turn_01",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_PLAINS_TERMINATORS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/plains/terminators",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/terminators/terminator_01",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/terminators/terminator_02",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/terminators/terminator_03",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/terminators/terminator_04",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_PLAINS_TOWN_CENTERS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/plains/town_centers",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 50u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/town_centers/plains_fountain_01",
                    processors: "minecraft:mossify_20_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 50u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/town_centers/plains_meeting_point_1",
                    processors: "minecraft:mossify_20_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 50u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/town_centers/plains_meeting_point_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 50u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/town_centers/plains_meeting_point_3",
                    processors: "minecraft:mossify_70_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/town_centers/plains_fountain_01",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/town_centers/plains_meeting_point_1",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/town_centers/plains_meeting_point_2",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/town_centers/plains_meeting_point_3",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_PLAINS_TREES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/plains/trees",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:oak"),
        }],
    };
    pub const VILLAGE_PLAINS_VILLAGERS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/plains/villagers",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/villagers/nitwit",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/villagers/baby",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 10u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/villagers/unemployed",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_PLAINS_ZOMBIE_DECOR: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/plains/zombie/decor",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/plains_lamp_1",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:oak"),
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:flower_plain"),
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:pile_hay"),
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_PLAINS_ZOMBIE_HOUSES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/plains/zombie/houses",
        fallback: "minecraft:village/plains/terminators",
        elements: &[
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/houses/plains_small_house_1",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/houses/plains_small_house_2",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/houses/plains_small_house_3",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/houses/plains_small_house_4",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/houses/plains_small_house_5",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/houses/plains_small_house_6",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/houses/plains_small_house_7",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/houses/plains_small_house_8",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/houses/plains_medium_house_1",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/houses/plains_medium_house_2",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/houses/plains_big_house_1",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_butcher_shop_1",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/houses/plains_butcher_shop_2",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_tool_smith_1",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/houses/plains_fletcher_house_1",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/houses/plains_shepherds_house_1",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_armorer_house_1",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_fisher_cottage_1",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_tannery_1",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_cartographer_1",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_library_1",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_library_2",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_masons_house_1",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_weaponsmith_1",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_temple_3",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_temple_4",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/houses/plains_stable_1",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_stable_2",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_large_farm_1",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_small_farm_1",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_animal_pen_1",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/houses/plains_animal_pen_2",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 5u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/houses/plains_animal_pen_3",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/houses/plains_meeting_point_4",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/houses/plains_meeting_point_5",
                    processors: "minecraft:zombie_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 10u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_PLAINS_ZOMBIE_STREETS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/plains/zombie/streets",
        fallback: "minecraft:village/plains/terminators",
        elements: &[
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/streets/corner_01",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/streets/corner_02",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/streets/corner_03",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/streets/straight_01",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/streets/straight_02",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 7u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/streets/straight_03",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 7u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/streets/straight_04",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/streets/straight_05",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/streets/straight_06",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/streets/crossroad_01",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/streets/crossroad_02",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/streets/crossroad_03",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/streets/crossroad_04",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/streets/crossroad_05",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/streets/crossroad_06",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/streets/turn_01",
                    processors: "minecraft:street_plains",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_PLAINS_ZOMBIE_VILLAGERS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/plains/zombie/villagers",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/villagers/nitwit",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 10u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/zombie/villagers/unemployed",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_SAVANNA_DECOR: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/savanna/decor",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/savanna_lamp_post_01",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:acacia"),
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:pile_hay"),
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:pile_melon"),
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_SAVANNA_HOUSES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/savanna/houses",
        fallback: "minecraft:village/savanna/terminators",
        elements: &[
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_small_house_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_small_house_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_small_house_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_small_house_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_small_house_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_small_house_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_small_house_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_small_house_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_medium_house_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_medium_house_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_butchers_shop_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_butchers_shop_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_tool_smith_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_fletcher_house_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 7u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_shepherd_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_armorer_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_fisher_cottage_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_tannery_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_cartographer_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_library_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_mason_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_weaponsmith_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_weaponsmith_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_temple_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_temple_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_large_farm_1",
                    processors: "minecraft:farm_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 6u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_large_farm_2",
                    processors: "minecraft:farm_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_small_farm",
                    processors: "minecraft:farm_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_animal_pen_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_animal_pen_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_animal_pen_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 5u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_SAVANNA_STREETS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/savanna/streets",
        fallback: "minecraft:village/savanna/terminators",
        elements: &[
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/streets/corner_01",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/streets/corner_03",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/streets/straight_02",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 7u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/streets/straight_04",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/streets/straight_05",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/streets/straight_06",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/streets/straight_08",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/streets/straight_09",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/streets/straight_10",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/streets/straight_11",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/streets/crossroad_02",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/streets/crossroad_03",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/streets/crossroad_04",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/streets/crossroad_05",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/streets/crossroad_06",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/streets/crossroad_07",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/streets/split_01",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/streets/split_02",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/streets/turn_01",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_SAVANNA_TERMINATORS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/savanna/terminators",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/terminators/terminator_01",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/terminators/terminator_02",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/terminators/terminator_03",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/terminators/terminator_04",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/terminators/terminator_05",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_SAVANNA_TOWN_CENTERS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/savanna/town_centers",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 100u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/town_centers/savanna_meeting_point_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 50u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/town_centers/savanna_meeting_point_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 150u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/town_centers/savanna_meeting_point_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 150u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/town_centers/savanna_meeting_point_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/town_centers/savanna_meeting_point_1",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/town_centers/savanna_meeting_point_2",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/town_centers/savanna_meeting_point_3",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/town_centers/savanna_meeting_point_4",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_SAVANNA_TREES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/savanna/trees",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:acacia"),
        }],
    };
    pub const VILLAGE_SAVANNA_VILLAGERS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/savanna/villagers",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/villagers/nitwit",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/villagers/baby",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 10u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/villagers/unemployed",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_SAVANNA_ZOMBIE_DECOR: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/savanna/zombie/decor",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/savanna_lamp_post_01",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:acacia"),
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:pile_hay"),
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:pile_melon"),
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_SAVANNA_ZOMBIE_HOUSES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/savanna/zombie/houses",
        fallback: "minecraft:village/savanna/zombie/terminators",
        elements: &[
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/houses/savanna_small_house_1",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/houses/savanna_small_house_2",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/houses/savanna_small_house_3",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/houses/savanna_small_house_4",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/houses/savanna_small_house_5",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/houses/savanna_small_house_6",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/houses/savanna_small_house_7",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/houses/savanna_small_house_8",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/houses/savanna_medium_house_1",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/houses/savanna_medium_house_2",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_butchers_shop_1",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_butchers_shop_2",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_tool_smith_1",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_fletcher_house_1",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_shepherd_1",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_armorer_1",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_fisher_cottage_1",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_tannery_1",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_cartographer_1",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_library_1",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_mason_1",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_weaponsmith_1",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_weaponsmith_2",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_temple_1",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_temple_2",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_large_farm_1",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/houses/savanna_large_farm_2",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_small_farm",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/houses/savanna_animal_pen_1",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/houses/savanna_animal_pen_2",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/houses/savanna_animal_pen_3",
                    processors: "minecraft:zombie_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 5u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_SAVANNA_ZOMBIE_STREETS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/savanna/zombie/streets",
        fallback: "minecraft:village/savanna/zombie/terminators",
        elements: &[
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/streets/corner_01",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/streets/corner_03",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/streets/straight_02",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 7u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/streets/straight_04",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/streets/straight_05",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/streets/straight_06",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/streets/straight_08",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/streets/straight_09",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/streets/straight_10",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/streets/straight_11",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/streets/crossroad_02",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/streets/crossroad_03",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/streets/crossroad_04",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/streets/crossroad_05",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/streets/crossroad_06",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/streets/crossroad_07",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/streets/split_01",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/streets/split_02",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/streets/turn_01",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_SAVANNA_ZOMBIE_TERMINATORS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/savanna/zombie/terminators",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/terminators/terminator_01",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/terminators/terminator_02",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/terminators/terminator_03",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/terminators/terminator_04",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/terminators/terminator_05",
                    processors: "minecraft:street_savanna",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_SAVANNA_ZOMBIE_VILLAGERS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/savanna/zombie/villagers",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/villagers/nitwit",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 10u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/savanna/zombie/villagers/unemployed",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_SNOWY_DECOR: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/snowy/decor",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/snowy_lamp_post_01",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/snowy_lamp_post_02",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/snowy_lamp_post_03",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:spruce"),
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:pile_snow"),
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:pile_ice"),
            },
            StaticPoolElement {
                weight: 9u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_SNOWY_HOUSES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/snowy/houses",
        fallback: "minecraft:village/snowy/terminators",
        elements: &[
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_small_house_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_small_house_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_small_house_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_small_house_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_small_house_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_small_house_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_small_house_7",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_small_house_8",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_medium_house_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_medium_house_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_medium_house_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_butchers_shop_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_butchers_shop_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_tool_smith_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_fletcher_house_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_shepherds_house_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_armorer_house_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_armorer_house_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_fisher_cottage",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_tannery_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_cartographer_house_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_library_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_masons_house_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_masons_house_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_weapon_smith_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_temple_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_farm_1",
                    processors: "minecraft:farm_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_farm_2",
                    processors: "minecraft:farm_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_animal_pen_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_animal_pen_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 6u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_SNOWY_STREETS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/snowy/streets",
        fallback: "minecraft:village/snowy/terminators",
        elements: &[
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/streets/corner_01",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/streets/corner_02",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/streets/corner_03",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/streets/square_01",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/streets/straight_01",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/streets/straight_02",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/streets/straight_03",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 7u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/streets/straight_04",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/streets/straight_06",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/streets/straight_08",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/streets/crossroad_02",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/streets/crossroad_03",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/streets/crossroad_04",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/streets/crossroad_05",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/streets/crossroad_06",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/streets/turn_01",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_SNOWY_TERMINATORS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/snowy/terminators",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/terminators/terminator_01",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/terminators/terminator_02",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/terminators/terminator_03",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/terminators/terminator_04",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_SNOWY_TOWN_CENTERS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/snowy/town_centers",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 100u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/town_centers/snowy_meeting_point_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 50u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/town_centers/snowy_meeting_point_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 150u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/town_centers/snowy_meeting_point_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/town_centers/snowy_meeting_point_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/town_centers/snowy_meeting_point_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/town_centers/snowy_meeting_point_3",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_SNOWY_TREES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/snowy/trees",
        fallback: "minecraft:empty",
        elements: &[StaticPoolElement {
            weight: 1u32,
            projection: TemplatePoolProjection::Rigid,
            kind: StaticPoolElementKind::Feature("minecraft:spruce"),
        }],
    };
    pub const VILLAGE_SNOWY_VILLAGERS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/snowy/villagers",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/villagers/nitwit",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/villagers/baby",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 10u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/villagers/unemployed",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_SNOWY_ZOMBIE_DECOR: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/snowy/zombie/decor",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/snowy_lamp_post_01",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/snowy_lamp_post_02",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/snowy_lamp_post_03",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:spruce"),
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:pile_snow"),
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:pile_ice"),
            },
            StaticPoolElement {
                weight: 7u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_SNOWY_ZOMBIE_HOUSES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/snowy/zombie/houses",
        fallback: "minecraft:village/snowy/terminators",
        elements: &[
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/houses/snowy_small_house_1",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/houses/snowy_small_house_2",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/houses/snowy_small_house_3",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/houses/snowy_small_house_4",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/houses/snowy_small_house_5",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/houses/snowy_small_house_6",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/houses/snowy_small_house_7",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/houses/snowy_small_house_8",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/houses/snowy_medium_house_1",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/houses/snowy_medium_house_2",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/houses/snowy_medium_house_3",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_butchers_shop_1",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_butchers_shop_2",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_tool_smith_1",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_fletcher_house_1",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_shepherds_house_1",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_armorer_house_1",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_armorer_house_2",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_fisher_cottage",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_tannery_1",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_cartographer_house_1",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_library_1",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_masons_house_1",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_masons_house_2",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_weapon_smith_1",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_temple_1",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_farm_1",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_farm_2",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_animal_pen_1",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/houses/snowy_animal_pen_2",
                    processors: "minecraft:zombie_snowy",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 6u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_SNOWY_ZOMBIE_STREETS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/snowy/zombie/streets",
        fallback: "minecraft:village/snowy/terminators",
        elements: &[
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/streets/corner_01",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/streets/corner_02",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/streets/corner_03",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/streets/square_01",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/streets/straight_01",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/streets/straight_02",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/streets/straight_03",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 7u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/streets/straight_04",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/streets/straight_06",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/streets/straight_08",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/streets/crossroad_02",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/streets/crossroad_03",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/streets/crossroad_04",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/streets/crossroad_05",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/streets/crossroad_06",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/streets/turn_01",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_SNOWY_ZOMBIE_VILLAGERS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/snowy/zombie/villagers",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/villagers/nitwit",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 10u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/snowy/zombie/villagers/unemployed",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_TAIGA_DECOR: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/taiga/decor",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 10u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/taiga_lamp_post_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/taiga_decoration_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/taiga_decoration_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/taiga_decoration_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/taiga_decoration_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/taiga_decoration_5",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/taiga_decoration_6",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:spruce"),
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:pine"),
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:pile_pumpkin"),
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:patch_taiga_grass"),
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:patch_berry_bush"),
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_TAIGA_HOUSES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/taiga/houses",
        fallback: "minecraft:village/taiga/terminators",
        elements: &[
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_small_house_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_small_house_2",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_small_house_3",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_small_house_4",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_small_house_5",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_medium_house_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_medium_house_2",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_medium_house_3",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_medium_house_4",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_butcher_shop_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_tool_smith_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_fletcher_house_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_shepherds_house_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_armorer_house_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_armorer_2",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_fisher_cottage_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_tannery_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_cartographer_house_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_library_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_masons_house_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_weaponsmith_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_weaponsmith_2",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_temple_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 6u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_large_farm_1",
                    processors: "minecraft:farm_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 6u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_large_farm_2",
                    processors: "minecraft:farm_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_small_farm_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_animal_pen_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 6u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_TAIGA_STREETS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/taiga/streets",
        fallback: "minecraft:village/taiga/terminators",
        elements: &[
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/streets/corner_01",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/streets/corner_02",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/streets/corner_03",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/streets/straight_01",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/streets/straight_02",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/streets/straight_03",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 7u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/streets/straight_04",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 7u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/streets/straight_05",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/streets/straight_06",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/streets/crossroad_01",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/streets/crossroad_02",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/streets/crossroad_03",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/streets/crossroad_04",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/streets/crossroad_05",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/streets/crossroad_06",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/streets/turn_01",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_TAIGA_TERMINATORS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/taiga/terminators",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/terminators/terminator_01",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/terminators/terminator_02",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/terminators/terminator_03",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/plains/terminators/terminator_04",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_TAIGA_TOWN_CENTERS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/taiga/town_centers",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 49u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/town_centers/taiga_meeting_point_1",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 49u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/town_centers/taiga_meeting_point_2",
                    processors: "minecraft:mossify_10_percent",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/town_centers/taiga_meeting_point_1",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/town_centers/taiga_meeting_point_2",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_TAIGA_VILLAGERS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/taiga/villagers",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/villagers/nitwit",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/villagers/baby",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 10u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/villagers/unemployed",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_TAIGA_ZOMBIE_DECOR: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/taiga/zombie/decor",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/taiga_decoration_1",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/taiga_decoration_2",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/taiga_decoration_3",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/taiga_decoration_4",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:spruce"),
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:pine"),
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:pile_pumpkin"),
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:patch_taiga_grass"),
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Feature("minecraft:patch_berry_bush"),
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_TAIGA_ZOMBIE_HOUSES: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/taiga/zombie/houses",
        fallback: "minecraft:village/taiga/terminators",
        elements: &[
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/houses/taiga_small_house_1",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/houses/taiga_small_house_2",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/houses/taiga_small_house_3",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/houses/taiga_small_house_4",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/houses/taiga_small_house_5",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/houses/taiga_medium_house_1",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/houses/taiga_medium_house_2",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/houses/taiga_medium_house_3",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/houses/taiga_medium_house_4",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_butcher_shop_1",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/houses/taiga_tool_smith_1",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_fletcher_house_1",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/houses/taiga_shepherds_house_1",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_armorer_house_1",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/houses/taiga_fisher_cottage_1",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_tannery_1",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/houses/taiga_cartographer_house_1",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/houses/taiga_library_1",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_masons_house_1",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_weaponsmith_1",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/houses/taiga_weaponsmith_2",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/houses/taiga_temple_1",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 6u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_large_farm_1",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 6u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/houses/taiga_large_farm_2",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_small_farm_1",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/houses/taiga_animal_pen_1",
                    processors: "minecraft:zombie_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 6u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Empty,
            },
        ],
    };
    pub const VILLAGE_TAIGA_ZOMBIE_STREETS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/taiga/zombie/streets",
        fallback: "minecraft:village/taiga/terminators",
        elements: &[
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/streets/corner_01",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/streets/corner_02",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/streets/corner_03",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/streets/straight_01",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/streets/straight_02",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/streets/straight_03",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 7u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/streets/straight_04",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 7u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/streets/straight_05",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 4u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/streets/straight_06",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/streets/crossroad_01",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/streets/crossroad_02",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/streets/crossroad_03",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/streets/crossroad_04",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/streets/crossroad_05",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 2u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/streets/crossroad_06",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 3u32,
                projection: TemplatePoolProjection::TerrainMatching,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/streets/turn_01",
                    processors: "minecraft:street_snowy_or_taiga",
                    legacy: true,
                },
            },
        ],
    };
    pub const VILLAGE_TAIGA_ZOMBIE_VILLAGERS: StaticTemplatePool = StaticTemplatePool {
        id: "minecraft:village/taiga/zombie/villagers",
        fallback: "minecraft:empty",
        elements: &[
            StaticPoolElement {
                weight: 1u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/villagers/nitwit",
                    processors: "",
                    legacy: true,
                },
            },
            StaticPoolElement {
                weight: 10u32,
                projection: TemplatePoolProjection::Rigid,
                kind: StaticPoolElementKind::Single {
                    location: "minecraft:village/taiga/zombie/villagers/unemployed",
                    processors: "",
                    legacy: true,
                },
            },
        ],
    };
    #[must_use]
    pub fn get(name: &str) -> Option<&'static Self> {
        let stripped = name.strip_prefix("minecraft:").unwrap_or(name);
        match stripped {
            "abandoned_camp/camp/bamboo_jungle" => Some(&Self::ABANDONED_CAMP_CAMP_BAMBOO_JUNGLE),
            "abandoned_camp/camp/birch_forest" => Some(&Self::ABANDONED_CAMP_CAMP_BIRCH_FOREST),
            "abandoned_camp/camp/cherry_grove" => Some(&Self::ABANDONED_CAMP_CAMP_CHERRY_GROVE),
            "abandoned_camp/camp/dappled_forest" => Some(&Self::ABANDONED_CAMP_CAMP_DAPPLED_FOREST),
            "abandoned_camp/camp/flower_forest" => Some(&Self::ABANDONED_CAMP_CAMP_FLOWER_FOREST),
            "abandoned_camp/camp/forest" => Some(&Self::ABANDONED_CAMP_CAMP_FOREST),
            "abandoned_camp/camp/meadow" => Some(&Self::ABANDONED_CAMP_CAMP_MEADOW),
            "abandoned_camp/camp/old_growth_birch_forest" => {
                Some(&Self::ABANDONED_CAMP_CAMP_OLD_GROWTH_BIRCH_FOREST)
            }
            "abandoned_camp/camp/old_growth_pine_taiga" => {
                Some(&Self::ABANDONED_CAMP_CAMP_OLD_GROWTH_PINE_TAIGA)
            }
            "abandoned_camp/camp/old_growth_spruce_taiga" => {
                Some(&Self::ABANDONED_CAMP_CAMP_OLD_GROWTH_SPRUCE_TAIGA)
            }
            "abandoned_camp/camp/pale_garden" => Some(&Self::ABANDONED_CAMP_CAMP_PALE_GARDEN),
            "abandoned_camp/camp/savanna" => Some(&Self::ABANDONED_CAMP_CAMP_SAVANNA),
            "abandoned_camp/camp/snowy_taiga" => Some(&Self::ABANDONED_CAMP_CAMP_SNOWY_TAIGA),
            "abandoned_camp/camp/sparse_jungle" => Some(&Self::ABANDONED_CAMP_CAMP_SPARSE_JUNGLE),
            "abandoned_camp/camp/swamp" => Some(&Self::ABANDONED_CAMP_CAMP_SWAMP),
            "abandoned_camp/camp/taiga" => Some(&Self::ABANDONED_CAMP_CAMP_TAIGA),
            "abandoned_camp/camp/windswept_forest" => {
                Some(&Self::ABANDONED_CAMP_CAMP_WINDSWEPT_FOREST)
            }
            "abandoned_camp/camp/wooded_badlands" => {
                Some(&Self::ABANDONED_CAMP_CAMP_WOODED_BADLANDS)
            }
            "abandoned_camp/tent/bamboo_jungle" => Some(&Self::ABANDONED_CAMP_TENT_BAMBOO_JUNGLE),
            "abandoned_camp/tent/birch_forest" => Some(&Self::ABANDONED_CAMP_TENT_BIRCH_FOREST),
            "abandoned_camp/tent/cherry_grove" => Some(&Self::ABANDONED_CAMP_TENT_CHERRY_GROVE),
            "abandoned_camp/tent/dappled_forest" => Some(&Self::ABANDONED_CAMP_TENT_DAPPLED_FOREST),
            "abandoned_camp/tent/flower_forest" => Some(&Self::ABANDONED_CAMP_TENT_FLOWER_FOREST),
            "abandoned_camp/tent/forest" => Some(&Self::ABANDONED_CAMP_TENT_FOREST),
            "abandoned_camp/tent/meadow" => Some(&Self::ABANDONED_CAMP_TENT_MEADOW),
            "abandoned_camp/tent/old_growth_birch_forest" => {
                Some(&Self::ABANDONED_CAMP_TENT_OLD_GROWTH_BIRCH_FOREST)
            }
            "abandoned_camp/tent/old_growth_pine_taiga" => {
                Some(&Self::ABANDONED_CAMP_TENT_OLD_GROWTH_PINE_TAIGA)
            }
            "abandoned_camp/tent/old_growth_spruce_taiga" => {
                Some(&Self::ABANDONED_CAMP_TENT_OLD_GROWTH_SPRUCE_TAIGA)
            }
            "abandoned_camp/tent/pale_garden" => Some(&Self::ABANDONED_CAMP_TENT_PALE_GARDEN),
            "abandoned_camp/tent/savanna" => Some(&Self::ABANDONED_CAMP_TENT_SAVANNA),
            "abandoned_camp/tent/snowy_taiga" => Some(&Self::ABANDONED_CAMP_TENT_SNOWY_TAIGA),
            "abandoned_camp/tent/sparse_jungle" => Some(&Self::ABANDONED_CAMP_TENT_SPARSE_JUNGLE),
            "abandoned_camp/tent/swamp" => Some(&Self::ABANDONED_CAMP_TENT_SWAMP),
            "abandoned_camp/tent/taiga" => Some(&Self::ABANDONED_CAMP_TENT_TAIGA),
            "abandoned_camp/tent/windswept_forest" => {
                Some(&Self::ABANDONED_CAMP_TENT_WINDSWEPT_FOREST)
            }
            "abandoned_camp/tent/wooded_badlands" => {
                Some(&Self::ABANDONED_CAMP_TENT_WOODED_BADLANDS)
            }
            "abandoned_camp/trees/acacia" => Some(&Self::ABANDONED_CAMP_TREES_ACACIA),
            "abandoned_camp/trees/bamboo" => Some(&Self::ABANDONED_CAMP_TREES_BAMBOO),
            "abandoned_camp/trees/birch" => Some(&Self::ABANDONED_CAMP_TREES_BIRCH),
            "abandoned_camp/trees/birch_bees" => Some(&Self::ABANDONED_CAMP_TREES_BIRCH_BEES),
            "abandoned_camp/trees/cherry" => Some(&Self::ABANDONED_CAMP_TREES_CHERRY),
            "abandoned_camp/trees/cherry_bees" => Some(&Self::ABANDONED_CAMP_TREES_CHERRY_BEES),
            "abandoned_camp/trees/fancy_oak" => Some(&Self::ABANDONED_CAMP_TREES_FANCY_OAK),
            "abandoned_camp/trees/fancy_oak_bees" => {
                Some(&Self::ABANDONED_CAMP_TREES_FANCY_OAK_BEES)
            }
            "abandoned_camp/trees/jungle" => Some(&Self::ABANDONED_CAMP_TREES_JUNGLE),
            "abandoned_camp/trees/mega_jungle" => Some(&Self::ABANDONED_CAMP_TREES_MEGA_JUNGLE),
            "abandoned_camp/trees/mega_pine" => Some(&Self::ABANDONED_CAMP_TREES_MEGA_PINE),
            "abandoned_camp/trees/oak" => Some(&Self::ABANDONED_CAMP_TREES_OAK),
            "abandoned_camp/trees/orange_poplar" => Some(&Self::ABANDONED_CAMP_TREES_ORANGE_POPLAR),
            "abandoned_camp/trees/pale_oak" => Some(&Self::ABANDONED_CAMP_TREES_PALE_OAK),
            "abandoned_camp/trees/pine" => Some(&Self::ABANDONED_CAMP_TREES_PINE),
            "abandoned_camp/trees/red_poplar" => Some(&Self::ABANDONED_CAMP_TREES_RED_POPLAR),
            "abandoned_camp/trees/spruce" => Some(&Self::ABANDONED_CAMP_TREES_SPRUCE),
            "abandoned_camp/trees/spruce_on_snow" => {
                Some(&Self::ABANDONED_CAMP_TREES_SPRUCE_ON_SNOW)
            }
            "abandoned_camp/trees/super_birch_bees" => {
                Some(&Self::ABANDONED_CAMP_TREES_SUPER_BIRCH_BEES)
            }
            "abandoned_camp/trees/thick_spruce" => Some(&Self::ABANDONED_CAMP_TREES_THICK_SPRUCE),
            "abandoned_camp/trees/yellow_poplar" => Some(&Self::ABANDONED_CAMP_TREES_YELLOW_POPLAR),
            "ancient_city/city/entrance" => Some(&Self::ANCIENT_CITY_CITY_ENTRANCE),
            "ancient_city/city_center" => Some(&Self::ANCIENT_CITY_CITY_CENTER),
            "ancient_city/city_center/walls" => Some(&Self::ANCIENT_CITY_CITY_CENTER_WALLS),
            "ancient_city/sculk" => Some(&Self::ANCIENT_CITY_SCULK),
            "ancient_city/structures" => Some(&Self::ANCIENT_CITY_STRUCTURES),
            "ancient_city/walls" => Some(&Self::ANCIENT_CITY_WALLS),
            "ancient_city/walls/no_corners" => Some(&Self::ANCIENT_CITY_WALLS_NO_CORNERS),
            "bastion/blocks/gold" => Some(&Self::BASTION_BLOCKS_GOLD),
            "bastion/bridge/bridge_pieces" => Some(&Self::BASTION_BRIDGE_BRIDGE_PIECES),
            "bastion/bridge/connectors" => Some(&Self::BASTION_BRIDGE_CONNECTORS),
            "bastion/bridge/legs" => Some(&Self::BASTION_BRIDGE_LEGS),
            "bastion/bridge/rampart_plates" => Some(&Self::BASTION_BRIDGE_RAMPART_PLATES),
            "bastion/bridge/ramparts" => Some(&Self::BASTION_BRIDGE_RAMPARTS),
            "bastion/bridge/starting_pieces" => Some(&Self::BASTION_BRIDGE_STARTING_PIECES),
            "bastion/bridge/walls" => Some(&Self::BASTION_BRIDGE_WALLS),
            "bastion/hoglin_stable/connectors" => Some(&Self::BASTION_HOGLIN_STABLE_CONNECTORS),
            "bastion/hoglin_stable/large_stables/inner" => {
                Some(&Self::BASTION_HOGLIN_STABLE_LARGE_STABLES_INNER)
            }
            "bastion/hoglin_stable/large_stables/outer" => {
                Some(&Self::BASTION_HOGLIN_STABLE_LARGE_STABLES_OUTER)
            }
            "bastion/hoglin_stable/mirrored_starting_pieces" => {
                Some(&Self::BASTION_HOGLIN_STABLE_MIRRORED_STARTING_PIECES)
            }
            "bastion/hoglin_stable/posts" => Some(&Self::BASTION_HOGLIN_STABLE_POSTS),
            "bastion/hoglin_stable/rampart_plates" => {
                Some(&Self::BASTION_HOGLIN_STABLE_RAMPART_PLATES)
            }
            "bastion/hoglin_stable/ramparts" => Some(&Self::BASTION_HOGLIN_STABLE_RAMPARTS),
            "bastion/hoglin_stable/small_stables/inner" => {
                Some(&Self::BASTION_HOGLIN_STABLE_SMALL_STABLES_INNER)
            }
            "bastion/hoglin_stable/small_stables/outer" => {
                Some(&Self::BASTION_HOGLIN_STABLE_SMALL_STABLES_OUTER)
            }
            "bastion/hoglin_stable/stairs" => Some(&Self::BASTION_HOGLIN_STABLE_STAIRS),
            "bastion/hoglin_stable/starting_pieces" => {
                Some(&Self::BASTION_HOGLIN_STABLE_STARTING_PIECES)
            }
            "bastion/hoglin_stable/wall_bases" => Some(&Self::BASTION_HOGLIN_STABLE_WALL_BASES),
            "bastion/hoglin_stable/walls" => Some(&Self::BASTION_HOGLIN_STABLE_WALLS),
            "bastion/mobs/hoglin" => Some(&Self::BASTION_MOBS_HOGLIN),
            "bastion/mobs/piglin" => Some(&Self::BASTION_MOBS_PIGLIN),
            "bastion/mobs/piglin_melee" => Some(&Self::BASTION_MOBS_PIGLIN_MELEE),
            "bastion/starts" => Some(&Self::BASTION_STARTS),
            "bastion/treasure/bases" => Some(&Self::BASTION_TREASURE_BASES),
            "bastion/treasure/bases/centers" => Some(&Self::BASTION_TREASURE_BASES_CENTERS),
            "bastion/treasure/brains" => Some(&Self::BASTION_TREASURE_BRAINS),
            "bastion/treasure/connectors" => Some(&Self::BASTION_TREASURE_CONNECTORS),
            "bastion/treasure/corners/bottom" => Some(&Self::BASTION_TREASURE_CORNERS_BOTTOM),
            "bastion/treasure/corners/edges" => Some(&Self::BASTION_TREASURE_CORNERS_EDGES),
            "bastion/treasure/corners/middle" => Some(&Self::BASTION_TREASURE_CORNERS_MIDDLE),
            "bastion/treasure/corners/top" => Some(&Self::BASTION_TREASURE_CORNERS_TOP),
            "bastion/treasure/entrances" => Some(&Self::BASTION_TREASURE_ENTRANCES),
            "bastion/treasure/extensions/houses" => Some(&Self::BASTION_TREASURE_EXTENSIONS_HOUSES),
            "bastion/treasure/extensions/large_pool" => {
                Some(&Self::BASTION_TREASURE_EXTENSIONS_LARGE_POOL)
            }
            "bastion/treasure/extensions/small_pool" => {
                Some(&Self::BASTION_TREASURE_EXTENSIONS_SMALL_POOL)
            }
            "bastion/treasure/ramparts" => Some(&Self::BASTION_TREASURE_RAMPARTS),
            "bastion/treasure/roofs" => Some(&Self::BASTION_TREASURE_ROOFS),
            "bastion/treasure/stairs" => Some(&Self::BASTION_TREASURE_STAIRS),
            "bastion/treasure/walls" => Some(&Self::BASTION_TREASURE_WALLS),
            "bastion/treasure/walls/bottom" => Some(&Self::BASTION_TREASURE_WALLS_BOTTOM),
            "bastion/treasure/walls/mid" => Some(&Self::BASTION_TREASURE_WALLS_MID),
            "bastion/treasure/walls/outer" => Some(&Self::BASTION_TREASURE_WALLS_OUTER),
            "bastion/treasure/walls/top" => Some(&Self::BASTION_TREASURE_WALLS_TOP),
            "bastion/units/center_pieces" => Some(&Self::BASTION_UNITS_CENTER_PIECES),
            "bastion/units/edge_wall_units" => Some(&Self::BASTION_UNITS_EDGE_WALL_UNITS),
            "bastion/units/edges" => Some(&Self::BASTION_UNITS_EDGES),
            "bastion/units/fillers/stage_0" => Some(&Self::BASTION_UNITS_FILLERS_STAGE_0),
            "bastion/units/large_ramparts" => Some(&Self::BASTION_UNITS_LARGE_RAMPARTS),
            "bastion/units/pathways" => Some(&Self::BASTION_UNITS_PATHWAYS),
            "bastion/units/rampart_plates" => Some(&Self::BASTION_UNITS_RAMPART_PLATES),
            "bastion/units/ramparts" => Some(&Self::BASTION_UNITS_RAMPARTS),
            "bastion/units/stages/rot/stage_1" => Some(&Self::BASTION_UNITS_STAGES_ROT_STAGE_1),
            "bastion/units/stages/stage_0" => Some(&Self::BASTION_UNITS_STAGES_STAGE_0),
            "bastion/units/stages/stage_1" => Some(&Self::BASTION_UNITS_STAGES_STAGE_1),
            "bastion/units/stages/stage_2" => Some(&Self::BASTION_UNITS_STAGES_STAGE_2),
            "bastion/units/stages/stage_3" => Some(&Self::BASTION_UNITS_STAGES_STAGE_3),
            "bastion/units/wall_units" => Some(&Self::BASTION_UNITS_WALL_UNITS),
            "bastion/units/walls/wall_bases" => Some(&Self::BASTION_UNITS_WALLS_WALL_BASES),
            "empty" => Some(&Self::EMPTY),
            "pillager_outpost/base_plates" => Some(&Self::PILLAGER_OUTPOST_BASE_PLATES),
            "pillager_outpost/feature_plates" => Some(&Self::PILLAGER_OUTPOST_FEATURE_PLATES),
            "pillager_outpost/features" => Some(&Self::PILLAGER_OUTPOST_FEATURES),
            "pillager_outpost/towers" => Some(&Self::PILLAGER_OUTPOST_TOWERS),
            "trail_ruins/buildings" => Some(&Self::TRAIL_RUINS_BUILDINGS),
            "trail_ruins/buildings/grouped" => Some(&Self::TRAIL_RUINS_BUILDINGS_GROUPED),
            "trail_ruins/decor" => Some(&Self::TRAIL_RUINS_DECOR),
            "trail_ruins/roads" => Some(&Self::TRAIL_RUINS_ROADS),
            "trail_ruins/tower" => Some(&Self::TRAIL_RUINS_TOWER),
            "trail_ruins/tower/additions" => Some(&Self::TRAIL_RUINS_TOWER_ADDITIONS),
            "trail_ruins/tower/tower_top" => Some(&Self::TRAIL_RUINS_TOWER_TOWER_TOP),
            "trial_chambers/atrium" => Some(&Self::TRIAL_CHAMBERS_ATRIUM),
            "trial_chambers/chamber/addon" => Some(&Self::TRIAL_CHAMBERS_CHAMBER_ADDON),
            "trial_chambers/chamber/assembly" => Some(&Self::TRIAL_CHAMBERS_CHAMBER_ASSEMBLY),
            "trial_chambers/chamber/end" => Some(&Self::TRIAL_CHAMBERS_CHAMBER_END),
            "trial_chambers/chamber/entrance_cap" => {
                Some(&Self::TRIAL_CHAMBERS_CHAMBER_ENTRANCE_CAP)
            }
            "trial_chambers/chamber/eruption" => Some(&Self::TRIAL_CHAMBERS_CHAMBER_ERUPTION),
            "trial_chambers/chamber/pedestal" => Some(&Self::TRIAL_CHAMBERS_CHAMBER_PEDESTAL),
            "trial_chambers/chamber/slanted" => Some(&Self::TRIAL_CHAMBERS_CHAMBER_SLANTED),
            "trial_chambers/chambers/end" => Some(&Self::TRIAL_CHAMBERS_CHAMBERS_END),
            "trial_chambers/chests/contents/supply" => {
                Some(&Self::TRIAL_CHAMBERS_CHESTS_CONTENTS_SUPPLY)
            }
            "trial_chambers/chests/supply" => Some(&Self::TRIAL_CHAMBERS_CHESTS_SUPPLY),
            "trial_chambers/corridor" => Some(&Self::TRIAL_CHAMBERS_CORRIDOR),
            "trial_chambers/corridor/slices" => Some(&Self::TRIAL_CHAMBERS_CORRIDOR_SLICES),
            "trial_chambers/corridors/addon/lower" => {
                Some(&Self::TRIAL_CHAMBERS_CORRIDORS_ADDON_LOWER)
            }
            "trial_chambers/corridors/addon/middle" => {
                Some(&Self::TRIAL_CHAMBERS_CORRIDORS_ADDON_MIDDLE)
            }
            "trial_chambers/corridors/addon/middle_upper" => {
                Some(&Self::TRIAL_CHAMBERS_CORRIDORS_ADDON_MIDDLE_UPPER)
            }
            "trial_chambers/decor" => Some(&Self::TRIAL_CHAMBERS_DECOR),
            "trial_chambers/decor/bed" => Some(&Self::TRIAL_CHAMBERS_DECOR_BED),
            "trial_chambers/decor/chamber" => Some(&Self::TRIAL_CHAMBERS_DECOR_CHAMBER),
            "trial_chambers/decor/disposal" => Some(&Self::TRIAL_CHAMBERS_DECOR_DISPOSAL),
            "trial_chambers/dispensers/chamber" => Some(&Self::TRIAL_CHAMBERS_DISPENSERS_CHAMBER),
            "trial_chambers/entrance" => Some(&Self::TRIAL_CHAMBERS_ENTRANCE),
            "trial_chambers/hallway" => Some(&Self::TRIAL_CHAMBERS_HALLWAY),
            "trial_chambers/hallway/fallback" => Some(&Self::TRIAL_CHAMBERS_HALLWAY_FALLBACK),
            "trial_chambers/reward/all" => Some(&Self::TRIAL_CHAMBERS_REWARD_ALL),
            "trial_chambers/reward/contents/default" => {
                Some(&Self::TRIAL_CHAMBERS_REWARD_CONTENTS_DEFAULT)
            }
            "trial_chambers/reward/ominous_vault" => {
                Some(&Self::TRIAL_CHAMBERS_REWARD_OMINOUS_VAULT)
            }
            "trial_chambers/spawner/all" => Some(&Self::TRIAL_CHAMBERS_SPAWNER_ALL),
            "trial_chambers/spawner/breeze" => Some(&Self::TRIAL_CHAMBERS_SPAWNER_BREEZE),
            "trial_chambers/spawner/contents/breeze" => {
                Some(&Self::TRIAL_CHAMBERS_SPAWNER_CONTENTS_BREEZE)
            }
            "trial_chambers/spawner/melee" => Some(&Self::TRIAL_CHAMBERS_SPAWNER_MELEE),
            "trial_chambers/spawner/melee/husk" => Some(&Self::TRIAL_CHAMBERS_SPAWNER_MELEE_HUSK),
            "trial_chambers/spawner/melee/spider" => {
                Some(&Self::TRIAL_CHAMBERS_SPAWNER_MELEE_SPIDER)
            }
            "trial_chambers/spawner/melee/zombie" => {
                Some(&Self::TRIAL_CHAMBERS_SPAWNER_MELEE_ZOMBIE)
            }
            "trial_chambers/spawner/ranged" => Some(&Self::TRIAL_CHAMBERS_SPAWNER_RANGED),
            "trial_chambers/spawner/ranged/poison_skeleton" => {
                Some(&Self::TRIAL_CHAMBERS_SPAWNER_RANGED_POISON_SKELETON)
            }
            "trial_chambers/spawner/ranged/skeleton" => {
                Some(&Self::TRIAL_CHAMBERS_SPAWNER_RANGED_SKELETON)
            }
            "trial_chambers/spawner/ranged/stray" => {
                Some(&Self::TRIAL_CHAMBERS_SPAWNER_RANGED_STRAY)
            }
            "trial_chambers/spawner/slow_ranged" => Some(&Self::TRIAL_CHAMBERS_SPAWNER_SLOW_RANGED),
            "trial_chambers/spawner/slow_ranged/poison_skeleton" => {
                Some(&Self::TRIAL_CHAMBERS_SPAWNER_SLOW_RANGED_POISON_SKELETON)
            }
            "trial_chambers/spawner/slow_ranged/skeleton" => {
                Some(&Self::TRIAL_CHAMBERS_SPAWNER_SLOW_RANGED_SKELETON)
            }
            "trial_chambers/spawner/slow_ranged/stray" => {
                Some(&Self::TRIAL_CHAMBERS_SPAWNER_SLOW_RANGED_STRAY)
            }
            "trial_chambers/spawner/small_melee" => Some(&Self::TRIAL_CHAMBERS_SPAWNER_SMALL_MELEE),
            "trial_chambers/spawner/small_melee/baby_zombie" => {
                Some(&Self::TRIAL_CHAMBERS_SPAWNER_SMALL_MELEE_BABY_ZOMBIE)
            }
            "trial_chambers/spawner/small_melee/cave_spider" => {
                Some(&Self::TRIAL_CHAMBERS_SPAWNER_SMALL_MELEE_CAVE_SPIDER)
            }
            "trial_chambers/spawner/small_melee/silverfish" => {
                Some(&Self::TRIAL_CHAMBERS_SPAWNER_SMALL_MELEE_SILVERFISH)
            }
            "trial_chambers/spawner/small_melee/slime" => {
                Some(&Self::TRIAL_CHAMBERS_SPAWNER_SMALL_MELEE_SLIME)
            }
            "village/common/animals" => Some(&Self::VILLAGE_COMMON_ANIMALS),
            "village/common/butcher_animals" => Some(&Self::VILLAGE_COMMON_BUTCHER_ANIMALS),
            "village/common/cats" => Some(&Self::VILLAGE_COMMON_CATS),
            "village/common/iron_golem" => Some(&Self::VILLAGE_COMMON_IRON_GOLEM),
            "village/common/sheep" => Some(&Self::VILLAGE_COMMON_SHEEP),
            "village/common/well_bottoms" => Some(&Self::VILLAGE_COMMON_WELL_BOTTOMS),
            "village/desert/camel" => Some(&Self::VILLAGE_DESERT_CAMEL),
            "village/desert/decor" => Some(&Self::VILLAGE_DESERT_DECOR),
            "village/desert/houses" => Some(&Self::VILLAGE_DESERT_HOUSES),
            "village/desert/streets" => Some(&Self::VILLAGE_DESERT_STREETS),
            "village/desert/terminators" => Some(&Self::VILLAGE_DESERT_TERMINATORS),
            "village/desert/town_centers" => Some(&Self::VILLAGE_DESERT_TOWN_CENTERS),
            "village/desert/villagers" => Some(&Self::VILLAGE_DESERT_VILLAGERS),
            "village/desert/zombie/decor" => Some(&Self::VILLAGE_DESERT_ZOMBIE_DECOR),
            "village/desert/zombie/houses" => Some(&Self::VILLAGE_DESERT_ZOMBIE_HOUSES),
            "village/desert/zombie/streets" => Some(&Self::VILLAGE_DESERT_ZOMBIE_STREETS),
            "village/desert/zombie/terminators" => Some(&Self::VILLAGE_DESERT_ZOMBIE_TERMINATORS),
            "village/desert/zombie/villagers" => Some(&Self::VILLAGE_DESERT_ZOMBIE_VILLAGERS),
            "village/plains/decor" => Some(&Self::VILLAGE_PLAINS_DECOR),
            "village/plains/houses" => Some(&Self::VILLAGE_PLAINS_HOUSES),
            "village/plains/streets" => Some(&Self::VILLAGE_PLAINS_STREETS),
            "village/plains/terminators" => Some(&Self::VILLAGE_PLAINS_TERMINATORS),
            "village/plains/town_centers" => Some(&Self::VILLAGE_PLAINS_TOWN_CENTERS),
            "village/plains/trees" => Some(&Self::VILLAGE_PLAINS_TREES),
            "village/plains/villagers" => Some(&Self::VILLAGE_PLAINS_VILLAGERS),
            "village/plains/zombie/decor" => Some(&Self::VILLAGE_PLAINS_ZOMBIE_DECOR),
            "village/plains/zombie/houses" => Some(&Self::VILLAGE_PLAINS_ZOMBIE_HOUSES),
            "village/plains/zombie/streets" => Some(&Self::VILLAGE_PLAINS_ZOMBIE_STREETS),
            "village/plains/zombie/villagers" => Some(&Self::VILLAGE_PLAINS_ZOMBIE_VILLAGERS),
            "village/savanna/decor" => Some(&Self::VILLAGE_SAVANNA_DECOR),
            "village/savanna/houses" => Some(&Self::VILLAGE_SAVANNA_HOUSES),
            "village/savanna/streets" => Some(&Self::VILLAGE_SAVANNA_STREETS),
            "village/savanna/terminators" => Some(&Self::VILLAGE_SAVANNA_TERMINATORS),
            "village/savanna/town_centers" => Some(&Self::VILLAGE_SAVANNA_TOWN_CENTERS),
            "village/savanna/trees" => Some(&Self::VILLAGE_SAVANNA_TREES),
            "village/savanna/villagers" => Some(&Self::VILLAGE_SAVANNA_VILLAGERS),
            "village/savanna/zombie/decor" => Some(&Self::VILLAGE_SAVANNA_ZOMBIE_DECOR),
            "village/savanna/zombie/houses" => Some(&Self::VILLAGE_SAVANNA_ZOMBIE_HOUSES),
            "village/savanna/zombie/streets" => Some(&Self::VILLAGE_SAVANNA_ZOMBIE_STREETS),
            "village/savanna/zombie/terminators" => Some(&Self::VILLAGE_SAVANNA_ZOMBIE_TERMINATORS),
            "village/savanna/zombie/villagers" => Some(&Self::VILLAGE_SAVANNA_ZOMBIE_VILLAGERS),
            "village/snowy/decor" => Some(&Self::VILLAGE_SNOWY_DECOR),
            "village/snowy/houses" => Some(&Self::VILLAGE_SNOWY_HOUSES),
            "village/snowy/streets" => Some(&Self::VILLAGE_SNOWY_STREETS),
            "village/snowy/terminators" => Some(&Self::VILLAGE_SNOWY_TERMINATORS),
            "village/snowy/town_centers" => Some(&Self::VILLAGE_SNOWY_TOWN_CENTERS),
            "village/snowy/trees" => Some(&Self::VILLAGE_SNOWY_TREES),
            "village/snowy/villagers" => Some(&Self::VILLAGE_SNOWY_VILLAGERS),
            "village/snowy/zombie/decor" => Some(&Self::VILLAGE_SNOWY_ZOMBIE_DECOR),
            "village/snowy/zombie/houses" => Some(&Self::VILLAGE_SNOWY_ZOMBIE_HOUSES),
            "village/snowy/zombie/streets" => Some(&Self::VILLAGE_SNOWY_ZOMBIE_STREETS),
            "village/snowy/zombie/villagers" => Some(&Self::VILLAGE_SNOWY_ZOMBIE_VILLAGERS),
            "village/taiga/decor" => Some(&Self::VILLAGE_TAIGA_DECOR),
            "village/taiga/houses" => Some(&Self::VILLAGE_TAIGA_HOUSES),
            "village/taiga/streets" => Some(&Self::VILLAGE_TAIGA_STREETS),
            "village/taiga/terminators" => Some(&Self::VILLAGE_TAIGA_TERMINATORS),
            "village/taiga/town_centers" => Some(&Self::VILLAGE_TAIGA_TOWN_CENTERS),
            "village/taiga/villagers" => Some(&Self::VILLAGE_TAIGA_VILLAGERS),
            "village/taiga/zombie/decor" => Some(&Self::VILLAGE_TAIGA_ZOMBIE_DECOR),
            "village/taiga/zombie/houses" => Some(&Self::VILLAGE_TAIGA_ZOMBIE_HOUSES),
            "village/taiga/zombie/streets" => Some(&Self::VILLAGE_TAIGA_ZOMBIE_STREETS),
            "village/taiga/zombie/villagers" => Some(&Self::VILLAGE_TAIGA_ZOMBIE_VILLAGERS),
            _ => None,
        }
    }
    #[must_use]
    pub const fn all_names() -> &'static [&'static str] {
        &[
            "minecraft:abandoned_camp/camp/bamboo_jungle",
            "minecraft:abandoned_camp/camp/birch_forest",
            "minecraft:abandoned_camp/camp/cherry_grove",
            "minecraft:abandoned_camp/camp/dappled_forest",
            "minecraft:abandoned_camp/camp/flower_forest",
            "minecraft:abandoned_camp/camp/forest",
            "minecraft:abandoned_camp/camp/meadow",
            "minecraft:abandoned_camp/camp/old_growth_birch_forest",
            "minecraft:abandoned_camp/camp/old_growth_pine_taiga",
            "minecraft:abandoned_camp/camp/old_growth_spruce_taiga",
            "minecraft:abandoned_camp/camp/pale_garden",
            "minecraft:abandoned_camp/camp/savanna",
            "minecraft:abandoned_camp/camp/snowy_taiga",
            "minecraft:abandoned_camp/camp/sparse_jungle",
            "minecraft:abandoned_camp/camp/swamp",
            "minecraft:abandoned_camp/camp/taiga",
            "minecraft:abandoned_camp/camp/windswept_forest",
            "minecraft:abandoned_camp/camp/wooded_badlands",
            "minecraft:abandoned_camp/tent/bamboo_jungle",
            "minecraft:abandoned_camp/tent/birch_forest",
            "minecraft:abandoned_camp/tent/cherry_grove",
            "minecraft:abandoned_camp/tent/dappled_forest",
            "minecraft:abandoned_camp/tent/flower_forest",
            "minecraft:abandoned_camp/tent/forest",
            "minecraft:abandoned_camp/tent/meadow",
            "minecraft:abandoned_camp/tent/old_growth_birch_forest",
            "minecraft:abandoned_camp/tent/old_growth_pine_taiga",
            "minecraft:abandoned_camp/tent/old_growth_spruce_taiga",
            "minecraft:abandoned_camp/tent/pale_garden",
            "minecraft:abandoned_camp/tent/savanna",
            "minecraft:abandoned_camp/tent/snowy_taiga",
            "minecraft:abandoned_camp/tent/sparse_jungle",
            "minecraft:abandoned_camp/tent/swamp",
            "minecraft:abandoned_camp/tent/taiga",
            "minecraft:abandoned_camp/tent/windswept_forest",
            "minecraft:abandoned_camp/tent/wooded_badlands",
            "minecraft:abandoned_camp/trees/acacia",
            "minecraft:abandoned_camp/trees/bamboo",
            "minecraft:abandoned_camp/trees/birch",
            "minecraft:abandoned_camp/trees/birch_bees",
            "minecraft:abandoned_camp/trees/cherry",
            "minecraft:abandoned_camp/trees/cherry_bees",
            "minecraft:abandoned_camp/trees/fancy_oak",
            "minecraft:abandoned_camp/trees/fancy_oak_bees",
            "minecraft:abandoned_camp/trees/jungle",
            "minecraft:abandoned_camp/trees/mega_jungle",
            "minecraft:abandoned_camp/trees/mega_pine",
            "minecraft:abandoned_camp/trees/oak",
            "minecraft:abandoned_camp/trees/orange_poplar",
            "minecraft:abandoned_camp/trees/pale_oak",
            "minecraft:abandoned_camp/trees/pine",
            "minecraft:abandoned_camp/trees/red_poplar",
            "minecraft:abandoned_camp/trees/spruce",
            "minecraft:abandoned_camp/trees/spruce_on_snow",
            "minecraft:abandoned_camp/trees/super_birch_bees",
            "minecraft:abandoned_camp/trees/thick_spruce",
            "minecraft:abandoned_camp/trees/yellow_poplar",
            "minecraft:ancient_city/city/entrance",
            "minecraft:ancient_city/city_center",
            "minecraft:ancient_city/city_center/walls",
            "minecraft:ancient_city/sculk",
            "minecraft:ancient_city/structures",
            "minecraft:ancient_city/walls",
            "minecraft:ancient_city/walls/no_corners",
            "minecraft:bastion/blocks/gold",
            "minecraft:bastion/bridge/bridge_pieces",
            "minecraft:bastion/bridge/connectors",
            "minecraft:bastion/bridge/legs",
            "minecraft:bastion/bridge/rampart_plates",
            "minecraft:bastion/bridge/ramparts",
            "minecraft:bastion/bridge/starting_pieces",
            "minecraft:bastion/bridge/walls",
            "minecraft:bastion/hoglin_stable/connectors",
            "minecraft:bastion/hoglin_stable/large_stables/inner",
            "minecraft:bastion/hoglin_stable/large_stables/outer",
            "minecraft:bastion/hoglin_stable/mirrored_starting_pieces",
            "minecraft:bastion/hoglin_stable/posts",
            "minecraft:bastion/hoglin_stable/rampart_plates",
            "minecraft:bastion/hoglin_stable/ramparts",
            "minecraft:bastion/hoglin_stable/small_stables/inner",
            "minecraft:bastion/hoglin_stable/small_stables/outer",
            "minecraft:bastion/hoglin_stable/stairs",
            "minecraft:bastion/hoglin_stable/starting_pieces",
            "minecraft:bastion/hoglin_stable/wall_bases",
            "minecraft:bastion/hoglin_stable/walls",
            "minecraft:bastion/mobs/hoglin",
            "minecraft:bastion/mobs/piglin",
            "minecraft:bastion/mobs/piglin_melee",
            "minecraft:bastion/starts",
            "minecraft:bastion/treasure/bases",
            "minecraft:bastion/treasure/bases/centers",
            "minecraft:bastion/treasure/brains",
            "minecraft:bastion/treasure/connectors",
            "minecraft:bastion/treasure/corners/bottom",
            "minecraft:bastion/treasure/corners/edges",
            "minecraft:bastion/treasure/corners/middle",
            "minecraft:bastion/treasure/corners/top",
            "minecraft:bastion/treasure/entrances",
            "minecraft:bastion/treasure/extensions/houses",
            "minecraft:bastion/treasure/extensions/large_pool",
            "minecraft:bastion/treasure/extensions/small_pool",
            "minecraft:bastion/treasure/ramparts",
            "minecraft:bastion/treasure/roofs",
            "minecraft:bastion/treasure/stairs",
            "minecraft:bastion/treasure/walls",
            "minecraft:bastion/treasure/walls/bottom",
            "minecraft:bastion/treasure/walls/mid",
            "minecraft:bastion/treasure/walls/outer",
            "minecraft:bastion/treasure/walls/top",
            "minecraft:bastion/units/center_pieces",
            "minecraft:bastion/units/edge_wall_units",
            "minecraft:bastion/units/edges",
            "minecraft:bastion/units/fillers/stage_0",
            "minecraft:bastion/units/large_ramparts",
            "minecraft:bastion/units/pathways",
            "minecraft:bastion/units/rampart_plates",
            "minecraft:bastion/units/ramparts",
            "minecraft:bastion/units/stages/rot/stage_1",
            "minecraft:bastion/units/stages/stage_0",
            "minecraft:bastion/units/stages/stage_1",
            "minecraft:bastion/units/stages/stage_2",
            "minecraft:bastion/units/stages/stage_3",
            "minecraft:bastion/units/wall_units",
            "minecraft:bastion/units/walls/wall_bases",
            "minecraft:empty",
            "minecraft:pillager_outpost/base_plates",
            "minecraft:pillager_outpost/feature_plates",
            "minecraft:pillager_outpost/features",
            "minecraft:pillager_outpost/towers",
            "minecraft:trail_ruins/buildings",
            "minecraft:trail_ruins/buildings/grouped",
            "minecraft:trail_ruins/decor",
            "minecraft:trail_ruins/roads",
            "minecraft:trail_ruins/tower",
            "minecraft:trail_ruins/tower/additions",
            "minecraft:trail_ruins/tower/tower_top",
            "minecraft:trial_chambers/atrium",
            "minecraft:trial_chambers/chamber/addon",
            "minecraft:trial_chambers/chamber/assembly",
            "minecraft:trial_chambers/chamber/end",
            "minecraft:trial_chambers/chamber/entrance_cap",
            "minecraft:trial_chambers/chamber/eruption",
            "minecraft:trial_chambers/chamber/pedestal",
            "minecraft:trial_chambers/chamber/slanted",
            "minecraft:trial_chambers/chambers/end",
            "minecraft:trial_chambers/chests/contents/supply",
            "minecraft:trial_chambers/chests/supply",
            "minecraft:trial_chambers/corridor",
            "minecraft:trial_chambers/corridor/slices",
            "minecraft:trial_chambers/corridors/addon/lower",
            "minecraft:trial_chambers/corridors/addon/middle",
            "minecraft:trial_chambers/corridors/addon/middle_upper",
            "minecraft:trial_chambers/decor",
            "minecraft:trial_chambers/decor/bed",
            "minecraft:trial_chambers/decor/chamber",
            "minecraft:trial_chambers/decor/disposal",
            "minecraft:trial_chambers/dispensers/chamber",
            "minecraft:trial_chambers/entrance",
            "minecraft:trial_chambers/hallway",
            "minecraft:trial_chambers/hallway/fallback",
            "minecraft:trial_chambers/reward/all",
            "minecraft:trial_chambers/reward/contents/default",
            "minecraft:trial_chambers/reward/ominous_vault",
            "minecraft:trial_chambers/spawner/all",
            "minecraft:trial_chambers/spawner/breeze",
            "minecraft:trial_chambers/spawner/contents/breeze",
            "minecraft:trial_chambers/spawner/melee",
            "minecraft:trial_chambers/spawner/melee/husk",
            "minecraft:trial_chambers/spawner/melee/spider",
            "minecraft:trial_chambers/spawner/melee/zombie",
            "minecraft:trial_chambers/spawner/ranged",
            "minecraft:trial_chambers/spawner/ranged/poison_skeleton",
            "minecraft:trial_chambers/spawner/ranged/skeleton",
            "minecraft:trial_chambers/spawner/ranged/stray",
            "minecraft:trial_chambers/spawner/slow_ranged",
            "minecraft:trial_chambers/spawner/slow_ranged/poison_skeleton",
            "minecraft:trial_chambers/spawner/slow_ranged/skeleton",
            "minecraft:trial_chambers/spawner/slow_ranged/stray",
            "minecraft:trial_chambers/spawner/small_melee",
            "minecraft:trial_chambers/spawner/small_melee/baby_zombie",
            "minecraft:trial_chambers/spawner/small_melee/cave_spider",
            "minecraft:trial_chambers/spawner/small_melee/silverfish",
            "minecraft:trial_chambers/spawner/small_melee/slime",
            "minecraft:village/common/animals",
            "minecraft:village/common/butcher_animals",
            "minecraft:village/common/cats",
            "minecraft:village/common/iron_golem",
            "minecraft:village/common/sheep",
            "minecraft:village/common/well_bottoms",
            "minecraft:village/desert/camel",
            "minecraft:village/desert/decor",
            "minecraft:village/desert/houses",
            "minecraft:village/desert/streets",
            "minecraft:village/desert/terminators",
            "minecraft:village/desert/town_centers",
            "minecraft:village/desert/villagers",
            "minecraft:village/desert/zombie/decor",
            "minecraft:village/desert/zombie/houses",
            "minecraft:village/desert/zombie/streets",
            "minecraft:village/desert/zombie/terminators",
            "minecraft:village/desert/zombie/villagers",
            "minecraft:village/plains/decor",
            "minecraft:village/plains/houses",
            "minecraft:village/plains/streets",
            "minecraft:village/plains/terminators",
            "minecraft:village/plains/town_centers",
            "minecraft:village/plains/trees",
            "minecraft:village/plains/villagers",
            "minecraft:village/plains/zombie/decor",
            "minecraft:village/plains/zombie/houses",
            "minecraft:village/plains/zombie/streets",
            "minecraft:village/plains/zombie/villagers",
            "minecraft:village/savanna/decor",
            "minecraft:village/savanna/houses",
            "minecraft:village/savanna/streets",
            "minecraft:village/savanna/terminators",
            "minecraft:village/savanna/town_centers",
            "minecraft:village/savanna/trees",
            "minecraft:village/savanna/villagers",
            "minecraft:village/savanna/zombie/decor",
            "minecraft:village/savanna/zombie/houses",
            "minecraft:village/savanna/zombie/streets",
            "minecraft:village/savanna/zombie/terminators",
            "minecraft:village/savanna/zombie/villagers",
            "minecraft:village/snowy/decor",
            "minecraft:village/snowy/houses",
            "minecraft:village/snowy/streets",
            "minecraft:village/snowy/terminators",
            "minecraft:village/snowy/town_centers",
            "minecraft:village/snowy/trees",
            "minecraft:village/snowy/villagers",
            "minecraft:village/snowy/zombie/decor",
            "minecraft:village/snowy/zombie/houses",
            "minecraft:village/snowy/zombie/streets",
            "minecraft:village/snowy/zombie/villagers",
            "minecraft:village/taiga/decor",
            "minecraft:village/taiga/houses",
            "minecraft:village/taiga/streets",
            "minecraft:village/taiga/terminators",
            "minecraft:village/taiga/town_centers",
            "minecraft:village/taiga/villagers",
            "minecraft:village/taiga/zombie/decor",
            "minecraft:village/taiga/zombie/houses",
            "minecraft:village/taiga/zombie/streets",
            "minecraft:village/taiga/zombie/villagers",
        ]
    }
}
#[must_use]
pub fn get_template_pool(name: &str) -> Option<&'static StaticTemplatePool> {
    StaticTemplatePool::get(name)
}
