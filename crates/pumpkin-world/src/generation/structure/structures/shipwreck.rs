use std::sync::Arc;

use pumpkin_data::block_rotation::Rotation;
use pumpkin_util::{
    math::{block_box::BlockBox, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};

use crate::{
    ProtoChunk,
    generation::{
        positions::chunk_pos::{get_center_x, get_center_z},
        structure::{
            piece::StructurePieceType,
            structures::{
                StructureGenerator, StructureGeneratorContext, StructurePiece, StructurePieceBase,
                StructurePiecesCollector, StructurePosition, WorldPortalExt,
            },
            template::{StructureTemplate, get_template, place_template},
        },
    },
};

const TEMPLATES: &[&str] = &[
    "shipwreck/rightsideup_backhalf",
    "shipwreck/rightsideup_backhalf_degraded",
    "shipwreck/rightsideup_fronthalf",
    "shipwreck/rightsideup_fronthalf_degraded",
    "shipwreck/rightsideup_full",
    "shipwreck/rightsideup_full_degraded",
    "shipwreck/sideways_backhalf",
    "shipwreck/sideways_backhalf_degraded",
    "shipwreck/sideways_fronthalf",
    "shipwreck/sideways_fronthalf_degraded",
    "shipwreck/sideways_full",
    "shipwreck/sideways_full_degraded",
    "shipwreck/upsidedown_backhalf",
    "shipwreck/upsidedown_backhalf_degraded",
    "shipwreck/upsidedown_fronthalf",
    "shipwreck/upsidedown_fronthalf_degraded",
    "shipwreck/upsidedown_full",
    "shipwreck/upsidedown_full_degraded",
    "shipwreck/with_mast",
    "shipwreck/with_mast_degraded",
];

pub struct ShipwreckGenerator {
    pub is_beached: bool,
}

impl StructureGenerator for ShipwreckGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        let chunk_center_x = get_center_x(context.chunk_x);
        let chunk_center_z = get_center_z(context.chunk_z);

        // Deterministically select rotation and template
        let rotation_idx = context.random.next_bounded_i32(4) as u8;
        let rotation = Rotation::from_index(rotation_idx);

        let template_idx = context.random.next_bounded_i32(TEMPLATES.len() as i32) as usize;
        let template_name = TEMPLATES[template_idx];
        let template = get_template(template_name)?;

        let size = template.size;
        let bounding_box = BlockBox::new(
            chunk_center_x - size.x / 2,
            context.min_y,
            chunk_center_z - size.z / 2,
            chunk_center_x + size.x / 2,
            256,
            chunk_center_z + size.z / 2,
        );

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(ShipwreckPiece {
            piece: StructurePiece::new(StructurePieceType::Shipwreck, bounding_box, 0),
            template,
            rotation,
            is_beached: self.is_beached,
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(chunk_center_x, 64, chunk_center_z),
            collector: Arc::new(collector.into()),
        })
    }
}

pub struct ShipwreckPiece {
    piece: StructurePiece,
    template: Arc<StructureTemplate>,
    rotation: Rotation,
    is_beached: bool,
}

impl StructurePieceBase for ShipwreckPiece {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece
    }
    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece
    }
    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _block_registry: &dyn WorldPortalExt,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let origin = self.piece.bounding_box.min;
        let height_map_type = if self.is_beached {
            pumpkin_util::HeightMap::WorldSurfaceWg
        } else {
            pumpkin_util::HeightMap::OceanFloorWg
        };

        let sample_y = chunk.get_top_y(&height_map_type, origin.x, origin.z);
        let target_y = if self.is_beached {
            sample_y - 1
        } else {
            sample_y - 3
        };
        let mut final_origin = origin;
        final_origin.y = target_y;

        place_template(
            chunk,
            &self.template,
            final_origin,
            (0, 0),
            self.rotation,
            true,
            !self.is_beached,
            &[],
            Some(chunk_box),
        );
    }
}
