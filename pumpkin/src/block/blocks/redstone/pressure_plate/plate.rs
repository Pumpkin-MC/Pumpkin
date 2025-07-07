use async_trait::async_trait;
use pumpkin_data::{
    Block, BlockDirection, BlockState,
    block_properties::BlockProperties,
    tag::{RegistryKey, get_tag_values},
};
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos};
use pumpkin_world::BlockStateId;

use crate::{
    block::pumpkin_block::{
        BlockMetadata, EmitsRedstonePowerArgs, GetRedstonePowerArgs, OnEntityCollisionArgs,
        OnScheduledTickArgs, OnStateReplacedArgs, PumpkinBlock,
    },
    world::World,
};

use super::PressurePlate;

/// This is for Normal Pressure plates, so not Gold or Iron
pub struct PressurePlateBlock;

type PressurePlateProps = pumpkin_data::block_properties::StonePressurePlateLikeProperties;

impl BlockMetadata for PressurePlateBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        let mut combined = Vec::new();
        combined.extend_from_slice(
            get_tag_values(RegistryKey::Block, "minecraft:wooden_pressure_plates").unwrap(),
        );
        combined.extend_from_slice(
            get_tag_values(RegistryKey::Block, "minecraft:stone_pressure_plates").unwrap(),
        );
        combined.leak()
    }
}

#[async_trait]
impl PumpkinBlock for PressurePlateBlock {
    async fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        self.on_entity_collision_pp(args).await;
    }

    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        self.on_scheduled_tick_pp(args).await;
    }

    async fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        self.on_state_replaced_pp(args).await;
    }

    async fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        self.get_redstone_output(args.block, args.state.id)
    }

    async fn get_strong_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        if args.direction == BlockDirection::Up {
            return self.get_redstone_output(args.block, args.state.id);
        }
        0
    }

    async fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }
}

impl PressurePlate for PressurePlateBlock {
    fn get_redstone_output(&self, block: &Block, state: BlockStateId) -> u8 {
        let props = PressurePlateProps::from_state_id(state, block);
        if props.powered { 15 } else { 0 }
    }

    async fn calculate_redstone_output(&self, world: &World, _block: &Block, pos: &BlockPos) -> u8 {
        // TODO: this is bad use real box
        let aabb = BoundingBox::from_block(pos);
        if !world.get_entities_at_box(&aabb).await.is_empty()
            || !world.get_players_at_box(&aabb).await.is_empty()
        {
            return 15;
        }
        0
    }

    fn set_redstone_output(
        &self,
        block: &Block,
        state: &BlockState,
        output: u8,
    ) -> pumpkin_world::BlockStateId {
        let mut props = PressurePlateProps::from_state_id(state.id, block);
        props.powered = output > 0;
        props.to_state_id(block)
    }
}
