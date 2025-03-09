use crate::block::properties::Direction;
use crate::block::pumpkin_block::PumpkinBlock;
use crate::server::Server;
use crate::world::World;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
use pumpkin_world::block::registry::Block;

pub trait VerticalAttachment: PumpkinBlock {
    fn get_standing_block(&self) -> &'static Block;
    #[allow(clippy::too_many_arguments)]
    async fn on_place(
        &self,
        server: &Server,
        world: &World,
        block: &Block,
        face: &BlockDirection,
        block_pos: &BlockPos,
        use_item_on: &SUseItemOn,
        player_direction: &Direction,
        other: bool,
    ) -> u16 {
        match &face {
            BlockDirection::Bottom => self.get_standing_block().default_state_id,

            _ => {
                server
                    .block_properties_manager
                    .on_place_state(
                        world,
                        block,
                        &face.opposite(),
                        block_pos,
                        use_item_on,
                        player_direction,
                        other,
                    )
                    .await
            }
        }
    }
}
