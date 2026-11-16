use crate::entity::player::Player;
use dashmap::DashMap;
use pumpkin_data::dimension::Dimension;
use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
use std::sync::{Arc, Mutex};

pub struct MapManager {
    pub maps: DashMap<i32, Arc<Mutex<MapData>>>,
}

impl Default for MapManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MapManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            maps: DashMap::new(),
        }
    }

    #[must_use]
    pub fn get_map(&self, id: i32) -> Option<Arc<Mutex<MapData>>> {
        self.maps.get(&id).map(|m| m.clone())
    }

    #[must_use]
    pub fn create_map(
        &self,
        id: i32,
        dimension: Dimension,
        x: i32,
        z: i32,
        scale: i8,
    ) -> Arc<Mutex<MapData>> {
        let map = Arc::new(Mutex::new(MapData::new(dimension, x, z, scale)));
        self.maps.insert(id, map.clone());
        map
    }
}

pub struct MapData {
    pub scale: i8,
    pub locked: bool,
    pub dimension: Dimension,
    pub center_x: i32,
    pub center_z: i32,
    pub colors: Box<[u8; 128 * 128]>,
    pub decorations: Vec<MapDecoration>,
    pub dirty: bool,
    pub fully_updated: bool,
}

impl MapData {
    #[must_use]
    pub fn new(dimension: Dimension, x: i32, z: i32, scale: i8) -> Self {
        Self {
            scale,
            locked: false,
            dimension,
            center_x: x,
            center_z: z,
            colors: Box::new([0; 128 * 128]),
            decorations: Vec::new(),
            dirty: true,
            fully_updated: false,
        }
    }

    pub fn set_color(&mut self, x: usize, z: usize, color: u8) {
        if x < 128 && z < 128 {
            let idx = z * 128 + x;
            if self.colors[idx] != color {
                self.colors[idx] = color;
                self.dirty = true;
            }
        }
    }

    pub fn update(&mut self, player: &Player) {
        let world = player.world();
        let scale = 1 << self.scale;
        let center_x = self.center_x;
        let center_z = self.center_z;

        let player_pos = player.position();
        let player_x = player_pos.x as i32;
        let player_z = player_pos.z as i32;

        let start_img_x = ((player_x - center_x) / scale + 64).clamp(0, 127) as usize;
        let start_img_z = ((player_z - center_z) / scale + 64).clamp(0, 127) as usize;

        let radius = 16;
        let (range_x, range_z) = if self.fully_updated {
            (
                (start_img_x.saturating_sub(radius))..(start_img_x + radius).min(128),
                (start_img_z.saturating_sub(radius))..(start_img_z + radius).min(128),
            )
        } else {
            self.fully_updated = true;
            (0..128, 0..128)
        };

        for img_x in range_x {
            let mut prev_y = -1;
            for img_z in range_z.clone() {
                let world_x = (img_x as i32 - 64) * scale + center_x;
                let world_z = (img_z as i32 - 64) * scale + center_z;

                let top_y = world.get_top_block(Vector2::new(world_x, world_z));
                let block = world.get_block(&BlockPos::new(world_x, top_y, world_z));

                let color_base = block.map_color;

                let mut brightness = 2; // Normal
                if prev_y != -1 {
                    if top_y > prev_y {
                        brightness = 3; // High
                    } else if top_y < prev_y {
                        brightness = 1; // Low
                    }
                }
                prev_y = top_y;

                let color = color_base * 4 + brightness;
                self.set_color(img_x, img_z, color);
            }
        }
    }
}

#[derive(Clone)]
pub struct MapDecoration {
    pub icon_type: i32,
    pub x: i8,
    pub z: i8,
    pub direction: i8,
    pub display_name: Option<String>,
}

/// Resolves `minecraft:map_post_processing` (`MapItem.onCraftedPostProcess`) on a crafted map stack.
pub fn resolve_map_post_processing(
    stack: &mut pumpkin_data::item_stack::ItemStack,
    server: &crate::server::Server,
) -> bool {
    use pumpkin_data::data_component::DataComponent;
    use pumpkin_data::data_component_impl::{MapIdImpl, MapPostProcessing, MapPostProcessingImpl};

    let Some(post) = stack
        .get_data_component::<MapPostProcessingImpl>()
        .and_then(|p| p.processing)
    else {
        return false;
    };
    let Some(orig_id) = stack.get_data_component::<MapIdImpl>().map(|m| m.id) else {
        return false;
    };
    let Some(orig_arc) = server.map_manager.get_map(orig_id) else {
        return false;
    };
    let Ok(orig) = orig_arc.try_lock() else {
        return false;
    };
    let new_id = server.next_map_id();
    match post {
        MapPostProcessing::Scale => {
            let new_scale = (orig.scale + 1).min(4);
            let _ = server.map_manager.create_map(
                new_id,
                orig.dimension.clone(),
                orig.center_x,
                orig.center_z,
                new_scale,
            );
        }
        MapPostProcessing::Lock => {
            let new_arc = server.map_manager.create_map(
                new_id,
                orig.dimension.clone(),
                orig.center_x,
                orig.center_z,
                orig.scale,
            );
            if let Ok(mut new_map) = new_arc.try_lock() {
                new_map.locked = true;
                *new_map.colors = *orig.colors;
                new_map.decorations.clone_from(&orig.decorations);
                new_map.dirty = true;
            }
        }
    }
    stack
        .patch
        .retain(|(id, _)| *id != DataComponent::MapPostProcessing);
    stack.set_data_component(MapIdImpl { id: new_id });
    true
}
