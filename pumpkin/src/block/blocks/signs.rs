use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::BlockProperties;
use pumpkin_data::block::HorizontalFacing;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
use pumpkin_world::block::registry::get_block;

use crate::block::pumpkin_block::BlockMetadata;
use crate::block::pumpkin_block::PumpkinBlock;
use crate::block::registry::BlockRegistry;
use crate::server::Server;
use crate::world::World;
use pumpkin_data::block::Integer0To15;

use pumpkin_data::tag::get_tag_values;

use pumpkin_data::tag::RegistryKey;

type SignLikeProperties = pumpkin_data::block::OakSignLikeProperties;
type LadderLikeProperties = pumpkin_data::block::LadderLikeProperties;

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
                face: &BlockDirection,
                _block_pos: &BlockPos,
                _use_item_on: &SUseItemOn,
                player_direction: &HorizontalFacing,
                _other: bool,
            ) -> u16 {
                // Is a standing sign or is a standing sign.
                if *face == BlockDirection::Down {
                    let mut block_properties = SignLikeProperties::default(block);
                    block_properties.rotation = match player_direction.opposite() {
                        // Taken from wiki
                        // TODO: There should be more enums in HorizontalFacing
                        HorizontalFacing::South => Integer0To15::L0,
                        HorizontalFacing::West => Integer0To15::L4,
                        HorizontalFacing::North => Integer0To15::L8,
                        HorizontalFacing::East => Integer0To15::L12,
                    };

                    block_properties.to_state_id(block)
                } else {
                    let key = self.name().replace("_sign", "_wall_sign");
                    // Sends "false" if no map which basically means don't place anything
                    get_block(&key).map_or(0, |new_block| {
                        let mut block_properties_wall_override =
                            LadderLikeProperties::default(&new_block);

                        block_properties_wall_override.facing = player_direction.opposite();

                        block_properties_wall_override.to_state_id(&new_block)
                    })
                }
            }
        }
        manager.register(SignBlock { id: block });
    }
}
