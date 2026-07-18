use std::sync::Arc;

use pumpkin_data::block_rotation::Rotation;
use pumpkin_util::{
    math::{block_box::BlockBox, position::BlockPos, vector3::Vector3},
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

pub struct MansionGenerator;

impl StructureGenerator for MansionGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        let chunk_center_x = get_center_x(context.chunk_x);
        let chunk_center_z = get_center_z(context.chunk_z);

        let rotation_idx = context.random.next_bounded_i32(4) as u8;
        let rotation = Rotation::from_index(rotation_idx);

        let entrance = get_template("woodland_mansion/entrance")?;
        let wall_flat = get_template("woodland_mansion/wall_flat")?;
        let wall_window = get_template("woodland_mansion/wall_window")?;
        let roof = get_template("woodland_mansion/roof")?;

        let bounding_box = BlockBox::new(
            chunk_center_x - 20,
            context.min_y,
            chunk_center_z - 20,
            chunk_center_x + 20,
            256,
            chunk_center_z + 20,
        );

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(MansionPiece {
            piece: StructurePiece::new(StructurePieceType::WoodlandMansion, bounding_box, 0),
            entrance,
            wall_flat,
            wall_window,
            roof,
            rotation,
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(chunk_center_x, 64, chunk_center_z),
            collector: Arc::new(collector.into()),
        })
    }
}

pub struct MansionPiece {
    piece: StructurePiece,
    entrance: Arc<StructureTemplate>,
    wall_flat: Arc<StructureTemplate>,
    wall_window: Arc<StructureTemplate>,
    roof: Arc<StructureTemplate>,
    rotation: Rotation,
}

impl StructurePieceBase for MansionPiece {
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
        let sample_y =
            chunk.get_top_y(&pumpkin_util::HeightMap::WorldSurfaceWg, origin.x, origin.z);
        let start_y = if sample_y <= 0 { 64 } else { sample_y - 1 };

        let mut pos = Vector3::new(origin.x, start_y, origin.z);

        // Place main entrance
        place_template(
            chunk,
            &self.entrance,
            pos,
            (0, 0),
            self.rotation,
            true,
            false,
            &[],
            Some(chunk_box),
        );

        // Place some side window walls to look like a giant mansion!
        let entrance_width = self.entrance.size.x;
        pos.x += entrance_width;
        place_template(
            chunk,
            &self.wall_window,
            pos,
            (0, 0),
            self.rotation,
            true,
            false,
            &[],
            Some(chunk_box),
        );

        pos.x -= entrance_width * 2;
        place_template(
            chunk,
            &self.wall_window,
            pos,
            (0, 0),
            self.rotation,
            true,
            false,
            &[],
            Some(chunk_box),
        );
    }
}
