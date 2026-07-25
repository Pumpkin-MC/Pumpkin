use std::sync::OnceLock;
use std::sync::atomic::{AtomicU16, Ordering};

use crate::block_properties::BlockProperties;
use crate::{Block, BlockState, BlockStateId, Mirror, Rotation};

const UNINITIALIZED_STATE: u16 = u16::MAX;
const TRANSFORM_COUNT: usize = 12;

// Structure placement repeatedly transforms the same static states. Cache the resolved
// state IDs so each transform pays the property decoding cost at most once.
static TRANSFORMED_STATE_CACHE: OnceLock<[Box<[AtomicU16]>; TRANSFORM_COUNT]> = OnceLock::new();

#[must_use]
pub fn transform_block_state(
    id: BlockStateId,
    mirror: Mirror,
    rotation: Rotation,
) -> &'static BlockState {
    if mirror == Mirror::None && rotation == Rotation::None {
        return BlockState::from_id(id);
    }

    let cache = &transformed_state_cache()[transform_index(mirror, rotation)];
    let cached = cache[usize::from(id.as_u16())].load(Ordering::Relaxed);
    if cached != UNINITIALIZED_STATE {
        return BlockState::from_id(
            BlockStateId::new(cached).expect("cached block-state transform must be valid"),
        );
    }

    let transformed = transform_block_state_uncached(id, mirror, rotation);
    cache[usize::from(id.as_u16())].store(transformed.id.as_u16(), Ordering::Relaxed);
    transformed
}

fn transformed_state_cache() -> &'static [Box<[AtomicU16]>; TRANSFORM_COUNT] {
    TRANSFORMED_STATE_CACHE.get_or_init(|| {
        std::array::from_fn(|_| {
            (0..usize::from(BlockStateId::STATE_COUNT))
                .map(|_| AtomicU16::new(UNINITIALIZED_STATE))
                .collect()
        })
    })
}

const fn transform_index(mirror: Mirror, rotation: Rotation) -> usize {
    let mirror = match mirror {
        Mirror::None => 0,
        Mirror::LeftRight => 1,
        Mirror::FrontBack => 2,
    };
    let rotation = match rotation {
        Rotation::None => 0,
        Rotation::Clockwise90 => 1,
        Rotation::Rotate180 => 2,
        Rotation::CounterClockwise90 => 3,
    };
    mirror * 4 + rotation
}

fn transform_block_state_uncached(
    id: BlockStateId,
    mirror: Mirror,
    rotation: Rotation,
) -> &'static BlockState {
    let block = Block::from_state_id(id);
    let Some(properties) = block.properties(id) else {
        return BlockState::from_id(id);
    };
    let properties = properties.to_props();
    let transformed = transform_properties(&properties, mirror, rotation);
    BlockState::from_id(block.from_properties(&transformed).to_state_id(block))
}

fn transform_properties(
    properties: &[(&'static str, &'static str)],
    mirror: Mirror,
    rotation: Rotation,
) -> Vec<(&'static str, &'static str)> {
    let facing = properties
        .iter()
        .find_map(|(key, value)| (*key == "facing").then_some(*value));

    properties
        .iter()
        .map(|&(key, value)| {
            let transformed_key = transform_direction_key(key, mirror, rotation);
            let transformed_value = match key {
                "facing" => transform_direction(value, mirror, rotation),
                "axis" => transform_axis(value, rotation),
                "rotation" => transform_rotation(value, mirror, rotation),
                "shape" => transform_shape(value, facing, mirror, rotation),
                "hinge" if mirror != Mirror::None => flip_hinge(value),
                "orientation" => transform_orientation(value, mirror, rotation),
                _ => value,
            };
            (transformed_key, transformed_value)
        })
        .collect()
}

fn transform_direction_key(key: &'static str, mirror: Mirror, rotation: Rotation) -> &'static str {
    match key {
        "north" | "south" | "east" | "west" => transform_direction(key, mirror, rotation),
        _ => key,
    }
}

fn transform_direction(
    direction: &'static str,
    mirror: Mirror,
    rotation: Rotation,
) -> &'static str {
    let direction = match mirror {
        Mirror::None => direction,
        Mirror::LeftRight => match direction {
            "north" => "south",
            "south" => "north",
            _ => direction,
        },
        Mirror::FrontBack => match direction {
            "east" => "west",
            "west" => "east",
            _ => direction,
        },
    };

    match rotation {
        Rotation::None => direction,
        Rotation::Clockwise90 => match direction {
            "north" => "east",
            "east" => "south",
            "south" => "west",
            "west" => "north",
            _ => direction,
        },
        Rotation::Rotate180 => match direction {
            "north" => "south",
            "south" => "north",
            "east" => "west",
            "west" => "east",
            _ => direction,
        },
        Rotation::CounterClockwise90 => match direction {
            "north" => "west",
            "west" => "south",
            "south" => "east",
            "east" => "north",
            _ => direction,
        },
    }
}

fn transform_axis(axis: &'static str, rotation: Rotation) -> &'static str {
    match rotation {
        Rotation::Clockwise90 | Rotation::CounterClockwise90 => match axis {
            "x" => "z",
            "z" => "x",
            _ => axis,
        },
        Rotation::None | Rotation::Rotate180 => axis,
    }
}

fn transform_rotation(value: &'static str, mirror: Mirror, rotation: Rotation) -> &'static str {
    let Ok(value) = value.parse::<i32>() else {
        return value;
    };
    let value = mirror.mirror_block_rotation(value);
    let value = rotation.rotate_block_rotation(value);
    rotation_to_str(value)
}

fn rotation_to_str(rotation: i32) -> &'static str {
    match rotation.rem_euclid(16) {
        0 => "0",
        1 => "1",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        9 => "9",
        10 => "10",
        11 => "11",
        12 => "12",
        13 => "13",
        14 => "14",
        _ => "15",
    }
}

fn transform_shape(
    shape: &'static str,
    facing: Option<&'static str>,
    mirror: Mirror,
    rotation: Rotation,
) -> &'static str {
    let shape = mirror_rail_shape(shape, mirror);
    let shape = mirror_stair_shape(shape, facing, mirror);
    rotate_rail_shape(shape, rotation)
}

fn mirror_stair_shape(
    shape: &'static str,
    facing: Option<&'static str>,
    mirror: Mirror,
) -> &'static str {
    let on_mirrored_axis = match mirror {
        Mirror::LeftRight => matches!(facing, Some("north" | "south")),
        Mirror::FrontBack => matches!(facing, Some("east" | "west")),
        Mirror::None => false,
    };
    if !on_mirrored_axis {
        return shape;
    }

    match (mirror, shape) {
        (Mirror::LeftRight, "inner_left") => "inner_right",
        (Mirror::LeftRight, "inner_right") => "inner_left",
        (Mirror::LeftRight, "outer_left") => "outer_right",
        (Mirror::LeftRight, "outer_right") => "outer_left",
        (Mirror::FrontBack, "outer_left") => "outer_right",
        (Mirror::FrontBack, "outer_right") => "outer_left",
        _ => shape,
    }
}

fn rotate_rail_shape(shape: &'static str, rotation: Rotation) -> &'static str {
    match rotation {
        Rotation::None => shape,
        Rotation::Clockwise90 => match shape {
            "north_south" => "east_west",
            "east_west" => "north_south",
            "ascending_east" => "ascending_south",
            "ascending_west" => "ascending_north",
            "ascending_north" => "ascending_east",
            "ascending_south" => "ascending_west",
            "south_east" => "south_west",
            "south_west" => "north_west",
            "north_west" => "north_east",
            "north_east" => "south_east",
            _ => shape,
        },
        Rotation::Rotate180 => match shape {
            "ascending_east" => "ascending_west",
            "ascending_west" => "ascending_east",
            "ascending_north" => "ascending_south",
            "ascending_south" => "ascending_north",
            "south_east" => "north_west",
            "south_west" => "north_east",
            "north_west" => "south_east",
            "north_east" => "south_west",
            _ => shape,
        },
        Rotation::CounterClockwise90 => match shape {
            "north_south" => "east_west",
            "east_west" => "north_south",
            "ascending_east" => "ascending_north",
            "ascending_west" => "ascending_south",
            "ascending_north" => "ascending_west",
            "ascending_south" => "ascending_east",
            "south_east" => "north_east",
            "south_west" => "south_east",
            "north_west" => "south_west",
            "north_east" => "north_west",
            _ => shape,
        },
    }
}

fn mirror_rail_shape(shape: &'static str, mirror: Mirror) -> &'static str {
    match mirror {
        Mirror::None => shape,
        Mirror::LeftRight => match shape {
            "ascending_north" => "ascending_south",
            "ascending_south" => "ascending_north",
            "south_east" => "north_east",
            "south_west" => "north_west",
            "north_west" => "south_west",
            "north_east" => "south_east",
            _ => shape,
        },
        Mirror::FrontBack => match shape {
            "ascending_east" => "ascending_west",
            "ascending_west" => "ascending_east",
            "south_east" => "south_west",
            "south_west" => "south_east",
            "north_west" => "north_east",
            "north_east" => "north_west",
            _ => shape,
        },
    }
}

fn flip_hinge(value: &'static str) -> &'static str {
    match value {
        "left" => "right",
        "right" => "left",
        _ => value,
    }
}

fn transform_orientation(
    orientation: &'static str,
    mirror: Mirror,
    rotation: Rotation,
) -> &'static str {
    let (front, top) = match orientation {
        "down_east" => ("down", "east"),
        "down_north" => ("down", "north"),
        "down_south" => ("down", "south"),
        "down_west" => ("down", "west"),
        "up_east" => ("up", "east"),
        "up_north" => ("up", "north"),
        "up_south" => ("up", "south"),
        "up_west" => ("up", "west"),
        "west_up" => ("west", "up"),
        "east_up" => ("east", "up"),
        "north_up" => ("north", "up"),
        "south_up" => ("south", "up"),
        _ => return orientation,
    };

    match (
        transform_direction(front, mirror, rotation),
        transform_direction(top, mirror, rotation),
    ) {
        ("down", "east") => "down_east",
        ("down", "north") => "down_north",
        ("down", "south") => "down_south",
        ("down", "west") => "down_west",
        ("up", "east") => "up_east",
        ("up", "north") => "up_north",
        ("up", "south") => "up_south",
        ("up", "west") => "up_west",
        ("west", "up") => "west_up",
        ("east", "up") => "east_up",
        ("north", "up") => "north_up",
        ("south", "up") => "south_up",
        _ => orientation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with(block: &'static Block, properties: &[(&str, &str)]) -> &'static BlockState {
        BlockState::from_id(block.from_properties(properties).to_state_id(block))
    }

    fn property(state: &BlockState, name: &str) -> &'static str {
        Block::from_state_id(state.id)
            .properties(state.id)
            .expect("state has properties")
            .to_props()
            .into_iter()
            .find_map(|(key, value)| (key == name).then_some(value))
            .expect("property exists")
    }

    #[test]
    fn transforms_stair_facing_and_shape() {
        let state = state_with(
            &Block::OAK_STAIRS,
            &[("facing", "north"), ("shape", "inner_left")],
        );
        let transformed = transform_block_state(state.id, Mirror::LeftRight, Rotation::Clockwise90);

        assert_eq!(property(transformed, "facing"), "west");
        assert_eq!(property(transformed, "shape"), "inner_right");
    }

    #[test]
    fn transforms_directional_connection_properties() {
        let state = state_with(
            &Block::REDSTONE_WIRE,
            &[
                ("north", "side"),
                ("east", "up"),
                ("south", "none"),
                ("west", "none"),
            ],
        );
        let transformed = transform_block_state(state.id, Mirror::None, Rotation::Clockwise90);

        assert_eq!(property(transformed, "north"), "none");
        assert_eq!(property(transformed, "east"), "side");
        assert_eq!(property(transformed, "south"), "up");
        assert_eq!(property(transformed, "west"), "none");
    }

    #[test]
    fn transforms_rail_shapes() {
        let state = state_with(&Block::RAIL, &[("shape", "south_east")]);
        let transformed = transform_block_state(state.id, Mirror::FrontBack, Rotation::Clockwise90);

        assert_eq!(property(transformed, "shape"), "north_west");
    }

    #[test]
    fn transforms_front_and_top_orientations() {
        let state = state_with(&Block::JIGSAW, &[("orientation", "north_up")]);
        let transformed = transform_block_state(state.id, Mirror::LeftRight, Rotation::Clockwise90);

        assert_eq!(property(transformed, "orientation"), "west_up");
    }
}
