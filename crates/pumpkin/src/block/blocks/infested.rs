use std::sync::Arc;

use pumpkin_data::Enchantment;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::GameMode;

use crate::block::BlockBehaviour;
use crate::block::BrokenArgs;
use crate::entity::Entity;

#[pumpkin_block_from_tag("c:cobblestones/infested")]
pub struct InfestedBlock;

impl BlockBehaviour for InfestedBlock {
    fn broken(&self, args: BrokenArgs<'_>) {
        {
            // TODO: ugly fix, use onStacksDropped
            if !should_spawn_silverfish(
                args.player.gamemode.load(),
                &args.player.inventory().held_item(),
            ) {
                return;
            }
            let entity = Entity::new(
                args.world.clone(),
                args.position.0.to_f64(),
                &EntityType::SILVERFISH,
            );

            args.world.spawn_entity(Arc::new(entity));
        }
    }
}

fn should_spawn_silverfish(gamemode: GameMode, held_item: &ItemStack) -> bool {
    gamemode != GameMode::Creative && held_item.get_enchantment_level(&Enchantment::SILK_TOUCH) == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::item::Item;

    #[test]
    fn silk_touch_and_creative_skip_silverfish() {
        let mut stack = ItemStack::new(1, &Item::STONE_PICKAXE);
        stack.add_enchantment(&Enchantment::SILK_TOUCH, 1);
        assert!(!should_spawn_silverfish(GameMode::Survival, &stack));

        let plain = ItemStack::new(1, &Item::STONE_PICKAXE);
        assert!(!should_spawn_silverfish(GameMode::Creative, &plain));
    }

    #[test]
    fn regular_breaks_spawn_silverfish() {
        let stack = ItemStack::new(1, &Item::STONE_PICKAXE);
        assert!(should_spawn_silverfish(GameMode::Survival, &stack));

        let mut enchanted = ItemStack::new(1, &Item::STONE_PICKAXE);
        enchanted.add_enchantment(&Enchantment::EFFICIENCY, 3);
        assert!(should_spawn_silverfish(GameMode::Survival, &enchanted));
    }
}
