use std::sync::Arc;

use crate::block::BlockBehaviour;
use crate::block::OnEntityCollisionArgs;
use crate::block::PlacedArgs;
use crate::block::entities::end_portal::EndPortalBlockEntity;
use pumpkin_data::dimension::Dimension;
use pumpkin_macros::pumpkin_block;

#[pumpkin_block("minecraft:end_portal")]
pub struct EndPortalBlock;

impl BlockBehaviour for EndPortalBlock {
    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        let target_world =
            if args.world.dimension.minecraft_name == Dimension::THE_END.minecraft_name {
                args.server.get_world_from_dimension(&Dimension::OVERWORLD)
            } else {
                args.server.get_world_from_dimension(&Dimension::THE_END)
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
            .try_use_portal(0, target_world, *args.position);
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let nbt = EndPortalBlockEntity::create_nbt(*args.position);
        args.world.add_block_entity_nbt(*args.position, &nbt);

        args.world
            .add_block_entity(Arc::new(EndPortalBlockEntity::new(*args.position)));
    }
}
