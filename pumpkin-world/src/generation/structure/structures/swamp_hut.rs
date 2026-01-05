use pumpkin_data::Block;
use pumpkin_util::{
    HeightMap,
    math::{
        block_box::BlockBox,
        position::BlockPos,
        vector3::{Axis, Vector3},
    },
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
    fn get_structure_position(
        &self,
        _seed: i64,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Option<StructurePosition> {
        let x = get_center_x(chunk_x);
        let z = get_center_z(chunk_z);

        // TODO: random axis
        let bounding_box = BlockBox::create_box(x, 64, z, Axis::X, 7, 7, 9);

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
        let min = self.bounding_box.min;
        let origin_x = min.x;
        let origin_z = min.z;

        // Vanilla uses adjustToAverageHeight.
        let surface_y = chunk.get_top_y(&HeightMap::OceanFloorWg, origin_x + 3, origin_z + 3);
        let origin_y = surface_y + 1;

        let spruce_planks = &Block::SPRUCE_PLANKS.default_state;
        let oak_log = &Block::OAK_LOG.default_state;
        let oak_fence = &Block::OAK_FENCE.default_state;
        let air = &Block::AIR.default_state;

        // 1. THE FLOOR
        fill(
            origin_x + 1,
            origin_y + 1,
            origin_z + 1,
            origin_x + 5,
            origin_y + 1,
            origin_z + 7,
            spruce_planks,
            chunk,
        );

        // 2. THE CEILING
        fill(
            origin_x + 1,
            origin_y + 4,
            origin_z + 2,
            origin_x + 5,
            origin_y + 4,
            origin_z + 7,
            spruce_planks,
            chunk,
        );

        // 3. THE ENTRANCE PLATFORM
        fill(
            origin_x + 2,
            origin_y + 1,
            origin_z,
            origin_x + 4,
            origin_y + 1,
            origin_z,
            spruce_planks,
            chunk,
        );

        // Front Wall
        fill(
            origin_x + 2,
            origin_y + 2,
            origin_z + 2,
            origin_x + 3,
            origin_y + 3,
            origin_z + 2,
            spruce_planks,
            chunk,
        );
        // Left Wall
        fill(
            origin_x + 1,
            origin_y + 2,
            origin_z + 3,
            origin_x + 1,
            origin_y + 3,
            origin_z + 6,
            spruce_planks,
            chunk,
        );
        // Right Wall
        fill(
            origin_x + 5,
            origin_y + 2,
            origin_z + 3,
            origin_x + 5,
            origin_y + 3,
            origin_z + 6,
            spruce_planks,
            chunk,
        );
        // Back Wall
        fill(
            origin_x + 2,
            origin_y + 2,
            origin_z + 7,
            origin_x + 4,
            origin_y + 3,
            origin_z + 7,
            spruce_planks,
            chunk,
        );

        // 5. THE LOGS
        let logs = [(1, 2), (5, 2), (1, 7), (5, 7)];
        for (lx, lz) in logs {
            fill(
                origin_x + lx,
                origin_y,
                origin_z + lz,
                origin_x + lx,
                origin_y + 3,
                origin_z + lz,
                oak_log,
                chunk,
            );
            fill_downwards(origin_x + lx, origin_y - 1, origin_z + lz, oak_log, chunk);
        }

        // Placed at Y=2 (feet level) or Y=3 (eye level)
        let y_floor = origin_y + 2;
        let y_eye = origin_y + 3;

        chunk.set_block_state(
            &Vector3::new(origin_x + 3, y_floor, origin_z + 6),
            Block::CRAFTING_TABLE.default_state,
        );
        chunk.set_block_state(
            &Vector3::new(origin_x + 4, y_floor, origin_z + 6),
            Block::CAULDRON.default_state,
        );
        chunk.set_block_state(
            &Vector3::new(origin_x + 1, y_eye, origin_z + 5),
            Block::POTTED_RED_MUSHROOM.default_state,
        );

        // Fences & Gaps
        chunk.set_block_state(&Vector3::new(origin_x + 2, y_eye, origin_z + 2), oak_fence); // Above the door
        chunk.set_block_state(&Vector3::new(origin_x + 3, y_eye, origin_z + 7), oak_fence); // Back window
        chunk.set_block_state(
            &Vector3::new(origin_x + 1, y_floor, origin_z + 1),
            oak_fence,
        ); // Balcony left
        chunk.set_block_state(
            &Vector3::new(origin_x + 5, y_floor, origin_z + 1),
            oak_fence,
        ); // Balcony right

        // Clear window air (Java: addBlock AIR at 1, 3, 4 and 5, 3, 4/5)
        chunk.set_block_state(&Vector3::new(origin_x + 1, y_eye, origin_z + 4), air);
        chunk.set_block_state(&Vector3::new(origin_x + 5, y_eye, origin_z + 4), air);
        chunk.set_block_state(&Vector3::new(origin_x + 5, y_eye, origin_z + 5), air);
    }
}
