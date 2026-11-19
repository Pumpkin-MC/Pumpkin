use pumpkin_data::poi_type::PoiType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::chunk::palette::BlockPalette;
use pumpkin_world::poi::{PoiStorage, SectionPos};

pub(crate) const MAX_CHUNKS_PER_TICK: usize = 8;

fn collect_section_palette(
    palette: &BlockPalette,
    chunk_x: i32,
    section_y: i32,
    chunk_z: i32,
) -> Vec<(BlockPos, PoiType)> {
    if !palette.contains_matching(|state| PoiType::from_state_id(state.as_u16()).is_some()) {
        return Vec::new();
    }
    let mut found = Vec::new();
    for y in 0..BlockPalette::SIZE {
        for z in 0..BlockPalette::SIZE {
            for x in 0..BlockPalette::SIZE {
                let state = palette.get(x, y, z);
                if let Some(poi_type) = PoiType::from_state_id(state.as_u16()) {
                    found.push((
                        BlockPos(Vector3::new(
                            chunk_x * BlockPalette::SIZE as i32 + x as i32,
                            section_y * BlockPalette::SIZE as i32 + y as i32,
                            chunk_z * BlockPalette::SIZE as i32 + z as i32,
                        )),
                        poi_type,
                    ));
                }
            }
        }
    }
    found
}

pub(crate) fn scan_chunk(storage: &mut PoiStorage, chunk: &ChunkData) {
    let mut sections = Vec::new();
    {
        let palettes = chunk
            .section
            .block_sections
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let base_y = chunk.section.min_y / 16;
        for (index, palette) in palettes.iter().enumerate() {
            let section_y = base_y + index as i32;
            let found = collect_section_palette(palette, chunk.x, section_y, chunk.z);
            sections.push((SectionPos::new(chunk.x, section_y, chunk.z), found));
        }
    }
    for (section, found) in sections {
        storage.refresh_section(section, &found);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::Block;

    #[test]
    fn scan_section_finds_poi_blocks() {
        let dir = std::env::temp_dir().join("pumpkin_poi_scan_test");
        let _ = std::fs::remove_dir_all(&dir);

        let bed_state = Block::WHITE_BED
            .states
            .iter()
            .find(|state| PoiType::from_state_id(state.id.as_u16()) == Some(PoiType::Home));
        assert!(bed_state.is_some());
        let Some(bed_state) = bed_state else {
            return;
        };
        let bell_state = Block::BELL.default_state;

        let mut palette = BlockPalette::default();
        palette.set(1, 2, 3, bed_state.id);
        palette.set(4, 5, 6, bell_state.id);

        let found = collect_section_palette(&palette, 0, 0, 0);
        let bed_pos = BlockPos(Vector3::new(1, 2, 3));
        let bell_pos = BlockPos(Vector3::new(4, 5, 6));
        assert_eq!(found.len(), 2);
        assert!(found.contains(&(bed_pos, PoiType::Home)));
        assert!(found.contains(&(bell_pos, PoiType::Meeting)));

        let mut storage = PoiStorage::new(dir);
        storage.refresh_section(SectionPos::new(0, 0, 0), &found);
        assert_eq!(storage.get_type(&bed_pos), Some(PoiType::Home));
        assert_eq!(storage.get_type(&bell_pos), Some(PoiType::Meeting));
    }
}
