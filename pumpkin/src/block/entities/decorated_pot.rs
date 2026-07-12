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
}
