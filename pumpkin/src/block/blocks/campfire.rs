use pumpkin_data::{
    Block, BlockDirection,
    block_properties::{BlockProperties, CampfireLikeProperties},
    fluid::Fluid,
};
use pumpkin_world::{BlockStateId, tick::TickPriority};

use crate::{
    block::{
        BlockBehaviour, BlockFuture, BlockIsReplacing, BlockMetadata,
        GetStateForNeighborUpdateArgs, OnEntityCollisionArgs, OnPlaceArgs,
    },
    entity::EntityBase,
};

pub struct CampfireBlock;

impl BlockMetadata for CampfireBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[Block::CAMPFIRE.name, Block::SOUL_CAMPFIRE.name]
    }
}

impl BlockBehaviour for CampfireBlock {
    // TODO: cooking food on campfire (CampfireBlockEntity)
    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if CampfireLikeProperties::from_state_id(args.state.id, args.block).lit
                && args.entity.get_living_entity().is_some()
            {
                // TODO
                //args.entity.damage(args.entity, 1.0, DamageType::CAMPFIRE).await;
            }
        })
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let is_replacing_water = matches!(args.replacing, BlockIsReplacing::Water(_));
            let mut props =
                CampfireLikeProperties::from_state_id(args.block.default_state.id, args.block);
            props.waterlogged = is_replacing_water;
            props.signal_fire =
                is_signal_fire_base_block(args.world.get_block(&args.position.down()).await);
            props.lit = !is_replacing_water;
            props.facing = args.player.get_entity().get_horizontal_facing();
            props.to_state_id(args.block)
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = CampfireLikeProperties::from_state_id(args.state_id, args.block);
            if props.waterlogged {
                props.lit = false;
                args.world
                    .schedule_fluid_tick(
                        &Fluid::WATER,
                        *args.position,
                        Fluid::WATER.flow_speed as u8,
                        TickPriority::Normal,
                    )
                    .await;
            }

            if args.direction == BlockDirection::Down {
                props.signal_fire =
                    is_signal_fire_base_block(args.world.get_block(args.neighbor_position).await);
            }

            props.to_state_id(args.block)
        })
    }

    // TODO: onProjectileHit
}

fn is_signal_fire_base_block(block: &Block) -> bool {
    block == &Block::HAY_BLOCK
}
