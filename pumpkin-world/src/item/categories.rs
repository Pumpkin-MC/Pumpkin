use pumpkin_data::tag::Tagable;

use crate::item::ItemStack;

const SWORDS_TAG: &str = "#minecraft:swords";
const HEAD_ARMOR_TAG: &str = "#minecraft:head_armor";
const SKULL_TAG: &str = "#minecraft:skulls";
const CHEST_ARMOR_TAG: &str = "#minecraft:chest_armor";
const LEG_ARMOR_TAG: &str = "#minecraft:leg_armor";
const FOOT_ARMOR_TAG: &str = "#minecraft:foot_armor";

impl ItemStack {
    #[inline]
    pub fn is_sword(&self) -> bool {
        self.item.is_tagged_with(SWORDS_TAG).expect(
            "This is a default minecraft tag that should have been gotten from the extractor",
        )
    }

    #[inline]
    pub fn is_helmet(&self) -> bool {
        self.item.is_tagged_with(HEAD_ARMOR_TAG).expect(
            "This is a default minecraft tag that should have been gotten from the extractor",
        )
    }

    #[inline]
    pub fn is_skull(&self) -> bool {
        self.item.is_tagged_with(SKULL_TAG).expect(
            "This is a default minecraft tag that should have been gotten from the extractor",
        )
    }

    #[inline]
    pub fn is_chestplate(&self) -> bool {
        self.item.is_tagged_with(CHEST_ARMOR_TAG).expect(
            "This is a default minecraft tag that should have been gotten from the extractor",
        )
    }

    #[inline]
    pub fn is_leggings(&self) -> bool {
        self.item.is_tagged_with(LEG_ARMOR_TAG).expect(
            "This is a default minecraft tag that should have been gotten from the extractor",
        )
    }

    #[inline]
    pub fn is_boots(&self) -> bool {
        self.item.is_tagged_with(FOOT_ARMOR_TAG).expect(
            "This is a default minecraft tag that should have been gotten from the extractor",
        )
    }
}
