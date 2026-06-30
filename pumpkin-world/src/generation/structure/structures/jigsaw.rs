use super::jigsaw_placement::{
    DimensionPadding, JigsawPlacement, LiquidSettings, MaxDistance, PoolAliasLookup,
};
use crate::generation::structure::structures::{
    StructureGenerator, StructureGeneratorContext, StructurePieceBase, StructurePosition,
};
use crate::generation::structure::template::{
    BlockMirror, BlockRotation, BlockStateResolver, PaletteEntry, StructureTemplate,
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::RandomImpl;
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
    pub template: &'static str,
    pub weight: u32,
    pub projection: JigsawProjection,
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
        let elements = crate::generation::structure::template::get_pool_elements(id)?;

        // Heuristic: roads and streets are usually TerrainMatching
        let projection = if id.contains("streets") {
            JigsawProjection::TerrainMatching
        } else {
            JigsawProjection::Rigid
        };

        Some(Self {
            id: id.to_string(),
            fallback: "minecraft:empty".to_string(),
            elements: elements
                .iter()
                .map(|e| PoolElement {
                    template: e,
                    weight: 1,
                    projection,
                })
                .collect(),
        })
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
            "north" => pumpkin_util::BlockDirection::North,
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
            "up" => pumpkin_util::BlockDirection::Up,
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
    pub template: Arc<StructureTemplate>,
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
        _random: &mut pumpkin_util::random::RandomGenerator,
        _seed: i64,
        chunk_box: &pumpkin_util::math::block_box::BlockBox,
    ) {
        let origin =
            pumpkin_util::math::vector3::Vector3::new(self.pos.0.x, self.pos.0.y, self.pos.0.z);

        let processors: Vec<Box<dyn crate::generation::structure::template::StructureProcessor>> =
            vec![Box::new(
                crate::generation::structure::template::processor::GravityProcessor {
                    heightmap: pumpkin_util::HeightMap::WorldSurfaceWg,
                    offset: -1,
                },
            )];

        crate::generation::structure::template::place_template(
            chunk,
            &self.template,
            origin,
            (0, 0),
            self.rotation,
            false,
            &processors,
            Some(chunk_box),
        );

        // Post-process: replace jigsaw blocks with their final_state
        for jigsaw in &self.jigsaw_blocks {
            let wx = jigsaw.pos.0.x;
            let wy = jigsaw.pos.0.y;
            let wz = jigsaw.pos.0.z;
            if wx < chunk_box.min.x
                || wx > chunk_box.max.x
                || wy < chunk_box.min.y
                || wy > chunk_box.max.y
                || wz < chunk_box.min.z
                || wz > chunk_box.max.z
            {
                continue;
            }
            if jigsaw.final_state == "minecraft:air" || jigsaw.final_state == "air" {
                chunk.set_block_state(wx, wy, wz, pumpkin_data::Block::AIR.default_state);
                continue;
            }

            let entry = PaletteEntry::from_string(&jigsaw.final_state);
            if let Some(state) = BlockStateResolver::resolve(&entry, self.rotation, self.mirror) {
                chunk.set_block_state(wx, wy, wz, state);
            }
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
                    "apply_waterlogging" => LiquidSettings::ApplyWaterlog,
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
            MaxDistance {
                horizontal: max_distance,
                vertical: max_distance,
            },
            dimension_padding,
            liquid_settings,
            &PoolAliasLookup,
        )
    }
}
