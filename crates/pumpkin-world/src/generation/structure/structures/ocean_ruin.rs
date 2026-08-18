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

const COLD_RUINS: &[&str] = &[
    "underwater_ruin/brick_1",
    "underwater_ruin/brick_2",
    "underwater_ruin/brick_3",
    "underwater_ruin/brick_4",
    "underwater_ruin/brick_5",
    "underwater_ruin/brick_6",
    "underwater_ruin/brick_7",
    "underwater_ruin/brick_8",
    "underwater_ruin/cracked_1",
    "underwater_ruin/cracked_2",
    "underwater_ruin/cracked_3",
    "underwater_ruin/mossy_1",
    "underwater_ruin/mossy_2",
    "underwater_ruin/mossy_3",
];

const WARM_RUINS: &[&str] = &[
    "underwater_ruin/warm_1",
    "underwater_ruin/warm_2",
    "underwater_ruin/warm_3",
    "underwater_ruin/warm_4",
    "underwater_ruin/warm_5",
    "underwater_ruin/warm_6",
    "underwater_ruin/warm_7",
    "underwater_ruin/warm_8",
    "underwater_ruin/big_warm_4",
    "underwater_ruin/big_warm_5",
];

pub struct OceanRuinGenerator {
    pub is_warm: bool,
}

impl StructureGenerator for OceanRuinGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        let chunk_center_x = get_center_x(context.chunk_x);
        let chunk_center_z = get_center_z(context.chunk_z);

        let rotation_idx = context.random.next_bounded_i32(4) as u8;
        let rotation = Rotation::from_index(rotation_idx);

        let pool = if self.is_warm { WARM_RUINS } else { COLD_RUINS };
        let template_idx = context.random.next_bounded_i32(pool.len() as i32) as usize;
        let template_name = pool[template_idx];
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
        collector.add_piece(Box::new(OceanRuinPiece {
            piece: StructurePiece::new(StructurePieceType::OceanTemple, bounding_box, 0),
            template,
            rotation,
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(chunk_center_x, 64, chunk_center_z),
            collector: Arc::new(collector.into()),
        })
    }
}

pub struct OceanRuinPiece {
    piece: StructurePiece,
    template: Arc<StructureTemplate>,
    rotation: Rotation,
}

impl StructurePieceBase for OceanRuinPiece {
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
        let sample_y = chunk.get_top_y(&pumpkin_util::HeightMap::OceanFloorWg, origin.x, origin.z);
        let target_y = sample_y - 2; // slightly buried on ocean floor
        let mut final_origin = origin;
        final_origin.y = target_y;

        place_template(
            chunk,
            &self.template,
            final_origin,
            (0, 0),
            self.rotation,
            true, // skip air
            true, // apply waterlogging
            &[],
            Some(chunk_box),
        );
    }
}
