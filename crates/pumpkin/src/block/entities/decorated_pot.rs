use super::BlockEntity;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use std::pin::Pin;
use tokio::sync::Mutex;

pub struct DecoratedPotBlockEntity {
    pub position: BlockPos,
    pub sherds: Mutex<Option<Vec<NbtTag>>>,
    pub item: Mutex<Option<ItemStack>>,
}

impl BlockEntity for DecoratedPotBlockEntity {
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
        let sherds = nbt.get_list("sherds").map(<[_]>::to_vec);
        let item = nbt
            .get_compound("item")
            .and_then(ItemStack::read_item_stack);
        Self {
            position,
            sherds: Mutex::new(sherds),
            item: Mutex::new(item),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if let Some(sh) = self.sherds.lock().await.as_ref() {
                nbt.put_list("sherds", sh.clone());
            }
            if let Some(it) = self.item.lock().await.as_ref() {
                let mut it_nbt = NbtCompound::new();
                it.write_item_stack(&mut it_nbt);
                nbt.put_compound("item", it_nbt);
            }
        })
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(sherds) = self.sherds.try_lock()
            && let Some(ref sh) = *sherds
        {
            nbt.put_list("sherds", sh.clone());
        }
        if let Ok(item) = self.item.try_lock()
            && let Some(ref it) = *item
        {
            let mut it_nbt = NbtCompound::new();
            it.write_item_stack(&mut it_nbt);
            nbt.put_compound("item", it_nbt);
        }
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl DecoratedPotBlockEntity {
    pub const ID: &'static str = "minecraft:decorated_pot";

    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            sherds: Mutex::const_new(None),
            item: Mutex::const_new(None),
        }
    }

    pub async fn get_item(&self) -> Option<ItemStack> {
        self.item.lock().await.clone()
    }

    pub async fn take_item(&self) -> Option<ItemStack> {
        self.item.lock().await.take()
    }

    pub async fn try_insert_item(&self, stack: &mut ItemStack, count: u8) -> bool {
        let mut item_guard = self.item.lock().await;
        if let Some(existing) = item_guard.as_mut() {
            if existing.item.id == stack.item.id {
                let add = count.min(64 - existing.item_count);
                if add > 0 {
                    existing.item_count += add;
                    stack.item_count -= add;
                    return true;
                }
            }
            false
        } else {
            let insert_count = count.min(stack.item_count);
            let mut inserted = stack.clone();
            inserted.item_count = insert_count;
            *item_guard = Some(inserted);
            stack.item_count -= insert_count;
            true
        }
    }

    pub async fn get_comparator_output(&self) -> u8 {
        self.item.lock().await.as_ref().map_or(0, |item| {
            if item.item_count == 0 {
                0
            } else {
                let max_count = 64f32;
                1 + ((item.item_count as f32 / max_count) * 14.0).floor() as u8
            }
        })
    }
}
