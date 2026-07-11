use super::BlockEntity;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use std::pin::Pin;
use tokio::sync::Mutex;

pub struct ConduitBlockEntity {
    pub position: BlockPos,
    pub active: Mutex<bool>,
    pub target: Mutex<Option<NbtTag>>,
}

impl BlockEntity for ConduitBlockEntity {
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
        let active = nbt.get_bool("Active").unwrap_or(false);
        let target = nbt.get("Target").cloned();
        Self {
            position,
            active: Mutex::new(active),
            target: Mutex::new(target),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            nbt.put_bool("Active", *self.active.lock().await);
            if let Some(tgt) = self.target.lock().await.as_ref() {
                nbt.put("Target", tgt.clone());
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ConduitBlockEntity {
    pub const ID: &'static str = "minecraft:conduit";
    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            active: Mutex::new(false),
            target: Mutex::new(None),
        }
    }
}
