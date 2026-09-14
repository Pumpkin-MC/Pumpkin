use std::sync::Arc;

use pumpkin_macros::pumpkin_block;

use crate::block::entities::end_gateway::EndGatewayBlockEntity;
use crate::block::{BlockBehaviour, OnEntityCollisionArgs, OnSyncedBlockEventArgs, PlacedArgs};

#[pumpkin_block("minecraft:end_gateway")]
pub struct EndGatewayBlock;

impl BlockBehaviour for EndGatewayBlock {
    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        let entity = args.entity.get_entity();
        if entity
            .portal_cooldown
            .load(std::sync::atomic::Ordering::Relaxed)
            > 0
        {
            return;
        }

        let Some(block_entity) = args.world.get_block_entity(args.position) else {
            return;
        };
        let Some(end_gateway) = block_entity
            .as_any()
            .downcast_ref::<EndGatewayBlockEntity>()
        else {
            return;
        };

        if end_gateway.is_cooling_down() {
            return;
        }

        end_gateway.trigger_cooldown(args.world, *args.position);

        if let Some(destination) = end_gateway.get_portal_position(args.world, *args.position) {
            let yaw = entity.yaw.load();
            let pitch = entity.pitch.load();
            args.entity
                .teleport(destination, Some(yaw), Some(pitch), args.world.clone());
        }
    }

    fn on_synced_block_event(&self, args: OnSyncedBlockEventArgs<'_>) -> bool {
        if let Some(block_entity) = args.world.get_block_entity(args.position)
            && let Some(end_gateway) = block_entity
                .as_any()
                .downcast_ref::<EndGatewayBlockEntity>()
        {
            end_gateway.trigger_event(args.r#type, args.data)
        } else {
            args.r#type == EndGatewayBlockEntity::EVENT_COOLDOWN
        }
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let block_entity = EndGatewayBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(block_entity));
    }
}
