use pumpkin_data::item::Item;
use pumpkin_world::item::ItemStack;

/// 8-slot hidden inventory for villagers (food collection for breeding).
pub struct VillagerInventory {
    slots: [ItemStack; 8],
}

impl Default for VillagerInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl VillagerInventory {
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| ItemStack::EMPTY.clone()),
        }
    }

    /// Food points for a given item (vanilla values).
    #[must_use]
    pub const fn food_points(item_id: u16) -> i32 {
        if item_id == Item::BREAD.id {
            4
        } else if item_id == Item::CARROT.id
            || item_id == Item::POTATO.id
            || item_id == Item::BEETROOT.id
        {
            1
        } else {
            0
        }
    }

    /// Total food points across all slots.
    #[must_use]
    pub fn total_food_points(&self) -> i32 {
        self.slots
            .iter()
            .map(|s| {
                if s.is_empty() {
                    0
                } else {
                    Self::food_points(s.item.id) * s.item_count as i32
                }
            })
            .sum()
    }

    /// Whether the villager is willing to breed (>= 12 food points).
    #[must_use]
    pub fn is_willing(&self) -> bool {
        self.total_food_points() >= 12
    }

    /// Consume 12 food points for breeding. Returns true if successful.
    pub fn consume_food_for_breeding(&mut self) -> bool {
        if !self.is_willing() {
            return false;
        }

        let mut remaining = 12;
        for slot in &mut self.slots {
            if slot.is_empty() {
                continue;
            }
            let pts = Self::food_points(slot.item.id);
            if pts <= 0 {
                continue;
            }
            while remaining > 0 && slot.item_count > 0 {
                slot.item_count -= 1;
                remaining -= pts;
            }
        }
        true
    }

    /// Try to add an item stack. Returns the leftover count that didn't fit.
    pub fn try_add(&mut self, item: &'static Item, mut count: u8) -> u8 {
        // First try to stack with existing matching slots
        for slot in &mut self.slots {
            if count == 0 {
                break;
            }
            if !slot.is_empty() && slot.item.id == item.id {
                let space = 64u8.saturating_sub(slot.item_count);
                let to_add = count.min(space);
                slot.item_count += to_add;
                count -= to_add;
            }
        }
        // Then try empty slots
        for slot in &mut self.slots {
            if count == 0 {
                break;
            }
            if slot.is_empty() {
                *slot = ItemStack::new(count, item);
                count = 0;
            }
        }
        count
    }

    /// Write inventory to NBT.
    pub fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        let mut items = Vec::new();
        for (i, slot) in self.slots.iter().enumerate() {
            if slot.is_empty() {
                continue;
            }
            let mut item_nbt = pumpkin_nbt::compound::NbtCompound::new();
            slot.write_item_stack(&mut item_nbt);
            item_nbt.put_byte("Slot", i as i8);
            items.push(pumpkin_nbt::tag::NbtTag::Compound(item_nbt));
        }
        if !items.is_empty() {
            nbt.put_list("Inventory", items);
        }
    }

    /// Read inventory from NBT.
    pub fn read_nbt(&mut self, nbt: &pumpkin_nbt::compound::NbtCompound) {
        if let Some(inv_list) = nbt.get_list("Inventory") {
            for tag in inv_list {
                if let pumpkin_nbt::tag::NbtTag::Compound(item_nbt) = tag {
                    let slot_raw = item_nbt.get_byte("Slot").unwrap_or(-1);
                    if slot_raw >= 0
                        && (slot_raw as usize) < 8
                        && let Some(stack) = ItemStack::read_item_stack(item_nbt)
                    {
                        self.slots[slot_raw as usize] = stack;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn food_points_values() {
        assert_eq!(VillagerInventory::food_points(Item::BREAD.id), 4);
        assert_eq!(VillagerInventory::food_points(Item::CARROT.id), 1);
        assert_eq!(VillagerInventory::food_points(Item::POTATO.id), 1);
        assert_eq!(VillagerInventory::food_points(Item::BEETROOT.id), 1);
        assert_eq!(VillagerInventory::food_points(Item::DIAMOND.id), 0);
    }

    #[test]
    fn willing_with_bread() {
        let mut inv = VillagerInventory::new();
        inv.try_add(&Item::BREAD, 3);
        assert!(inv.is_willing()); // 3 * 4 = 12
    }

    #[test]
    fn consume_food() {
        let mut inv = VillagerInventory::new();
        inv.try_add(&Item::BREAD, 4); // 16 points
        assert!(inv.consume_food_for_breeding());
        // Should have consumed 3 bread (12 points)
        assert_eq!(inv.total_food_points(), 4); // 1 bread left
    }
}
