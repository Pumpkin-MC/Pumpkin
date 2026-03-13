use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component::DataComponent::Enchantments;
use pumpkin_data::data_component_impl::{
    BlocksAttacksImpl, ConsumableImpl, DamageImpl, DataComponentImpl, EnchantmentsImpl, IDSet,
    MaxDamageImpl, MaxStackSizeImpl, ToolImpl, UnbreakableImpl, get, get_mut, read_data,
};
use pumpkin_data::item::Item;
use pumpkin_data::recipes::RecipeResultStruct;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, Enchantment};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::GameMode;
use std::borrow::Cow;
use std::cmp::{max, min};

mod categories;

/// The outcome of a [`ItemStack::damage_item`] call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageResult {
    /// No damage was applied (zero/negative amount, not damageable, unbreakable,
    /// or Unbreaking negated every point).
    Untouched,
    /// Damage was applied and the item is still alive.
    Damaged,
    /// The item broke: either the stack became empty or a multi-item stack lost
    /// one count (durability resets to 0).
    Broken,
}

#[derive(Clone)]
pub struct ItemStack {
    pub item_count: u8,
    pub item: &'static Item,
    pub patch: Vec<(DataComponent, Option<Box<dyn DataComponentImpl>>)>,
}

// impl Hash for ItemStack {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.item_count.hash(state);
//         self.item.id.hash(state);
//         self.patch.hash(state);
//     }
// }

/*
impl PartialEq for ItemStack {
    fn eq(&self, other: &Self) -> bool {
        self.item.id == other.item.id
    }
} */

impl ItemStack {
    #[must_use]
    pub fn new(item_count: u8, item: &'static Item) -> Self {
        Self {
            item_count,
            item,
            patch: Vec::new(),
        }
    }

    #[must_use]
    pub fn new_with_component(
        item_count: u8,
        item: &'static Item,
        component: Vec<(DataComponent, Option<Box<dyn DataComponentImpl>>)>,
    ) -> Self {
        Self {
            item_count,
            item,
            patch: component,
        }
    }

    #[must_use]
    pub fn get_data_component<T: DataComponentImpl + 'static>(&self) -> Option<&T> {
        let to_get_id = &T::get_enum();
        for (id, component) in &self.patch {
            if id == to_get_id {
                return component
                    .as_ref()
                    .map(|component| get::<T>(component.as_ref()));
            }
        }
        for (id, component) in self.item.components {
            if id == to_get_id {
                return Some(get::<T>(*component));
            }
        }
        None
    }
    pub fn get_data_component_mut<T: DataComponentImpl + 'static>(&mut self) -> Option<&mut T> {
        let to_get_id = &T::get_enum();
        for (id, component) in &mut self.patch {
            if id == to_get_id {
                return component
                    .as_mut()
                    .map(|component| get_mut::<T>(component.as_mut()));
            }
        }
        None
    }

    pub const EMPTY: &'static Self = &Self {
        item_count: 0,
        item: &Item::AIR,
        patch: Vec::new(),
    };

    #[must_use]
    pub fn get_max_stack_size(&self) -> u8 {
        self.get_data_component::<MaxStackSizeImpl>()
            .map_or(1, |value| value.size)
    }

    #[must_use]
    pub fn get_max_damage(&self) -> Option<i32> {
        self.get_data_component::<MaxDamageImpl>()
            .map(|value| value.max_damage)
    }

    #[must_use]
    pub fn get_damage(&self) -> i32 {
        self.get_data_component::<DamageImpl>()
            .map_or(0, |value| value.damage)
    }

    #[must_use]
    pub fn get_enchantment_level(&self, enchantment: &'static Enchantment) -> i32 {
        let Some(data) = self.get_data_component::<EnchantmentsImpl>() else {
            return 0;
        };
        for (enc, level) in data.enchantment.iter() {
            if *enc == enchantment {
                return *level;
            }
        }
        0
    }

    #[must_use]
    pub fn is_unbreakable(&self) -> bool {
        self.get_data_component::<UnbreakableImpl>().is_some()
    }

    pub fn set_damage(&mut self, damage: i32) {
        let damage = damage.max(0);
        if damage == 0 {
            self.patch.retain(|(id, _)| *id != DataComponent::Damage);
            return;
        }

        for (id, component) in &mut self.patch {
            if *id == DataComponent::Damage {
                *component = Some(DamageImpl { damage }.to_dyn());
                return;
            }
        }

        self.patch
            .push((DataComponent::Damage, Some(DamageImpl { damage }.to_dyn())));
    }

    #[must_use]
    pub fn is_damageable(&self) -> bool {
        self.get_max_damage().unwrap_or(0) > 0
    }

    pub fn repair_item(&mut self, amount: i32) -> i32 {
        if amount <= 0 {
            return 0;
        }
        let damage = self.get_damage();
        if damage <= 0 {
            return 0;
        }
        let repaired = amount.min(damage);
        self.set_damage(damage - repaired);
        repaired
    }

    fn should_apply_durability_damage(&self) -> bool {
        let unbreaking_level = self.get_enchantment_level(&Enchantment::UNBREAKING);
        if unbreaking_level <= 0 {
            return true;
        }

        // `#minecraft:enchantable/armor` uses the armor formula; all others use the tool formula.
        if self.is_armor() {
            let chance = 0.6 + (0.4 / (unbreaking_level as f32 + 1.0));
            rand::random::<f32>() < chance
        } else {
            rand::random::<u32>().is_multiple_of(unbreaking_level as u32 + 1)
        }
    }

    pub fn damage_item(&mut self, amount: i32) -> DamageResult {
        if amount <= 0 || !self.is_damageable() || self.is_unbreakable() {
            return DamageResult::Untouched;
        }

        let max_damage = self.get_max_damage().unwrap_or(0);
        if max_damage <= 0 {
            return DamageResult::Untouched;
        }

        let mut applied = 0;
        for _ in 0..amount {
            if self.should_apply_durability_damage() {
                applied += 1;
            }
        }

        if applied <= 0 {
            return DamageResult::Untouched;
        }

        let new_damage = self.get_damage().saturating_add(applied);
        if new_damage >= max_damage {
            if self.item_count > 1 {
                self.item_count = self.item_count.saturating_sub(1);
                self.set_damage(0);
            } else {
                *self = Self::EMPTY.clone();
            }
            return DamageResult::Broken;
        }

        self.set_damage(new_damage);
        DamageResult::Damaged
    }

    #[must_use]
    pub fn get_max_use_time(&self) -> i32 {
        if let Some(value) = self.get_data_component::<ConsumableImpl>() {
            return value.consume_ticks();
        }
        if self.get_data_component::<BlocksAttacksImpl>().is_some() {
            return 72000;
        }
        0
    }

    #[must_use]
    pub const fn get_item(&self) -> &Item {
        if self.is_empty() {
            &Item::AIR
        } else {
            self.item
        }
    }

    #[must_use]
    pub fn is_stackable(&self) -> bool {
        self.get_max_stack_size() > 1 // TODO: && (!this.isDamageable() || !this.isDamaged());
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.item_count == 0 || self.item.id == Item::AIR.id
    }

    #[must_use]
    pub fn split(&mut self, amount: u8) -> Self {
        let min = amount.min(self.item_count);
        let stack = self.copy_with_count(min);
        self.decrement(min);
        stack
    }

    #[must_use]
    pub fn split_unless_creative(&mut self, gamemode: GameMode, amount: u8) -> Self {
        let min = amount.min(self.item_count);
        let stack = self.copy_with_count(min);
        if gamemode != GameMode::Creative {
            self.decrement(min);
        }
        stack
    }

    #[must_use]
    pub fn copy_with_count(&self, count: u8) -> Self {
        let mut stack = self.clone();
        stack.item_count = count;
        stack
    }

    pub const fn set_count(&mut self, count: u8) {
        self.item_count = count;
    }

    pub fn decrement_unless_creative(&mut self, gamemode: GameMode, amount: u8) {
        if gamemode != GameMode::Creative {
            self.item_count = self.item_count.saturating_sub(amount);
        }
    }

    pub const fn decrement(&mut self, amount: u8) {
        self.item_count = self.item_count.saturating_sub(amount);
    }

    pub const fn increment(&mut self, amount: u8) {
        self.item_count = self.item_count.saturating_add(amount);
    }

    /// Completely resets the stack to air
    pub fn clear(&mut self) {
        *self = Self::EMPTY.clone();
    }

    pub fn enchant(&mut self, enchantment: &'static Enchantment, level: i32) {
        // TODO itemstack may not send update packet to client
        if level <= 0 {
            return;
        }
        let level = min(level, 255);
        if let Some(data) = self.get_data_component_mut::<EnchantmentsImpl>() {
            for (enc, old_level) in data.enchantment.to_mut() {
                if *enc == enchantment {
                    *old_level = max(*old_level, level);
                    return;
                }
            }
            data.enchantment.to_mut().push((enchantment, level));
        } else {
            self.patch.push((
                Enchantments,
                Some(
                    EnchantmentsImpl {
                        enchantment: Cow::Owned(vec![(enchantment, level)]),
                    }
                    .to_dyn(),
                ),
            ));
        }
    }

    #[must_use]
    pub fn are_items_and_components_equal(&self, other: &Self) -> bool {
        if self.item != other.item || self.patch.len() != other.patch.len() {
            return false;
        }
        for (id, data) in &self.patch {
            let mut not_found = true;
            'out: for (other_id, other_data) in &other.patch {
                if id == other_id {
                    if let (Some(data), Some(other_data)) = (data, other_data) {
                        if !data.equal(other_data.as_ref()) {
                            return false;
                        }
                        not_found = false;
                        break 'out;
                    } else if data.is_none() && other_data.is_none() {
                        not_found = false;
                        break 'out;
                    }
                    return false;
                }
            }
            if not_found {
                return false;
            }
        }
        true
    }

    #[must_use]
    pub fn are_equal(&self, other: &Self) -> bool {
        self.item_count == other.item_count && self.are_items_and_components_equal(other)
    }

    /// Determines the mining speed for a block based on tool rules.
    /// Direct matches return immediately, tagged blocks are checked separately.
    /// If no match is found, returns the tool's default mining speed or `1.0`.
    #[must_use]
    pub fn get_speed(&self, block: &'static Block) -> f32 {
        // No tool? Use default speed
        if let Some(tool) = self.get_data_component::<ToolImpl>() {
            for rule in tool.rules.iter() {
                // Skip if speed is not set
                let Some(speed) = rule.speed else {
                    continue;
                };
                match &rule.blocks {
                    IDSet::Tag(tag) => {
                        if block.has_tag(tag) {
                            return speed;
                        }
                    }
                    IDSet::Blocks(blocks) => {
                        if blocks.contains(&block) {
                            return speed;
                        }
                    }
                }
            }
            tool.default_mining_speed
        } else {
            1.0
        }
    }

    /// Determines if a tool is valid for block drops based on tool rules.
    /// Direct matches return immediately, while tagged blocks are checked separately.
    #[must_use]
    pub fn is_correct_for_drops(&self, block: &'static Block) -> bool {
        if let Some(tool) = self.get_data_component::<ToolImpl>() {
            for rule in tool.rules.iter() {
                // Skip if speed is not set
                let Some(correct) = rule.correct_for_drops else {
                    continue;
                };
                match &rule.blocks {
                    IDSet::Tag(tag) => {
                        if block.has_tag(tag) {
                            return correct;
                        }
                    }
                    IDSet::Blocks(blocks) => {
                        if blocks.contains(&block) {
                            return correct;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn write_item_stack(&self, compound: &mut NbtCompound) {
        // Minecraft 1.21.4 uses "id" as string with namespaced ID (minecraft:diamond_sword)
        compound.put_string("id", format!("minecraft:{}", self.item.registry_key));
        compound.put_int("count", self.item_count as i32);

        // Create a tag compound for additional data
        let mut tag = NbtCompound::new();

        for (id, data) in &self.patch {
            if let Some(data) = data {
                tag.put(id.to_name(), data.write_data());
            } else {
                let name = '!'.to_string() + id.to_name();
                tag.put(name.as_str(), NbtCompound::new());
            }
        }

        // Store custom data like enchantments, display name, etc. would go here
        compound.put_compound("components", tag);
    }

    #[must_use]
    pub fn read_item_stack(compound: &NbtCompound) -> Option<Self> {
        // Get ID, which is a string like "minecraft:diamond_sword"
        let full_id = compound.get_string("id")?;

        // Remove the "minecraft:" prefix if present
        let registry_key = full_id.strip_prefix("minecraft:").unwrap_or(full_id);

        // Try to get item by registry key
        let item = Item::from_registry_key(registry_key)?;

        let count = compound.get_int("count")? as u8;

        // Create the item stack
        let mut item_stack = Self::new(count, item);

        // Process any additional data in the components compound
        if let Some(tag) = compound.get_compound("components") {
            for (name, data) in &tag.child_tags {
                if let Some(name) = name.strip_prefix("!") {
                    item_stack
                        .patch
                        .push((DataComponent::try_from_name(name)?, None));
                } else {
                    let id = DataComponent::try_from_name(name)?;
                    item_stack.patch.push((id, Some(read_data(id, data)?)));
                }
            }
        }

        Some(item_stack)
    }
}

impl From<&RecipeResultStruct> for ItemStack {
    fn from(value: &RecipeResultStruct) -> Self {
        Self {
            item_count: value.count,
            item: Item::from_registry_key(value.id.strip_prefix("minecraft:").unwrap_or(value.id))
                .expect("Crafting recipe gives invalid item"),
            patch: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::data_component::DataComponent;
    use pumpkin_data::data_component_impl::{DataComponentImpl, EnchantmentsImpl, UnbreakableImpl};

    /// Helper: creates a fresh Iron Sword (max_damage 250, damage 0).
    fn iron_sword() -> ItemStack {
        ItemStack::new(1, &Item::IRON_SWORD)
    }

    // ── damage_item ───────────────────────────────────────────────

    #[test]
    fn damage_zero_amount_is_noop() {
        let mut stack = iron_sword();
        assert_eq!(stack.damage_item(0), DamageResult::Untouched);
        assert_eq!(stack.get_damage(), 0);
    }

    #[test]
    fn damage_negative_amount_is_noop() {
        let cases: &[i32] = &[-1, -5, -10, -100];
        for &amount in cases {
            let mut stack = iron_sword();
            assert_eq!(
                stack.damage_item(amount),
                DamageResult::Untouched,
                "expected no damage for amount={amount}"
            );
            assert_eq!(stack.get_damage(), 0, "damage mismatch for amount={amount}");
        }
    }

    #[test]
    fn damage_non_damageable_item_is_noop() {
        // AIR has no MaxDamage component.
        let mut stack = ItemStack::new(1, &Item::AIR);
        assert_eq!(stack.damage_item(1), DamageResult::Untouched);
    }

    #[test]
    fn damage_unbreakable_item_is_noop() {
        let cases: &[i32] = &[1, 5, 10, 100, 250];
        for &amount in cases {
            let mut stack = iron_sword();
            stack
                .patch
                .push((DataComponent::Unbreakable, Some(UnbreakableImpl.to_dyn())));
            assert_eq!(
                stack.damage_item(amount),
                DamageResult::Untouched,
                "expected no damage for unbreakable item, amount={amount}"
            );
            assert_eq!(
                stack.get_damage(),
                0,
                "damage mismatch for unbreakable item, amount={amount}"
            );
        }
    }

    #[test]
    fn damage_increases_damage_value() {
        // Without Unbreaking, every point of damage is applied.
        // Each sub-array is (amount, expected_damage); each case gets a fresh iron_sword.
        let cases: &[(i32, i32)] = &[(1, 1), (5, 5), (10, 10), (100, 100), (249, 249)];
        for &(amount, expected) in cases {
            let mut stack = iron_sword();
            assert_eq!(
                stack.damage_item(amount),
                DamageResult::Damaged,
                "expected damage_item to return Damaged for amount={amount}"
            );
            assert_eq!(
                stack.get_damage(),
                expected,
                "damage mismatch for amount={amount}"
            );
        }
    }

    #[test]
    fn damage_accumulates() {
        // Each entry: (first_amount, second_amount, expected_total)
        let cases: &[(i32, i32, i32)] = &[(100, 50, 150), (10, 20, 30), (1, 1, 2), (50, 100, 150)];
        for &(first, second, expected) in cases {
            let mut stack = iron_sword();
            stack.damage_item(first);
            stack.damage_item(second);
            assert_eq!(
                stack.get_damage(),
                expected,
                "accumulated damage mismatch for first={first}, second={second}"
            );
        }
    }

    #[test]
    fn damage_breaks_item_when_exceeding_max() {
        // Iron Sword max_damage = 250; any amount >= 250 should destroy it.
        let cases: &[i32] = &[250, 260, 300, 1000];
        for &amount in cases {
            let mut stack = iron_sword();
            assert_eq!(
                stack.damage_item(amount),
                DamageResult::Broken,
                "expected item to break for amount={amount}"
            );
            assert!(
                stack.is_empty(),
                "item should be destroyed for amount={amount}"
            );
        }
    }

    #[test]
    fn damage_breaks_single_item_to_empty() {
        let mut stack = iron_sword();
        stack.damage_item(300);
        assert!(stack.is_empty());
        assert_eq!(stack.item_count, 0);
    }

    // ── repair_item ──────────────────────────────────────────────────

    #[test]
    fn repair_zero_amount_is_noop() {
        let initial_damages: &[i32] = &[1, 5, 10, 100, 249];
        for &initial in initial_damages {
            let mut stack = iron_sword();
            stack.set_damage(initial);
            assert_eq!(
                stack.repair_item(0),
                0,
                "repair(0) should return 0 for initial={initial}"
            );
            assert_eq!(
                stack.get_damage(),
                initial,
                "damage should be unchanged for initial={initial}"
            );
        }
    }

    #[test]
    fn repair_negative_amount_is_noop() {
        let cases: &[i32] = &[-1, -5, -10, -100];
        for &amount in cases {
            let mut stack = iron_sword();
            stack.set_damage(10);
            assert_eq!(
                stack.repair_item(amount),
                0,
                "repair({amount}) should return 0"
            );
            assert_eq!(
                stack.get_damage(),
                10,
                "damage should be unchanged for repair({amount})"
            );
        }
    }

    #[test]
    fn repair_undamaged_item_is_noop() {
        let amounts: &[i32] = &[1, 5, 10, 100];
        for &amount in amounts {
            let mut stack = iron_sword();
            assert_eq!(
                stack.repair_item(amount),
                0,
                "repair({amount}) on undamaged item should return 0"
            );
            assert_eq!(
                stack.get_damage(),
                0,
                "undamaged item should remain at 0 after repair({amount})"
            );
        }
    }

    #[test]
    fn repair_partial() {
        // Each entry: (initial_damage, repair_amount, expected_repaired, expected_remaining)
        let cases: &[(i32, i32, i32, i32)] = &[
            (20, 8, 8, 12),
            (50, 25, 25, 25),
            (100, 30, 30, 70),
            (249, 1, 1, 248),
        ];
        for &(initial, repair, exp_repaired, exp_remaining) in cases {
            let mut stack = iron_sword();
            stack.set_damage(initial);
            let repaired = stack.repair_item(repair);
            assert_eq!(
                repaired, exp_repaired,
                "repaired amount mismatch for initial={initial}, repair={repair}"
            );
            assert_eq!(
                stack.get_damage(),
                exp_remaining,
                "remaining damage mismatch for initial={initial}, repair={repair}"
            );
        }
    }

    #[test]
    fn repair_capped_at_current_damage() {
        // Each entry: (initial_damage, repair_amount); repair exceeds damage, so repaired == initial.
        let cases: &[(i32, i32)] = &[(5, 6), (5, 100), (10, 11), (100, 200)];
        for &(initial, repair) in cases {
            let mut stack = iron_sword();
            stack.set_damage(initial);
            let repaired = stack.repair_item(repair);
            assert_eq!(
                repaired, initial,
                "repaired amount mismatch for initial={initial}, repair={repair}"
            );
            assert_eq!(
                stack.get_damage(),
                0,
                "damage should be 0 after over-repair for initial={initial}"
            );
        }
    }

    #[test]
    fn repair_fully_clears_damage_component() {
        let mut stack = iron_sword();
        stack.set_damage(10);
        stack.repair_item(10);
        assert_eq!(stack.get_damage(), 0);
        // set_damage(0) removes the Damage patch entry.
        assert!(
            !stack
                .patch
                .iter()
                .any(|(id, _)| *id == DataComponent::Damage)
        );
    }

    // ── stacked item breaking ────────────────────────────────────────

    #[test]
    fn damage_stacked_item_breaks_one_and_resets_durability() {
        // Two Iron Swords (max_damage 250) at damage 249 — one hit away from breaking.
        // Without Unbreaking the damage roll is always applied, so this is deterministic.
        let mut stack = ItemStack::new(2, &Item::IRON_SWORD);
        stack.set_damage(249);

        let result = stack.damage_item(1);

        assert_eq!(
            result,
            DamageResult::Broken,
            "stacked item at max damage should return Broken"
        );
        assert_eq!(stack.item_count, 1, "stack count should drop from 2 to 1");
        assert_eq!(
            stack.get_damage(),
            0,
            "remaining sword's durability should reset to 0 after breaking"
        );
        assert!(
            !stack.is_empty(),
            "one sword should still remain in the stack"
        );
    }

    // ── weapon category predicates ───────────────────────────────────

    /// 2-durability combat weapons (axes/pickaxes/shovels/hoes) must match their category predicate.
    #[test]
    fn weapon_categories_identify_2_cost_items() {
        // Items that should have is_axe / is_pickaxe / is_shovel / is_hoe = true.
        let axes: &[&Item] = &[
            &Item::WOODEN_AXE,
            &Item::STONE_AXE,
            &Item::IRON_AXE,
            &Item::GOLDEN_AXE,
            &Item::DIAMOND_AXE,
            &Item::NETHERITE_AXE,
        ];
        let pickaxes: &[&Item] = &[
            &Item::WOODEN_PICKAXE,
            &Item::STONE_PICKAXE,
            &Item::IRON_PICKAXE,
            &Item::GOLDEN_PICKAXE,
            &Item::DIAMOND_PICKAXE,
            &Item::NETHERITE_PICKAXE,
        ];
        let shovels: &[&Item] = &[
            &Item::WOODEN_SHOVEL,
            &Item::STONE_SHOVEL,
            &Item::IRON_SHOVEL,
            &Item::GOLDEN_SHOVEL,
            &Item::DIAMOND_SHOVEL,
            &Item::NETHERITE_SHOVEL,
        ];
        let hoes: &[&Item] = &[
            &Item::WOODEN_HOE,
            &Item::STONE_HOE,
            &Item::IRON_HOE,
            &Item::GOLDEN_HOE,
            &Item::DIAMOND_HOE,
            &Item::NETHERITE_HOE,
        ];

        for item in axes {
            let stack = ItemStack::new(1, item);
            assert!(stack.is_axe(), "{} should be an axe", item.registry_key);
            assert!(
                !stack.is_sword(),
                "{} should not be a sword",
                item.registry_key
            );
        }
        for item in pickaxes {
            let stack = ItemStack::new(1, item);
            assert!(
                stack.is_pickaxe(),
                "{} should be a pickaxe",
                item.registry_key
            );
        }
        for item in shovels {
            let stack = ItemStack::new(1, item);
            assert!(
                stack.is_shovel(),
                "{} should be a shovel",
                item.registry_key
            );
        }
        for item in hoes {
            let stack = ItemStack::new(1, item);
            assert!(stack.is_hoe(), "{} should be a hoe", item.registry_key);
        }

        // Swords should cost 1, so they must NOT match any 2-cost predicate.
        let swords: &[&Item] = &[
            &Item::IRON_SWORD,
            &Item::DIAMOND_SWORD,
            &Item::NETHERITE_SWORD,
        ];
        for item in swords {
            let stack = ItemStack::new(1, item);
            assert!(stack.is_sword(), "{} should be a sword", item.registry_key);
            assert!(
                !stack.is_axe(),
                "{} should not be an axe",
                item.registry_key
            );
            assert!(
                !stack.is_pickaxe(),
                "{} should not be a pickaxe",
                item.registry_key
            );
        }
    }

    // ── Unbreaking (statistical) ─────────────────────────────────────

    /// Helper: iron sword with Unbreaking at `level`.
    fn with_unbreaking(item: &'static Item, level: i32) -> ItemStack {
        let mut s = ItemStack::new(1, item);
        s.patch.push((
            DataComponent::Enchantments,
            Some(
                EnchantmentsImpl {
                    enchantment: std::borrow::Cow::Owned(vec![(&Enchantment::UNBREAKING, level)]),
                }
                .to_dyn(),
            ),
        ));
        s
    }

    /// Unbreaking III tool: 25% apply probability. 4 000 trials, expect ~1 000 hits (window 700–1300).
    #[test]
    fn unbreaking_iii_tool_applies_roughly_25_percent_of_hits() {
        let mut stack = with_unbreaking(&Item::NETHERITE_PICKAXE, 3);
        let mut applied: u32 = 0;
        for _ in 0..4_000 {
            if stack.damage_item(1) != DamageResult::Untouched {
                applied += 1;
            }
        }
        assert!(
            (700..=1_300).contains(&applied),
            "Unbreaking III tool: expected ~1 000 applications in 4 000 trials, got {applied}"
        );
    }

    /// Unbreaking III armor: 70% apply probability. 500 trials, expect ~350 hits (window 300–400).
    #[test]
    fn unbreaking_iii_armor_applies_roughly_70_percent_of_hits() {
        let mut stack = with_unbreaking(&Item::DIAMOND_CHESTPLATE, 3);
        let mut applied: u32 = 0;
        for _ in 0..500 {
            if stack.damage_item(1) != DamageResult::Untouched {
                applied += 1;
            }
        }
        // ~350 expected with 70% probability, ±5σ confidence (300–400 window, ~99.7% non-flaky).
        assert!(
            (300..=400).contains(&applied),
            "Unbreaking III armor: expected ~350 applications in 500 trials, got {applied}"
        );
    }

    // ── set_damage ───────────────────────────────────────────────────

    #[test]
    fn set_damage_negative_clamps_to_zero() {
        let cases: &[i32] = &[-1, -10, -100, i32::MIN];
        for &amount in cases {
            let mut stack = iron_sword();
            stack.set_damage(amount);
            assert_eq!(
                stack.get_damage(),
                0,
                "damage should clamp to 0 for set_damage({amount})"
            );
        }
    }
}
