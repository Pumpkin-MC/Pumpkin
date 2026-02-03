use crate::chunk_system::chunk_state::Chunk;
use crate::chunk_system::generation_cache::Cache;
use crate::generation::height_limit::HeightLimitView;
use pumpkin_util::math::position::BlockPos;

fn get_chunk_index(cache: &Cache, chunk_x: i32, chunk_z: i32) -> Option<usize> {
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

pub fn get_block_light(cache: &Cache, pos: BlockPos) -> u8 {
    let chunk_x = pos.0.x >> 4;
    let chunk_z = pos.0.z >> 4;
    
    let Some(idx) = get_chunk_index(cache, chunk_x, chunk_z) else { return 0; };
    let chunk = &cache.chunks[idx];
    
    let Some(section_y) = get_section_y(cache, pos.0.y) else { return 0; };
    
    let x = (pos.0.x & 15) as usize;
    let y = (pos.0.y & 15) as usize;
    let z = (pos.0.z & 15) as usize;

    match chunk {
        Chunk::Level(c) => {
            let read = c.blocking_read();
            if section_y >= read.light_engine.block_light.len() { return 0; }
            read.light_engine.block_light[section_y].get(x, y, z)
        },
        Chunk::Proto(c) => {
             if section_y >= c.light.block_light.len() { return 0; }
             c.light.block_light[section_y].get(x, y, z)
        }
    }
}

pub fn set_block_light(cache: &mut Cache, pos: BlockPos, level: u8) {
    let chunk_x = pos.0.x >> 4;
    let chunk_z = pos.0.z >> 4;
    
    let Some(idx) = get_chunk_index(cache, chunk_x, chunk_z) else { return; };
    let Some(section_y) = get_section_y(cache, pos.0.y) else { return; };
    
    let x = (pos.0.x & 15) as usize;
    let y = (pos.0.y & 15) as usize;
    let z = (pos.0.z & 15) as usize;
    let chunk = &mut cache.chunks[idx];

    match chunk {
        Chunk::Level(c) => {
            let mut write = c.blocking_write();
            if section_y < write.light_engine.block_light.len() {
                write.light_engine.block_light[section_y].set(x, y, z, level);
                write.dirty = true;
            }
        },
        Chunk::Proto(c) => {
             if section_y < c.light.block_light.len() {
                 c.light.block_light[section_y].set(x, y, z, level);
             }
        }
    }
}

pub fn get_sky_light(cache: &Cache, pos: BlockPos) -> u8 {
    let chunk_x = pos.0.x >> 4;
    let chunk_z = pos.0.z >> 4;
    
    let Some(idx) = get_chunk_index(cache, chunk_x, chunk_z) else { return 0; };
    let chunk = &cache.chunks[idx];
    
    let Some(section_y) = get_section_y(cache, pos.0.y) else { return 0; };
    
    let x = (pos.0.x & 15) as usize;
    let y = (pos.0.y & 15) as usize;
    let z = (pos.0.z & 15) as usize;

    match chunk {
        Chunk::Level(c) => {
            let read = c.blocking_read();
            if section_y >= read.light_engine.sky_light.len() { return 0; }
            read.light_engine.sky_light[section_y].get(x, y, z)
        },
        Chunk::Proto(c) => {
             if section_y >= c.light.sky_light.len() { return 0; }
             c.light.sky_light[section_y].get(x, y, z)
        }
    }
}

pub fn set_sky_light(cache: &mut Cache, pos: BlockPos, level: u8) {
    let chunk_x = pos.0.x >> 4;
    let chunk_z = pos.0.z >> 4;
    
    let Some(idx) = get_chunk_index(cache, chunk_x, chunk_z) else { return; };
    let Some(section_y) = get_section_y(cache, pos.0.y) else { return; };
    
    let x = (pos.0.x & 15) as usize;
    let y = (pos.0.y & 15) as usize;
    let z = (pos.0.z & 15) as usize;
    let chunk = &mut cache.chunks[idx];

    match chunk {
        Chunk::Level(c) => {
            let mut write = c.blocking_write();
            if section_y < write.light_engine.sky_light.len() {
                 write.light_engine.sky_light[section_y].set(x, y, z, level);
                 write.dirty = true;
            }
        },
        Chunk::Proto(c) => {
             if section_y < c.light.sky_light.len() {
                 c.light.sky_light[section_y].set(x, y, z, level);
             }
        }
    }
}
