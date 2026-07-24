use pumpkin_data::entity::EntityType;
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::GameMode;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::block::BrokenArgs;
use crate::block::{BlockBehaviour, BlockFuture};
use crate::entity::r#type::from_type;

#[pumpkin_block_from_tag("c:cobblestones/infested")]
pub struct InfestedBlock;

impl BlockBehaviour for InfestedBlock {
    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // TODO: ugly fix, use onStacksDropped
            if args.player.gamemode.load() == GameMode::Creative {
                return;
            }
            let pos = args.position.0.to_f64() + Vector3::new(0.5, 0.0, 0.5);
            let silver = from_type(&EntityType::SILVERFISH, pos, args.world, Uuid::new_v4());
            silver.get_entity().set_pos(pos);
            args.world.spawn_entity(silver).await;
        })
    }
}
