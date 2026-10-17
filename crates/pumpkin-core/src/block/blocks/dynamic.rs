use crate::block::{BlockBehaviour, GetStateForNeighborUpdateArgs, OnPlaceArgs};
use crate::entity::EntityBase;
use pumpkin_data::block_registry::PlacementContext;
use pumpkin_data::{Block, BlockStateId, FacingExt, HorizontalFacingExt};

/// Behaviour of every block that a plugin registers, driven by its registration.
pub struct DynamicBlock;

fn state_id(block: &Block, state_index: u16) -> BlockStateId {
    block
        .states
        .get(usize::from(state_index))
        .map_or(block.default_state.id, |state| state.id)
}

impl BlockBehaviour for DynamicBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let Some(info) = args.block.dynamic_info() else {
            return args.block.default_state.id;
        };
        let entity = args.player.get_entity();
        let context = PlacementContext {
            // `direction` points from the placed block to the clicked one.
            clicked_face: args.direction.opposite(),
            horizontal_facing: entity.get_horizontal_facing().to_block_direction(),
            looking_direction: entity.get_facing().to_block_direction(),
            pitch: entity.pitch.load(),
            cursor_y: args.use_item_on.cursor_pos.y,
            in_water: args.replacing.water_source(),
            sneaking: entity.is_sneaking(),
        };
        // Placement rules first, then connect rules from the neighbours.
        let placed = info.state_for_placement(info.default_state_index(), &context);
        let index = info.state_for_neighbours(placed, |direction| {
            args.world
                .get_block(&args.position.offset(direction.to_offset()))
        });
        state_id(args.block, index)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let Some(info) = args.block.dynamic_info() else {
            return args.state_id;
        };
        let Some(index) = info.state_index(args.state_id) else {
            return args.state_id;
        };
        let neighbour = args.world.get_block(args.neighbor_position);
        state_id(
            args.block,
            info.update_connection(index, args.direction, neighbour),
        )
    }
}
