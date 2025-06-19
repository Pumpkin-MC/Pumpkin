use std::collections::HashMap;

use pumpkin_data::{item::Item, tag::Tagable};

#[derive(Debug)]
pub struct FuelRegistry {
    fuel_values: HashMap<&'static str, i32>,
    fuel_values_tag: HashMap<&'static str, i32>,
}
impl Default for FuelRegistry {
    fn default() -> Self {
        let fuel_values = Self::default_fuel_values();
        let fuel_values_tag = Self::default_fuel_values_tag();

        Self {
            fuel_values,
            fuel_values_tag,
        }
    }
}

impl FuelRegistry {
    const ITEM_SMELT_TIME: i32 = 200;

    pub fn is_fuel(&self, item: &Item) -> bool {
        if self.fuel_values.contains_key(item.registry_key()) {
            return true;
        }

        for fuel_tag in self.fuel_values_tag.keys() {
            if item.is_tagged_with(fuel_tag) == Some(true) {
                return true;
            }
        }

        false
    }

    pub fn get_fuel_tick(&self, item: &Item) -> Option<i32> {
        if let Some(fuel_tick) = self.fuel_values.get(item.registry_key) {
            return Some(*fuel_tick);
        }

        for fuel_tag in self.fuel_values_tag.keys() {
            if item.is_tagged_with(fuel_tag) == Some(true) {
                return self.fuel_values_tag.get(fuel_tag).copied();
            }
        }

        None
    }

    fn default_fuel_values_tag() -> HashMap<&'static str, i32> {
        let mut fuel_values = HashMap::new();
        fuel_values.insert("minecraft:logs", Self::ITEM_SMELT_TIME * 3 / 2);
        fuel_values.insert("minecraft:bamboo_blocks", Self::ITEM_SMELT_TIME * 3 / 2);
        fuel_values.insert("minecraft:planks", Self::ITEM_SMELT_TIME * 3 / 2);
        fuel_values.insert("minecraft:wooden_stairs", Self::ITEM_SMELT_TIME * 3 / 2);
        fuel_values.insert("minecraft:wooden_slabs", Self::ITEM_SMELT_TIME * 3 / 4);
        fuel_values.insert("minecraft:wooden_trapdoors", Self::ITEM_SMELT_TIME * 3 / 2);
        fuel_values.insert(
            "minecraft:wooden_pressure_plates",
            Self::ITEM_SMELT_TIME * 3 / 2,
        );
        fuel_values.insert("minecraft:wooden_fences", Self::ITEM_SMELT_TIME * 3 / 2);
        fuel_values.insert("minecraft:fence_gates", Self::ITEM_SMELT_TIME * 3 / 2);
        fuel_values.insert("minecraft:banners", Self::ITEM_SMELT_TIME * 3 / 2);
        fuel_values.insert("minecraft:signs", Self::ITEM_SMELT_TIME);
        fuel_values.insert("minecraft:hanging_signs", Self::ITEM_SMELT_TIME * 4);
        fuel_values.insert("minecraft:wooden_doors", Self::ITEM_SMELT_TIME);
        fuel_values.insert("minecraft:boats", Self::ITEM_SMELT_TIME * 6);
        fuel_values.insert("minecraft:wool", Self::ITEM_SMELT_TIME / 2);
        fuel_values.insert("minecraft:wooden_buttons", Self::ITEM_SMELT_TIME / 2);
        fuel_values.insert("minecraft:saplings", Self::ITEM_SMELT_TIME / 2);
        fuel_values.insert("minecraft:wool_carpets", 1 + Self::ITEM_SMELT_TIME / 3);

        fuel_values
    }

    fn default_fuel_values() -> HashMap<&'static str, i32> {
        let mut fuel_values = HashMap::new();
        fuel_values.insert(Item::LAVA_BUCKET.registry_key, Self::ITEM_SMELT_TIME * 100);
        fuel_values.insert(
            Item::COAL_BLOCK.registry_key,
            Self::ITEM_SMELT_TIME * 8 * 10,
        );
        fuel_values.insert(Item::BLAZE_ROD.registry_key, Self::ITEM_SMELT_TIME * 12);
        fuel_values.insert(Item::COAL.registry_key, Self::ITEM_SMELT_TIME * 8);
        fuel_values.insert(Item::CHARCOAL.registry_key, Self::ITEM_SMELT_TIME * 8);
        fuel_values.insert(
            Item::BAMBOO_MOSAIC.registry_key,
            Self::ITEM_SMELT_TIME * 3 / 2,
        );
        fuel_values.insert(
            Item::BAMBOO_MOSAIC_STAIRS.registry_key,
            Self::ITEM_SMELT_TIME * 3 / 2,
        );
        fuel_values.insert(
            Item::BAMBOO_MOSAIC_SLAB.registry_key,
            Self::ITEM_SMELT_TIME * 3 / 4,
        );
        fuel_values.insert(Item::NOTE_BLOCK.registry_key, Self::ITEM_SMELT_TIME * 3 / 2);
        fuel_values.insert(Item::BOOKSHELF.registry_key, Self::ITEM_SMELT_TIME * 3 / 2);
        fuel_values.insert(
            Item::CHISELED_BOOKSHELF.registry_key,
            Self::ITEM_SMELT_TIME * 3 / 2,
        );
        fuel_values.insert(Item::LECTERN.registry_key, Self::ITEM_SMELT_TIME * 3 / 2);
        fuel_values.insert(Item::JUKEBOX.registry_key, Self::ITEM_SMELT_TIME * 3 / 2);
        fuel_values.insert(Item::CHEST.registry_key, Self::ITEM_SMELT_TIME * 3 / 2);
        fuel_values.insert(
            Item::TRAPPED_CHEST.registry_key,
            Self::ITEM_SMELT_TIME * 3 / 2,
        );
        fuel_values.insert(
            Item::CRAFTING_TABLE.registry_key,
            Self::ITEM_SMELT_TIME * 3 / 2,
        );
        fuel_values.insert(
            Item::DAYLIGHT_DETECTOR.registry_key,
            Self::ITEM_SMELT_TIME * 3 / 2,
        );
        fuel_values.insert(Item::BOW.registry_key, Self::ITEM_SMELT_TIME * 3 / 2);
        fuel_values.insert(
            Item::FISHING_ROD.registry_key,
            Self::ITEM_SMELT_TIME * 3 / 2,
        );
        fuel_values.insert(Item::LADDER.registry_key, Self::ITEM_SMELT_TIME * 3 / 2);
        fuel_values.insert(Item::WOODEN_SHOVEL.registry_key, Self::ITEM_SMELT_TIME);
        fuel_values.insert(Item::WOODEN_SWORD.registry_key, Self::ITEM_SMELT_TIME);
        fuel_values.insert(Item::WOODEN_HOE.registry_key, Self::ITEM_SMELT_TIME);
        fuel_values.insert(Item::WOODEN_AXE.registry_key, Self::ITEM_SMELT_TIME);
        fuel_values.insert(Item::WOODEN_PICKAXE.registry_key, Self::ITEM_SMELT_TIME);
        fuel_values.insert(Item::STICK.registry_key, Self::ITEM_SMELT_TIME / 2);
        fuel_values.insert(Item::BOWL.registry_key, Self::ITEM_SMELT_TIME / 2);
        fuel_values.insert(
            Item::DRIED_KELP_BLOCK.registry_key,
            1 + Self::ITEM_SMELT_TIME * 20,
        );
        fuel_values.insert(Item::CROSSBOW.registry_key, Self::ITEM_SMELT_TIME * 3 / 2);
        fuel_values.insert(Item::BAMBOO.registry_key, Self::ITEM_SMELT_TIME / 4);
        fuel_values.insert(Item::DEAD_BUSH.registry_key, Self::ITEM_SMELT_TIME / 2);
        fuel_values.insert(
            Item::SHORT_DRY_GRASS.registry_key,
            Self::ITEM_SMELT_TIME / 2,
        );
        fuel_values.insert(Item::TALL_DRY_GRASS.registry_key, Self::ITEM_SMELT_TIME / 2);
        fuel_values.insert(Item::SCAFFOLDING.registry_key, Self::ITEM_SMELT_TIME / 4);
        fuel_values.insert(Item::LOOM.registry_key, Self::ITEM_SMELT_TIME * 3 / 2);
        fuel_values.insert(Item::BARREL.registry_key, Self::ITEM_SMELT_TIME * 3 / 2);
        fuel_values.insert(
            Item::CARTOGRAPHY_TABLE.registry_key,
            Self::ITEM_SMELT_TIME * 3 / 2,
        );
        fuel_values.insert(
            Item::FLETCHING_TABLE.registry_key,
            Self::ITEM_SMELT_TIME * 3 / 2,
        );
        fuel_values.insert(
            Item::SMITHING_TABLE.registry_key,
            Self::ITEM_SMELT_TIME * 3 / 2,
        );
        fuel_values.insert(Item::COMPOSTER.registry_key, Self::ITEM_SMELT_TIME * 3 / 2);
        fuel_values.insert(Item::AZALEA.registry_key, Self::ITEM_SMELT_TIME / 2);
        fuel_values.insert(
            Item::FLOWERING_AZALEA.registry_key,
            Self::ITEM_SMELT_TIME / 2,
        );
        fuel_values.insert(
            Item::MANGROVE_ROOTS.registry_key,
            Self::ITEM_SMELT_TIME * 3 / 2,
        );
        fuel_values.insert(Item::LEAF_LITTER.registry_key, Self::ITEM_SMELT_TIME / 2);

        fuel_values
    }
}
