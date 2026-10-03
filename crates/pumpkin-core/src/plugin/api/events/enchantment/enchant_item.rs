use pumpkin_data::Enchantment;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

/// An event triggered when an item is enchanted at an enchanting table.
#[cancellable]
#[derive(Event, Clone)]
pub struct EnchantItemEvent {
    /// The player enchanting the item.
    pub player: Arc<Player>,

    /// The item being enchanted.
    pub item: ItemStack,

    /// The button index selected (0, 1, or 2).
    pub option: i32,

    /// The cost in experience levels for the enchantment.
    pub exp_level_cost: i32,

    /// The list of enchantments and levels to apply.
    pub enchantments_to_add: Vec<(&'static Enchantment, i32)>,
}

impl EnchantItemEvent {
    #[must_use]
    pub const fn new(
        player: Arc<Player>,
        item: ItemStack,
        option: i32,
        exp_level_cost: i32,
        enchantments_to_add: Vec<(&'static Enchantment, i32)>,
    ) -> Self {
        Self {
            player,
            item,
            option,
            exp_level_cost,
            enchantments_to_add,
            cancelled: false,
        }
    }
}
