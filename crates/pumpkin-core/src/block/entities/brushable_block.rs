use super::BlockEntity;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::sync::Mutex;

pub struct BrushableBlockBlockEntity {
    pub position: BlockPos,
    pub item: Mutex<Option<ItemStack>>,
    pub hits: Mutex<i32>,
    pub hit_direction: Mutex<Option<u8>>,
    pub loot_table: Mutex<Option<String>>,
    pub loot_table_seed: Mutex<i64>,
}

impl BlockEntity for BrushableBlockBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let item = nbt
            .get_compound("item")
            .and_then(ItemStack::read_item_stack);
        let hits = nbt.get_int("hits").unwrap_or(0);
        let hit_direction = nbt
            .get_byte("hit_direction")
            .or_else(|| nbt.get_byte("direction"))
            .map(|b| b as u8);
        let loot_table = nbt.get_string("LootTable").map(ToString::to_string);
        let loot_table_seed = nbt.get_long("LootTableSeed").unwrap_or(0);
        Self {
            position,
            item: Mutex::new(item),
            hits: Mutex::new(hits),
            hit_direction: Mutex::new(hit_direction),
            loot_table: Mutex::new(loot_table),
            loot_table_seed: Mutex::new(loot_table_seed),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(loot_table) = self.loot_table.lock()
            && let Some(table) = loot_table.as_ref()
        {
            nbt.put_string("LootTable", table.clone());
            if let Ok(seed) = self.loot_table_seed.lock()
                && *seed != 0
            {
                nbt.put_long("LootTableSeed", *seed);
            }
        } else if let Ok(item) = self.item.lock()
            && let Some(it) = item.as_ref()
        {
            let mut it_nbt = NbtCompound::new();
            it.write_item_stack(&mut it_nbt);
            nbt.put_compound("item", it_nbt);
        }
        if let Ok(hits) = self.hits.lock()
            && *hits != 0
        {
            nbt.put_int("hits", *hits);
        }
        if let Ok(direction) = self.hit_direction.lock()
            && let Some(dir) = *direction
        {
            nbt.put_byte("hit_direction", dir as i8);
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(item) = self.item.try_lock()
            && let Some(ref it) = *item
        {
            let mut it_nbt = NbtCompound::new();
            it.write_item_stack(&mut it_nbt);
            nbt.put_compound("item", it_nbt);
        }
        if let Ok(direction) = self.hit_direction.try_lock()
            && let Some(dir) = *direction
        {
            nbt.put_byte("hit_direction", dir as i8);
        }
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl BrushableBlockBlockEntity {
    pub const ID: &'static str = "minecraft:brushable_block";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            item: Mutex::new(None),
            hits: Mutex::new(0),
            hit_direction: Mutex::new(None),
            loot_table: Mutex::new(None),
            loot_table_seed: Mutex::new(0),
        }
    }
}
