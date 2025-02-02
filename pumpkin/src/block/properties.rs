use std::collections::HashMap;

use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::block::{
    block_registry::{Block, State, BLOCKS},
    BlockDirection,
};

use crate::world::World;

#[derive(Clone, Debug)]
pub enum BlockProperty {
    Waterlogged(bool),
    Facing(Direction),
    Face(BlockFace),
    Powered(bool),
    SlabType(SlabPosition),
    StairShape(StairShape),
    Layers(u8),
    Half(BlockHalf), // Add other properties as needed
}

#[derive(Clone, Debug)]
pub enum BlockFace {
    Floor,
    Wall,
    Ceiling,
}

#[derive(Clone, Debug)]
pub enum BlockHalf {
    Top,
    Bottom,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SlabPosition {
    Top,
    Bottom,
    Double,
}

#[derive(Clone, Debug)]
pub enum StairShape {
    Straight,
    InnerLeft,
    InnerRight,
    OuterLeft,
    OuterRight,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

// TODO: We can automatically parse them ig
#[must_use]
pub fn get_property_key(property_name: &str) -> Option<BlockProperty> {
    match property_name {
        "waterlogged" => Some(BlockProperty::Waterlogged(false)),
        "facing" => Some(BlockProperty::Facing(Direction::North)),
        "type" => Some(BlockProperty::SlabType(SlabPosition::Top)),
        "shape" => Some(BlockProperty::StairShape(StairShape::Straight)),
        "half" => Some(BlockProperty::Half(BlockHalf::Bottom)),
        "powered" => Some(BlockProperty::Powered(false)),
        "layers" => Some(BlockProperty::Layers(1)),
        "face" => Some(BlockProperty::Face(BlockFace::Wall)),
        _ => None,
    }
}

#[must_use]
pub fn evaluate_layers(
    block: &Block,
    clicked_block: &Block,
    clicked_block_state: &State,
    face: BlockDirection,
    use_item_on: &SUseItemOn,
    properties: &BlockProperties,
    _other: bool,
) -> (String, bool) {
    let state =
        &properties.property_mappings[&(clicked_block_state.id - clicked_block.states[0].id)];
    for property in state {
        // Max
        if property == "layers8" {
            return (property.clone(), false);
        }

        if block.id == clicked_block.id {
            // bro its is so hacky :crying:
            let mut layer: u8 = property.replace("layers", "").parse().unwrap();
            // lets add a new layer
            layer += 1;
            return (format!("{}{}", "layers", layer), true);
        }
    }

    let y_pos = use_item_on.cursor_pos.y;
    if (y_pos > 0.5 && face != BlockDirection::Bottom) || face == BlockDirection::Top {
        return (format!("{}{}", "layers", "1"), false);
    }

    (format!("{}{}", "layers", "1"), false)
}

#[must_use]
pub fn evaluate_property_type(
    block: &Block,
    clicked_block: &Block,
    clicked_block_state: &State,
    face: BlockDirection,
    use_item_on: &SUseItemOn,
    properties: &BlockProperties,
    other: bool,
) -> (String, bool) {
    let state =
        &properties.property_mappings[&(clicked_block_state.id - clicked_block.states[0].id)];
    for property in state {
        if property == "typedouble" {
            return (property.clone(), false);
        }

        if block.id == clicked_block.id {
            if property == "typebottom" && face == BlockDirection::Top {
                return (format!("{}{}", "type", "double"), true);
            }
            if property == "typetop" && face == BlockDirection::Bottom {
                return (format!("{}{}", "type", "double"), true);
            }
            if !other {
                return (format!("{}{}", "type", "double"), true);
            }
        }
    }

    let y_pos = use_item_on.cursor_pos.y;
    if (y_pos > 0.5 && face != BlockDirection::Bottom) || face == BlockDirection::Top {
        return (format!("{}{}", "type", "top"), false);
    }

    (format!("{}{}", "type", "bottom"), false)
}

#[must_use]
pub fn evaluate_property_waterlogged(block: &Block) -> String {
    if block.name == "water" {
        return format!("{}{}", "waterlogged", "true");
    }
    format!("{}{}", "waterlogged", "false")
}

fn calculate_positions(player_direction: &Direction, block_pos: &BlockPos) -> (BlockPos, BlockPos) {
    match player_direction {
        Direction::North => (
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
        Direction::South => (
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
        Direction::East => (
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
        Direction::West => (
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

#[expect(clippy::implicit_hasher)]
pub async fn evaluate_property_shape(
    world: &World,
    block_pos: &BlockPos,
    face: &BlockDirection,
    use_item_on: &SUseItemOn,
    player_direction: &Direction,
    property_mappings: &HashMap<u16, Vec<String>>,
) -> String {
    let block_half = evaluate_property_half(*face, use_item_on);
    let (front_block_pos, back_block_pos) = calculate_positions(player_direction, block_pos);

    let front_block_and_state = world.get_block_and_block_state(&front_block_pos).await;
    let back_block_and_state = world.get_block_and_block_state(&back_block_pos).await;

    match front_block_and_state {
        Ok((block, state)) => {
            if block.name.ends_with("stairs") {
                log::debug!("Block in front is a stair block");

                let key = state.id - block.states[0].id;
                if let Some(properties) = property_mappings.get(&key) {
                    if properties.contains(&"shapestraight".to_owned())
                        && properties.contains(&block_half)
                    {
                        let is_facing_north = properties.contains(&"facingnorth".to_owned());
                        let is_facing_west = properties.contains(&"facingwest".to_owned());
                        let is_facing_south = properties.contains(&"facingsouth".to_owned());
                        let is_facing_east = properties.contains(&"facingeast".to_owned());

                        if (is_facing_north && *player_direction == Direction::West)
                            || (is_facing_west && *player_direction == Direction::South)
                            || (is_facing_south && *player_direction == Direction::East)
                            || (is_facing_east && *player_direction == Direction::North)
                        {
                            return "shapeouter_right".to_owned();
                        }

                        if (is_facing_north && *player_direction == Direction::East)
                            || (is_facing_west && *player_direction == Direction::North)
                            || (is_facing_south && *player_direction == Direction::West)
                            || (is_facing_east && *player_direction == Direction::South)
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
                if let Some(properties) = property_mappings.get(&key) {
                    if properties.contains(&"shapestraight".to_owned())
                        && properties.contains(&block_half)
                    {
                        let is_facing_north = properties.contains(&"facingnorth".to_owned());
                        let is_facing_west = properties.contains(&"facingwest".to_owned());
                        let is_facing_south = properties.contains(&"facingsouth".to_owned());
                        let is_facing_east = properties.contains(&"facingeast".to_owned());

                        if (is_facing_north && *player_direction == Direction::West)
                            || (is_facing_west && *player_direction == Direction::South)
                            || (is_facing_south && *player_direction == Direction::East)
                            || (is_facing_east && *player_direction == Direction::North)
                        {
                            return "shapeinner_right".to_owned();
                        }

                        if (is_facing_north && *player_direction == Direction::East)
                            || (is_facing_west && *player_direction == Direction::North)
                            || (is_facing_south && *player_direction == Direction::West)
                            || (is_facing_east && *player_direction == Direction::South)
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

#[must_use]
pub fn evaluate_property_facing(face: BlockDirection, player_direction: &Direction) -> String {
    let facing = match face {
        BlockDirection::North => "south",
        BlockDirection::South => "north",
        BlockDirection::East => "west",
        BlockDirection::West => "east",
        BlockDirection::Top | BlockDirection::Bottom => match player_direction {
            Direction::North => "north",
            Direction::South => "south",
            Direction::East => "east",
            Direction::West => "west",
        },
    };

    format!("facing{facing}")
}

#[must_use]
pub fn evaluate_property_block_face(dir: BlockDirection) -> String {
    let block_face = if dir == BlockDirection::Bottom || dir == BlockDirection::Top {
        if dir == BlockDirection::Top {
            BlockFace::Ceiling
        } else {
            BlockFace::Floor
        }
    } else {
        BlockFace::Wall
    };

    let facing = match block_face {
        BlockFace::Floor => "floor",
        BlockFace::Wall => "wall",
        BlockFace::Ceiling => "ceiling",
    };

    format!("face{facing}")
}

#[must_use]
pub fn evaluate_property_half(face: BlockDirection, use_item_on: &SUseItemOn) -> String {
    match face {
        BlockDirection::Bottom => format!("{}{}", "half", "bottom"),
        BlockDirection::Top => format!("{}{}", "half", "top"),
        _ => {
            if use_item_on.cursor_pos.y > 0.5 {
                format!("{}{}", "half", "top")
            } else {
                format!("{}{}", "half", "bottom")
            }
        }
    }
}

#[derive(Default)]
pub struct BlockPropertiesManager {
    properties_registry: HashMap<u16, BlockProperties>,
}

pub struct BlockProperties {
    // Mappings from property state strings -> offset
    state_mappings: HashMap<Vec<String>, u16>,
    // Mappings from offset -> property state strings
    property_mappings: HashMap<u16, Vec<String>>,
}

impl BlockPropertiesManager {
    pub fn build_properties_registry(&mut self) {
        for block in &BLOCKS.blocks {
            let properties = &block.properties;
            if properties.is_empty() {
                continue;
            }
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
            self.properties_registry.insert(
                block.id,
                BlockProperties {
                    state_mappings: forward_map,
                    property_mappings: reverse_map,
                },
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn get_state_data(
        &self,
        world: &World,
        block: &Block,
        face: &BlockDirection,
        block_pos: &BlockPos,
        use_item_on: &SUseItemOn,
        player_direction: &Direction,
        other: bool,
    ) -> (u16, bool) {
        if let Some(properties) = self.properties_registry.get(&block.id) {
            let mut hmap_key: Vec<String> = Vec::with_capacity(block.properties.len());
            let mut updateable = false;

            for raw_property in &block.properties {
                let property = get_property_key(raw_property.name.as_str());
                if let Some(property) = property {
                    let state = match property {
                        BlockProperty::SlabType(_) => {
                            let clicked_block = world.get_block(block_pos).await.unwrap();
                            let clicked_block_state =
                                world.get_block_state(block_pos).await.unwrap();
                            let (state, can_update) = evaluate_property_type(
                                block,
                                clicked_block,
                                clicked_block_state,
                                *face,
                                use_item_on,
                                properties,
                                other,
                            );
                            updateable = can_update;
                            state
                        }
                        BlockProperty::Waterlogged(_) => evaluate_property_waterlogged(block),
                        BlockProperty::Facing(_) => {
                            evaluate_property_facing(*face, player_direction)
                        }
                        BlockProperty::Half(_) => evaluate_property_half(*face, use_item_on),
                        BlockProperty::StairShape(_) => {
                            evaluate_property_shape(
                                world,
                                block_pos,
                                face,
                                use_item_on,
                                player_direction,
                                &properties.property_mappings,
                            )
                            .await
                        }
                        BlockProperty::Powered(_) => "poweredfalse".to_string(), // todo
                        BlockProperty::Face(_) => evaluate_property_block_face(*face),
                        BlockProperty::Layers(_) => {
                            let clicked_block = world.get_block(block_pos).await.unwrap();
                            let clicked_block_state =
                                world.get_block_state(block_pos).await.unwrap();
                            let (state, can_update) = evaluate_layers(
                                block,
                                clicked_block,
                                clicked_block_state,
                                *face,
                                use_item_on,
                                properties,
                                other,
                            );
                            updateable = can_update;
                            state
                        }
                    };
                    hmap_key.push(state.to_string());
                } else {
                    log::warn!("Unknown Block Property: {}", &raw_property.name);
                    // if one property is not found everything will not work
                    return (block.default_state_id, false);
                }
            }
            // Base state id plus offset
            let mapping = properties.state_mappings.get(&hmap_key);
            if let Some(mapping) = mapping {
                return (block.states[0].id + mapping, updateable);
            }
            log::error!("Failed to get Block Properties mapping for {}", block.name);
        }
        (block.default_state_id, false)
    }
}
