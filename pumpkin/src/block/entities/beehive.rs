use super::BlockEntity;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use std::pin::Pin;
use tokio::sync::Mutex;

pub struct BeehiveBlockEntity {
    pub position: BlockPos,
    pub bees: Mutex<Option<Vec<NbtTag>>>,
    pub flower_pos: Mutex<Option<BlockPos>>,
}

impl BlockEntity for BeehiveBlockEntity {
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
        let bees = nbt.get_list("Bees").map(<[_]>::to_vec);
        let flower_pos = nbt.get_compound("FlowerPos").map(|c| {
            BlockPos::new(
                c.get_int("X").unwrap_or(0),
                c.get_int("Y").unwrap_or(0),
                c.get_int("Z").unwrap_or(0),
            )
        });
        Self {
            position,
            bees: Mutex::new(bees),
            flower_pos: Mutex::new(flower_pos),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if let Some(b) = self.bees.lock().await.as_ref() {
                nbt.put_list("Bees", b.clone());
            }
            if let Some(fp) = self.flower_pos.lock().await.as_ref() {
                let mut fp_nbt = NbtCompound::new();
                fp_nbt.put_int("X", fp.0.x);
                fp_nbt.put_int("Y", fp.0.y);
                fp_nbt.put_int("Z", fp.0.z);
                nbt.put_compound("FlowerPos", fp_nbt);
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl BeehiveBlockEntity {
    pub const ID: &'static str = "minecraft:beehive";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            bees: Mutex::const_new(None),
            flower_pos: Mutex::const_new(None),
        }
    }
}
