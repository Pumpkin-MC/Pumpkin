use async_trait::async_trait;
use pumpkin_data::{
    block::{Block, BlockProperties},
    tag::{RegistryKey, get_tag_values},
};
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::{BlockDirection, precise_direction::PreciseDirection};

use crate::{
    block::{
        pumpkin_block::{BlockMetadata, PumpkinBlock},
        registry::BlockRegistry,
    },
    server::Server,
    world::World,
};

type WallSignProps = pumpkin_data::block::LadderLikeProperties;
type SignProps = pumpkin_data::block::OakSignLikeProperties;

pub fn register_sign_blocks(manager: &mut BlockRegistry) {
    let tag_values: &'static [&'static str] =
        if let Some(values) = get_tag_values(RegistryKey::Block, "minecraft:standing_signs") {
            values
        } else {
            log::error!("Couldn't get tags for minecraft:standing_signs");
            return;
        };

    let wall_tag_values: &'static [&'static str] =
        if let Some(values) = get_tag_values(RegistryKey::Block, "minecraft:wall_signs") {
            values
        } else {
            log::error!("Couldn't get tags for minecraft:wall_signs");
            return;
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
                player_direction: &f32,
                _other: bool,
            ) -> u16 {
                if face.is_horizontal() {
                    let wall_block = Block::from_registry_key(self.wall_block).map_or_else(
                        || {
                            log::error!("Failed to the block {}", block.id);
                            Block::OAK_WALL_SIGN
                        },
                        |block| block,
                    );

                    let mut props = WallSignProps::default(&wall_block);
                    if let Some(facing) = face.to_horizontal_facing() {
                        props.facing = facing.opposite();
                    } else {
                        log::error!("Failed to get horizontal facing for sign");
                        return wall_block.default_state_id;
                    }
                    return props.to_state_id(&wall_block);
                }

                let direction = PreciseDirection::from(*player_direction).opposite();

                let mut props = SignProps::default(block);
                props.rotation = direction.to_integer_0_to_15();

                props.to_state_id(block)
            }
        }

        manager.register(SignBlock {
            id: block,
            wall_block: wall_tag_values[index],
        });
    }
}
