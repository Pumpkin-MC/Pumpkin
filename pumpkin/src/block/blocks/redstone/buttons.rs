use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::BlockFace;
use pumpkin_data::block::BlockState;
use pumpkin_data::block::HorizontalFacing;
use pumpkin_data::block::{BlockProperties, Boolean};
use pumpkin_data::item::Item;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
use pumpkin_world::chunk::TickPriority;

type ButtonLikeProperties = pumpkin_data::block::LeverLikeProperties;

use crate::block::blocks::redstone::lever::LeverLikePropertiesExt;
use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::block::registry::BlockActionResult;
use crate::block::registry::BlockRegistry;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::BlockFlags;
use crate::world::World;

#[allow(clippy::too_many_lines)]
pub fn register_button_blocks(manager: &mut BlockRegistry) {
    let tag_values: &'static [&'static str] =
        get_tag_values(RegistryKey::Block, "minecraft:buttons").unwrap();

    for block in tag_values {
        async fn click_button(world: &Arc<World>, block_pos: &BlockPos) {
            let (block, state) = world.get_block_and_block_state(block_pos).await.unwrap();

            let mut button_props = ButtonLikeProperties::from_state_id(state.id, &block);
            if !button_props.powered.to_bool() {
                button_props.powered = Boolean::True;
                world
                    .set_block_state(
                        block_pos,
                        button_props.to_state_id(&block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
                let delay = if block == Block::STONE_BUTTON { 20 } else { 30 };
                world
                    .schedule_block_tick(&block, *block_pos, delay, TickPriority::Normal)
                    .await;
                ButtonBlock::update_neighbors(world, block_pos, &button_props).await;
            }
        }

        pub struct ButtonBlock {
            id: &'static str,
        }
        impl BlockMetadata for ButtonBlock {
            fn namespace(&self) -> &'static str {
                "minecraft"
            }

            fn id(&self) -> &'static str {
                self.id
            }
        }

        #[async_trait]
        impl PumpkinBlock for ButtonBlock {
            async fn on_place(
                &self,
                _server: &Server,
                _world: &World,
                block: &Block,
                face: &BlockDirection,
                _block_pos: &BlockPos,
                _use_item_on: &SUseItemOn,
                player_direction: &HorizontalFacing,
                _other: bool,
            ) -> u16 {
                let mut props = ButtonLikeProperties::default(block);

                match face {
                    BlockDirection::Up => props.face = BlockFace::Ceiling,
                    BlockDirection::Down => props.face = BlockFace::Floor,
                    _ => props.face = BlockFace::Wall,
                }

                if face == &BlockDirection::Up || face == &BlockDirection::Down {
                    props.facing = *player_direction;
                } else {
                    props.facing = face.opposite().to_cardinal_direction();
                };

                props.to_state_id(block)
            }

            async fn normal_use(
                &self,
                _block: &Block,
                _player: &Player,
                location: BlockPos,
                _server: &Server,
                world: &Arc<World>,
            ) {
                click_button(world, &location).await;
            }

            async fn use_with_item(
                &self,
                _block: &Block,
                _player: &Player,
                location: BlockPos,
                _item: &Item,
                _server: &Server,
                world: &Arc<World>,
            ) -> BlockActionResult {
                click_button(world, &location).await;
                BlockActionResult::Consume
            }

            async fn on_scheduled_tick(
                &self,
                world: &Arc<World>,
                block: &Block,
                block_pos: &BlockPos,
            ) {
                let state = world.get_block_state(block_pos).await.unwrap();
                let mut props = ButtonLikeProperties::from_state_id(state.id, block);
                props.powered = Boolean::False;
                world
                    .set_block_state(block_pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
                    .await;
                Self::update_neighbors(world, block_pos, &props).await;
            }

            async fn emits_redstone_power(
                &self,
                _block: &Block,
                _state: &BlockState,
                _direction: &BlockDirection,
            ) -> bool {
                true
            }

            async fn get_weak_redstone_power(
                &self,
                block: &Block,
                _world: &World,
                _block_pos: &BlockPos,
                state: &BlockState,
                _direction: &BlockDirection,
            ) -> u8 {
                let button_props = ButtonLikeProperties::from_state_id(state.id, block);
                if button_props.powered.to_bool() {
                    15
                } else {
                    0
                }
            }

            async fn get_strong_redstone_power(
                &self,
                block: &Block,
                _world: &World,
                _block_pos: &BlockPos,
                state: &BlockState,
                direction: &BlockDirection,
            ) -> u8 {
                let button_props = ButtonLikeProperties::from_state_id(state.id, block);
                if button_props.powered.to_bool() && button_props.get_direction() == *direction {
                    15
                } else {
                    0
                }
            }

            async fn on_state_replaced(
                &self,
                world: &Arc<World>,
                block: &Block,
                location: BlockPos,
                old_state_id: u16,
                moved: bool,
            ) {
                if !moved {
                    let button_props = ButtonLikeProperties::from_state_id(old_state_id, block);
                    if button_props.powered.to_bool() {
                        Self::update_neighbors(world, &location, &button_props).await;
                    }
                }
            }
        }

        impl ButtonBlock {
            async fn update_neighbors(
                world: &Arc<World>,
                block_pos: &BlockPos,
                props: &ButtonLikeProperties,
            ) {
                let direction = props.get_direction().opposite();
                world.update_neighbors(block_pos, None).await;
                world
                    .update_neighbors(&block_pos.offset(direction.to_offset()), None)
                    .await;
            }
        }

        manager.register(ButtonBlock { id: block });
    }
}
