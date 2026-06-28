use pumpkin_data::{Block, BlockState, Rotation, tag};
use pumpkin_util::{
    HeightMap,
    math::{block_box::BlockBox, position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::{
    proto_chunk::GenerationCache,
    structure::template::{BlockStateResolver, StructureTemplate, get_template},
};

pub struct FossilFeature {
    pub fossil_structures: Vec<&'static str>,
    pub overlay_structures: Vec<&'static str>,
    pub fossil_processor: FossilProcessor,
    pub overlay_processor: FossilProcessor,
    pub max_empty_corners_allowed: u8,
}

#[derive(Clone, Copy)]
pub enum FossilProcessor {
    FossilRot,
    Coal,
    Diamonds,
}

impl FossilProcessor {
    const fn integrity(self) -> f32 {
        match self {
            Self::FossilRot => 0.9,
            Self::Coal | Self::Diamonds => 0.1,
        }
    }

    fn process(self, state: &'static BlockState) -> &'static BlockState {
        if matches!(self, Self::Diamonds) && Block::from_state_id(state.id) == &Block::COAL_ORE {
            Block::DEEPSLATE_DIAMOND_ORE.default_state
        } else {
            state
        }
    }
}

impl FossilFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        min_y: i8,
        height: u16,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if self.fossil_structures.is_empty()
            || self.fossil_structures.len() != self.overlay_structures.len()
        {
            return false;
        }

        let rotation = Rotation::from_index(random.next_bounded_i32(4) as u8);
        let template_index = random.next_bounded_i32(self.fossil_structures.len() as i32) as usize;

        let Some(fossil_name) = self.fossil_structures.get(template_index) else {
            return false;
        };
        let Some(overlay_name) = self.overlay_structures.get(template_index) else {
            return false;
        };
        let Some(fossil) = get_template(
            fossil_name
                .strip_prefix("minecraft:")
                .unwrap_or(fossil_name),
        ) else {
            return false;
        };
        let Some(overlay) = get_template(
            overlay_name
                .strip_prefix("minecraft:")
                .unwrap_or(overlay_name),
        ) else {
            return false;
        };

        let rotated_size = rotation.transform_size(fossil.size);
        let origin_x = pos.0.x - rotated_size.x / 2;
        let origin_z = pos.0.z - rotated_size.z / 2;
        let mut ocean_floor = pos.0.y;

        for x in origin_x..origin_x + rotated_size.x {
            for z in origin_z..origin_z + rotated_size.z {
                ocean_floor = ocean_floor.min(chunk.get_top_y(&HeightMap::OceanFloorWg, x, z));
            }
        }

        let origin_y = (ocean_floor - 15 - random.next_bounded_i32(10)).max(i32::from(min_y) + 10);
        let origin = Vector3::new(origin_x, origin_y, origin_z);
        let fossil_box = BlockBox::new(
            origin.x,
            origin.y,
            origin.z,
            origin.x + rotated_size.x - 1,
            origin.y + rotated_size.y - 1,
            origin.z + rotated_size.z - 1,
        );

        if count_empty_corners(chunk, &fossil_box) > usize::from(self.max_empty_corners_allowed) {
            return false;
        }

        let chunk_min_x = (pos.0.x >> 4) << 4;
        let chunk_min_z = (pos.0.z >> 4) << 4;
        let placement_box = BlockBox::new(
            chunk_min_x - 16,
            i32::from(min_y),
            chunk_min_z - 16,
            chunk_min_x + 31,
            i32::from(min_y) + i32::from(height) - 1,
            chunk_min_z + 31,
        );

        place_fossil_template(
            chunk,
            &fossil,
            origin,
            rotation,
            self.fossil_processor,
            random,
            &placement_box,
        );
        place_fossil_template(
            chunk,
            &overlay,
            origin,
            rotation,
            self.overlay_processor,
            random,
            &placement_box,
        );

        true
    }
}

fn count_empty_corners<T: GenerationCache>(chunk: &T, bounding_box: &BlockBox) -> usize {
    let mut empty_corners = 0;

    for x in [bounding_box.min.x, bounding_box.max.x] {
        for y in [bounding_box.min.y, bounding_box.max.y] {
            for z in [bounding_box.min.z, bounding_box.max.z] {
                let state =
                    GenerationCache::get_block_state(chunk, &Vector3::new(x, y, z)).to_state();
                let block = Block::from_state_id(state.id);
                if state.is_air() || block == &Block::LAVA || block == &Block::WATER {
                    empty_corners += 1;
                }
            }
        }
    }

    empty_corners
}

fn place_fossil_template<T: GenerationCache>(
    chunk: &mut T,
    template: &StructureTemplate,
    origin: Vector3<i32>,
    rotation: Rotation,
    processor: FossilProcessor,
    random: &mut RandomGenerator,
    placement_box: &BlockBox,
) {
    for block in &template.blocks {
        let local_pos = rotation.transform_pos(block.pos, template.size);
        let world_pos = origin + local_pos;

        if !placement_box.contains_pos(&world_pos) || random.next_f32() > processor.integrity() {
            continue;
        }

        let current_state = GenerationCache::get_block_state(chunk, &world_pos).to_state();
        if tag::Block::MINECRAFT_FEATURES_CANNOT_REPLACE
            .1
            .contains(&current_state.id)
        {
            continue;
        }

        let palette_entry = &template.palette[block.state as usize];
        if palette_entry.name == "minecraft:structure_void" {
            continue;
        }

        if let Some(state) =
            BlockStateResolver::resolve(palette_entry, rotation, Default::default())
        {
            chunk.set_block_state(&world_pos, processor.process(state));
        }
    }
}
