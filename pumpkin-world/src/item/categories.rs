use pumpkin_data::item::Item;

use crate::item::ItemStack;

impl ItemStack {
    pub fn is_sword(&self) -> bool {
        [
            Item::WOODEN_SWORD.id,    // Wooden
            Item::STONE_SWORD.id,     // Stone
            Item::GOLDEN_SWORD.id,    // Gold
            Item::IRON_SWORD.id,      // Iron
            Item::DIAMOND_SWORD.id,   // Diamond
            Item::NETHERITE_SWORD.id, // Netherite
        ]
        .contains(&self.item.id)
    }

    pub fn is_helmet(&self) -> bool {
        [
            // Leather
            Item::NETHERITE_HELMET.id, // Netherite
            Item::TURTLE_HELMET.id,    // Turtle helmet
            Item::CHAINMAIL_HELMET.id, // Chainmail
            Item::DIAMOND_HELMET.id,   // Diamond
            Item::GOLDEN_HELMET.id,    // Gold
            Item::IRON_HELMET.id,      // Iron
            Item::LEATHER_HELMET.id,
        ]
        .contains(&self.item.id)
    }

    pub fn is_chestplate(&self) -> bool {
        [
            // Leather
            Item::NETHERITE_CHESTPLATE.id, // Netherite
            Item::CHAINMAIL_CHESTPLATE.id, // Chainmail
            Item::DIAMOND_CHESTPLATE.id,   // Diamond
            Item::GOLDEN_CHESTPLATE.id,    // Gold
            Item::IRON_CHESTPLATE.id,      // Iron
            Item::ELYTRA.id,               // Elytra
            Item::LEATHER_CHESTPLATE.id,
        ]
        .contains(&self.item.id)
    }

    pub fn is_leggings(&self) -> bool {
        [
            // Leather
            Item::NETHERITE_LEGGINGS.id, // Netherite
            Item::CHAINMAIL_LEGGINGS.id, // Chainmail
            Item::DIAMOND_LEGGINGS.id,   // Diamond
            Item::GOLDEN_LEGGINGS.id,    // Gold
            Item::IRON_LEGGINGS.id,      // Iron
            Item::LEATHER_LEGGINGS.id,
        ]
        .contains(&self.item.id)
    }

    pub fn is_boots(&self) -> bool {
        [
            // Leather
            Item::NETHERITE_BOOTS.id, // Netherite
            Item::CHAINMAIL_BOOTS.id, // Chainmail
            Item::DIAMOND_BOOTS.id,   // Diamond
            Item::GOLDEN_BOOTS.id,    // Gold
            Item::IRON_BOOTS.id,      // Iron
            Item::LEATHER_BOOTS.id,
        ]
        .contains(&self.item.id)
    }
}
