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

const PORTALS: &[&str] = &[
    "ruined_portal/portal_1",
    "ruined_portal/portal_2",
    "ruined_portal/portal_3",
    "ruined_portal/portal_4",
    "ruined_portal/portal_5",
    "ruined_portal/portal_6",
    "ruined_portal/portal_7",
    "ruined_portal/portal_8",
    "ruined_portal/portal_9",
    "ruined_portal/portal_10",
    "ruined_portal/giant_portal_1",
    "ruined_portal/giant_portal_2",
    "ruined_portal/giant_portal_3",
];

pub struct RuinedPortalGenerator {
    pub variant: pumpkin_data::structures::StructureKeys,
}

impl StructureGenerator for RuinedPortalGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        let chunk_center_x = get_center_x(context.chunk_x);
        let chunk_center_z = get_center_z(context.chunk_z);

        let rotation_idx = context.random.next_bounded_i32(4) as u8;
        let rotation = Rotation::from_index(rotation_idx);

        let template_idx = context.random.next_bounded_i32(PORTALS.len() as i32) as usize;
        let template_name = PORTALS[template_idx];
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
        collector.add_piece(Box::new(RuinedPortalPiece {
            piece: StructurePiece::new(StructurePieceType::RuinedPortal, bounding_box, 0),
            template,
            rotation,
            variant: self.variant,
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(chunk_center_x, 64, chunk_center_z),
            collector: Arc::new(collector.into()),
        })
    }
}

pub struct RuinedPortalPiece {
    piece: StructurePiece,
    template: Arc<StructureTemplate>,
    rotation: Rotation,
    variant: pumpkin_data::structures::StructureKeys,
}

impl StructurePieceBase for RuinedPortalPiece {
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
        let (height_map_type, vertical_offset) = match self.variant {
            pumpkin_data::structures::StructureKeys::RuinedPortalOcean => {
                (pumpkin_util::HeightMap::OceanFloorWg, -2)
            }
            pumpkin_data::structures::StructureKeys::RuinedPortalDesert => {
                (pumpkin_util::HeightMap::WorldSurfaceWg, -3) // partly buried
            }
            pumpkin_data::structures::StructureKeys::RuinedPortalNether => {
                (pumpkin_util::HeightMap::WorldSurfaceWg, 0)
            }
            _ => (pumpkin_util::HeightMap::WorldSurfaceWg, -1),
        };

        let sample_y =
            if self.variant == pumpkin_data::structures::StructureKeys::RuinedPortalNether {
                45
            } else {
                chunk.get_top_y(&height_map_type, origin.x, origin.z)
            };

        let target_y = sample_y + vertical_offset;
        let mut final_origin = origin;
        final_origin.y = target_y;

        place_template(
            chunk,
            &self.template,
            final_origin,
            (0, 0),
            self.rotation,
            true,  // skip_air
            false, // no waterlogging
            &[],
            Some(chunk_box),
        );
    }
}
