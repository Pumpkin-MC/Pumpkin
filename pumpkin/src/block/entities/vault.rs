use super::BlockEntity;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::pin::Pin;
use tokio::sync::Mutex;

pub struct VaultBlockEntity {
    pub position: BlockPos,
    pub config: Mutex<Option<NbtCompound>>,
    pub server_data: Mutex<Option<NbtCompound>>,
}

impl BlockEntity for VaultBlockEntity {
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
        Self {
            position,
            config: Mutex::new(nbt.get_compound("config").cloned()),
            server_data: Mutex::new(nbt.get_compound("server_data").cloned()),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if let Some(cfg) = self.config.lock().await.as_ref() {
                nbt.put_compound("config", cfg.clone());
            }
            if let Some(data) = self.server_data.lock().await.as_ref() {
                nbt.put_compound("server_data", data.clone());
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl VaultBlockEntity {
    pub const ID: &'static str = "minecraft:vault";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            config: Mutex::const_new(None),
            server_data: Mutex::const_new(None),
        }
    }
}
