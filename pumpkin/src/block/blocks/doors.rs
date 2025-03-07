use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::BlockProperties;
use pumpkin_data::block::BlockState;
use pumpkin_data::block::CardinalDirection;
use pumpkin_data::block::DoorHinge;
use pumpkin_data::block::OakDoorProps;
use pumpkin_data::block::VerticalHalf;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
use std::sync::Arc;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::block::registry::BlockActionResult;
use crate::block::registry::BlockRegistry;
use crate::entity::player::Player;
use pumpkin_data::item::Item;
use pumpkin_protocol::server::play::SUseItemOn;

use crate::server::Server;
use crate::world::World;

async fn toggle_door(world: &World, block_pos: &BlockPos, index_id: u16) {
    let block = world.get_block(block_pos).await.unwrap();
    let mut door_props = OakDoorProps::from_index(index_id);
    door_props.open = door_props.open.flip();

    let other_half = match door_props.half {
        VerticalHalf::Upper => BlockDirection::Down,
        VerticalHalf::Lower => BlockDirection::Up,
    };
    let other_pos = block_pos.offset(other_half.to_offset());

    let (other_block, other_state_id) = world.get_block_and_block_state(&other_pos).await.unwrap();

    // Create a new scope to ensure other_door_props doesn't live across await points
    let other_new_state_id = {
        let other_door_props = other_block
            .properties_from_state_id(other_state_id.id)
            .unwrap();

        let mut other_door_props = OakDoorProps::from_index(other_door_props.to_index());
        other_door_props.open = door_props.open;

        other_block
            .properties_from_index(other_door_props.to_index())
            .unwrap()
            .to_state_id()
    };

    let new_state_id = block
        .properties_from_index(door_props.to_index())
        .unwrap()
        .to_state_id();

    world.set_block_state(block_pos, new_state_id).await;
    world.set_block_state(&other_pos, other_new_state_id).await;
}

fn place_state_index(player_direction: CardinalDirection) -> u16 {
    let mut door_props = OakDoorProps::default();
    door_props.half = VerticalHalf::Lower;
    door_props.facing = player_direction;
    door_props.hinge = DoorHinge::Left;

    door_props.to_index()
}

fn set_half_index(index: u16, half: VerticalHalf) -> u16 {
    let mut door_props = OakDoorProps::from_index(index);
    door_props.half = half;
    door_props.to_index()
}

fn can_open_door(block: &Block, player: &Player) -> bool {
    if block.id == Block::IRON_DOOR.id && player.gamemode.load() != GameMode::Creative {
        return false;
    }

    true
}

// Macro to easily define new door block variants
macro_rules! define_door_block {
    ($block_name:ident, $block:expr) => {
        pub struct $block_name;
        impl BlockMetadata for $block_name {
            const NAMESPACE: &'static str = "minecraft";
            const ID: &'static str = $block.name;
        }

        #[async_trait]
        impl PumpkinBlock for $block_name {
            async fn on_place(
                &self,
                _server: &Server,
                _world: &World,
                block: &Block,
                _face: &BlockDirection,
                _block_pos: &BlockPos,
                _use_item_on: &SUseItemOn,
                player_direction: &CardinalDirection,
                _other: bool,
            ) -> u16 {
                let door_index = place_state_index(*player_direction);

                block
                    .properties_from_index(door_index)
                    .unwrap()
                    .to_state_id()
            }

            async fn can_place(
                &self,
                _server: &Server,
                world: &World,
                _block: &Block,
                _face: &BlockDirection,
                block_pos: &BlockPos,
                _player_direction: &CardinalDirection,
            ) -> bool {
                if world
                    .get_block_state(&block_pos.offset(BlockDirection::Up.to_offset()))
                    .await
                    .is_ok_and(|state| state.replaceable)
                {
                    return true;
                }
                false
            }

            async fn placed(
                &self,
                block: &Block,
                _player: &Player,
                location: BlockPos,
                _server: &Server,
                world: &World,
            ) {
                let state_id = world.get_block_state_id(&location).await.unwrap();
                let index = block.properties_from_state_id(state_id).unwrap().to_index();
                let upper_index = set_half_index(index, VerticalHalf::Upper);

                let upper_state_id = block
                    .properties_from_index(upper_index)
                    .unwrap()
                    .to_state_id();

                world
                    .set_block_state(
                        &location.offset(BlockDirection::Up.to_offset()),
                        upper_state_id,
                    )
                    .await;
            }

            async fn broken(
                &self,
                block: &Block,
                _player: &Player,
                location: BlockPos,
                server: &Server,
                world: Arc<World>,
                state: BlockState,
            ) {
                let door_index = block.properties_from_state_id(state.id).unwrap().to_index();

                let door_props = OakDoorProps::from_index(door_index);

                let other_half = match door_props.half {
                    VerticalHalf::Upper => BlockDirection::Down,
                    VerticalHalf::Lower => BlockDirection::Up,
                };

                let other_pos = location.offset(other_half.to_offset());

                if let Ok(other_block) = world.get_block(&other_pos).await {
                    if other_block.id == block.id {
                        world
                            .break_block(&other_pos, None, true, Some(server))
                            .await;
                    }
                }
            }

            async fn use_with_item(
                &self,
                block: &Block,
                player: &Player,
                location: BlockPos,
                _item: &Item,
                _server: &Server,
                world: &World,
            ) -> BlockActionResult {
                if !can_open_door(block, player) {
                    return BlockActionResult::Continue;
                }

                let index_id = block
                    .properties_from_state_id(world.get_block_state_id(&location).await.unwrap())
                    .unwrap()
                    .to_index();

                toggle_door(world, &location, index_id).await;
                BlockActionResult::Consume
            }

            async fn normal_use(
                &self,
                block: &Block,
                player: &Player,
                location: BlockPos,
                _server: &Server,
                world: &World,
            ) {
                if !can_open_door(block, player) {
                    return;
                }

                let index_id = block
                    .properties_from_state_id(world.get_block_state_id(&location).await.unwrap())
                    .unwrap()
                    .to_index();
                toggle_door(world, &location, index_id).await;
            }
        }
    };
}

// https://minecraft.fandom.com/wiki/Door

define_door_block!(OakDoorBlock, Block::OAK_DOOR);
define_door_block!(SpruceDoorBlock, Block::SPRUCE_DOOR);
define_door_block!(BirchDoorBlock, Block::BIRCH_DOOR);
define_door_block!(JungleDoorBlock, Block::JUNGLE_DOOR);
define_door_block!(AcaciaDoorBlock, Block::ACACIA_DOOR);
define_door_block!(DarkOakDoorBlock, Block::DARK_OAK_DOOR);
define_door_block!(MangroveDoorBlock, Block::MANGROVE_DOOR);
define_door_block!(CherryDoorBlock, Block::CHERRY_DOOR);
define_door_block!(BambooDoorBlock, Block::BAMBOO_DOOR);
define_door_block!(CrimsonDoorBlock, Block::CRIMSON_DOOR);
define_door_block!(WarpedDoorBlock, Block::WARPED_DOOR);
define_door_block!(PaleOakDoorBlock, Block::PALE_OAK_DOOR);
define_door_block!(IronDoorBlock, Block::IRON_DOOR);
define_door_block!(CopperDoorBlock, Block::COPPER_DOOR);
define_door_block!(ExposedCopperDoorBlock, Block::EXPOSED_COPPER_DOOR);
define_door_block!(OxidizedCopperDoorBlock, Block::OXIDIZED_COPPER_DOOR);
define_door_block!(WeatheredCopperDoorBlock, Block::WEATHERED_COPPER_DOOR);
define_door_block!(WaxedCopperDoorBlock, Block::WAXED_COPPER_DOOR);
define_door_block!(
    WaxedExposedCopperDoorBlock,
    Block::WAXED_EXPOSED_COPPER_DOOR
);
define_door_block!(
    WaxedOxidizedCopperDoorBlock,
    Block::WAXED_OXIDIZED_COPPER_DOOR
);
define_door_block!(
    WaxedWeatheredCopperDoorBlock,
    Block::WAXED_WEATHERED_COPPER_DOOR
);

pub fn register_door_blocks(manager: &mut BlockRegistry) {
    manager.register(OakDoorBlock);
    manager.register(SpruceDoorBlock);
    manager.register(BirchDoorBlock);
    manager.register(JungleDoorBlock);
    manager.register(AcaciaDoorBlock);
    manager.register(DarkOakDoorBlock);
    manager.register(MangroveDoorBlock);
    manager.register(CherryDoorBlock);
    manager.register(BambooDoorBlock);
    manager.register(CrimsonDoorBlock);
    manager.register(WarpedDoorBlock);
    manager.register(PaleOakDoorBlock);

    manager.register(IronDoorBlock);

    manager.register(CopperDoorBlock);
    manager.register(ExposedCopperDoorBlock);
    manager.register(OxidizedCopperDoorBlock);
    manager.register(WeatheredCopperDoorBlock);
    manager.register(WaxedCopperDoorBlock);
    manager.register(WaxedExposedCopperDoorBlock);
    manager.register(WaxedOxidizedCopperDoorBlock);
    manager.register(WaxedWeatheredCopperDoorBlock);
}
