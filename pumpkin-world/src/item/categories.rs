use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;

use crate::item::ItemStack;

impl ItemStack {
    #[inline]
    pub fn is_sword(&self) -> bool {
        self.item.has_tag(&tag::Item::MINECRAFT_SWORDS)
    }

    #[inline]
    pub fn is_helmet(&self) -> bool {
        self.item.has_tag(&tag::Item::MINECRAFT_HEAD_ARMOR)
    }

    #[inline]
    pub fn is_skull(&self) -> bool {
        self.item.has_tag(&tag::Item::MINECRAFT_SKULLS)
    }

    #[inline]
    pub fn is_chestplate(&self) -> bool {
        self.item.has_tag(&tag::Item::MINECRAFT_CHEST_ARMOR)
    }

    #[inline]
    pub fn is_leggings(&self) -> bool {
        self.item.has_tag(&tag::Item::MINECRAFT_LEG_ARMOR)
    }

    #[inline]
    pub fn is_boots(&self) -> bool {
        self.item.has_tag(&tag::Item::MINECRAFT_FOOT_ARMOR)
    }
}
