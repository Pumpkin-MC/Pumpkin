use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use crate::{
    ProtoChunk,
    generation::{
        positions::chunk_pos::{get_center_x, get_center_z},
        structure::{
            piece::StructurePieceType,
            shiftable_piece::ShiftableStructurePiece,
            structures::{
                StructureGenerator, StructureGeneratorContext, StructurePiece, StructurePieceBase,
                StructurePiecesCollector, StructurePosition,
            },
        },
    },
};

/// Village biome variants determine the block palette.
#[derive(Clone, Copy)]
pub enum VillageVariant {
    Plains,
    Desert,
    Savanna,
    Snowy,
    Taiga,
}

/// Village generator for all 5 biome variants.
pub struct VillagePlainsGenerator;
pub struct VillageDesertGenerator;
pub struct VillageSavannaGenerator;
pub struct VillageSnowyGenerator;
pub struct VillageTaigaGenerator;

fn generate_village(
    variant: VillageVariant,
    mut context: StructureGeneratorContext,
) -> Option<StructurePosition> {
    let x = get_center_x(context.chunk_x);
    let z = get_center_z(context.chunk_z);

    let mut collector = StructurePiecesCollector::default();
    collector.add_piece(Box::new(VillagePiece {
        shiftable_structure_piece: ShiftableStructurePiece::new(
            StructurePieceType::Jigsaw,
            x,
            64,
            z,
            16,
            8,
            16,
            StructurePiece::get_random_horizontal_direction(&mut context.random).get_axis(),
        ),
        variant,
    }));

    Some(StructurePosition {
        start_pos: BlockPos::new(x, 64, z),
        collector: Arc::new(collector.into()),
    })
}

impl StructureGenerator for VillagePlainsGenerator {
    fn get_structure_position(
        &self,
        context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        generate_village(VillageVariant::Plains, context)
    }
}

impl StructureGenerator for VillageDesertGenerator {
    fn get_structure_position(
        &self,
        context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        generate_village(VillageVariant::Desert, context)
    }
}

impl StructureGenerator for VillageSavannaGenerator {
    fn get_structure_position(
        &self,
        context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        generate_village(VillageVariant::Savanna, context)
    }
}

impl StructureGenerator for VillageSnowyGenerator {
    fn get_structure_position(
        &self,
        context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        generate_village(VillageVariant::Snowy, context)
    }
}

impl StructureGenerator for VillageTaigaGenerator {
    fn get_structure_position(
        &self,
        context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        generate_village(VillageVariant::Taiga, context)
    }
}

#[derive(Clone)]
pub struct VillagePiece {
    shiftable_structure_piece: ShiftableStructurePiece,
    variant: VillageVariant,
}

impl StructurePieceBase for VillagePiece {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }

    fn place(&mut self, chunk: &mut ProtoChunk, _random: &mut RandomGenerator, _seed: i64) {
        if !self
            .shiftable_structure_piece
            .adjust_to_average_height(chunk)
        {
            return;
        }

        let box_limit = self.shiftable_structure_piece.piece.bounding_box;
        let p = &self.shiftable_structure_piece.piece;

        let (planks, log, path_block, roof_block) = match self.variant {
            VillageVariant::Plains => (
                Block::OAK_PLANKS.default_state,
                Block::OAK_LOG.default_state,
                Block::GRAVEL.default_state,
                Block::OAK_PLANKS.default_state,
            ),
            VillageVariant::Desert => (
                Block::SMOOTH_SANDSTONE.default_state,
                Block::SANDSTONE.default_state,
                Block::SMOOTH_SANDSTONE.default_state,
                Block::SMOOTH_SANDSTONE.default_state,
            ),
            VillageVariant::Savanna => (
                Block::ACACIA_PLANKS.default_state,
                Block::ACACIA_LOG.default_state,
                Block::GRAVEL.default_state,
                Block::ACACIA_PLANKS.default_state,
            ),
            VillageVariant::Snowy => (
                Block::SPRUCE_PLANKS.default_state,
                Block::STRIPPED_SPRUCE_LOG.default_state,
                Block::SNOW_BLOCK.default_state,
                Block::SPRUCE_PLANKS.default_state,
            ),
            VillageVariant::Taiga => (
                Block::SPRUCE_PLANKS.default_state,
                Block::SPRUCE_LOG.default_state,
                Block::GRAVEL.default_state,
                Block::SPRUCE_PLANKS.default_state,
            ),
        };
        let cobblestone = Block::COBBLESTONE.default_state;
        let air = Block::AIR.default_state;

        // Village is a simplified single-house + well + path layout

        // === Path (center road) ===
        p.fill(chunk, &box_limit, 6, 0, 0, 9, 0, 15, path_block);

        // === Well (center) ===
        p.fill(chunk, &box_limit, 6, 0, 6, 9, 0, 9, cobblestone);
        p.fill(chunk, &box_limit, 6, 1, 6, 6, 3, 6, cobblestone);
        p.fill(chunk, &box_limit, 9, 1, 6, 9, 3, 6, cobblestone);
        p.fill(chunk, &box_limit, 6, 1, 9, 6, 3, 9, cobblestone);
        p.fill(chunk, &box_limit, 9, 1, 9, 9, 3, 9, cobblestone);
        // Well roof
        p.fill(chunk, &box_limit, 6, 3, 6, 9, 3, 9, planks);
        // Well interior (water would go here)
        p.fill(chunk, &box_limit, 7, 0, 7, 8, 2, 8, air);

        // === House 1 (left side) ===
        // Foundation
        p.fill(chunk, &box_limit, 0, 0, 2, 4, 0, 6, cobblestone);
        // Walls
        p.fill(chunk, &box_limit, 0, 1, 2, 0, 4, 6, planks);
        p.fill(chunk, &box_limit, 4, 1, 2, 4, 4, 6, planks);
        p.fill(chunk, &box_limit, 1, 1, 2, 3, 4, 2, planks);
        p.fill(chunk, &box_limit, 1, 1, 6, 3, 4, 6, planks);
        // Interior
        p.fill(chunk, &box_limit, 1, 1, 3, 3, 3, 5, air);
        // Door
        p.fill(chunk, &box_limit, 4, 1, 4, 4, 2, 4, air);
        // Window
        p.add_block(
            chunk,
            Block::GLASS_PANE.default_state,
            0,
            2,
            4,
            &box_limit,
        );
        // Corner logs
        p.fill(chunk, &box_limit, 0, 1, 2, 0, 4, 2, log);
        p.fill(chunk, &box_limit, 4, 1, 2, 4, 4, 2, log);
        p.fill(chunk, &box_limit, 0, 1, 6, 0, 4, 6, log);
        p.fill(chunk, &box_limit, 4, 1, 6, 4, 4, 6, log);
        // Roof
        p.fill(chunk, &box_limit, 0, 4, 2, 4, 4, 6, roof_block);
        // Crafting table
        p.add_block(
            chunk,
            Block::CRAFTING_TABLE.default_state,
            1,
            1,
            3,
            &box_limit,
        );

        // === House 2 (right side) ===
        // Foundation
        p.fill(chunk, &box_limit, 11, 0, 2, 15, 0, 6, cobblestone);
        // Walls
        p.fill(chunk, &box_limit, 11, 1, 2, 11, 4, 6, planks);
        p.fill(chunk, &box_limit, 15, 1, 2, 15, 4, 6, planks);
        p.fill(chunk, &box_limit, 12, 1, 2, 14, 4, 2, planks);
        p.fill(chunk, &box_limit, 12, 1, 6, 14, 4, 6, planks);
        // Interior
        p.fill(chunk, &box_limit, 12, 1, 3, 14, 3, 5, air);
        // Door
        p.fill(chunk, &box_limit, 11, 1, 4, 11, 2, 4, air);
        // Window
        p.add_block(
            chunk,
            Block::GLASS_PANE.default_state,
            15,
            2,
            4,
            &box_limit,
        );
        // Corner logs
        p.fill(chunk, &box_limit, 11, 1, 2, 11, 4, 2, log);
        p.fill(chunk, &box_limit, 15, 1, 2, 15, 4, 2, log);
        p.fill(chunk, &box_limit, 11, 1, 6, 11, 4, 6, log);
        p.fill(chunk, &box_limit, 15, 1, 6, 15, 4, 6, log);
        // Roof
        p.fill(chunk, &box_limit, 11, 4, 2, 15, 4, 6, roof_block);
        // Furnace
        p.add_block(
            chunk,
            Block::FURNACE.default_state,
            14,
            1,
            3,
            &box_limit,
        );

        // === Farm (south end) ===
        p.fill(chunk, &box_limit, 2, 0, 11, 5, 0, 14, Block::FARMLAND.default_state);
        p.add_block(
            chunk,
            Block::COMPOSTER.default_state,
            1,
            1,
            12,
            &box_limit,
        );

        // === Bell (center of village) ===
        p.add_block(
            chunk,
            Block::BELL.default_state,
            7,
            1,
            5,
            &box_limit,
        );

        // === Lamp post ===
        p.fill(chunk, &box_limit, 7, 1, 1, 7, 3, 1, log);
        p.add_block(
            chunk,
            Block::LANTERN.default_state,
            7,
            4,
            1,
            &box_limit,
        );

        // Fill downward from structure corners
        for &x in &[0, 4, 11, 15] {
            for &z in &[2, 6] {
                p.fill_downwards(chunk, cobblestone, x, -1, z, &box_limit);
            }
        }
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.shiftable_structure_piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.shiftable_structure_piece.piece
    }
}
