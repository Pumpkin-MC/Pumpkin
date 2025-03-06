use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::BlockProperties;
use pumpkin_data::block::BlockState;
use pumpkin_data::block::CardinalDirection;
use pumpkin_data::block::CopperDoorProps;
use pumpkin_data::block::DoorHinge;
use pumpkin_data::block::ExposedCopperDoorProps;
use pumpkin_data::block::IronDoorProps;
use pumpkin_data::block::OxidizedCopperDoorProps;
use pumpkin_data::block::PaleOakDoorProps;
use pumpkin_data::block::VerticalHalf;
use pumpkin_data::block::WeatheredCopperDoorProps;
use pumpkin_data::block::{
    AcaciaDoorProps, BambooDoorProps, BirchDoorProps, CherryDoorProps, CrimsonDoorProps,
    DarkOakDoorProps, JungleDoorProps, MangroveDoorProps, OakDoorProps, SpruceDoorProps,
    WarpedDoorProps, WaxedCopperDoorProps, WaxedExposedCopperDoorProps,
    WaxedOxidizedCopperDoorProps, WaxedWeatheredCopperDoorProps,
};
use pumpkin_data::item::Item;
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::block::registry::{BlockActionResult, BlockRegistry};
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;

#[allow(dead_code)]
pub trait DoorProperties: BlockProperties + Send + Sync {
    fn set_half(&mut self, half: VerticalHalf);
    fn get_half(&self) -> VerticalHalf;
    fn set_facing(&mut self, facing: CardinalDirection);
    fn get_facing(&self) -> CardinalDirection;
    fn set_hinge(&mut self, hinge: DoorHinge);
    fn get_hinge(&self) -> DoorHinge;
    fn set_open(&mut self, open: bool);
    fn get_open(&self) -> bool;
    fn flip_open(&mut self);
}

async fn toggle_door<T: DoorProperties>(world: &World, block_pos: &BlockPos, state_id: u16) {
    let mut door_props = T::from_state_id(state_id).unwrap();
    door_props.flip_open();

    let other_half = match door_props.get_half() {
        VerticalHalf::Upper => BlockDirection::Down,
        VerticalHalf::Lower => BlockDirection::Up,
    };
    let other_pos = block_pos.offset(other_half.to_offset());

    let other_state_id = world.get_block_state_id(&other_pos).await.unwrap();
    let other_door_props = T::from_state_id(other_state_id);
    if let Some(mut other_door_props) = other_door_props {
        other_door_props.set_open(door_props.get_open());

        world
            .set_block_state(block_pos, door_props.to_state_id())
            .await;
        world
            .set_block_state(&other_pos, other_door_props.to_state_id())
            .await;
    }
}

fn can_open_door(block: &Block, player: &Player) -> bool {
    if block.id == Block::IRON_DOOR.id && player.gamemode.load() != GameMode::Creative {
        return false;
    }

    true
}

// Macro to easily define new door block variants
macro_rules! define_door_block {
    ($block_name:ident, $block_id:expr, $props_type:ty, $default_state_id:expr) => {
        #[pumpkin_block($block_id)]
        pub struct $block_name;

        impl DoorProperties for $props_type {
            fn set_half(&mut self, half: VerticalHalf) {
                self.half = half;
            }

            fn get_half(&self) -> VerticalHalf {
                self.half
            }

            fn set_facing(&mut self, facing: CardinalDirection) {
                self.facing = facing;
            }

            fn get_facing(&self) -> CardinalDirection {
                self.facing
            }

            fn set_hinge(&mut self, hinge: DoorHinge) {
                self.hinge = hinge;
            }

            fn get_hinge(&self) -> DoorHinge {
                self.hinge
            }

            fn set_open(&mut self, open: bool) {
                self.open = if open {
                    pumpkin_data::block::Boolean::True
                } else {
                    pumpkin_data::block::Boolean::False
                };
            }

            fn get_open(&self) -> bool {
                self.open.to_bool()
            }

            fn flip_open(&mut self) {
                self.open = self.open.flip();
            }
        }

        #[async_trait]
        impl PumpkinBlock for $block_name {
            async fn on_place(
                &self,
                _server: &Server,
                _world: &World,
                _block: &Block,
                _face: &BlockDirection,
                _block_pos: &BlockPos,
                _use_item_on: &SUseItemOn,
                player_direction: &CardinalDirection,
                _other: bool,
            ) -> u16 {
                let mut door_props = <$props_type>::from_state_id($default_state_id).unwrap();
                door_props.set_half(VerticalHalf::Lower);
                door_props.set_facing(*player_direction);
                door_props.set_hinge(DoorHinge::Left); // TODO: Calculate hinge based on surroundings

                door_props.to_state_id()
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
                _block: &Block,
                _player: &Player,
                location: BlockPos,
                _server: &Server,
                world: &World,
            ) {
                let state_id = world.get_block_state_id(&location).await.unwrap();
                let mut upper_door_props = <$props_type>::from_state_id(state_id).unwrap();

                upper_door_props.set_half(VerticalHalf::Upper);

                world
                    .set_block_state(
                        &location.offset(BlockDirection::Up.to_offset()),
                        upper_door_props.to_state_id(),
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
                let door_props = <$props_type>::from_state_id(state.id).unwrap();

                let other_half = match door_props.get_half() {
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

                let state_id = world
                    .get_block_state_id(&location)
                    .await
                    .unwrap_or($default_state_id);
                toggle_door::<$props_type>(world, &location, state_id).await;
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

                let state_id = world
                    .get_block_state_id(&location)
                    .await
                    .unwrap_or($default_state_id);
                toggle_door::<$props_type>(world, &location, state_id).await;
            }
        }
    };
}

// https://minecraft.fandom.com/wiki/Door

define_door_block!(
    OakDoorBlock,
    "minecraft:oak_door",
    OakDoorProps,
    Block::OAK_DOOR.default_state_id
);

define_door_block!(
    SpruceDoorBlock,
    "minecraft:spruce_door",
    SpruceDoorProps,
    Block::SPRUCE_DOOR.default_state_id
);

define_door_block!(
    BirchDoorBlock,
    "minecraft:birch_door",
    BirchDoorProps,
    Block::BIRCH_DOOR.default_state_id
);

define_door_block!(
    JungleDoorBlock,
    "minecraft:jungle_door",
    JungleDoorProps,
    Block::JUNGLE_DOOR.default_state_id
);

define_door_block!(
    AcaciaDoorBlock,
    "minecraft:acacia_door",
    AcaciaDoorProps,
    Block::ACACIA_DOOR.default_state_id
);

define_door_block!(
    DarkOakDoorBlock,
    "minecraft:dark_oak_door",
    DarkOakDoorProps,
    Block::DARK_OAK_DOOR.default_state_id
);

define_door_block!(
    MangroveDoorBlock,
    "minecraft:mangrove_door",
    MangroveDoorProps,
    Block::MANGROVE_DOOR.default_state_id
);

define_door_block!(
    CherryDoorBlock,
    "minecraft:cherry_door",
    CherryDoorProps,
    Block::CHERRY_DOOR.default_state_id
);

define_door_block!(
    BambooDoorBlock,
    "minecraft:bamboo_door",
    BambooDoorProps,
    Block::BAMBOO_DOOR.default_state_id
);

define_door_block!(
    CrimsonDoorBlock,
    "minecraft:crimson_door",
    CrimsonDoorProps,
    Block::CRIMSON_DOOR.default_state_id
);

define_door_block!(
    WarpedDoorBlock,
    "minecraft:warped_door",
    WarpedDoorProps,
    Block::WARPED_DOOR.default_state_id
);

define_door_block!(
    PaleOakDoorBlock,
    "minecraft:pale_oak_door",
    PaleOakDoorProps,
    Block::PALE_OAK_DOOR.default_state_id
);

define_door_block!(
    IronDoorBlock,
    "minecraft:iron_door",
    IronDoorProps,
    Block::IRON_DOOR.default_state_id
);

define_door_block!(
    CopperDoorBlock,
    "minecraft:copper_door",
    CopperDoorProps,
    Block::COPPER_DOOR.default_state_id
);

define_door_block!(
    ExposedCopperDoorBlock,
    "minecraft:exposed_copper_door",
    ExposedCopperDoorProps,
    Block::EXPOSED_COPPER_DOOR.default_state_id
);

define_door_block!(
    OxidizedCopperDoorBlock,
    "minecraft:oxidized_copper_door",
    OxidizedCopperDoorProps,
    Block::OXIDIZED_COPPER_DOOR.default_state_id
);

define_door_block!(
    WeatheredCopperDoorBlock,
    "minecraft:weathered_copper_door",
    WeatheredCopperDoorProps,
    Block::WEATHERED_COPPER_DOOR.default_state_id
);

define_door_block!(
    WaxedCopperDoorBlock,
    "minecraft:waxed_copper_door",
    WaxedCopperDoorProps,
    Block::WAXED_COPPER_DOOR.default_state_id
);

define_door_block!(
    WaxedExposedCopperDoorBlock,
    "minecraft:waxed_exposed_copper_door",
    WaxedExposedCopperDoorProps,
    Block::WAXED_EXPOSED_COPPER_DOOR.default_state_id
);

define_door_block!(
    WaxedOxidizedCopperDoorBlock,
    "minecraft:waxed_oxidized_copper_door",
    WaxedOxidizedCopperDoorProps,
    Block::WAXED_OXIDIZED_COPPER_DOOR.default_state_id
);

define_door_block!(
    WaxedWeatheredCopperDoorBlock,
    "minecraft:waxed_weathered_copper_door",
    WaxedWeatheredCopperDoorProps,
    Block::WAXED_WEATHERED_COPPER_DOOR.default_state_id
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
