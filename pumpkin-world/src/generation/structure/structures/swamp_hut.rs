use pumpkin_data::Block;
use pumpkin_util::{
    HeightMap,
    math::{block_box::BlockBox, position::BlockPos, vector3::Vector3},
};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::{
        positions::chunk_pos::{get_center_x, get_center_z},
        structure::structures::{
            StructureGenerator, StructurePiece, StructurePiecesCollector, StructurePosition, fill,
            fill_downwards,
        },
    },
};

#[derive(Deserialize, Clone)]
pub struct SwampHutGenerator;

impl StructureGenerator for SwampHutGenerator {
    fn try_generate(&self, _seed: i64, chunk_x: i32, chunk_z: i32) -> Option<StructurePosition> {
        let x = get_center_x(chunk_x);
        let z = get_center_z(chunk_z);
        let box_min_x = x;
        let box_min_z = z;

        let bounding_box =
            BlockBox::new(box_min_x, 60, box_min_z, box_min_x + 7, 80, box_min_z + 7);

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(SwampHutPiece { bounding_box }));

        Some(StructurePosition {
            start_pos: BlockPos::new(x, 64, z),
            collector: collector.into(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct SwampHutPiece {
    bounding_box: BlockBox,
}

impl StructurePiece for SwampHutPiece {
    fn bounding_box(&self) -> &BlockBox {
        &self.bounding_box
    }

    fn place(&self, chunk: &mut ProtoChunk, _seed: i64) {
        let origin_x = self.bounding_box.min.x;
        let origin_z = self.bounding_box.min.z;

        // Find surface height (approximate, usually slightly submerged in swamp)
        let floor_y = chunk.get_top_y(&HeightMap::OceanFloorWg, origin_x + 3, origin_z + 3) + 3; // +3 to lift it up

        let origin_y = floor_y;

        let fill_rel = |min_x, min_y, min_z, max_x, max_y, max_z, block, c: &mut ProtoChunk| {
            fill(
                origin_x + min_x,
                origin_y + min_y,
                origin_z + min_z,
                origin_x + max_x,
                origin_y + max_y,
                origin_z + max_z,
                block,
                c,
            )
        };

        let offset = |x, y, z| Vector3::new(origin_x + x, origin_y + y, origin_z + z);

        // 1. Stilts (Logs downwards)
        let spruce_log = &Block::SPRUCE_LOG.default_state;
        let stilts = [(1, 1), (1, 6), (6, 1), (6, 6)];

        for (sx, sz) in stilts {
            fill_downwards(origin_x + sx, origin_y, origin_z + sz, spruce_log, chunk);
        }

        // 2. Main Platform (Spruce Planks)
        let spruce_planks = &Block::SPRUCE_PLANKS.default_state;
        fill_rel(0, 0, 0, 7, 0, 7, spruce_planks, chunk);

        // 3. Walls
        fill_rel(0, 1, 0, 7, 3, 0, spruce_planks, chunk); // North
        fill_rel(0, 1, 7, 7, 3, 7, spruce_planks, chunk); // South
        fill_rel(0, 1, 0, 0, 3, 7, spruce_planks, chunk); // West
        fill_rel(7, 1, 0, 7, 3, 7, spruce_planks, chunk); // East

        // 4. Roof (Simple block roof for now, stairs are complex without proper state orientation helpers)
        fill_rel(0, 4, 0, 7, 4, 7, spruce_planks, chunk);
        fill_rel(1, 5, 1, 6, 5, 6, spruce_planks, chunk);

        // 5. Windows/Air
        fill_rel(1, 1, 1, 6, 3, 6, Block::AIR.default_state, chunk); // Clear inside

        // 6. Interior Decorations
        chunk.set_block_state(&offset(2, 1, 2), Block::CRAFTING_TABLE.default_state);
        chunk.set_block_state(&offset(2, 1, 3), Block::CAULDRON.default_state);

        // Note: Flower Pot usually requires block entity data or specific state, simplifying here
        chunk.set_block_state(&offset(5, 1, 3), Block::POTTED_BLUE_ORCHID.default_state);

        // Fence on balcony
        chunk.set_block_state(&offset(3, 1, 0), Block::OAK_FENCE.default_state);
    }
}
