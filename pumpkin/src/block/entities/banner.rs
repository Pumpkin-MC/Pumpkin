use super::BlockEntity;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use std::pin::Pin;
use tokio::sync::Mutex;

pub struct BannerBlockEntity {
    pub position: BlockPos,
    pub custom_name: Mutex<Option<String>>,
    pub patterns: Mutex<Option<Vec<NbtTag>>>,
}

impl BlockEntity for BannerBlockEntity {
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
        let custom_name = nbt.get_string("CustomName").map(ToString::to_string);
        let patterns = nbt.get_list("patterns").map(<[_]>::to_vec);
        Self {
            position,
            custom_name: Mutex::new(custom_name),
            patterns: Mutex::new(patterns),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if let Some(name) = self.custom_name.lock().await.as_ref() {
                nbt.put_string("CustomName", name.clone());
            }
            if let Some(pats) = self.patterns.lock().await.as_ref() {
                nbt.put_list("patterns", pats.clone());
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl BannerBlockEntity {
    pub const ID: &'static str = "minecraft:banner";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            custom_name: Mutex::const_new(None),
            patterns: Mutex::const_new(None),
        }
    }
}
