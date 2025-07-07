use async_trait::async_trait;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;

use crate::block::pumpkin_block::CanPlaceAtArgs;
use crate::block::pumpkin_block::OnNeighborUpdateArgs;
use crate::block::pumpkin_block::OnPlaceArgs;
use crate::block::pumpkin_block::PlacedArgs;
use crate::block::pumpkin_block::PumpkinBlock;

use super::super::block_receives_redstone_power;
use super::RailProperties;
use super::common::{
    can_place_rail_at, compute_placed_rail_shape, rail_placement_is_valid,
    update_flanking_rails_shape,
};

// TODO: Fix redstone rail power extension behavior
// Currently, redstone sources (like redstone torch) can incorrectly extend rail power
// when placed at any powered rail position. In Minecraft, power should only extend
// when a redstone source is placed at the LAST powered rail or at an unpowered rail.
// 
// Example of INCORRECT current behavior:
// redstone_torch [powered_rail×9] [unpowered_rail×3]
// If redstone torch is placed at 6th powered rail → power extends (WRONG)
//
// Example of CORRECT Minecraft behavior:
// redstone_torch [powered_rail×9] [unpowered_rail×3]
// Power should only extend when redstone source is at 9th rail (last powered) or unpowered rail

#[pumpkin_block("minecraft:activator_rail")]
pub struct ActivatorRailBlock;

#[async_trait]
impl PumpkinBlock for ActivatorRailBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut rail_props = RailProperties::default(args.block);
        let player_facing = args.player.living_entity.entity.get_horizontal_facing();

        rail_props.set_waterlogged(args.replacing.water_source());
        rail_props.set_straight_shape(
            compute_placed_rail_shape(args.world, args.position, player_facing).await,
        );

        rail_props.to_state_id(args.block)
    }

    async fn placed(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: BlockStateId,
        block_pos: &BlockPos,
        _old_state_id: BlockStateId,
        _notify: bool,
    ) {
        update_flanking_rails_shape(world, block, state_id, block_pos).await;

        self.update_powered_state(world, block, block_pos).await;

        let final_state_id = world.get_block_state_id(block_pos).await;
        let rail_props = RailProperties::new(final_state_id, block);

        self.update_connected_rails(world, block_pos, &rail_props, true, 0)
            .await;
        self.update_connected_rails(world, block_pos, &rail_props, false, 0)
            .await;

        for direction in rail_props.directions() {
            let neighbor_pos = block_pos.offset(direction.to_offset());

            if let Some(neighbor_rail) = self.find_rail_at_position(world, &neighbor_pos).await {
                self.update_powered_state_internal(world, &neighbor_rail.0, &neighbor_pos, false)
                    .await;
                self.update_connected_rails(world, &neighbor_pos, &neighbor_rail.1, true, 0)
                    .await;
                self.update_connected_rails(world, &neighbor_pos, &neighbor_rail.1, false, 0)
                    .await;
            }

            let up_pos = neighbor_pos.up();
            if let Some(neighbor_rail) = self.find_rail_at_position(world, &up_pos).await {
                self.update_powered_state_internal(world, &neighbor_rail.0, &up_pos, false)
                    .await;
                self.update_connected_rails(world, &up_pos, &neighbor_rail.1, true, 0)
                    .await;
                self.update_connected_rails(world, &up_pos, &neighbor_rail.1, false, 0)
                    .await;
            }

            let down_pos = neighbor_pos.down();
            if let Some(neighbor_rail) = self.find_rail_at_position(world, &down_pos).await {
                self.update_powered_state_internal(world, &neighbor_rail.0, &down_pos, false)
                    .await;
                self.update_connected_rails(world, &down_pos, &neighbor_rail.1, true, 0)
                    .await;
                self.update_connected_rails(world, &down_pos, &neighbor_rail.1, false, 0)
                    .await;
            }
        }
    }

    async fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        if !rail_placement_is_valid(args.world, args.block, args.position).await {
            args.world
                .break_block(args.position, None, BlockFlags::NOTIFY_ALL)
                .await;
            return;
        }

        self.update_powered_state(world, block, block_pos).await;

        let state_id = world.get_block_state_id(block_pos).await;
        let rail_props = RailProperties::new(state_id, block);

        self.update_connected_rails(world, block_pos, &rail_props, true, 0)
            .await;
        self.update_connected_rails(world, block_pos, &rail_props, false, 0)
            .await;
    }

    async fn on_state_replaced(
        &self,
        world: &Arc<World>,
        block: &Block,
        block_pos: BlockPos,
        state_id: BlockStateId,
        _moved: bool,
    ) {
        let rail_props = RailProperties::new(state_id, block);

        if rail_props.shape().is_ascending() {
            world.update_neighbor(&block_pos.up(), block).await;
        }

        world.update_neighbor(&block_pos, block).await;
        world.update_neighbor(&block_pos.down(), block).await;

        let directions = rail_props.directions();
        for direction in directions {
            let neighbor_pos = block_pos.offset(direction.to_offset());

            if let Some(neighbor_rail) = self.find_rail_at_position(world, &neighbor_pos).await {
                self.update_powered_state(world, &neighbor_rail.0, &neighbor_pos)
                    .await;
                self.update_connected_rails(world, &neighbor_pos, &neighbor_rail.1, true, 0)
                    .await;
                self.update_connected_rails(world, &neighbor_pos, &neighbor_rail.1, false, 0)
                    .await;
            }

            let up_pos = neighbor_pos.up();
            if let Some(neighbor_rail) = self.find_rail_at_position(world, &up_pos).await {
                self.update_powered_state(world, &neighbor_rail.0, &up_pos)
                    .await;
                self.update_connected_rails(world, &up_pos, &neighbor_rail.1, true, 0)
                    .await;
                self.update_connected_rails(world, &up_pos, &neighbor_rail.1, false, 0)
                    .await;
            }

            let down_pos = neighbor_pos.down();
            if let Some(neighbor_rail) = self.find_rail_at_position(world, &down_pos).await {
                self.update_powered_state(world, &neighbor_rail.0, &down_pos)
                    .await;
                self.update_connected_rails(world, &down_pos, &neighbor_rail.1, true, 0)
                    .await;
                self.update_connected_rails(world, &down_pos, &neighbor_rail.1, false, 0)
                    .await;
            }
        }
    }

    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_rail_at(args.block_accessor, args.position).await
    }

    async fn emits_redstone_power(
        &self,
        _block: &Block,
        _state: &pumpkin_data::BlockState,
        _direction: BlockDirection,
    ) -> bool {
        false
    }

    async fn get_weak_redstone_power(
        &self,
        _block: &Block,
        _world: &World,
        _pos: &BlockPos,
        _state: &pumpkin_data::BlockState,
        _direction: BlockDirection,
    ) -> u8 {
        0
    }
}

impl ActivatorRailBlock {
    async fn is_powered_by_other_rails(
        &self,
        world: &World,
        pos: &BlockPos,
        state: &RailProperties,
        direction: bool,
        distance: u8,
    ) -> bool {
        if distance >= 8 {
            return false;
        }

        let mut x = pos.0.x;
        let mut y = pos.0.y;
        let mut z = pos.0.z;
        let mut check_down = true;
        let mut next_shape = state.shape();

        match next_shape {
            pumpkin_data::block_properties::RailShape::NorthSouth => {
                if direction {
                    z += 1;
                } else {
                    z -= 1;
                }
            }
            pumpkin_data::block_properties::RailShape::EastWest => {
                if direction {
                    x -= 1;
                } else {
                    x += 1;
                }
            }
            pumpkin_data::block_properties::RailShape::AscendingEast => {
                if direction {
                    x -= 1;
                } else {
                    x += 1;
                    y += 1;
                    check_down = false;
                }
                next_shape = pumpkin_data::block_properties::RailShape::EastWest;
            }
            pumpkin_data::block_properties::RailShape::AscendingWest => {
                if direction {
                    x -= 1;
                    y += 1;
                    check_down = false;
                } else {
                    x += 1;
                }
                next_shape = pumpkin_data::block_properties::RailShape::EastWest;
            }
            pumpkin_data::block_properties::RailShape::AscendingNorth => {
                if direction {
                    z += 1;
                } else {
                    z -= 1;
                    y += 1;
                    check_down = false;
                }
                next_shape = pumpkin_data::block_properties::RailShape::NorthSouth;
            }
            pumpkin_data::block_properties::RailShape::AscendingSouth => {
                if direction {
                    z += 1;
                    y += 1;
                    check_down = false;
                } else {
                    z -= 1;
                }
                next_shape = pumpkin_data::block_properties::RailShape::NorthSouth;
            }
            _ => return false,
        }

        let next_pos = BlockPos::new(x, y, z);

        if self
            .is_powered_by_other_rails_at(world, &next_pos, direction, distance, next_shape)
            .await
        {
            return true;
        }

        if check_down {
            let down_pos = BlockPos::new(x, y - 1, z);
            if self
                .is_powered_by_other_rails_at(world, &down_pos, direction, distance, next_shape)
                .await
            {
                return true;
            }
        }

        false
    }

    async fn is_powered_by_other_rails_at(
        &self,
        world: &World,
        pos: &BlockPos,
        direction: bool,
        distance: u8,
        expected_shape: pumpkin_data::block_properties::RailShape,
    ) -> bool {
        let block = world.get_block(pos).await;
        if block != Block::ACTIVATOR_RAIL {
            return false;
        }

        let state_id = world.get_block_state_id(pos).await;
        let rail_props = RailProperties::new(state_id, &block);
        let rail_shape = rail_props.shape();

        match expected_shape {
            pumpkin_data::block_properties::RailShape::EastWest => {
                if matches!(
                    rail_shape,
                    pumpkin_data::block_properties::RailShape::NorthSouth
                        | pumpkin_data::block_properties::RailShape::AscendingNorth
                        | pumpkin_data::block_properties::RailShape::AscendingSouth
                ) {
                    return false;
                }
            }
            pumpkin_data::block_properties::RailShape::NorthSouth => {
                if matches!(
                    rail_shape,
                    pumpkin_data::block_properties::RailShape::EastWest
                        | pumpkin_data::block_properties::RailShape::AscendingEast
                        | pumpkin_data::block_properties::RailShape::AscendingWest
                ) {
                    return false;
                }
            }
            _ => {}
        }

        if !rail_props.is_powered() {
            return false;
        }

        if block_receives_redstone_power(world, pos).await {
            return true;
        }

        Box::pin(self.is_powered_by_other_rails(world, pos, &rail_props, direction, distance + 1))
            .await
    }

    async fn update_powered_state(&self, world: &Arc<World>, block: &Block, pos: &BlockPos) {
        self.update_powered_state_internal(world, block, pos, true)
            .await;
    }

    async fn update_powered_state_internal(
        &self,
        world: &Arc<World>,
        block: &Block,
        pos: &BlockPos,
        propagate: bool,
    ) {
        let state_id = world.get_block_state_id(pos).await;
        let mut rail_props = RailProperties::new(state_id, block);
        let current_powered = rail_props.is_powered();

        let direct_power = block_receives_redstone_power(world, pos).await;

        let rail_power = self
            .is_powered_by_other_rails(world, pos, &rail_props, true, 0)
            .await
            || self
                .is_powered_by_other_rails(world, pos, &rail_props, false, 0)
                .await;

        let should_be_powered = direct_power || rail_power;

        if current_powered != should_be_powered {
            rail_props.set_powered(should_be_powered);
            world
                .set_block_state(pos, rail_props.to_state_id(block), BlockFlags::NOTIFY_ALL)
                .await;

            world.update_neighbor(&pos.down(), block).await;

            if rail_props.shape().is_ascending() {
                world.update_neighbor(&pos.up(), block).await;
            }

            if propagate {
                let updated_rail_props = RailProperties::new(rail_props.to_state_id(block), block);
                Box::pin(self.update_connected_rails(world, pos, &updated_rail_props, true, 0))
                    .await;
                Box::pin(self.update_connected_rails(world, pos, &updated_rail_props, false, 0))
                    .await;
            }
        }
    }

    async fn update_connected_rails(
        &self,
        world: &Arc<World>,
        pos: &BlockPos,
        state: &RailProperties,
        direction: bool,
        distance: u8,
    ) {
        if distance >= 8 {
            return;
        }

        let mut x = pos.0.x;
        let mut y = pos.0.y;
        let mut z = pos.0.z;
        let mut check_down = true;
        let mut next_shape = state.shape();

        match next_shape {
            pumpkin_data::block_properties::RailShape::NorthSouth => {
                if direction {
                    z += 1;
                } else {
                    z -= 1;
                }
            }
            pumpkin_data::block_properties::RailShape::EastWest => {
                if direction {
                    x -= 1;
                } else {
                    x += 1;
                }
            }
            pumpkin_data::block_properties::RailShape::AscendingEast => {
                if direction {
                    x -= 1;
                } else {
                    x += 1;
                    y += 1;
                    check_down = false;
                }
                next_shape = pumpkin_data::block_properties::RailShape::EastWest;
            }
            pumpkin_data::block_properties::RailShape::AscendingWest => {
                if direction {
                    x -= 1;
                    y += 1;
                    check_down = false;
                } else {
                    x += 1;
                }
                next_shape = pumpkin_data::block_properties::RailShape::EastWest;
            }
            pumpkin_data::block_properties::RailShape::AscendingNorth => {
                if direction {
                    z += 1;
                } else {
                    z -= 1;
                    y += 1;
                    check_down = false;
                }
                next_shape = pumpkin_data::block_properties::RailShape::NorthSouth;
            }
            pumpkin_data::block_properties::RailShape::AscendingSouth => {
                if direction {
                    z += 1;
                    y += 1;
                    check_down = false;
                } else {
                    z -= 1;
                }
                next_shape = pumpkin_data::block_properties::RailShape::NorthSouth;
            }
            _ => return,
        }

        let next_pos = BlockPos::new(x, y, z);
        self.update_rail_at_position(world, &next_pos, direction, distance, next_shape)
            .await;

        if check_down {
            let down_pos = BlockPos::new(x, y - 1, z);
            self.update_rail_at_position(world, &down_pos, direction, distance, next_shape)
                .await;
        }
    }

    async fn update_rail_at_position(
        &self,
        world: &Arc<World>,
        pos: &BlockPos,
        direction: bool,
        distance: u8,
        expected_shape: pumpkin_data::block_properties::RailShape,
    ) {
        let block = world.get_block(pos).await;

        if block != Block::ACTIVATOR_RAIL {
            return;
        }

        let state_id = world.get_block_state_id(pos).await;
        let rail_props = RailProperties::new(state_id, &block);
        let rail_shape = rail_props.shape();

        let shapes_compatible = match expected_shape {
            pumpkin_data::block_properties::RailShape::EastWest => !matches!(
                rail_shape,
                pumpkin_data::block_properties::RailShape::NorthSouth
                    | pumpkin_data::block_properties::RailShape::AscendingNorth
                    | pumpkin_data::block_properties::RailShape::AscendingSouth
            ),
            pumpkin_data::block_properties::RailShape::NorthSouth => !matches!(
                rail_shape,
                pumpkin_data::block_properties::RailShape::EastWest
                    | pumpkin_data::block_properties::RailShape::AscendingEast
                    | pumpkin_data::block_properties::RailShape::AscendingWest
            ),
            _ => true,
        };

        if shapes_compatible {
            self.update_powered_state_internal(world, &block, pos, false)
                .await;

            Box::pin(self.update_connected_rails(world, pos, &rail_props, direction, distance + 1))
                .await;
        }
    }

    async fn find_rail_at_position(
        &self,
        world: &World,
        pos: &BlockPos,
    ) -> Option<(Block, RailProperties)> {
        let block = world.get_block(pos).await;
        if block == Block::ACTIVATOR_RAIL {
            let state_id = world.get_block_state_id(pos).await;
            let rail_props = RailProperties::new(state_id, &block);
            Some((block, rail_props))
        } else {
            None
        }
    }
}
