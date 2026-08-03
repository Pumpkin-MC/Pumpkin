use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, GetComparatorOutputArgs, OnEntityCollisionArgs,
};
use crate::world::World;
use crate::world::game_event::{GameEventContext, emit_game_event};
use pumpkin_data::block_properties::{BlockProperties, WaterCauldronLikeProperties};
use pumpkin_data::damage::DamageType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::{Block, BlockId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

pub struct CauldronBlock;

impl BlockMetadata for CauldronBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::CAULDRON,
            BlockId::WATER_CAULDRON,
            BlockId::LAVA_CAULDRON,
            BlockId::POWDER_SNOW_CAULDRON,
        ]
        .into()
    }
}

impl BlockBehaviour for CauldronBlock {
    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let entity = args.entity.get_entity();
            match args.block.id {
                BlockId::LAVA_CAULDRON => {
                    if !entity.entity_type.fire_immune
                        && !entity.fire_immune.load(Ordering::Relaxed)
                    {
                        args.entity.set_on_fire_for(15.0);
                        args.entity.damage(args.entity, 4.0, DamageType::LAVA).await;
                    }
                }
                BlockId::WATER_CAULDRON | BlockId::POWDER_SNOW_CAULDRON => {
                    if entity.fire_ticks.load(Ordering::Relaxed) > 0 {
                        lower_fill_level(args.world, args.position, args.block).await;
                    }
                    entity.extinguish();
                }
                _ => {}
            }
        })
    }

    fn get_comparator_output<'a>(
        &'a self,
        args: GetComparatorOutputArgs<'a>,
    ) -> BlockFuture<'a, Option<u8>> {
        Box::pin(async move {
            match args.block.id {
                BlockId::WATER_CAULDRON | BlockId::POWDER_SNOW_CAULDRON => {
                    let state_id = args.world.get_block_state_id(args.position);
                    let props = WaterCauldronLikeProperties::from_state_id(state_id, args.block);
                    Some(props.level)
                }
                BlockId::LAVA_CAULDRON => Some(3),
                _ => Some(0),
            }
        })
    }
}

/// `LayeredCauldronBlock#lowerFillLevel`, preceded by the powder-snow conversion
/// `handleEntityOnFireInside` does: a burning entity melts powder snow into water first,
/// so a powder snow cauldron always leaves a water cauldron behind.
async fn lower_fill_level(world: &Arc<World>, position: &BlockPos, block: &Block) {
    let state_id = world.get_block_state_id(position);
    let level = WaterCauldronLikeProperties::from_state_id(state_id, block).level;

    let new_state_id = if level <= 1 {
        Block::CAULDRON.default_state.id
    } else {
        let mut props = WaterCauldronLikeProperties::default(&Block::WATER_CAULDRON);
        props.level = level - 1;
        props.to_state_id(&Block::WATER_CAULDRON)
    };

    world
        .set_block_state(position, new_state_id, BlockFlags::NOTIFY_ALL)
        .await;
    emit_game_event(
        world,
        GameEvent::BlockChange,
        position.to_centered_f64(),
        GameEventContext::none(),
    )
    .await;
}
