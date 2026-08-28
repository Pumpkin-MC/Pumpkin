use super::BlockEntity;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::sync::Mutex;

pub struct BrushableBlockBlockEntity {
    pub position: BlockPos,
    pub item: Mutex<Option<ItemStack>>,
    pub hits: Mutex<i32>,
    pub direction: Mutex<u8>,
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
        let direction = nbt.get_byte("direction").unwrap_or(0) as u8;
        Self {
            position,
            item: Mutex::new(item),
            hits: Mutex::new(hits),
            direction: Mutex::new(direction),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(item) = self.item.lock()
            && let Some(it) = item.as_ref()
        {
            let mut it_nbt = NbtCompound::new();
            it.write_item_stack(&mut it_nbt);
            nbt.put_compound("item", it_nbt);
        }
        if let Ok(hits) = self.hits.lock() {
            nbt.put_int("hits", *hits);
        }
        if let Ok(direction) = self.direction.lock() {
            nbt.put_byte("direction", *direction as i8);
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
        nbt.put_int("hits", *self.hits.try_lock().ok()?);
        nbt.put_byte("direction", *self.direction.try_lock().ok()? as i8);
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
            direction: Mutex::new(0),
        }
    }
}
