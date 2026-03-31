use crate::tag;
use crate::tag::Taggable;

use crate::item_stack::ItemStack;

impl ItemStack {
    #[inline]
    #[must_use]
    pub fn is_sword(&self) -> bool {
        self.item.has_tag(&tag::Item::MINECRAFT_SWORDS)
    }

    #[inline]
    #[must_use]
    pub fn is_helmet(&self) -> bool {
        self.item.has_tag(&tag::Item::MINECRAFT_HEAD_ARMOR)
    }

    #[inline]
    #[must_use]
    pub fn is_skull(&self) -> bool {
        self.item.has_tag(&tag::Item::MINECRAFT_SKULLS)
    }

    #[inline]
    #[must_use]
    pub fn is_chestplate(&self) -> bool {
        self.item.has_tag(&tag::Item::MINECRAFT_CHEST_ARMOR)
    }

    #[inline]
    #[must_use]
    pub fn is_leggings(&self) -> bool {
        self.item.has_tag(&tag::Item::MINECRAFT_LEG_ARMOR)
    }

    #[inline]
    #[must_use]
    pub fn is_boots(&self) -> bool {
        self.item.has_tag(&tag::Item::MINECRAFT_FOOT_ARMOR)
    }

    /// `true` if item is in `#minecraft:enchantable/armor` (selects armor Unbreaking formula).
    #[inline]
    #[must_use]
    pub fn is_armor(&self) -> bool {
        self.item.has_tag(&tag::Item::MINECRAFT_ENCHANTABLE_ARMOR)
    }

    /// Test-only predicates: identify 2-durability tools (axes/pickaxes/shovels/hoes).
    /// In production, durability cost is data-driven via the `Weapon` component.
    /// These helpers exist only to validate item categorization in tests.
    #[cfg(test)]
    #[inline]
    #[must_use]
    pub fn is_axe(&self) -> bool {
        self.item.has_tag(&tag::Item::MINECRAFT_AXES)
    }

    #[cfg(test)]
    #[inline]
    #[must_use]
    pub fn is_pickaxe(&self) -> bool {
        self.item.has_tag(&tag::Item::MINECRAFT_PICKAXES)
    }

    #[cfg(test)]
    #[inline]
    #[must_use]
    pub fn is_shovel(&self) -> bool {
        self.item.has_tag(&tag::Item::MINECRAFT_SHOVELS)
    }

    #[cfg(test)]
    #[inline]
    #[must_use]
    pub fn is_hoe(&self) -> bool {
        self.item.has_tag(&tag::Item::MINECRAFT_HOES)
    }
}
