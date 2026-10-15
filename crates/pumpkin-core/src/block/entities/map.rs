use std::any::Any;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{CMapItemData, MapPatch};
use pumpkin_util::math::position::BlockPos;

use crate::block::entities::BlockEntity;

pub const MAP_BLOCK_ENTITY_ID: &str = "minecraft:map";

pub struct MapBlockEntity {
    position: BlockPos,
    map_id: AtomicI32,
    colors: Mutex<Box<[u8; 128 * 128]>>,
    dirty: AtomicBool,
}

impl MapBlockEntity {
    #[must_use]
    pub fn new(position: BlockPos, map_id: i32) -> Self {
        Self {
            position,
            map_id: AtomicI32::new(map_id),
            colors: Mutex::new(Box::new([0; 128 * 128])),
            dirty: AtomicBool::new(true),
        }
    }

    #[must_use]
    pub fn get_map_id(&self) -> i32 {
        self.map_id.load(Ordering::Relaxed)
    }

    pub fn set_map_id(&self, map_id: i32) {
        self.map_id.store(map_id, Ordering::Relaxed);
        self.dirty.store(true, Ordering::Relaxed);
    }

    pub fn get_colors(&self) -> Vec<u8> {
        let colors = self
            .colors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        colors.to_vec()
    }

    pub fn set_colors(&self, new_colors: &[u8]) {
        let mut colors = self
            .colors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if new_colors.len() == 128 * 128 * 3 {
            // RGB -> map color byte conversion
            for i in 0..(128 * 128) {
                let r = new_colors[i * 3];
                let g = new_colors[i * 3 + 1];
                let b = new_colors[i * 3 + 2];
                colors[i] = rgb_to_map_color(r, g, b);
            }
        } else if new_colors.len() == 128 * 128 * 4 {
            // RGBA -> map color byte conversion
            for i in 0..(128 * 128) {
                let r = new_colors[i * 4];
                let g = new_colors[i * 4 + 1];
                let b = new_colors[i * 4 + 2];
                colors[i] = rgb_to_map_color(r, g, b);
            }
        } else {
            let len = new_colors.len().min(128 * 128);
            colors[..len].copy_from_slice(&new_colors[..len]);
        }
        self.dirty.store(true, Ordering::Relaxed);
    }

    pub fn set_pixel(&self, x: usize, y: usize, color: u8) {
        if x < 128 && y < 128 {
            let mut colors = self
                .colors
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            colors[y * 128 + x] = color;
            self.dirty.store(true, Ordering::Relaxed);
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        if x < 128 && y < 128 {
            let colors = self
                .colors
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            colors[y * 128 + x]
        } else {
            0
        }
    }

    pub fn stream_frame(&self, frame_data: &[u8], server: Option<&crate::server::Server>) {
        self.set_colors(frame_data);
        if let Some(server) = server {
            self.broadcast_map_data(server);
        }
    }

    pub fn broadcast_map_data(&self, server: &crate::server::Server) {
        let map_id = self.get_map_id();
        let colors = self
            .colors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let data = MapPatch {
            columns: 128,
            rows: 128,
            x: 0,
            z: 0,
            data: &colors[..],
        };
        let packet = CMapItemData {
            map_id: VarInt(map_id),
            scale: 0,
            tracking_position: false,
            locked: false,
            icons: Some(&[]),
            data: Some(data),
        };
        server.broadcast_packet_all(&packet);
        self.dirty.store(false, Ordering::Relaxed);
    }
}

impl BlockEntity for MapBlockEntity {
    fn resource_location(&self) -> &'static str {
        MAP_BLOCK_ENTITY_ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let map_id = nbt.get_int("MapId").unwrap_or(0);
        let entity = Self::new(position, map_id);
        if let Some(colors_nbt) = nbt.get("Colors").and_then(|tag| tag.extract_byte_array()) {
            let mut colors = entity
                .colors
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let len = colors_nbt.len().min(128 * 128);
            for (i, &b) in colors_nbt.iter().take(len).enumerate() {
                colors[i] = b as u8;
            }
        }
        entity
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("MapId", self.get_map_id());
        let colors = self
            .colors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let colors_i8: Vec<i8> = colors.iter().map(|&b| b as i8).collect();
        nbt.put("Colors", NbtTag::ByteArray(colors_i8.into_boxed_slice()));
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Helper function to convert an RGB value to the closest Minecraft map color index.
fn rgb_to_map_color(r: u8, g: u8, b: u8) -> u8 {
    use pumpkin_data::map_color::MapColor;

    // Brightness multipliers: 180 (low), 220 (normal), 255 (high), 135 (lowest)
    const BRIGHTNESS: &[(u8, u32)] = &[(0, 180), (1, 220), (2, 255), (3, 135)];

    let mut best_color: u8 = 0;
    let mut min_diff: u32 = u32::MAX;

    let r32 = u32::from(r);
    let g32 = u32::from(g);
    let b32 = u32::from(b);

    for map_color in MapColor::ALL {
        if map_color.id == 0 {
            continue;
        }
        let (cr, cg, cb) = map_color.rgb;
        for &(b_idx, mult) in BRIGHTNESS {
            let pr = (u32::from(cr) * mult) / 255;
            let pg = (u32::from(cg) * mult) / 255;
            let pb = (u32::from(cb) * mult) / 255;

            let dr = r32.abs_diff(pr);
            let dg = g32.abs_diff(pg);
            let db = b32.abs_diff(pb);

            let diff = dr * dr + dg * dg + db * db;
            if diff < min_diff {
                min_diff = diff;
                best_color = map_color.id * 4 + b_idx;
            }
        }
    }

    best_color
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::math::position::BlockPos;

    #[test]
    fn map_block_entity_pixels_and_colors() {
        let pos = BlockPos::new(10, 64, -5);
        let map_be = MapBlockEntity::new(pos, 42);

        assert_eq!(map_be.get_map_id(), 42);
        assert_eq!(map_be.get_position(), pos);

        map_be.set_map_id(100);
        assert_eq!(map_be.get_map_id(), 100);

        map_be.set_pixel(10, 20, 15);
        assert_eq!(map_be.get_pixel(10, 20), 15);

        let mut custom_frame = vec![0u8; 128 * 128];
        custom_frame[0] = 5;
        custom_frame[127] = 20;
        map_be.set_colors(&custom_frame);

        assert_eq!(map_be.get_pixel(0, 0), 5);
        assert_eq!(map_be.get_pixel(127, 0), 20);

        // Test RGB conversion frame
        let mut rgb_frame = vec![0u8; 128 * 128 * 3];
        // Red pixel at (0, 0)
        rgb_frame[0] = 255;
        rgb_frame[1] = 0;
        rgb_frame[2] = 0;
        map_be.set_colors(&rgb_frame);
        assert!(map_be.get_pixel(0, 0) > 0);
    }
}
