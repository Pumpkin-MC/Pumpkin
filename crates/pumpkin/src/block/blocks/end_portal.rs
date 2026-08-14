use std::sync::Arc;

use crate::block::BlockBehaviour;
use crate::block::BlockFuture;
use crate::block::OnEntityCollisionArgs;
use crate::block::PlacedArgs;
use crate::block::entities::end_portal::EndPortalBlockEntity;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::identifier::Identifier;

#[pumpkin_block("minecraft:end_portal")]
pub struct EndPortalBlock;

impl BlockBehaviour for EndPortalBlock {
    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let overworld = Identifier::vanilla_static("overworld");
            let end = Identifier::vanilla_static("the_end");
            let target_world = if args.world.world_key() == &end {
                args.server.get_world_by_key(&overworld)
            } else {
                args.server.get_world_by_key(&end)
            };
            if Arc::ptr_eq(&target_world, args.world) {
                return;
            }
            tracing::info!(
                "End portal collision at {:?}, targeting world {:?}",
                args.position,
                target_world.dimension.minecraft_name
            );
            args.entity
                .get_entity()
                .try_use_portal(0, target_world, *args.position)
                .await;
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let nbt = EndPortalBlockEntity::create_nbt(*args.position);
            args.world.add_block_entity_nbt(*args.position, &nbt);

            args.world
                .add_block_entity(Arc::new(EndPortalBlockEntity::new(*args.position)));
        })
    }
}
