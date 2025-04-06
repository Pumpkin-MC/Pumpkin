use async_trait::async_trait;
use pumpkin_data::block::Axis;
use pumpkin_data::block::Block;
use pumpkin_data::block::BlockProperties;
use pumpkin_data::block::Boolean;
use pumpkin_data::block::DoorHinge;
use pumpkin_data::block::DoubleBlockHalf;
use pumpkin_data::block::HorizontalFacing;
use pumpkin_data::sound::Sound;
use pumpkin_data::sound::SoundCategory;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::Tagable;
use pumpkin_data::tag::get_tag_values;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
use pumpkin_world::block::HorizontalFacingExt;
use std::sync::Arc;

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::block::registry::BlockActionResult;
use crate::block::registry::BlockRegistry;
use crate::entity::player::Player;
use crate::world::BlockFlags;
use pumpkin_data::item::Item;
use pumpkin_protocol::server::play::SUseItemOn;

use crate::server::Server;
use crate::world::World;

type DoorProperties = pumpkin_data::block::OakDoorLikeProperties;

async fn toggle_door(world: &Arc<World>, block_pos: &BlockPos) {
    let (block, block_state) = world.get_block_and_block_state(block_pos).await.unwrap();
    let mut door_props = DoorProperties::from_state_id(block_state.id, &block);
    door_props.open = door_props.open.flip();

    let other_half = match door_props.half {
        DoubleBlockHalf::Upper => BlockDirection::Down,
        DoubleBlockHalf::Lower => BlockDirection::Up,
    };
    let other_pos = block_pos.offset(other_half.to_offset());

    let (other_block, other_state_id) = world.get_block_and_block_state(&other_pos).await.unwrap();
    let mut other_door_props = DoorProperties::from_state_id(other_state_id.id, &other_block);
    other_door_props.open = door_props.open;

    world
        .set_block_state(
            block_pos,
            door_props.to_state_id(&block),
            BlockFlags::NOTIFY_LISTENERS,
        )
        .await;
    world
        .set_block_state(
            &other_pos,
            other_door_props.to_state_id(&other_block),
            BlockFlags::NOTIFY_LISTENERS,
        )
        .await;
}

fn can_open_door(block: &Block) -> bool {
    if block.id == Block::IRON_DOOR.id {
        return false;
    }

    true
}

// Todo: The sounds should be from BlockSetType
fn get_sound(block: &Block, open: bool) -> Sound {
    if open {
        if block.is_tagged_with("minecraft:wooden_doors").unwrap() {
            Sound::BlockWoodenDoorOpen
        } else if block.id == Block::IRON_DOOR.id {
            Sound::BlockIronDoorOpen
        } else {
            Sound::BlockCopperDoorOpen
        }
    } else {
        if block.is_tagged_with("minecraft:wooden_doors").unwrap() {
            Sound::BlockWoodenDoorClose
        } else if block.id == Block::IRON_DOOR.id {
            Sound::BlockIronDoorClose
        } else {
            Sound::BlockCopperDoorClose
        }
    }
}

#[inline]
async fn get_hinge(
    world: &World,
    block: &Block,
    block_pos: &BlockPos,
    use_item_on: &SUseItemOn,
    player_direction: &HorizontalFacing,
) -> DoorHinge {
    let lv4 = block_pos.up();
    let lv5 = player_direction.rotate_ccw();
    let lv6 = block_pos.offset(lv5.to_block_direction().to_offset());
    let lv7 = world.get_block_state(&lv6).await.unwrap();
    let lv8 = lv4.offset(player_direction.to_block_direction().to_offset());
    let lv9 = world.get_block_state(&lv8).await.unwrap();
    let lv10 = player_direction.rotate();
    let lv11 = block_pos.offset(lv10.to_block_direction().to_offset());
    let lv12 = world.get_block_state(&lv11).await.unwrap();
    let lv13 = lv4.offset(player_direction.to_block_direction().to_offset());
    let lv14 = world.get_block_state(&lv13).await.unwrap();
    let bl = world
        .get_block(&lv6)
        .await
        .unwrap()
        .is_tagged_with("minecraft:doors")
        .unwrap()
        && DoorProperties::from_state_id(lv7.id, &block).half == DoubleBlockHalf::Lower;

    let bl2 = world
        .get_block(&lv11)
        .await
        .unwrap()
        .is_tagged_with("minecraft:doors")
        .unwrap()
        && DoorProperties::from_state_id(lv12.id, &block).half == DoubleBlockHalf::Lower;

    let i = (lv7.is_full_cube() as i32) * -1
        + (lv9.is_full_cube() as i32) * -1
        + (lv12.is_full_cube() as i32)
        + (lv14.is_full_cube() as i32);

    if (!bl || bl2) && i <= 0 {
        if (!bl2 || bl) && i >= 0 {
            let j = player_direction.to_block_direction().to_offset();
            let lv15 = use_item_on.cursor_pos;
            if (j.x >= 0 || !(lv15.z < 0.5))
                && (j.x <= 0 || !(lv15.z > 0.5))
                && (j.z >= 0 || !(lv15.x > 0.5))
                && (j.z <= 0 || !(lv15.x < 0.5))
            {
                DoorHinge::Left
            } else {
                DoorHinge::Right
            }
        } else {
            DoorHinge::Left
        }
    } else {
        DoorHinge::Right
    }
}

#[allow(clippy::too_many_lines)]
pub fn register_door_blocks(manager: &mut BlockRegistry) {
    let tag_values = get_tag_values(RegistryKey::Block, "minecraft:doors").unwrap();

    for block in tag_values {
        pub struct DoorBlock {
            id: &'static str,
        }
        impl BlockMetadata for DoorBlock {
            fn namespace(&self) -> &'static str {
                "minecraft"
            }

            fn id(&self) -> &'static str {
                self.id
            }
        }

        #[async_trait]
        impl PumpkinBlock for DoorBlock {
            async fn on_place(
                &self,
                _server: &Server,
                world: &World,
                block: &Block,
                _face: &BlockDirection,
                block_pos: &BlockPos,
                use_item_on: &SUseItemOn,
                player_direction: &HorizontalFacing,
                _other: bool,
            ) -> u16 {
                let powered = block_receives_redstone_power(world, block_pos).await
                    || block_receives_redstone_power(world, &block_pos.up()).await;

                let hinge = get_hinge(world, block, block_pos, use_item_on, player_direction).await;

                let mut door_props = DoorProperties::default(block);
                door_props.half = DoubleBlockHalf::Lower;
                door_props.facing = *player_direction;
                door_props.hinge = hinge;
                door_props.powered = Boolean::from_bool(powered);
                door_props.open = Boolean::from_bool(powered);

                door_props.to_state_id(block)
            }

            async fn can_place_at(&self, world: &World, block_pos: &BlockPos) -> bool {
                if world
                    .get_block_state(&block_pos.offset(BlockDirection::Up.to_offset()))
                    .await
                    .is_ok_and(|state| state.replaceable)
                    && world
                        .get_block_state(&block_pos.offset(BlockDirection::Down.to_offset()))
                        .await
                        .is_ok_and(|state| state.is_solid && state.is_full_cube())
                {
                    return true;
                }
                false
            }

            async fn placed(
                &self,
                world: &Arc<World>,
                block: &Block,
                state_id: u16,
                block_pos: &BlockPos,
                _old_state_id: u16,
                _notify: bool,
            ) {
                let mut door_props = DoorProperties::from_state_id(state_id, block);
                door_props.half = DoubleBlockHalf::Upper;

                world
                    .set_block_state(
                        &block_pos.offset(BlockDirection::Up.to_offset()),
                        door_props.to_state_id(block),
                        BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
                    )
                    .await;
            }

            async fn use_with_item(
                &self,
                block: &Block,
                _player: &Player,
                location: BlockPos,
                _item: &Item,
                _server: &Server,
                world: &Arc<World>,
            ) -> BlockActionResult {
                if !can_open_door(block) {
                    return BlockActionResult::Continue;
                }

                toggle_door(world, &location).await;

                BlockActionResult::Consume
            }

            async fn normal_use(
                &self,
                block: &Block,
                _player: &Player,
                location: BlockPos,
                _server: &Server,
                world: &Arc<World>,
            ) {
                if can_open_door(block) {
                    toggle_door(world, &location).await;
                }
            }

            async fn on_neighbor_update(
                &self,
                world: &Arc<World>,
                block: &Block,
                pos: &BlockPos,
                _source_block: &Block,
                _notify: bool,
            ) {
                let block_state = world.get_block_state(pos).await.unwrap();
                let mut door_props = DoorProperties::from_state_id(block_state.id, &block);

                let other_half = match door_props.half {
                    DoubleBlockHalf::Upper => BlockDirection::Down,
                    DoubleBlockHalf::Lower => BlockDirection::Up,
                };
                let other_pos = pos.offset(other_half.to_offset());
                let (other_block, other_state_id) =
                    world.get_block_and_block_state(&other_pos).await.unwrap();

                let powered = block_receives_redstone_power(world, pos).await
                    || block_receives_redstone_power(world, &other_pos).await;

                if block.id == other_block.id && powered != door_props.powered.to_bool() {
                    let mut other_door_props =
                        DoorProperties::from_state_id(other_state_id.id, &other_block);
                    door_props.powered = door_props.powered.flip();
                    other_door_props.powered = door_props.powered;

                    if powered != door_props.open.to_bool() {
                        door_props.open = door_props.powered;
                        other_door_props.open = other_door_props.powered;

                        world
                            .play_block_sound(
                                get_sound(block, powered),
                                SoundCategory::Blocks,
                                *pos,
                            )
                            .await;
                    }

                    world
                        .set_block_state(
                            pos,
                            door_props.to_state_id(&block),
                            BlockFlags::NOTIFY_LISTENERS,
                        )
                        .await;
                    world
                        .set_block_state(
                            &other_pos,
                            other_door_props.to_state_id(&other_block),
                            BlockFlags::NOTIFY_LISTENERS,
                        )
                        .await;
                }
            }

            async fn get_state_for_neighbor_update(
                &self,
                world: &World,
                block: &Block,
                state: u16,
                block_pos: &BlockPos,
                direction: &BlockDirection,
                _neighbor_pos: &BlockPos,
                neighbor_state: u16,
            ) -> u16 {
                let lv = DoorProperties::from_state_id(state, block).half;
                if direction.to_axis() != Axis::Y
                    || (lv == DoubleBlockHalf::Lower) != (direction == &BlockDirection::Up)
                {
                    if lv == DoubleBlockHalf::Lower
                        && direction == &BlockDirection::Down
                        && !self.can_place_at(world, block_pos).await
                    {
                        return 0;
                    }
                } else if Block::from_state_id(neighbor_state).unwrap().id == block.id
                    && DoorProperties::from_state_id(neighbor_state, block).half != lv
                {
                    let mut new_state = DoorProperties::from_state_id(neighbor_state, block);
                    new_state.half = lv;
                    return new_state.to_state_id(block);
                } else {
                    return 0;
                }
                state
            }
        }

        manager.register(DoorBlock { id: block });
    }
}
