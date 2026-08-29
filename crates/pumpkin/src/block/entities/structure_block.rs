use super::BlockEntity;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::sync::Mutex;

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

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(name) = self.name.lock() {
            nbt.put_string("name", name.clone());
        }
        if let Ok(author) = self.author.lock() {
            nbt.put_string("author", author.clone());
        }
        if let Ok(metadata) = self.metadata.lock() {
            nbt.put_string("metadata", metadata.clone());
        }
        if let Ok(pos_x) = self.pos_x.lock() {
            nbt.put_int("posX", *pos_x);
        }
        if let Ok(pos_y) = self.pos_y.lock() {
            nbt.put_int("posY", *pos_y);
        }
        if let Ok(pos_z) = self.pos_z.lock() {
            nbt.put_int("posZ", *pos_z);
        }
        if let Ok(size_x) = self.size_x.lock() {
            nbt.put_int("sizeX", *size_x);
        }
        if let Ok(size_y) = self.size_y.lock() {
            nbt.put_int("sizeY", *size_y);
        }
        if let Ok(size_z) = self.size_z.lock() {
            nbt.put_int("sizeZ", *size_z);
        }
        if let Ok(rotation) = self.rotation.lock() {
            nbt.put_string("rotation", rotation.clone());
        }
        if let Ok(mirror) = self.mirror.lock() {
            nbt.put_string("mirror", mirror.clone());
        }
        if let Ok(mode) = self.mode.lock() {
            nbt.put_string("mode", mode.clone());
        }
        if let Ok(ignore_entities) = self.ignore_entities.lock() {
            nbt.put_bool("ignoreEntities", *ignore_entities);
        }
        if let Ok(show_air) = self.show_air.lock() {
            nbt.put_bool("showAir", *show_air);
        }
        if let Ok(show_bounding_box) = self.show_bounding_box.lock() {
            nbt.put_bool("showBoundingBox", *show_bounding_box);
        }
        if let Ok(integrity) = self.integrity.lock() {
            nbt.put_float("integrity", *integrity);
        }
        if let Ok(seed) = self.seed.lock() {
            nbt.put_long("seed", *seed);
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        nbt.put_string("name", self.name.try_lock().ok()?.clone());
        nbt.put_string("author", self.author.try_lock().ok()?.clone());
        nbt.put_string("metadata", self.metadata.try_lock().ok()?.clone());
        nbt.put_int("posX", *self.pos_x.try_lock().ok()?);
        nbt.put_int("posY", *self.pos_y.try_lock().ok()?);
        nbt.put_int("posZ", *self.pos_z.try_lock().ok()?);
        nbt.put_int("sizeX", *self.size_x.try_lock().ok()?);
        nbt.put_int("sizeY", *self.size_y.try_lock().ok()?);
        nbt.put_int("sizeZ", *self.size_z.try_lock().ok()?);
        nbt.put_string("rotation", self.rotation.try_lock().ok()?.clone());
        nbt.put_string("mirror", self.mirror.try_lock().ok()?.clone());
        nbt.put_string("mode", self.mode.try_lock().ok()?.clone());
        nbt.put_bool("ignoreEntities", *self.ignore_entities.try_lock().ok()?);
        nbt.put_bool("showAir", *self.show_air.try_lock().ok()?);
        nbt.put_bool("showBoundingBox", *self.show_bounding_box.try_lock().ok()?);
        nbt.put_float("integrity", *self.integrity.try_lock().ok()?);
        nbt.put_long("seed", *self.seed.try_lock().ok()?);
        Some(nbt)
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
