use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::BlockProperties;
use pumpkin_data::block::HorizontalFacing;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
use pumpkin_world::block::entities::sign::SignBlockEntity;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::block::registry::BlockRegistry;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;

type SignProperties = pumpkin_data::block::OakSignLikeProperties;

pub fn register_sign_blocks(manager: &mut BlockRegistry) {
    let tag_values: &'static [&'static str] =
        get_tag_values(RegistryKey::Block, "minecraft:signs").unwrap();

    for block in tag_values {
        pub struct SignBlock {
            id: &'static str,
        }
        impl BlockMetadata for SignBlock {
            fn namespace(&self) -> &'static str {
                "minecraft"
            }

            fn id(&self) -> &'static str {
                self.id
            }
        }

        #[async_trait]
        impl PumpkinBlock for SignBlock {
            async fn on_place(
                &self,
                _server: &Server,
                _world: &World,
                block: &Block,
                _face: &BlockDirection,
                _block_pos: &BlockPos,
                _use_item_on: &SUseItemOn,
                _player_direction: &HorizontalFacing,
                _other: bool,
            ) -> u16 {
                let sign_props = SignProperties::default(block);

                sign_props.to_state_id(block)
            }

            async fn placed(
                &self,
                world: &Arc<World>,
                _block: &Block,
                _state_id: u16,
                pos: &BlockPos,
                _old_state_id: u16,
                _notify: bool,
            ) {
                world
                    .add_block_entity(Arc::new(SignBlockEntity::empty(*pos)))
                    .await;
            }

            async fn player_placed(
                &self,
                _world: &Arc<World>,
                _block: &Block,
                _state_id: u16,
                pos: &BlockPos,
                _face: &BlockDirection,
                player: &Player,
            ) {
                player.send_sign_packet(*pos).await;
            }

            async fn on_state_replaced(
                &self,
                world: &Arc<World>,
                _block: &Block,
                location: BlockPos,
                _old_state_id: u16,
                _moved: bool,
            ) {
                world.remove_block_entity(&location).await;
            }
        }

        manager.register(SignBlock { id: block });
    }
}
