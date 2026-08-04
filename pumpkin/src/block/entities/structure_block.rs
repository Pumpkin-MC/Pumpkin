use super::BlockEntity;
use pumpkin_data::Rotation;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::generation::structure::template::{get_template, place_template};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::world::World;
use crate::world::block_placer::WorldBlockPlacer;

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

    /// Mirrors `StructureBlockEntity.placeStructure` (`StructureBlockEntity.java:402-430`) for
    /// the LOAD half only: looks up the named template in the embedded/worldgen
    /// `TemplateCache` and places it via the same `WorldBlockPlacer` machinery the `/place
    /// template` command uses. Returns `false` if no template by that name is known.
    ///
    /// Known divergences from vanilla, left as documented follow-ups rather than expanded
    /// scope (see `designs/unimplemented-blocks.md`):
    /// - `mirror` is read from NBT but not applied: `place_template`'s signature has no mirror
    ///   parameter (it hardcodes `Mirror::default()` internally), and threading mirror support
    ///   through it touches shared worldgen infrastructure used by `/place` and jigsaw pieces.
    /// - `integrity < 1.0` block-dropout (`BlockRotProcessor`) is not applied.
    /// - Entities embedded in the template are not placed (`place_template` does not place
    ///   entities at all currently).
    /// - This only covers the `TemplateCache`'s embedded/worldgen template set, since there is
    ///   no filesystem-backed structure manager for player-saved templates (that's the SAVE-side
    ///   gap, out of scope here).
    pub async fn place_structure(&self, world: &Arc<World>) -> bool {
        let name = self.name.lock().await.clone();
        if name.is_empty() {
            return false;
        }
        let lookup_name = name.strip_prefix("minecraft:").unwrap_or(&name);
        let Some(template) = get_template(lookup_name) else {
            return false;
        };

        let rotation = match self.rotation.lock().await.as_str() {
            "CLOCKWISE_90" => Rotation::Clockwise90,
            "CLOCKWISE_180" => Rotation::Rotate180,
            "COUNTERCLOCKWISE_90" => Rotation::CounterClockwise90,
            _ => Rotation::None,
        };

        let origin = Vector3::new(
            self.position.0.x + *self.pos_x.lock().await,
            self.position.0.y + *self.pos_y.lock().await,
            self.position.0.z + *self.pos_z.lock().await,
        );

        let mut placer = WorldBlockPlacer::new(world);
        place_template(
            &mut placer,
            &template,
            origin,
            (0, 0),
            rotation,
            false,
            false,
            &[],
            None,
        );
        placer.finalize();
        world.queue_block_updates(&placer.changed_positions).await;
        world.flush_block_updates().await;
        true
    }
}
