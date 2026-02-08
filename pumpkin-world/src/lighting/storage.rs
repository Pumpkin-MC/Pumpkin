use crate::chunk_system::chunk_state::Chunk;
use crate::chunk_system::generation_cache::Cache;
use crate::generation::height_limit::HeightLimitView;
use pumpkin_util::math::position::BlockPos;

#[inline(always)]
fn get_chunk_index(cache: &Cache, chunk_x: i32, chunk_z: i32) -> Option<usize> {
    let rel_x = chunk_x - cache.x;
    let rel_z = chunk_z - cache.z;
    if rel_x < 0 || rel_x >= cache.size || rel_z < 0 || rel_z >= cache.size {
        return None;
    }
    Some((rel_x * cache.size + rel_z) as usize)
}

#[inline(always)]
fn get_section_y(cache: &Cache, pos_y: i32) -> Option<usize> {
    let bottom = cache.bottom_y() as i32;
    if pos_y < bottom {
        return None;
    }
    let section = ((pos_y - bottom) >> 4) as usize;
    Some(section)
}

#[inline(always)]
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

#[inline(always)]
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
            let mut light_engine = c.light_engine.lock().unwrap();
            if section_y < light_engine.block_light.len() {
                light_engine.block_light[section_y].set(x, y, z, level);
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

#[inline(always)]
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

    match &cache.chunks[idx] {
        Chunk::Level(c) => {
            let light_engine = c.light_engine.lock().unwrap();
            if section_y >= light_engine.sky_light.len() {
                return 0;
            }
            light_engine.sky_light[section_y].get(x, y, z)
        }
        Chunk::Proto(c) => {
            if section_y >= c.light.sky_light.len() {
                return 0;
            }
            c.light.sky_light[section_y].get(x, y, z)
        }
    }
}

#[inline(always)]
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
            let mut light_engine = c.light_engine.lock().unwrap();
            if section_y < light_engine.sky_light.len() {
                light_engine.sky_light[section_y].set(x, y, z, level);
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