use crate::{
    block::{
        BlockBehaviour, CanPlaceAtArgs, CanUpdateAtArgs, GetStateForNeighborUpdateArgs,
        OnPlaceArgs, RandomTickArgs, UseWithItemArgs, registry::BlockActionResult,
    },
    entity::{EntityBase, player::Player},
    world::World,
};
use pumpkin_data::{
    Block, BlockDirection, BlockState, BlockStateId, FacingExt,
    block_properties::{BlockProperties, VineLikeProperties},
    block_rotation::{Mirror, Rotation},
    game_rules::{GameRule, GameRuleValue},
    item::Item,
    tag::{self, Taggable},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use rand::RngExt;

#[pumpkin_block("minecraft:vine")]
pub struct VineBlock;

/// Checks if a neighboring block can support a vine attached to the face towards it.
///
/// In vanilla, this is `MultifaceBlock.canAttachTo`: face is full/solid or block is leaves.
#[must_use]
pub fn is_acceptable_neighbour(
    block: &Block,
    block_state: &BlockState,
    direction_to_neighbour: BlockDirection,
) -> bool {
    block_state.is_side_solid(direction_to_neighbour.opposite())
        || block.has_tag(&tag::Block::MINECRAFT_LEAVES)
}

/// Checks if a face of the vine at `pos` can be supported by adjacent blocks or hanging from above.
#[must_use]
pub fn can_support_at_face(
    world: &dyn BlockAccessor,
    pos: &BlockPos,
    direction: BlockDirection,
) -> bool {
    if direction == BlockDirection::Down {
        return false;
    }

    let relative = pos.offset(direction.to_offset());
    let (neighbour_block, neighbour_state) = world.get_block_and_state(&relative);
    if is_acceptable_neighbour(neighbour_block, neighbour_state, direction) {
        return true;
    }

    if direction == BlockDirection::Up {
        return false;
    }

    let (above_block, above_state) = world.get_block_and_state(&pos.up());
    if above_block == &Block::VINE {
        let above_props = VineLikeProperties::from_state_id(above_state.id, above_block);
        has_face_property(&above_props, direction)
    } else {
        false
    }
}

/// Recalculates vine faces based on current support blocks and vines above.
#[must_use]
pub fn get_updated_state(
    mut props: VineLikeProperties,
    world: &dyn BlockAccessor,
    pos: &BlockPos,
    _block: &Block,
) -> VineLikeProperties {
    let above_pos = pos.up();
    if props.up {
        let (above_block, above_state) = world.get_block_and_state(&above_pos);
        props.up = is_acceptable_neighbour(above_block, above_state, BlockDirection::Up);
    }

    let mut above_props: Option<VineLikeProperties> = None;

    for direction in [
        BlockDirection::North,
        BlockDirection::South,
        BlockDirection::West,
        BlockDirection::East,
    ] {
        if has_face_property(&props, direction) {
            let mut can_support = can_support_at_face(world, pos, direction);
            if !can_support {
                let above_p = above_props.get_or_insert_with(|| {
                    let (above_block, above_state) = world.get_block_and_state(&above_pos);
                    if above_block == &Block::VINE {
                        VineLikeProperties::from_state_id(above_state.id, above_block)
                    } else {
                        VineLikeProperties {
                            north: false,
                            south: false,
                            east: false,
                            west: false,
                            up: false,
                        }
                    }
                });
                can_support = has_face_property(above_p, direction);
            }
            set_face_property(&mut props, direction, can_support);
        }
    }

    props
}

#[must_use]
pub const fn has_face_property(props: &VineLikeProperties, direction: BlockDirection) -> bool {
    match direction {
        BlockDirection::Up => props.up,
        BlockDirection::North => props.north,
        BlockDirection::South => props.south,
        BlockDirection::West => props.west,
        BlockDirection::East => props.east,
        BlockDirection::Down => false,
    }
}

pub const fn set_face_property(
    props: &mut VineLikeProperties,
    direction: BlockDirection,
    value: bool,
) {
    match direction {
        BlockDirection::Up => props.up = value,
        BlockDirection::North => props.north = value,
        BlockDirection::South => props.south = value,
        BlockDirection::West => props.west = value,
        BlockDirection::East => props.east = value,
        BlockDirection::Down => {}
    }
}

#[must_use]
pub const fn count_faces(props: &VineLikeProperties) -> usize {
    let mut count = 0;
    if props.up {
        count += 1;
    }
    if props.north {
        count += 1;
    }
    if props.south {
        count += 1;
    }
    if props.west {
        count += 1;
    }
    if props.east {
        count += 1;
    }
    count
}

#[must_use]
pub const fn has_horizontal_connection(props: &VineLikeProperties) -> bool {
    props.north || props.east || props.south || props.west
}

/// Checks if vine density in a 9x3x9 volume is below 5 (vanilla `canSpread`).
#[must_use]
pub fn can_spread(world: &World, pos: &BlockPos) -> bool {
    let min_x = pos.0.x - 4;
    let max_x = pos.0.x + 4;
    let min_y = pos.0.y - 1;
    let max_y = pos.0.y + 1;
    let min_z = pos.0.z - 4;
    let max_z = pos.0.z + 4;

    let mut count = 0;
    for x in min_x..=max_x {
        for y in min_y..=max_y {
            for z in min_z..=max_z {
                let check_pos = BlockPos::new(x, y, z);
                if world.get_block(&check_pos) == &Block::VINE {
                    count += 1;
                    if count >= 5 {
                        return false;
                    }
                }
            }
        }
    }
    true
}

#[must_use]
pub fn get_nearest_looking_directions(
    player: &Player,
    replace_clicked: bool,
    clicked_face: BlockDirection,
) -> [BlockDirection; 6] {
    let mut directions: [BlockDirection; 6] = {
        let fs = player.get_entity().get_entity_facing_order();
        [
            fs[0].to_block_direction(),
            fs[1].to_block_direction(),
            fs[2].to_block_direction(),
            fs[3].to_block_direction(),
            fs[4].to_block_direction(),
            fs[5].to_block_direction(),
        ]
    };

    if !replace_clicked {
        let target = clicked_face.opposite();
        let mut index = 0;
        while index < directions.len() && directions[index] != target {
            index += 1;
        }

        if index > 0 && index < directions.len() {
            directions.copy_within(0..index, 1);
            directions[0] = target;
        }
    }
    directions
}

impl BlockBehaviour for VineBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let (clicked_block, clicked_state_id) = args.world.get_block_and_state_id(args.position);
        let clicked_is_vine = clicked_block == &Block::VINE;
        let mut result = if clicked_is_vine {
            VineLikeProperties::from_state_id(clicked_state_id, args.block)
        } else {
            VineLikeProperties::default(args.block)
        };

        let nearest_directions =
            get_nearest_looking_directions(args.player, clicked_is_vine, args.direction);

        for direction in nearest_directions {
            if direction != BlockDirection::Down {
                let face_occupied = clicked_is_vine && has_face_property(&result, direction);
                if !face_occupied && can_support_at_face(args.world, args.position, direction) {
                    set_face_property(&mut result, direction, true);
                    return result.to_state_id(args.block);
                }
            }
        }

        if clicked_is_vine && count_faces(&result) > 0 {
            result.to_state_id(args.block)
        } else {
            Block::AIR.default_state.id
        }
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let (clicked_block, clicked_state) = args.block_accessor.get_block_and_state(args.position);
        let clicked_is_vine = clicked_block == &Block::VINE;
        let result = if clicked_is_vine {
            VineLikeProperties::from_state_id(clicked_state.id, args.block)
        } else {
            VineLikeProperties::default(args.block)
        };

        if clicked_is_vine && count_faces(&result) >= 5 {
            return false;
        }

        let nearest_directions = args.player.map_or_else(
            || {
                args.direction.map_or(
                    [
                        BlockDirection::North,
                        BlockDirection::South,
                        BlockDirection::West,
                        BlockDirection::East,
                        BlockDirection::Up,
                        BlockDirection::Down,
                    ],
                    |click_dir| {
                        [
                            click_dir.opposite(),
                            BlockDirection::Up,
                            BlockDirection::North,
                            BlockDirection::South,
                            BlockDirection::West,
                            BlockDirection::East,
                        ]
                    },
                )
            },
            |player| {
                let click_dir = args.direction.unwrap_or(BlockDirection::Down);
                get_nearest_looking_directions(player, clicked_is_vine, click_dir)
            },
        );

        for direction in nearest_directions {
            if direction != BlockDirection::Down {
                let face_occupied = clicked_is_vine && has_face_property(&result, direction);
                if !face_occupied
                    && can_support_at_face(args.block_accessor, args.position, direction)
                {
                    return true;
                }
            }
        }

        clicked_is_vine && count_faces(&result) > 0
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.direction == BlockDirection::Down {
            return args.state_id;
        }

        let updated_props = get_updated_state(
            VineLikeProperties::from_state_id(args.state_id, args.block),
            args.world,
            args.position,
            args.block,
        );
        if count_faces(&updated_props) == 0 {
            Block::AIR.default_state.id
        } else {
            updated_props.to_state_id(args.block)
        }
    }

    fn can_update_at(&self, args: CanUpdateAtArgs<'_>) -> bool {
        let (clicked_block, clicked_state) = args.world.get_block_and_state(args.position);
        if clicked_block != &Block::VINE {
            return false;
        }
        let props = VineLikeProperties::from_state_id(clicked_state.id, clicked_block);
        count_faces(&props) < 5
    }

    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        {
            if args.item_stack.item.id != Item::VINE.id {
                return BlockActionResult::Pass;
            }

            let state = args.world.get_block_state(args.position);
            let mut props = VineLikeProperties::from_state_id(state.id, args.block);
            if count_faces(&props) >= 5 {
                return BlockActionResult::Pass;
            }

            let nearest_directions =
                get_nearest_looking_directions(args.player, true, BlockDirection::Down);

            for direction in nearest_directions {
                if direction != BlockDirection::Down {
                    let face_occupied = has_face_property(&props, direction);
                    if !face_occupied
                        && can_support_at_face(&**args.world, args.position, direction)
                    {
                        set_face_property(&mut props, direction, true);
                        args.world.set_block_state(
                            args.position,
                            props.to_state_id(args.block),
                            BlockFlags::NOTIFY_ALL,
                        );
                        return BlockActionResult::Consume;
                    }
                }
            }

            BlockActionResult::Pass
        }
    }

    #[expect(clippy::too_many_lines)]
    fn random_tick(&self, args: RandomTickArgs<'_>) {
        let do_spread = matches!(
            args.world
                .level_info
                .load()
                .game_rules
                .get(&GameRule::SpreadVines),
            GameRuleValue::Bool(true)
        );
        if !do_spread {
            return;
        }

        if rand::rng().random_range(0..4) != 0 {
            return;
        }

        let test_direction = BlockDirection::all()[rand::rng().random_range(0..6)];
        let above_pos = args.position.up();
        let state_id = args.world.get_block_state_id(args.position);
        let state_props = VineLikeProperties::from_state_id(state_id, args.block);

        if test_direction.is_horizontal() && !has_face_property(&state_props, test_direction) {
            if can_spread(args.world, args.position) {
                let test_pos = args.position.offset(test_direction.to_offset());
                let (edge_block, edge_state) = args.world.get_block_and_state(&test_pos);
                if edge_block.default_state.is_air() {
                    let cw_direction = test_direction.rotate_clockwise();
                    let ccw_direction = test_direction.rotate_counter_clockwise();
                    let cw_has_connecting_face = has_face_property(&state_props, cw_direction);
                    let ccw_has_connecting_face = has_face_property(&state_props, ccw_direction);
                    let cw_test_pos = test_pos.offset(cw_direction.to_offset());
                    let ccw_test_pos = test_pos.offset(ccw_direction.to_offset());

                    let (cw_test_block, cw_test_state) =
                        args.world.get_block_and_state(&cw_test_pos);
                    let (ccw_test_block, ccw_test_state) =
                        args.world.get_block_and_state(&ccw_test_pos);

                    if cw_has_connecting_face
                        && is_acceptable_neighbour(cw_test_block, cw_test_state, cw_direction)
                    {
                        let mut new_props = VineLikeProperties::default(args.block);
                        set_face_property(&mut new_props, cw_direction, true);
                        args.world.set_block_state(
                            &test_pos,
                            new_props.to_state_id(args.block),
                            BlockFlags::NOTIFY_ALL,
                        );
                    } else if ccw_has_connecting_face
                        && is_acceptable_neighbour(ccw_test_block, ccw_test_state, ccw_direction)
                    {
                        let mut new_props = VineLikeProperties::default(args.block);
                        set_face_property(&mut new_props, ccw_direction, true);
                        args.world.set_block_state(
                            &test_pos,
                            new_props.to_state_id(args.block),
                            BlockFlags::NOTIFY_ALL,
                        );
                    } else {
                        let opposite = test_direction.opposite();
                        let (cw_support_block, cw_support_state) = args
                            .world
                            .get_block_and_state(&args.position.offset(cw_direction.to_offset()));
                        let (ccw_support_block, ccw_support_state) = args
                            .world
                            .get_block_and_state(&args.position.offset(ccw_direction.to_offset()));

                        if cw_has_connecting_face
                            && cw_test_block.default_state.is_air()
                            && is_acceptable_neighbour(cw_support_block, cw_support_state, opposite)
                        {
                            let mut new_props = VineLikeProperties::default(args.block);
                            set_face_property(&mut new_props, opposite, true);
                            args.world.set_block_state(
                                &cw_test_pos,
                                new_props.to_state_id(args.block),
                                BlockFlags::NOTIFY_ALL,
                            );
                        } else if ccw_has_connecting_face
                            && ccw_test_block.default_state.is_air()
                            && is_acceptable_neighbour(
                                ccw_support_block,
                                ccw_support_state,
                                opposite,
                            )
                        {
                            let mut new_props = VineLikeProperties::default(args.block);
                            set_face_property(&mut new_props, opposite, true);
                            args.world.set_block_state(
                                &ccw_test_pos,
                                new_props.to_state_id(args.block),
                                BlockFlags::NOTIFY_ALL,
                            );
                        } else if rand::rng().random_range(0.0..1.0f32) < 0.05
                            && is_acceptable_neighbour(
                                args.world.get_block(&test_pos.up()),
                                args.world.get_block_state(&test_pos.up()),
                                BlockDirection::Up,
                            )
                        {
                            let mut new_props = VineLikeProperties::default(args.block);
                            new_props.up = true;
                            args.world.set_block_state(
                                &test_pos,
                                new_props.to_state_id(args.block),
                                BlockFlags::NOTIFY_ALL,
                            );
                        }
                    }
                } else if is_acceptable_neighbour(edge_block, edge_state, test_direction) {
                    let mut new_props = state_props;
                    set_face_property(&mut new_props, test_direction, true);
                    args.world.set_block_state(
                        args.position,
                        new_props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    );
                }
            }
        } else if test_direction == BlockDirection::Up
            && args.position.0.y < args.world.dimension.min_y + args.world.dimension.height - 1
        {
            if can_support_at_face(&**args.world, args.position, test_direction) {
                let mut new_props = state_props;
                new_props.up = true;
                args.world.set_block_state(
                    args.position,
                    new_props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                );
                return;
            }

            if args.world.get_block(&above_pos).default_state.is_air() {
                if !can_spread(args.world, args.position) {
                    return;
                }

                let mut above_props = state_props;
                for direction in [
                    BlockDirection::North,
                    BlockDirection::South,
                    BlockDirection::West,
                    BlockDirection::East,
                ] {
                    let rel_pos = above_pos.offset(direction.to_offset());
                    let (rel_block, rel_state) = args.world.get_block_and_state(&rel_pos);
                    if rand::rng().random_range(0..2) == 0
                        || !is_acceptable_neighbour(rel_block, rel_state, direction)
                    {
                        set_face_property(&mut above_props, direction, false);
                    }
                }

                if has_horizontal_connection(&above_props) {
                    args.world.set_block_state(
                        &above_pos,
                        above_props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    );
                }
            }
        } else if args.position.0.y > args.world.dimension.min_y {
            let below_pos = args.position.down();
            let (below_block, below_state) = args.world.get_block_and_state(&below_pos);
            if below_block.default_state.is_air() || below_block == &Block::VINE {
                let before_props = if below_block.default_state.is_air() {
                    VineLikeProperties::default(args.block)
                } else {
                    VineLikeProperties::from_state_id(below_state.id, below_block)
                };

                let mut after_props = before_props;
                for direction in [
                    BlockDirection::North,
                    BlockDirection::South,
                    BlockDirection::West,
                    BlockDirection::East,
                ] {
                    if rand::rng().random_range(0..2) == 0
                        && has_face_property(&state_props, direction)
                    {
                        set_face_property(&mut after_props, direction, true);
                    }
                }

                if before_props != after_props && has_horizontal_connection(&after_props) {
                    args.world.set_block_state(
                        &below_pos,
                        after_props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    );
                }
            }
        }
    }

    fn rotate(
        &self,
        block: &Block,
        state_id: BlockStateId,
        rotation: Rotation,
    ) -> &'static BlockState {
        let props = VineLikeProperties::from_state_id(state_id, block);
        let mut rotated_props = props;
        match rotation {
            Rotation::Rotate180 => {
                rotated_props.north = props.south;
                rotated_props.east = props.west;
                rotated_props.south = props.north;
                rotated_props.west = props.east;
            }
            Rotation::CounterClockwise90 => {
                rotated_props.north = props.east;
                rotated_props.east = props.south;
                rotated_props.south = props.west;
                rotated_props.west = props.north;
            }
            Rotation::Clockwise90 => {
                rotated_props.north = props.west;
                rotated_props.east = props.north;
                rotated_props.south = props.east;
                rotated_props.west = props.south;
            }
            Rotation::None => {}
        }
        BlockState::from_id(rotated_props.to_state_id(block))
    }

    fn mirror(&self, block: &Block, state_id: BlockStateId, mirror: Mirror) -> &'static BlockState {
        let props = VineLikeProperties::from_state_id(state_id, block);
        let mut mirrored_props = props;
        match mirror {
            Mirror::LeftRight => {
                mirrored_props.north = props.south;
                mirrored_props.south = props.north;
            }
            Mirror::FrontBack => {
                mirrored_props.east = props.west;
                mirrored_props.west = props.east;
            }
            Mirror::None => {}
        }
        BlockState::from_id(mirrored_props.to_state_id(block))
    }
}
