use std::sync::Arc;

use pumpkin_data::{
    Block, BlockId, BlockState, BlockStateId, Enchantment, Rotation,
    block_properties::{Axis, PaleOakWoodLikeProperties},
    entity::EntityType,
};
use pumpkin_util::{GameMode, math::vector3::Vector3};

use crate::{
    block::{BlockBehaviour, BlockMetadata, BrokenArgs, OnPlaceArgs},
    entity::Entity,
};

pub struct InfestedBlock;

impl BlockMetadata for InfestedBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::INFESTED_STONE,
            BlockId::INFESTED_COBBLESTONE,
            BlockId::INFESTED_STONE_BRICKS,
            BlockId::INFESTED_MOSSY_STONE_BRICKS,
            BlockId::INFESTED_CRACKED_STONE_BRICKS,
            BlockId::INFESTED_CHISELED_STONE_BRICKS,
            BlockId::INFESTED_DEEPSLATE,
        ]
        .into()
    }
}

impl BlockBehaviour for InfestedBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if args.block == &Block::INFESTED_DEEPSLATE {
            let mut props = PaleOakWoodLikeProperties::default(args.block);
            props.axis = args.direction.to_axis();
            props.to_state_id(args.block)
        } else {
            args.block.default_state.id
        }
    }

    fn rotate(
        &self,
        block: &Block,
        state_id: BlockStateId,
        rotation: Rotation,
    ) -> &'static BlockState {
        if block == &Block::INFESTED_DEEPSLATE {
            let mut props = PaleOakWoodLikeProperties::from_state_id(state_id);
            if matches!(
                rotation,
                Rotation::Clockwise90 | Rotation::CounterClockwise90
            ) {
                props.axis = match props.axis {
                    Axis::X => Axis::Z,
                    Axis::Z => Axis::X,
                    Axis::Y => Axis::Y,
                };
            }
            BlockState::from_id(props.to_state_id(block))
        } else {
            BlockState::from_id(state_id)
        }
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        // TODO: ugly fix, use onStacksDropped
        if args.player.gamemode.load() == GameMode::Creative
            || !args.world.level_info.load().game_rules.block_drops
            || args
                .player
                .inventory()
                .held_item()
                .get_enchantment_level(&Enchantment::SILK_TOUCH)
                > 0
        {
            return;
        }
        let pos = Vector3::new(
            f64::from(args.position.0.x) + 0.5,
            f64::from(args.position.0.y),
            f64::from(args.position.0.z) + 0.5,
        );
        let entity = Entity::new(args.world.clone(), pos, &EntityType::SILVERFISH);

        args.world.spawn_entity(Arc::new(entity));
    }
}
