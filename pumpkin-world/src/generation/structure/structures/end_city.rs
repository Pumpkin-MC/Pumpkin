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

pub struct EndCityGenerator;

impl StructureGenerator for EndCityGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        let chunk_center_x = get_center_x(context.chunk_x);
        let chunk_center_z = get_center_z(context.chunk_z);

        let rotation_idx = context.random.next_bounded_i32(4) as u8;
        let rotation = Rotation::from_index(rotation_idx);

        let base_floor = get_template("end_city/base_floor")?;
        let tower_base = get_template("end_city/tower_base")?;
        let tower_piece = get_template("end_city/tower_piece")?;
        let tower_top = get_template("end_city/tower_top")?;
        let ship = get_template("end_city/ship")?;

        let bounding_box = BlockBox::new(
            chunk_center_x - 30,
            context.min_y,
            chunk_center_z - 30,
            chunk_center_x + 30,
            256,
            chunk_center_z + 30,
        );

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(EndCityPiece {
            piece: StructurePiece::new(StructurePieceType::EndCity, bounding_box, 0),
            base_floor,
            tower_base,
            tower_piece,
            tower_top,
            ship,
            rotation,
            has_ship: context.random.next_f32() < 0.5,
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(chunk_center_x, 64, chunk_center_z),
            collector: Arc::new(collector.into()),
        })
    }
}

pub struct EndCityPiece {
    piece: StructurePiece,
    base_floor: Arc<StructureTemplate>,
    tower_base: Arc<StructureTemplate>,
    tower_piece: Arc<StructureTemplate>,
    tower_top: Arc<StructureTemplate>,
    ship: Arc<StructureTemplate>,
    rotation: Rotation,
    has_ship: bool,
}

impl StructurePieceBase for EndCityPiece {
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
        let start_y = if sample_y <= 0 { 60 } else { sample_y };

        // 1. Place Base Floor
        let mut pos = Vector3::new(origin.x, start_y, origin.z);
        place_template(
            chunk,
            &self.base_floor,
            pos,
            (0, 0),
            self.rotation,
            true,
            false,
            &[],
            Some(chunk_box),
        );

        // 2. Place Tower Base
        pos.y += self.base_floor.size.y;
        place_template(
            chunk,
            &self.tower_base,
            pos,
            (0, 0),
            self.rotation,
            true,
            false,
            &[],
            Some(chunk_box),
        );

        // 3. Place Tower Piece
        pos.y += self.tower_base.size.y;
        place_template(
            chunk,
            &self.tower_piece,
            pos,
            (0, 0),
            self.rotation,
            true,
            false,
            &[],
            Some(chunk_box),
        );

        // 4. Place Tower Top
        pos.y += self.tower_piece.size.y;
        place_template(
            chunk,
            &self.tower_top,
            pos,
            (0, 0),
            self.rotation,
            true,
            false,
            &[],
            Some(chunk_box),
        );

        // 5. Place End Ship
        if self.has_ship {
            let ship_pos = Vector3::new(origin.x + 16, start_y + 20, origin.z + 16);
            place_template(
                chunk,
                &self.ship,
                ship_pos,
                (0, 0),
                self.rotation,
                true,
                false,
                &[],
                Some(chunk_box),
            );
        }
    }
}
