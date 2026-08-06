use crate::chunk_system::chunk_state::Chunk;
use crate::chunk_system::generation_cache::Cache;
use crate::generation::height_limit::HeightLimitView;
use pumpkin_util::math::position::BlockPos;

const fn get_chunk_index(cache: &Cache, chunk_x: i32, chunk_z: i32) -> Option<usize> {
    let rel_x = chunk_x - cache.x;
    let rel_z = chunk_z - cache.z;
    if rel_x < 0 || rel_x >= cache.size || rel_z < 0 || rel_z >= cache.size {
        return None;
    }
    Some((rel_x * cache.size + rel_z) as usize)
}

fn get_section_y(cache: &Cache, pos_y: i32) -> Option<usize> {
    let bottom = cache.bottom_y() as i32;
    if pos_y < bottom {
        return None;
    }
    let section = ((pos_y - bottom) >> 4) as usize;
    Some(section)
}

#[must_use]
pub fn get_block_light(cache: &Cache, pos: BlockPos) -> u8 {
    let chunk_x = pos.0.x >> 4;
    let chunk_z = pos.0.z >> 4;

    let Some(idx) = get_chunk_index(cache, chunk_x, chunk_z) else {
        return 0;
    };

    let Some(section_y) = get_section_y(cache, pos.0.y) else {
        return 0;
    };

    let x = (pos.0.x & 15) as usize;
    let y = (pos.0.y & 15) as usize;
    let z = (pos.0.z & 15) as usize;

    match &cache.chunks[idx] {
        Chunk::Level(c) => {
            let light_engine = c.light_engine.lock().unwrap();
            if section_y >= light_engine.block_light.len() {
                return 0;
            }
            light_engine.block_light[section_y].get(x, y, z)
        }
        Chunk::Proto(c) => {
            if section_y >= c.light.block_light.len() {
                return 0;
            }
            c.light.block_light[section_y].get(x, y, z)
        }
    }
}

pub fn set_block_light(cache: &mut Cache, pos: BlockPos, level: u8) {
    let chunk_x = pos.0.x >> 4;
    let chunk_z = pos.0.z >> 4;

    let Some(idx) = get_chunk_index(cache, chunk_x, chunk_z) else {
        return;
    };
    let Some(section_y) = get_section_y(cache, pos.0.y) else {
        return;
    };

    let x = (pos.0.x & 15) as usize;
    let y = (pos.0.y & 15) as usize;
    let z = (pos.0.z & 15) as usize;

    match &mut cache.chunks[idx] {
        Chunk::Level(c) => {
            let marked_dirty = {
                let mut light_engine = c
                    .light_engine
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if section_y < light_engine.block_light.len() {
                    light_engine.block_light[section_y].set(x, y, z, level);
                    true
                } else {
                    false
                }
            };
            if marked_dirty {
                c.dirty.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
        Chunk::Proto(c) => {
            if section_y < c.light.block_light.len() {
                c.light.block_light[section_y].set(x, y, z, level);
            }
        }
    }
}

#[must_use]
pub fn get_sky_light(cache: &Cache, pos: BlockPos) -> u8 {
    let chunk_x = pos.0.x >> 4;
    let chunk_z = pos.0.z >> 4;

    let Some(idx) = get_chunk_index(cache, chunk_x, chunk_z) else {
        return 0;
    };

    let Some(section_y) = get_section_y(cache, pos.0.y) else {
        return 0;
    };

    let x = (pos.0.x & 15) as usize;
    let y = (pos.0.y & 15) as usize;
    let z = (pos.0.z & 15) as usize;

    // The out-of-range answers below are deliberately asymmetric with
    // `get_block_light`, which returns 0 in both directions. This is not an
    // oversight:
    //
    // - Below the world (`get_section_y` returned `None`) there is no sky, so 0.
    // - Above the stored sections there is nothing between the query and the
    //   sky, so 15. That is the open-sky answer, not a "don't know" fallback.
    //   For `Chunk::Level` the arrays span every section the chunk stores
    //   (padded to the full dimension height by `upgrade_to_level_chunk`, and
    //   sized from the saved section list on the disk path); for `Chunk::Proto`
    //   they span only the generated height. In both cases anything past the
    //   end is air that was never lit, i.e. open sky.
    //
    // Known limitation, deliberately left alone here: this is wrong in a
    // dimension with no sky light (Nether/End), where the answer should be 0.
    // `Cache` carries no dimension or `has_sky_light` flag, and the sky light
    // engine is not gated on one either, so that is a separate change.
    match &cache.chunks[idx] {
        Chunk::Level(c) => {
            let light_engine = c.light_engine.lock().unwrap();
            if section_y >= light_engine.sky_light.len() {
                return 15;
            }
            light_engine.sky_light[section_y].get(x, y, z)
        }
        Chunk::Proto(c) => {
            if section_y >= c.light.sky_light.len() {
                return 15;
            }
            c.light.sky_light[section_y].get(x, y, z)
        }
    }
}

pub fn set_sky_light(cache: &mut Cache, pos: BlockPos, level: u8) {
    let chunk_x = pos.0.x >> 4;
    let chunk_z = pos.0.z >> 4;

    let Some(idx) = get_chunk_index(cache, chunk_x, chunk_z) else {
        return;
    };
    let Some(section_y) = get_section_y(cache, pos.0.y) else {
        return;
    };

    let x = (pos.0.x & 15) as usize;
    let y = (pos.0.y & 15) as usize;
    let z = (pos.0.z & 15) as usize;

    match &mut cache.chunks[idx] {
        Chunk::Level(c) => {
            let marked_dirty = {
                let mut light_engine = c
                    .light_engine
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if section_y < light_engine.sky_light.len() {
                    light_engine.sky_light[section_y].set(x, y, z, level);
                    true
                } else {
                    false
                }
            };
            if marked_dirty {
                c.dirty.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
        Chunk::Proto(c) => {
            if section_y < c.light.sky_light.len() {
                c.light.sky_light[section_y].set(x, y, z, level);
            }
        }
    }
}
