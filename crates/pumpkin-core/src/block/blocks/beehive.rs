use std::sync::Arc;

use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, BlockMetadata, GetComparatorOutputArgs, UseWithItemArgs};
use crate::entity::Entity;
use crate::entity::item::ItemEntity;
use pumpkin_data::block_properties::BeeNestLikeProperties;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{Block, BlockId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

pub struct BeehiveBlock;

impl BlockMetadata for BeehiveBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::BEEHIVE, BlockId::BEE_NEST].into()
    }
}

pub fn is_smokey_pos(world: &Arc<crate::world::World>, pos: BlockPos) -> bool {
    use pumpkin_data::block_properties::CampfireLikeProperties;
    for i in 1..=5 {
        let check_pos = pos.down_height(i);
        let (block, state_id) = world.get_block_and_state_id(&check_pos);
        if block == &Block::CAMPFIRE || block == &Block::SOUL_CAMPFIRE {
            let props = CampfireLikeProperties::from_state_id(state_id);
            if props.lit {
                return true;
            }
        }
    }
    false
}

impl BlockBehaviour for BeehiveBlock {
    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        let state_id = args.world.get_block_state_id(args.position);
        let props = BeeNestLikeProperties::from_state_id(state_id);
        Some(props.honey_level)
    }

    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position);
        let props = BeeNestLikeProperties::from_state_id(state_id);
        if props.honey_level < 5 {
            return BlockActionResult::Pass;
        }

        let item = args.item_stack.get_item();
        if item == &Item::SHEARS {
            let entity = Entity::new(
                args.world.clone(),
                args.position.to_f64() + Vector3::new(0.5, 0.5, 0.5),
                &EntityType::ITEM,
            );
            let item_entity = Arc::new(ItemEntity::new(
                entity,
                ItemStack::new(3, &Item::HONEYCOMB),
            ));
            args.world.spawn_entity(item_entity);

            args.world.play_block_sound(
                pumpkin_data::sound::Sound::BlockBeehiveShear,
                pumpkin_data::sound::SoundCategory::Blocks,
                *args.position,
            );
            args.player.damage_item_in_slot(args.equipment_slot, 1);
            args.player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Used,
                Item::SHEARS.id as i32,
                1,
            );

            let mut new_props = props;
            new_props.honey_level = 0;
            let (block, _) = args.world.get_block_and_state_id(args.position);
            args.world.set_block_state(
                args.position,
                new_props.to_state_id(block),
                BlockFlags::NOTIFY_ALL,
            );

            return BlockActionResult::Success;
        } else if item == &Item::GLASS_BOTTLE {
            args.item_stack.decrement_unless_creative(args.player.gamemode.load(), 1);

            let mut honey_bottle = ItemStack::new(1, &Item::HONEY_BOTTLE);
            if !args.player.inventory().insert_stack_anywhere(&mut honey_bottle) && !honey_bottle.is_empty() {
                let entity = Entity::new(
                    args.world.clone(),
                    args.position.to_f64() + Vector3::new(0.5, 0.5, 0.5),
                    &EntityType::ITEM,
                );
                let item_entity = Arc::new(ItemEntity::new(entity, honey_bottle));
                args.world.spawn_entity(item_entity);
            }

            args.world.play_block_sound(
                pumpkin_data::sound::Sound::ItemBottleFill,
                pumpkin_data::sound::SoundCategory::Blocks,
                *args.position,
            );
            args.player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Used,
                Item::GLASS_BOTTLE.id as i32,
                1,
            );

            let is_smokey = is_smokey_pos(args.world, *args.position);
            if is_smokey {
                args.player.trigger_advancement(
                    crate::entity::player::advancement::trigger::AdvancementTrigger::SafelyHarvestHoney,
                );
            }

            let mut new_props = props;
            new_props.honey_level = 0;
            let (block, _) = args.world.get_block_and_state_id(args.position);
            args.world.set_block_state(
                args.position,
                new_props.to_state_id(block),
                BlockFlags::NOTIFY_ALL,
            );

            return BlockActionResult::Success;
        }

        BlockActionResult::Pass
    }
}
