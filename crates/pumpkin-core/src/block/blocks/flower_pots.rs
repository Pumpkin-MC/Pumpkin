use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, NormalUseArgs, PathComputationType, RandomTickArgs, UseWithItemArgs,
};
use pumpkin_data::flower_pot_transformations::get_potted_item;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockId, BlockState};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::world::BlockFlags;

#[pumpkin_block_from_tag("minecraft:flower_pots")]
pub struct FlowerPotBlock;

fn get_plant_item_from_potted_block(block: &Block) -> Option<&'static Item> {
    let suffix = block.name.strip_prefix("potted_")?;
    let item_key = match suffix {
        "azalea_bush" => "azalea",
        "flowering_azalea_bush" => "flowering_azalea",
        other => other,
    };
    Item::from_registry_key(item_key)
}

impl BlockBehaviour for FlowerPotBlock {
    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let item = args.item_stack.item;
        let potted_block_id = get_potted_item(item.id);
        if potted_block_id == BlockId::AIR {
            return BlockActionResult::PassToDefaultBlockAction;
        }

        if !args.block.eq(&Block::FLOWER_POT) {
            return BlockActionResult::Consume;
        }

        args.world.set_block_state(
            args.position,
            Block::from_id(potted_block_id).default_state.id,
            BlockFlags::NOTIFY_ALL,
        );
        args.world.emit_game_event(
            GameEvent::BlockChange.name(),
            args.position.to_centered_f64(),
        );
        args.player.increment_stat(
            pumpkin_data::statistic::StatisticCategory::Custom,
            pumpkin_data::statistic::CustomStatistic::PotFlower as i32,
            1,
        );
        args.item_stack
            .decrement_unless_creative(args.player.gamemode.load(), 1);
        BlockActionResult::Success
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if args.block.eq(&Block::FLOWER_POT) {
            return BlockActionResult::Consume;
        }

        if let Some(plant_item) = get_plant_item_from_potted_block(args.block) {
            let mut stack = ItemStack::new(1, plant_item);
            args.player.inventory.insert_stack_anywhere(&mut stack);
            if !stack.is_empty() {
                args.player.drop_item(stack);
            }
        }

        args.world.set_block_state(
            args.position,
            Block::FLOWER_POT.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );
        args.world.emit_game_event(
            GameEvent::BlockChange.name(),
            args.position.to_centered_f64(),
        );
        BlockActionResult::Success
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        let is_open_potted = args.block.eq(&Block::POTTED_OPEN_EYEBLOSSOM);
        let is_closed_potted = args.block.eq(&Block::POTTED_CLOSED_EYEBLOSSOM);
        if !is_open_potted && !is_closed_potted {
            return;
        }

        let is_open = is_open_potted;
        let should_be_open = args.world.eyeblossom_open(args.position).unwrap_or(is_open);

        if is_open != should_be_open {
            let (next_block, sound) = if should_be_open {
                (
                    &Block::POTTED_OPEN_EYEBLOSSOM,
                    Sound::BlockEyeblossomOpenLong,
                )
            } else {
                (
                    &Block::POTTED_CLOSED_EYEBLOSSOM,
                    Sound::BlockEyeblossomCloseLong,
                )
            };
            args.world.set_block_state(
                args.position,
                next_block.default_state.id,
                BlockFlags::NOTIFY_ALL,
            );
            args.world.play_sound_fine(
                sound,
                SoundCategory::Blocks,
                &args.position.to_centered_f64(),
                1.0,
                1.0,
            );
        }
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}
