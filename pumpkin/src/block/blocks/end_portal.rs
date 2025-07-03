use std::sync::Arc;

use crate::block::pumpkin_block::OnEntityCollisionArgs;
use crate::block::pumpkin_block::OnStateReplacedArgs;
use crate::block::pumpkin_block::PlacedArgs;
use crate::block::pumpkin_block::PumpkinBlock;
use async_trait::async_trait;
use pumpkin_macros::pumpkin_block;
use pumpkin_registry::VanillaDimensionType;
use pumpkin_world::block::entities::end_portal::EndPortalBlockEntity;

#[pumpkin_block("minecraft:end_portal")]
pub struct EndPortalBlock;

#[async_trait]
impl PumpkinBlock for EndPortalBlock {
    async fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        let world = if args.world.dimension_type == VanillaDimensionType::TheEnd {
            args.server
                .get_world_from_dimension(VanillaDimensionType::Overworld)
                .await
        } else {
            args.server
                .get_world_from_dimension(VanillaDimensionType::TheEnd)
                .await
        };
        args.entity
            .get_entity()
            .try_use_portal(0, world, *args.location)
            .await;
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        args.world
            .add_block_entity(Arc::new(EndPortalBlockEntity::new(*args.location)))
            .await;
    }

    async fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        args.world.remove_block_entity(args.location).await;
    }
}
