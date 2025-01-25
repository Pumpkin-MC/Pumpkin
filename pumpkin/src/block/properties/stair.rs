use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::block::block_registry::Block;
use pumpkin_world::{
    block::{block_registry::Property, BlockFace},
    entity::FacingDirection,
};

use crate::{
    block::block_properties_manager::{get_property_key, BlockBehavior, BlockProperty},
    world::World,
};

/// Global static for `StairBehavior`
pub static STAIRS_BEHAVIOR: OnceLock<Arc<StairBehavior>> = OnceLock::new();

/// Behavior for Stairs
pub struct StairBehavior {
    // Mappings from property state strings -> offset
    state_mappings: HashMap<Vec<String>, u16>,
    // Mappings from offset -> property state strings
    property_mappings: HashMap<u16, Vec<String>>,
}

impl StairBehavior {
    /// Initialize or return the existing `StairBehavior`
    pub fn get_or_init(properties: &[Property]) -> Arc<Self> {
        STAIRS_BEHAVIOR
            .get_or_init(|| Arc::new(Self::new(properties)))
            .clone()
    }

    /// Returns the global `StairBehavior` (must have been init once)
    pub fn get() -> Arc<Self> {
        STAIRS_BEHAVIOR
            .get()
            .expect("StairsBehavior not initialized")
            .clone()
    }

    /// Build up our forward/reverse property state maps
    pub fn new(properties: &[Property]) -> Self {
        let total_combinations: usize = properties.iter().map(|p| p.values.len()).product();

        let mut forward_map = HashMap::with_capacity(total_combinations);
        let mut reverse_map = HashMap::with_capacity(total_combinations);

        for i in 0..total_combinations {
            let mut current = i;
            let mut combination = Vec::with_capacity(properties.len());

            for property in properties.iter().rev() {
                let property_size = property.values.len();
                combination.push(current % property_size);
                current /= property_size;
            }

            combination.reverse();

            let key: Vec<String> = combination
                .iter()
                .enumerate()
                .map(|(prop_idx, &state_idx)| {
                    // Build "namevalue" strings, e.g. "facingnorth", "halfbottom", etc.
                    format!(
                        "{}{}",
                        properties[prop_idx].name, properties[prop_idx].values[state_idx]
                    )
                })
                .collect();

            forward_map.insert(key.clone(), i as u16);
            reverse_map.insert(i as u16, key);
        }

        Self {
            state_mappings: forward_map,
            property_mappings: reverse_map,
        }
    }

    fn calculate_positions(
        player_direction: FacingDirection,
        block_pos: BlockPos,
    ) -> (BlockPos, BlockPos) {
        match player_direction {
            FacingDirection::North => (
                BlockPos(Vector3::new(
                    block_pos.0.x,
                    block_pos.0.y,
                    block_pos.0.z - 1,
                )),
                BlockPos(Vector3::new(
                    block_pos.0.x,
                    block_pos.0.y,
                    block_pos.0.z + 1,
                )),
            ),
            FacingDirection::South => (
                BlockPos(Vector3::new(
                    block_pos.0.x,
                    block_pos.0.y,
                    block_pos.0.z + 1,
                )),
                BlockPos(Vector3::new(
                    block_pos.0.x,
                    block_pos.0.y,
                    block_pos.0.z - 1,
                )),
            ),
            FacingDirection::East => (
                BlockPos(Vector3::new(
                    block_pos.0.x + 1,
                    block_pos.0.y,
                    block_pos.0.z,
                )),
                BlockPos(Vector3::new(
                    block_pos.0.x - 1,
                    block_pos.0.y,
                    block_pos.0.z,
                )),
            ),
            FacingDirection::West => (
                BlockPos(Vector3::new(
                    block_pos.0.x - 1,
                    block_pos.0.y,
                    block_pos.0.z,
                )),
                BlockPos(Vector3::new(
                    block_pos.0.x + 1,
                    block_pos.0.y,
                    block_pos.0.z,
                )),
            ),
        }
    }

    pub async fn evaluate_property_shape(
        world: &World,
        block_pos: &BlockPos,
        face: &BlockFace,
        use_item_on: &SUseItemOn,
        player_direction: &FacingDirection,
    ) -> String {
        let block_half = Self::evaluate_property_half(*face, use_item_on);
        let (front_block_pos, back_block_pos) =
            Self::calculate_positions(*player_direction, *block_pos);

        let front_block_and_state = world.get_block_and_block_state(&front_block_pos).await;
        let back_block_and_state = world.get_block_and_block_state(&back_block_pos).await;

        match front_block_and_state {
            Ok((block, state)) => {
                if block.name.ends_with("stairs") {
                    log::debug!("Block in front is a stair block");

                    let key = state.id - block.states[0].id;
                    if let Some(properties) = Self::get().property_mappings.get(&key) {
                        if properties.contains(&"shapestraight".to_owned())
                            && properties.contains(&block_half)
                        {
                            let is_facing_north = properties.contains(&"facingnorth".to_owned());
                            let is_facing_west = properties.contains(&"facingwest".to_owned());
                            let is_facing_south = properties.contains(&"facingsouth".to_owned());
                            let is_facing_east = properties.contains(&"facingeast".to_owned());

                            if (is_facing_north && *player_direction == FacingDirection::West)
                                || (is_facing_west && *player_direction == FacingDirection::South)
                                || (is_facing_south && *player_direction == FacingDirection::East)
                                || (is_facing_east && *player_direction == FacingDirection::North)
                            {
                                return "shapeouter_right".to_owned();
                            }

                            if (is_facing_north && *player_direction == FacingDirection::East)
                                || (is_facing_west && *player_direction == FacingDirection::North)
                                || (is_facing_south && *player_direction == FacingDirection::West)
                                || (is_facing_east && *player_direction == FacingDirection::South)
                            {
                                return "shapeouter_left".to_owned();
                            }
                        }
                    }
                } else {
                    log::debug!("Block to the left is not a stair block");
                }
            }
            Err(_) => {
                log::debug!("There is no block to the left");
            }
        }

        match back_block_and_state {
            Ok((block, state)) => {
                if block.name.ends_with("stairs") {
                    log::debug!("Block in back is a stair block");

                    let key = state.id - block.states[0].id;
                    if let Some(properties) = Self::get().property_mappings.get(&key) {
                        if properties.contains(&"shapestraight".to_owned())
                            && properties.contains(&block_half)
                        {
                            let is_facing_north = properties.contains(&"facingnorth".to_owned());
                            let is_facing_west = properties.contains(&"facingwest".to_owned());
                            let is_facing_south = properties.contains(&"facingsouth".to_owned());
                            let is_facing_east = properties.contains(&"facingeast".to_owned());

                            if (is_facing_north && *player_direction == FacingDirection::West)
                                || (is_facing_west && *player_direction == FacingDirection::South)
                                || (is_facing_south && *player_direction == FacingDirection::East)
                                || (is_facing_east && *player_direction == FacingDirection::North)
                            {
                                return "shapeinner_right".to_owned();
                            }

                            if (is_facing_north && *player_direction == FacingDirection::East)
                                || (is_facing_west && *player_direction == FacingDirection::North)
                                || (is_facing_south && *player_direction == FacingDirection::West)
                                || (is_facing_east && *player_direction == FacingDirection::South)
                            {
                                return "shapeinner_left".to_owned();
                            }
                        }
                    }
                } else {
                    log::debug!("Block to the right is not a stair block");
                }
            }
            Err(_) => {
                log::debug!("There is no block to the right");
            }
        }

        // TODO: We currently don't notify adjacent stair blocks to update their shape after placement.
        //       We should implement a block update mechanism (e.g., tracking state changes and triggering
        //       a server-wide or chunk-level update) so that neighbors properly recalculate their shape.

        format!("{}{}", "shape", "straight")
    }

    pub fn evaluate_property_waterlogged(block: &Block) -> String {
        if block.name == "water" {
            return format!("{}{}", "waterlogged", "true");
        }
        format!("{}{}", "waterlogged", "false")
    }

    pub fn evaluate_property_facing(face: BlockFace, player_direction: FacingDirection) -> String {
        let facing = match face {
            BlockFace::North => "south",
            BlockFace::South => "north",
            BlockFace::East => "west",
            BlockFace::West => "east",
            BlockFace::Top | BlockFace::Bottom => match player_direction {
                FacingDirection::North => "north",
                FacingDirection::South => "south",
                FacingDirection::East => "east",
                FacingDirection::West => "west",
            },
        };

        format!("facing{facing}")
    }

    pub fn evaluate_property_half(face: BlockFace, use_item_on: &SUseItemOn) -> String {
        match face {
            BlockFace::Top => format!("{}{}", "half", "bottom"),
            BlockFace::Bottom => format!("{}{}", "half", "top"),
            _ => {
                if use_item_on.cursor_pos.y > 0.5 {
                    format!("{}{}", "half", "top")
                } else {
                    format!("{}{}", "half", "bottom")
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl BlockBehavior for StairBehavior {
    /// Given the block and environment, compute the correct state ID.
    async fn map_state_id(
        &self,
        world: &World,
        block: &Block,
        face: &BlockFace,
        block_pos: &BlockPos,
        use_item_on: &SUseItemOn,
        player_direction: &FacingDirection,
    ) -> u16 {
        let mut hmap_key: Vec<String> = Vec::with_capacity(block.properties.len());
        let stair_behaviour = Self::get();

        for property in &block.properties {
            let state = match get_property_key(property.name.as_str()).expect("Property not found")
            {
                BlockProperty::Facing(_) => {
                    Self::evaluate_property_facing(*face, *player_direction)
                }
                BlockProperty::Half(_) => Self::evaluate_property_half(*face, use_item_on),
                BlockProperty::StairShape(_) => {
                    Self::evaluate_property_shape(
                        world,
                        block_pos,
                        face,
                        use_item_on,
                        player_direction,
                    )
                    .await
                }
                BlockProperty::Waterlogged(_) => Self::evaluate_property_waterlogged(block),
                BlockProperty::SlabType(_) => panic!("SlabType BlockProperty invalid for Stairs"),
            };
            hmap_key.push(state);
        }

        block.states[0].id + stair_behaviour.state_mappings[&hmap_key]
    }

    async fn is_updateable(
        &self,
        _world: &World,
        _block: &Block,
        _face: &BlockFace,
        _block_pos: &BlockPos,
    ) -> bool {
        false
    }
}
