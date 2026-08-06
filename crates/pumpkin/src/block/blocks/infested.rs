use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::GameMode;

use crate::block::BrokenArgs;
use crate::block::{BlockBehaviour, BlockFuture};
use crate::entity::Entity;

#[pumpkin_block_from_tag("c:cobblestones/infested")]
pub struct InfestedBlock;

impl BlockBehaviour for InfestedBlock {
    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async {
            // TODO: ugly fix, use onStacksDropped
            if args.player.gamemode.load() == GameMode::Creative {
                return;
            }
            let entity = Entity::new(
                args.world.clone(),
                args.position.0.to_f64(),
                &EntityType::SILVERFISH,
            );

            args.world.spawn_entity(Arc::new(entity)).await;
        })
    }
}
