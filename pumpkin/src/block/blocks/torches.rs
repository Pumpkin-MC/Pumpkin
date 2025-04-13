use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::BlockProperties;
use pumpkin_data::block::HorizontalFacing;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::BlockDirection;

type WallTorchProps = pumpkin_data::block::WallTorchLikeProperties;
// Normal tourches don't have properties

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::block::registry::BlockRegistry;
use crate::server::Server;
use crate::world::World;

pub fn register_torch_blocks(manager: &mut BlockRegistry) {
    for block in ["torch", "soul_torch"] {
        pub struct TorchBlock {
            id: &'static str,
        }
        impl BlockMetadata for TorchBlock {
            fn namespace(&self) -> &'static str {
                "minecraft"
            }

            fn id(&self) -> &'static str {
                self.id
            }
        }

        #[async_trait]
        impl PumpkinBlock for TorchBlock {
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
            ) -> BlockStateId {
                if face.is_horizontal() {
                    let wall_block = match block.name {
                        "torch" => Block::WALL_TORCH,
                        "soul_torch" => Block::SOUL_WALL_TORCH,
                        _ => unreachable!(),
                    };
                    let mut torch_props = WallTorchProps::default(&wall_block);
                    torch_props.facing = face.to_horizontal_facing().unwrap().opposite();
                    return torch_props.to_state_id(&wall_block);
                }
                block.default_state_id
            }
        }

        manager.register(TorchBlock { id: block });
    }
}
