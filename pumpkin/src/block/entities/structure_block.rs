use super::BlockEntity;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::pin::Pin;
use tokio::sync::Mutex;

pub struct StructureBlockBlockEntity {
    pub position: BlockPos,
    pub name: Mutex<String>,
    pub author: Mutex<String>,
    pub metadata: Mutex<String>,
    pub pos_x: Mutex<i32>,
    pub pos_y: Mutex<i32>,
    pub pos_z: Mutex<i32>,
    pub size_x: Mutex<i32>,
    pub size_y: Mutex<i32>,
    pub size_z: Mutex<i32>,
    pub rotation: Mutex<String>,
    pub mirror: Mutex<String>,
    pub mode: Mutex<String>,
    pub ignore_entities: Mutex<bool>,
    pub show_air: Mutex<bool>,
    pub show_bounding_box: Mutex<bool>,
    pub integrity: Mutex<f32>,
    pub seed: Mutex<i64>,
}

impl BlockEntity for StructureBlockBlockEntity {
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
            name: Mutex::new(nbt.get_string("name").unwrap_or("").to_string()),
            author: Mutex::new(nbt.get_string("author").unwrap_or("").to_string()),
            metadata: Mutex::new(nbt.get_string("metadata").unwrap_or("").to_string()),
            pos_x: Mutex::new(nbt.get_int("posX").unwrap_or(0)),
            pos_y: Mutex::new(nbt.get_int("posY").unwrap_or(0)),
            pos_z: Mutex::new(nbt.get_int("posZ").unwrap_or(0)),
            size_x: Mutex::new(nbt.get_int("sizeX").unwrap_or(0)),
            size_y: Mutex::new(nbt.get_int("sizeY").unwrap_or(0)),
            size_z: Mutex::new(nbt.get_int("sizeZ").unwrap_or(0)),
            rotation: Mutex::new(nbt.get_string("rotation").unwrap_or("NONE").to_string()),
            mirror: Mutex::new(nbt.get_string("mirror").unwrap_or("NONE").to_string()),
            mode: Mutex::new(nbt.get_string("mode").unwrap_or("DATA").to_string()),
            ignore_entities: Mutex::new(nbt.get_bool("ignoreEntities").unwrap_or(true)),
            show_air: Mutex::new(nbt.get_bool("showAir").unwrap_or(false)),
            show_bounding_box: Mutex::new(nbt.get_bool("showBoundingBox").unwrap_or(true)),
            integrity: Mutex::new(nbt.get_float("integrity").unwrap_or(1.0)),
            seed: Mutex::new(nbt.get_long("seed").unwrap_or(0)),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            nbt.put_string("name", self.name.lock().await.clone());
            nbt.put_string("author", self.author.lock().await.clone());
            nbt.put_string("metadata", self.metadata.lock().await.clone());
            nbt.put_int("posX", *self.pos_x.lock().await);
            nbt.put_int("posY", *self.pos_y.lock().await);
            nbt.put_int("posZ", *self.pos_z.lock().await);
            nbt.put_int("sizeX", *self.size_x.lock().await);
            nbt.put_int("sizeY", *self.size_y.lock().await);
            nbt.put_int("sizeZ", *self.size_z.lock().await);
            nbt.put_string("rotation", self.rotation.lock().await.clone());
            nbt.put_string("mirror", self.mirror.lock().await.clone());
            nbt.put_string("mode", self.mode.lock().await.clone());
            nbt.put_bool("ignoreEntities", *self.ignore_entities.lock().await);
            nbt.put_bool("showAir", *self.show_air.lock().await);
            nbt.put_bool("showBoundingBox", *self.show_bounding_box.lock().await);
            nbt.put_float("integrity", *self.integrity.lock().await);
            nbt.put_long("seed", *self.seed.lock().await);
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl StructureBlockBlockEntity {
    pub const ID: &'static str = "minecraft:structure_block";
    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            name: Mutex::new(String::new()),
            author: Mutex::new(String::new()),
            metadata: Mutex::new(String::new()),
            pos_x: Mutex::new(0),
            pos_y: Mutex::new(0),
            pos_z: Mutex::new(0),
            size_x: Mutex::new(0),
            size_y: Mutex::new(0),
            size_z: Mutex::new(0),
            rotation: Mutex::new("NONE".to_string()),
            mirror: Mutex::new("NONE".to_string()),
            mode: Mutex::new("DATA".to_string()),
            ignore_entities: Mutex::new(true),
            show_air: Mutex::new(false),
            show_bounding_box: Mutex::new(true),
            integrity: Mutex::new(1.0),
            seed: Mutex::new(0),
        }
    }
}
