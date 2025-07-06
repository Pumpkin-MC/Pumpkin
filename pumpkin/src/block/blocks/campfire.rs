use async_trait::async_trait;
use pumpkin_data::{
    Block, BlockDirection,
    block_properties::{BlockProperties, CampfireLikeProperties},
    damage::DamageType,
    fluid::Fluid,
};
use pumpkin_world::BlockStateId;

use crate::{
    block::{
        BlockIsReplacing,
        pumpkin_block::{
            BlockMetadata, GetStateForNeighborUpdateArgs, OnEntityCollisionArgs, OnPlaceArgs,
            PumpkinBlock,
        },
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

#[async_trait]
impl PumpkinBlock for CampfireBlock {
    // TODO: cooking food on campfire (CampfireBlockEntity)
    async fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        if CampfireLikeProperties::from_state_id(args.state.id, args.block).lit
            && args.entity.get_living_entity().is_some()
        {
            args.entity.damage(1.0, DamageType::CAMPFIRE).await;
        }
    }

    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let is_replacing_water = matches!(args.replacing, BlockIsReplacing::Water(_));
        let mut props =
            CampfireLikeProperties::from_state_id(args.block.default_state.id, args.block);
        props.waterlogged = is_replacing_water;
        props.signal_fire =
            is_signal_fire_base_block(args.world.get_block(&args.location.down()).await);
        props.lit = !is_replacing_water;
        props.facing = args.player.get_entity().get_horizontal_facing();
        props.to_state_id(args.block)
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let mut props = CampfireLikeProperties::from_state_id(args.state_id, args.block);
        if props.waterlogged {
            props.lit = false;
            args.world
                .schedule_fluid_tick(
                    args.block.id,
                    *args.location,
                    Fluid::WATER.flow_speed as u16,
                )
                .await;
        }

        if args.direction == BlockDirection::Down {
            props.signal_fire =
                is_signal_fire_base_block(args.world.get_block(args.neighbor_location).await);
        }

        props.to_state_id(args.block)
    }

    // TODO: onProjectileHit
}

fn is_signal_fire_base_block(block: &Block) -> bool {
    block == &Block::HAY_BLOCK
}
