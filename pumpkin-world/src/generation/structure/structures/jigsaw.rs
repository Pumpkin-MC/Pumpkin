use super::jigsaw_placement::{
    DimensionPadding, JigsawPlacement, LiquidSettings, MaxDistance, PoolAliasLookup,
};
use crate::generation::structure::structures::{
    StructureGenerator, StructureGeneratorContext, StructurePieceBase, StructurePosition,
};
use crate::generation::structure::template::{
    BlockMirror, BlockRotation, PaletteEntry, StructureTemplate,
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::RandomImpl;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum JigsawProjection {
    Rigid,
    TerrainMatching,
}

#[derive(Clone)]
pub struct TemplatePool {
    pub id: String,
    pub fallback: String,
    pub elements: Vec<PoolElement>,
}

#[derive(Clone)]
pub struct PoolElement {
    pub weight: u32,
    pub projection: JigsawProjection,
    pub kind: PoolElementKind,
}

#[derive(Clone)]
pub enum PoolElementKind {
    Empty,
    Single {
        template: String,
        processors: ProcessorListRef,
    },
    List(Vec<Self>),
    Feature(pumpkin_data::placed_feature::PlacedFeature),
}

#[derive(Clone, Default)]
pub enum ProcessorListRef {
    Named(String),
    #[default]
    Empty,
}

#[derive(Deserialize)]
struct RawTemplatePool {
    fallback: String,
    elements: Vec<RawWeightedPoolElement>,
}

#[derive(Deserialize)]
struct RawWeightedPoolElement {
    element: RawPoolElement,
    weight: u32,
}

#[derive(Deserialize)]
#[serde(tag = "element_type")]
enum RawPoolElement {
    #[serde(rename = "minecraft:empty_pool_element")]
    Empty,
    #[serde(rename = "minecraft:single_pool_element")]
    Single {
        location: String,
        processors: RawProcessorList,
        projection: RawProjection,
    },
    #[serde(rename = "minecraft:list_pool_element")]
    List {
        elements: Vec<Self>,
        projection: RawProjection,
    },
    #[serde(rename = "minecraft:feature_pool_element")]
    Feature {
        feature: String,
        projection: RawProjection,
    },
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawProcessorList {
    Named(String),
    Inline { processors: Vec<serde_json::Value> },
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RawProjection {
    Rigid,
    TerrainMatching,
}

impl From<RawProjection> for JigsawProjection {
    fn from(value: RawProjection) -> Self {
        match value {
            RawProjection::Rigid => Self::Rigid,
            RawProjection::TerrainMatching => Self::TerrainMatching,
        }
    }
}

impl RawPoolElement {
    fn into_element(self) -> Option<(PoolElementKind, JigsawProjection)> {
        match self {
            Self::Empty => Some((PoolElementKind::Empty, JigsawProjection::Rigid)),
            Self::Single {
                location,
                processors,
                projection,
            } => {
                let processors = match processors {
                    RawProcessorList::Named(name) => ProcessorListRef::Named(name),
                    RawProcessorList::Inline { processors } => {
                        debug_assert!(processors.is_empty());
                        ProcessorListRef::Empty
                    }
                };
                Some((
                    PoolElementKind::Single {
                        template: location,
                        processors,
                    },
                    projection.into(),
                ))
            }
            Self::List {
                elements,
                projection,
            } => {
                let projection = projection.into();
                let elements = elements
                    .into_iter()
                    .filter_map(|element| element.into_element().map(|(kind, _)| kind))
                    .collect();
                Some((PoolElementKind::List(elements), projection))
            }
            Self::Feature {
                feature,
                projection,
            } => {
                let feature = feature.strip_prefix("minecraft:").unwrap_or(&feature);
                pumpkin_data::placed_feature::PlacedFeature::from_name(feature)
                    .map(|feature| (PoolElementKind::Feature(feature), projection.into()))
            }
        }
    }
}

impl PoolElement {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        matches!(self.kind, PoolElementKind::Empty)
    }

    #[must_use]
    pub fn first_template(&self) -> Option<Arc<StructureTemplate>> {
        fn find(kind: &PoolElementKind) -> Option<Arc<StructureTemplate>> {
            match kind {
                PoolElementKind::Single { template, .. } => {
                    crate::generation::structure::template::get_template(template)
                }
                PoolElementKind::List(elements) => elements.iter().find_map(find),
                PoolElementKind::Empty | PoolElementKind::Feature(_) => None,
            }
        }

        find(&self.kind)
    }

    pub fn for_each_template(
        &self,
        mut consumer: impl FnMut(&str, &ProcessorListRef, Arc<StructureTemplate>),
    ) {
        fn visit(
            kind: &PoolElementKind,
            consumer: &mut impl FnMut(&str, &ProcessorListRef, Arc<StructureTemplate>),
        ) {
            match kind {
                PoolElementKind::Single {
                    template,
                    processors,
                } => {
                    if let Some(structure_template) =
                        crate::generation::structure::template::get_template(template)
                    {
                        consumer(template, processors, structure_template);
                    }
                }
                PoolElementKind::List(elements) => {
                    for element in elements {
                        visit(element, consumer);
                    }
                }
                PoolElementKind::Empty | PoolElementKind::Feature(_) => {}
            }
        }

        visit(&self.kind, &mut consumer);
    }

    #[must_use]
    pub const fn feature(&self) -> Option<pumpkin_data::placed_feature::PlacedFeature> {
        match self.kind {
            PoolElementKind::Feature(feature) => Some(feature),
            _ => None,
        }
    }
}

impl TemplatePool {
    pub fn get_random_element(
        &self,
        random: &mut pumpkin_util::random::RandomGenerator,
    ) -> &PoolElement {
        let total_weight: u32 = self.elements.iter().map(|e| e.weight).sum();
        if total_weight == 0 {
            return &self.elements[0];
        }
        let mut r = random.next_bounded_i32(total_weight as i32) as u32;
        for element in &self.elements {
            if r < element.weight {
                return element;
            }
            r -= element.weight;
        }
        &self.elements[0]
    }

    /// Discovers a pool from the filesystem/embedded assets.
    #[must_use]
    pub fn discover(id: &str) -> Option<Self> {
        static CACHE: std::sync::LazyLock<dashmap::DashMap<String, TemplatePool>> =
            std::sync::LazyLock::new(dashmap::DashMap::new);

        if let Some(pool) = CACHE.get(id) {
            return Some(pool.clone());
        }

        let pool = if id == "minecraft:empty" || id == "empty" {
            Self {
                id: "minecraft:empty".to_string(),
                fallback: "minecraft:empty".to_string(),
                elements: Vec::new(),
            }
        } else if let Some(json) =
            crate::generation::structure::template::get_template_pool_json(id)
        {
            let raw: RawTemplatePool = match serde_json::from_str(json) {
                Ok(pool) => pool,
                Err(error) => {
                    tracing::error!("Failed to parse template pool {id}: {error}");
                    return None;
                }
            };
            let elements = raw
                .elements
                .into_iter()
                .filter_map(|weighted| {
                    weighted
                        .element
                        .into_element()
                        .map(|(kind, projection)| PoolElement {
                            weight: weighted.weight,
                            projection,
                            kind,
                        })
                })
                .collect();
            Self {
                id: id.to_string(),
                fallback: raw.fallback,
                elements,
            }
        } else {
            let elements = crate::generation::structure::template::get_pool_elements(id)?;
            let projection = if id.contains("streets") {
                JigsawProjection::TerrainMatching
            } else {
                JigsawProjection::Rigid
            };

            Self {
                id: id.to_string(),
                fallback: "minecraft:empty".to_string(),
                elements: elements
                    .iter()
                    .map(|e| PoolElement {
                        weight: 1,
                        projection,
                        kind: PoolElementKind::Single {
                            template: (*e).to_string(),
                            processors: ProcessorListRef::Empty,
                        },
                    })
                    .collect(),
            }
        };
        CACHE.insert(id.to_owned(), pool.clone());
        Some(pool)
    }

    #[must_use]
    pub fn get_shuffled_elements(
        &self,
        random: &mut pumpkin_util::random::RandomGenerator,
    ) -> Vec<PoolElement> {
        let mut elements = self
            .elements
            .iter()
            .flat_map(|element| std::iter::repeat_n(element.clone(), element.weight as usize))
            .collect::<Vec<_>>();
        for index in (1..elements.len()).rev() {
            let other = random.next_bounded_i32(index as i32 + 1) as usize;
            elements.swap(index, other);
        }
        elements
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum JigsawJointType {
    Rollable,
    Aligned,
}

impl JigsawJointType {
    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn from_str(s: &str) -> Self {
        match s {
            "aligned" => Self::Aligned,
            _ => Self::Rollable,
        }
    }
}

#[derive(Clone)]
pub struct JigsawBlock {
    pub pos: BlockPos,
    pub name: String,
    pub target: String,
    pub pool: String,
    pub final_state: String,
    pub joint: JigsawJointType,
    pub facing: pumpkin_util::BlockDirection,
    pub up: pumpkin_util::BlockDirection,
    pub selection_priority: i32,
    pub placement_priority: i32,
}

impl JigsawBlock {
    #[must_use]
    pub fn from_template_block(
        block: &crate::generation::structure::template::TemplateBlock,
        palette: &PaletteEntry,
    ) -> Option<Self> {
        if palette.name != "minecraft:jigsaw" {
            return None;
        }

        let nbt = block.nbt.as_ref()?;

        // Resolve facing from properties
        let facing_str = palette
            .properties
            .iter()
            .find(|(k, _)| *k == "orientation")
            .map_or_else(|| "north_up".to_string(), |(_, v)| v.clone());

        let mut parts = facing_str.split('_');
        let facing_part = parts.next().unwrap_or("north");
        let up_part = parts.next().unwrap_or("up");

        let facing = match facing_part {
            "south" => pumpkin_util::BlockDirection::South,
            "east" => pumpkin_util::BlockDirection::East,
            "west" => pumpkin_util::BlockDirection::West,
            "up" => pumpkin_util::BlockDirection::Up,
            "down" => pumpkin_util::BlockDirection::Down,
            _ => pumpkin_util::BlockDirection::North,
        };

        let up = match up_part {
            "north" => pumpkin_util::BlockDirection::North,
            "south" => pumpkin_util::BlockDirection::South,
            "east" => pumpkin_util::BlockDirection::East,
            "west" => pumpkin_util::BlockDirection::West,
            "down" => pumpkin_util::BlockDirection::Down,
            _ => pumpkin_util::BlockDirection::Up,
        };

        Some(Self {
            pos: BlockPos(block.pos),
            name: nbt.get_string("name").unwrap_or_default().to_string(),
            target: nbt.get_string("target").unwrap_or_default().to_string(),
            pool: nbt.get_string("pool").unwrap_or_default().to_string(),
            final_state: nbt
                .get_string("final_state")
                .unwrap_or_default()
                .to_string(),
            joint: JigsawJointType::from_str(nbt.get_string("joint").unwrap_or_default()),
            facing,
            up,
            selection_priority: nbt.get_int("selection_priority").unwrap_or(0),
            placement_priority: nbt.get_int("placement_priority").unwrap_or(0),
        })
    }

    #[must_use]
    pub fn can_attach(
        source: &Self,
        target_facing: pumpkin_util::BlockDirection,
        target_name: &str,
    ) -> bool {
        source.facing.opposite() == target_facing && source.target == target_name
    }
}

#[derive(Clone)]
pub struct JigsawJunction {
    pub source_x: i32,
    pub source_ground_y: i32,
    pub source_z: i32,
    pub delta_y: i32,
    pub projection: JigsawProjection,
}

pub struct PoolElementStructurePiece {
    pub piece: crate::generation::structure::structures::StructurePiece,
    pub element: PoolElement,
    pub pos: BlockPos,
    pub rotation: BlockRotation,
    pub mirror: BlockMirror,
    pub jigsaw_blocks: Vec<JigsawBlock>,
    pub junctions: Vec<JigsawJunction>,
    pub ground_level_delta: i32,
    pub liquid_settings: LiquidSettings,
    pub projection: JigsawProjection,
}

impl StructurePieceBase for PoolElementStructurePiece {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_structure_piece(&self) -> &crate::generation::structure::structures::StructurePiece {
        &self.piece
    }

    fn get_structure_piece_mut(
        &mut self,
    ) -> &mut crate::generation::structure::structures::StructurePiece {
        &mut self.piece
    }

    fn place(
        &mut self,
        chunk: &mut crate::ProtoChunk,
        _block_registry: &dyn crate::world::WorldPortalExt,
        random: &mut pumpkin_util::random::RandomGenerator,
        _seed: i64,
        chunk_box: &pumpkin_util::math::block_box::BlockBox,
    ) {
        let origin =
            pumpkin_util::math::vector3::Vector3::new(self.pos.0.x, self.pos.0.y, self.pos.0.z);

        self.element
            .for_each_template(|_name, processor_list, template| {
                let processors = match processor_list {
                    ProcessorListRef::Named(name) => {
                        crate::generation::structure::template::processor::load_processor_list(name)
                    }
                    ProcessorListRef::Empty => Arc::from([]),
                };
                crate::generation::structure::template::place_template(
                    chunk,
                    &template,
                    origin,
                    (0, 0),
                    self.rotation,
                    false,
                    self.liquid_settings == LiquidSettings::ApplyWaterlog,
                    processors.as_ref(),
                    Some(chunk_box),
                );
            });

        if let Some(feature) = self.element.feature()
            && let Some(placed_feature) =
                crate::generation::feature::placed_features::PLACED_FEATURES.get(&feature)
        {
            placed_feature.generate_in_proto_chunk(chunk, feature, random, self.pos);
        }
    }
}

impl PoolElementStructurePiece {
    pub fn add_junction(&mut self, junction: JigsawJunction) {
        self.junctions.push(junction);
    }
}

pub struct JigsawGenerator {
    pub start_pool: String,
    pub size: i32,
    pub start_jigsaw_name: Option<String>,
    pub use_expansion_hack: bool,
}

impl JigsawGenerator {
    #[must_use]
    pub fn new(start_pool: &str, size: i32) -> Self {
        Self {
            start_pool: start_pool.to_string(),
            size,
            start_jigsaw_name: None,
            use_expansion_hack: false,
        }
    }

    #[must_use]
    pub fn with_start_jigsaw(mut self, name: &str) -> Self {
        self.start_jigsaw_name = Some(name.to_string());
        self
    }

    #[must_use]
    pub const fn with_expansion_hack(mut self, use_hack: bool) -> Self {
        self.use_expansion_hack = use_hack;
        self
    }
}

impl StructureGenerator for JigsawGenerator {
    fn get_structure_position(
        &self,
        context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        let mut context = context;
        let structure = context
            .structure_key
            .map(|key| pumpkin_data::structures::Structure::get(&key));

        let start_y = if let Some(s) = structure {
            s.start_height.unwrap_or(context.sea_level as i16) as i32
        } else {
            context.sea_level
        };

        let start_pos = BlockPos::new(
            crate::generation::positions::chunk_pos::start_block_x(context.chunk_x),
            start_y,
            crate::generation::positions::chunk_pos::start_block_z(context.chunk_z),
        );

        let project_start_to_heightmap = structure
            .and_then(|s| s.project_start_to_heightmap)
            .is_some();

        let max_distance = structure
            .and_then(|s| s.max_distance_from_center)
            .unwrap_or(80); // Vanilla default is 80

        let liquid_settings =
            structure
                .and_then(|s| s.liquid_settings)
                .map_or(LiquidSettings::ApplyWaterlog, |ls| match ls {
                    "ignore_waterlogging" => LiquidSettings::IgnoreWaterlogDone,
                    _ => LiquidSettings::ApplyWaterlog,
                });

        let dimension_padding =
            structure
                .and_then(|s| s.dimension_padding)
                .map_or(DimensionPadding::ZERO, |dp| DimensionPadding {
                    top: dp,
                    bottom: dp,
                });

        JigsawPlacement::add_pieces(
            &mut context,
            &self.start_pool,
            self.start_jigsaw_name.as_deref(),
            self.size,
            start_pos,
            self.use_expansion_hack,
            project_start_to_heightmap,
            &MaxDistance::new(max_distance),
            &dimension_padding,
            liquid_settings,
            &PoolAliasLookup,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ancient_city_pools_match_vanilla_weights() {
        let expected = [
            ("minecraft:ancient_city/city_center", 3, 3),
            ("minecraft:ancient_city/sculk", 2, 7),
            ("minecraft:ancient_city/structures", 20, 46),
            ("minecraft:ancient_city/walls", 16, 27),
            ("minecraft:ancient_city/city/entrance", 6, 6),
            ("minecraft:ancient_city/city_center/walls", 10, 10),
            ("minecraft:ancient_city/walls/no_corners", 8, 8),
        ];

        for (id, element_count, total_weight) in expected {
            let pool = TemplatePool::discover(id).unwrap_or_else(|| panic!("missing pool {id}"));
            assert_eq!(pool.elements.len(), element_count, "{id}");
            assert_eq!(
                pool.elements
                    .iter()
                    .map(|element| element.weight)
                    .sum::<u32>(),
                total_weight,
                "{id}"
            );
            assert_eq!(pool.fallback, "minecraft:empty", "{id}");
        }
    }

    #[test]
    fn ancient_city_start_templates_and_anchor_exist() {
        let pool = TemplatePool::discover("minecraft:ancient_city/city_center").unwrap();
        for element in pool.elements {
            let template = element.first_template().expect("missing start template");
            assert!(
                template.blocks.iter().any(|block| {
                    JigsawBlock::from_template_block(block, &template.palette[block.state as usize])
                        .is_some_and(|jigsaw| jigsaw.name == "minecraft:city_anchor")
                }),
                "start template has no city_anchor"
            );
        }
    }

    #[test]
    fn ancient_city_pool_templates_are_embedded() {
        fn check(kind: &PoolElementKind) {
            match kind {
                PoolElementKind::Single { template, .. } => {
                    // This entry exists in vanilla's pool data but has no corresponding
                    // template in the vanilla server jar.
                    if template == "minecraft:ancient_city/walls/intact_horizontal_wall_stairs_5" {
                        assert!(
                            crate::generation::structure::template::get_template(template)
                                .is_none()
                        );
                    } else {
                        assert!(
                            crate::generation::structure::template::get_template(template)
                                .is_some(),
                            "missing template {template}"
                        );
                    }
                }
                PoolElementKind::List(elements) => elements.iter().for_each(check),
                PoolElementKind::Empty | PoolElementKind::Feature(_) => {}
            }
        }

        for id in [
            "minecraft:ancient_city/city_center",
            "minecraft:ancient_city/structures",
            "minecraft:ancient_city/walls",
            "minecraft:ancient_city/city/entrance",
            "minecraft:ancient_city/city_center/walls",
            "minecraft:ancient_city/walls/no_corners",
        ] {
            for element in TemplatePool::discover(id).unwrap().elements {
                check(&element.kind);
            }
        }
    }

    #[test]
    fn ancient_city_builds_a_multi_piece_graph() {
        let generator = JigsawGenerator::new("minecraft:ancient_city/city_center", 7)
            .with_start_jigsaw("minecraft:city_anchor");
        let context = StructureGeneratorContext {
            seed: 0,
            chunk_x: 0,
            chunk_z: 0,
            random: super::super::create_chunk_random(0, 0, 0),
            sea_level: 63,
            min_y: -64,
            height_sampler: None,
            structure_key: Some(pumpkin_data::structures::StructureKeys::AncientCity),
        };

        let position = generator
            .get_structure_position(context)
            .expect("ancient city graph should generate");
        let collector = position.collector.lock().unwrap();
        assert!(
            collector.pieces.len() > 10,
            "ancient city generated only {} pieces",
            collector.pieces.len()
        );
        assert_eq!(position.start_pos.0.y, -27);
    }
}
