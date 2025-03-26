use async_trait::async_trait;
use pumpkin_data::{block::{Block, BlockProperties, HorizontalFacing}, tag::{get_tag_values, RegistryKey}};
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;

use crate::{block::{pumpkin_block::{BlockMetadata, PumpkinBlock}, registry::BlockRegistry}, server::Server, world::World};

type WallSignProps = pumpkin_data::block::LadderLikeProperties;


pub fn register_sign_blocks(manager: &mut BlockRegistry) {
    let tag_values: &'static [&'static str] =
        match get_tag_values(RegistryKey::Block, "minecraft:standing_signs") {
            Some(value) => value,
            None => {
                log::error!("Couldn't get tags for minecraft:standing_signs");
                return;
            }
        };

    let wall_tag_values: &'static [&'static str] =
        match get_tag_values(RegistryKey::Block, "minecraft:wall_signs") {
            Some(value) => value,
            None => {
                log::error!("Couldn't get tags for minecraft:wall_signs");
                return;
            }
        };
    

    for (index, block) in tag_values.iter().enumerate() {
        pub struct SignBlock {
            id: &'static str,
            wall_block: &'static str,
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
                face: &BlockDirection,
                _block_pos: &BlockPos,
                _use_item_on: &SUseItemOn,
                _player_direction: &HorizontalFacing,
                _other: bool,
            ) -> u16 {
                if face.is_horizontal() {
                    let wall_block = match Block::from_registry_key(self.wall_block) {
                        Some(b) => b,
                        None => {
                            log::error!("Failed to the block {}", block.id);
                            Block::OAK_WALL_SIGN
                        }
                    };
                    
                    let mut props = WallSignProps::default(&wall_block);
                    props.facing = match face.to_horizontal_facing() {
                        Some (f) => f.opposite(),
                        None => {
                            log::error!("Failed to get horizontal facing for sign");
                            return wall_block.default_state_id;
                        }
                    };
                    return props.to_state_id(&wall_block);
                }

                block.default_state_id
            }
        }


        manager.register(SignBlock { id: block, wall_block: wall_tag_values[index] });
    }
}