use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, WallTorchLikeProperties},
    entity::EntityType,
    world::WorldEvent,
};
use pumpkin_world::{BlockStateId, world::BlockFlags};

use crate::{
    block::{BlockBehaviour, BlockFuture, BlockMetadata, OnPlaceArgs, PlacedArgs},
    entity::{Entity, passive::snow_golem::SnowGolemEntity},
};

pub struct CarvedPumpkinBlock;

impl BlockMetadata for CarvedPumpkinBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[Block::JACK_O_LANTERN.name, Block::CARVED_PUMPKIN.name]
    }
}

impl BlockBehaviour for CarvedPumpkinBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = WallTorchLikeProperties::default(args.block);
            props.facing = args
                .player
                .living_entity
                .entity
                .get_horizontal_facing()
                .opposite();
            props.to_state_id(args.block)
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        // Mojang uses some BlockPattern magic, way too complex tbh
        Box::pin(async {
            let down_pos = args.position.down();
            let upper = args.world.get_block(&down_pos).await;
            let lower = args.world.get_block(&down_pos.down()).await;
            if upper == &Block::SNOW_BLOCK && lower == &Block::SNOW_BLOCK {
                for i in 0..3 {
                    let pos = args.position.down_height(i);
                    args.world
                        .set_block_state(
                            &pos,
                            Block::AIR.default_state.id,
                            BlockFlags::NOTIFY_LISTENERS,
                        )
                        .await;
                    args.world
                        .sync_world_event(
                            WorldEvent::BlockBroken,
                            pos,
                            Block::SNOW_BLOCK.default_state.id.into(),
                        )
                        .await;
                }
                let entity = Entity::new(
                    args.world.clone(),
                    down_pos.down().to_centered_f64(),
                    &EntityType::SNOW_GOLEM,
                );
                let golem = SnowGolemEntity::make(entity).await;
                args.world.spawn_entity(golem).await;
            }
        })
    }
}
