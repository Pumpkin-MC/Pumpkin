use super::BlockEntity;
use pumpkin_data::{Block, BlockState, BlockStateId, Rotation};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::generation::structure::template::{
    CapturedBlock, PaletteEntry, StructureTemplate, TemplateEntity, get_template, global_cache,
    place_template,
};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::entity::NBTStorage;
use crate::world::World;
use crate::world::block_placer::WorldBlockPlacer;

/// `StructureBlockEntity.MAX_SIZE_PER_AXIS` (`StructureBlockEntity.java:42`): vanilla clamps
/// the structure size to this on the client-set-structure-block packet. That packet doesn't
/// exist in this codebase yet (same gap LOAD's `placeStructureIfSameSize` documents), so the
/// NBT fields backing size can only come from a save file or hand-edited chunk data; clamp
/// here too so a SAVE scan can't be tricked into walking millions of blocks on the tick thread.
const MAX_SIZE_PER_AXIS: i32 = 48;

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

    /// Mirrors `StructureTemplate.fillFromWorld` (`StructureTemplate.java:97-135`): scans the
    /// structure block's bounding box (position + size) and captures each block's state,
    /// block-entity NBT, and (unless `ignoreEntities`) entities inside the box into a
    /// `StructureTemplate`.
    ///
    /// Returns `None` if the size is degenerate (any axis < 1, matching vanilla's guard) or if
    /// any position in the box is in an unloaded chunk - unlike vanilla, which has no such
    /// concept here since `Level.getBlockState` always returns a real (possibly generated)
    /// state; silently capturing air for an unloaded region would produce a corrupt save.
    async fn capture_structure(&self, world: &Arc<World>) -> Option<StructureTemplate> {
        let size_x = (*self.size_x.lock().await).clamp(0, MAX_SIZE_PER_AXIS);
        let size_y = (*self.size_y.lock().await).clamp(0, MAX_SIZE_PER_AXIS);
        let size_z = (*self.size_z.lock().await).clamp(0, MAX_SIZE_PER_AXIS);
        if size_x < 1 || size_y < 1 || size_z < 1 {
            return None;
        }
        let size = Vector3::new(size_x, size_y, size_z);

        let corner1 = Vector3::new(
            self.position.0.x + *self.pos_x.lock().await,
            self.position.0.y + *self.pos_y.lock().await,
            self.position.0.z + *self.pos_z.lock().await,
        );
        let corner2 = Vector3::new(
            corner1.x + size_x - 1,
            corner1.y + size_y - 1,
            corner1.z + size_z - 1,
        );
        let min = Vector3::new(
            corner1.x.min(corner2.x),
            corner1.y.min(corner2.y),
            corner1.z.min(corner2.z),
        );
        let max = Vector3::new(
            corner1.x.max(corner2.x),
            corner1.y.max(corner2.y),
            corner1.z.max(corner2.z),
        );

        let mut blocks = Vec::new();
        for x in min.x..=max.x {
            for y in min.y..=max.y {
                for z in min.z..=max.z {
                    let pos = BlockPos::new(x, y, z);
                    let state_id = world.get_block_state_id_if_loaded(&pos)?;
                    let block = Block::from_state_id(state_id);
                    if block.name == "structure_void" {
                        continue;
                    }

                    let nbt = block_entity_nbt(world, &pos).await;
                    let full_cube = BlockState::from_id(state_id).is_full_cube();
                    blocks.push(CapturedBlock {
                        pos: Vector3::new(x - min.x, y - min.y, z - min.z),
                        palette_entry: palette_entry_from_state_id(state_id),
                        nbt,
                        full_cube,
                    });
                }
            }
        }

        let entities = if *self.ignore_entities.lock().await {
            Vec::new()
        } else {
            capture_entities(world, min, max).await
        };

        Some(StructureTemplate::from_captured(size, blocks, entities))
    }

    /// Mirrors `StructureBlockEntity.saveStructure` (`StructureBlockEntity.java:322-329`):
    /// captures the structure and makes it available under its name.
    ///
    /// `save_to_disk` selects between vanilla's two call sites: the redstone-triggered path
    /// (`StructureBlock.trigger`, `StructureBlock.java:88`) always passes `false` and only
    /// refreshes the in-memory `StructureTemplateManager` cache; the actual file write
    /// (`StructureTemplateManager.save`) is only reachable from
    /// `ServerGamePacketListenerImpl.handleSetStructureBlock`'s `SAVE_AREA` branch
    /// (`ServerGamePacketListenerImpl.java:853`), gated on the client's structure-block editor
    /// GUI - the same `ServerboundSetStructureBlockPacket` gap `place_structure`'s doc comment
    /// already notes for LOAD. `save_to_disk: true` is implemented and tested here so the
    /// writer path is exercised, but in this codebase it currently has no in-game trigger.
    pub async fn save_structure(&self, world: &Arc<World>, save_to_disk: bool) -> bool {
        let name = self.name.lock().await.clone();
        if name.is_empty() {
            return false;
        }
        let Some((namespace, path)) = split_structure_identifier(&name) else {
            return false;
        };

        let Some(template) = self.capture_structure(world).await else {
            return false;
        };
        let template = Arc::new(template);

        global_cache().insert(&format!("{namespace}:{path}"), Arc::clone(&template));

        if !save_to_disk {
            return true;
        }

        let Ok(bytes) = template.to_nbt_bytes() else {
            return false;
        };
        let file_path = world
            .save_root_folder()
            .join("generated")
            .join(&namespace)
            .join("structure")
            .join(format!("{path}.nbt"));
        let Some(parent) = file_path.parent() else {
            return false;
        };
        if std::fs::create_dir_all(parent).is_err() {
            return false;
        }
        std::fs::write(&file_path, bytes).is_ok()
    }
}

/// Converts a resolved block state back into the template palette shape (the inverse of
/// `BlockStateResolver::resolve`): the namespaced block name plus its non-default properties.
fn palette_entry_from_state_id(state_id: BlockStateId) -> PaletteEntry {
    let block = Block::from_state_id(state_id);
    let properties = block.properties(state_id).map_or_else(Vec::new, |props| {
        props
            .to_props()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    });
    PaletteEntry::with_properties(format!("minecraft:{}", block.name), properties)
}

/// Captures a loaded position's block entity as `id` + its fields, with no position - the
/// shape of vanilla's `BlockEntity.saveWithId` (`saveAdditional` + `saveId`, no `x`/`y`/`z`;
/// those are added back by the position-carrying caller, e.g. `place_template`).
async fn block_entity_nbt(world: &Arc<World>, pos: &BlockPos) -> Option<NbtCompound> {
    let block_entity = world.get_block_entity(pos)?;
    let mut nbt = NbtCompound::new();
    block_entity.write_nbt(&mut nbt).await;
    nbt.put_string("id", block_entity.resource_location().to_string());
    Some(nbt)
}

/// Mirrors `StructureTemplate.fillEntityList` (`StructureTemplate.java:174-190`): every entity
/// (except players) whose position falls inside the capture box, saved with position relative
/// to the box's minimum corner.
async fn capture_entities(
    world: &Arc<World>,
    min: Vector3<i32>,
    max: Vector3<i32>,
) -> Vec<TemplateEntity> {
    let mut entities = Vec::new();
    for entity in world.entities.load().iter() {
        if entity.get_player().is_some() {
            continue;
        }

        let pos = entity.get_entity().pos.load();
        if pos.x < f64::from(min.x)
            || pos.x >= f64::from(max.x + 1)
            || pos.y < f64::from(min.y)
            || pos.y >= f64::from(max.y + 1)
            || pos.z < f64::from(min.z)
            || pos.z >= f64::from(max.z + 1)
        {
            continue;
        }

        let mut nbt = NbtCompound::new();
        if let Some(living) = entity.get_living_entity() {
            living.write_nbt(&mut nbt).await;
        } else {
            entity.get_entity().write_nbt(&mut nbt).await;
        }
        entity.write_nbt(&mut nbt).await;

        let rel_pos = Vector3::new(
            pos.x - f64::from(min.x),
            pos.y - f64::from(min.y),
            pos.z - f64::from(min.z),
        );
        let block_pos = Vector3::new(
            rel_pos.x.floor() as i32,
            rel_pos.y.floor() as i32,
            rel_pos.z.floor() as i32,
        );
        entities.push(TemplateEntity {
            pos: rel_pos,
            block_pos,
            nbt,
        });
    }
    entities
}

/// Splits a structure name into `(namespace, path)`, defaulting the namespace to `minecraft`
/// (matching `Identifier`'s default namespace), and rejects anything that isn't safe to join
/// into a filesystem path - both segments restricted to the resource-location charset
/// (`[a-z0-9_.-]` for namespace, additionally `/` for path), no empty segments, and no `..`
/// path component. Mirrors the intent of vanilla's `FileUtil.validatePath`, called from
/// `TemplatePathFactory.createAndValidatePathToStructure`.
fn split_structure_identifier(name: &str) -> Option<(String, String)> {
    let (namespace, path) = name.strip_prefix("minecraft:").map_or_else(
        || name.split_once(':').unwrap_or(("minecraft", name)),
        |rest| ("minecraft", rest),
    );

    let valid_namespace_char =
        |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.' || c == '-';
    let valid_path_char = |c: char| valid_namespace_char(c) || c == '/';

    if namespace.is_empty() || !namespace.chars().all(valid_namespace_char) {
        return None;
    }
    if path.is_empty() || !path.chars().all(valid_path_char) {
        return None;
    }
    if path
        .split('/')
        .any(|segment| segment.is_empty() || segment == "..")
    {
        return None;
    }

    Some((namespace.to_string(), path.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_defaults_to_minecraft_namespace() {
        assert_eq!(
            split_structure_identifier("my_house"),
            Some(("minecraft".to_string(), "my_house".to_string()))
        );
    }

    #[test]
    fn split_accepts_explicit_minecraft_prefix() {
        assert_eq!(
            split_structure_identifier("minecraft:village/well"),
            Some(("minecraft".to_string(), "village/well".to_string()))
        );
    }

    #[test]
    fn split_accepts_custom_namespace() {
        assert_eq!(
            split_structure_identifier("my_pack:castle"),
            Some(("my_pack".to_string(), "castle".to_string()))
        );
    }

    #[test]
    fn split_rejects_path_traversal() {
        assert_eq!(
            split_structure_identifier("minecraft:../../etc/passwd"),
            None
        );
        assert_eq!(split_structure_identifier("../escape"), None);
    }

    #[test]
    fn split_rejects_empty_segments_and_invalid_chars() {
        assert_eq!(split_structure_identifier(""), None);
        assert_eq!(split_structure_identifier("minecraft:"), None);
        assert_eq!(split_structure_identifier(":foo"), None);
        assert_eq!(split_structure_identifier("Foo:bar"), None);
        assert_eq!(split_structure_identifier("minecraft:foo//bar"), None);
        assert_eq!(split_structure_identifier("minecraft:foo\\bar"), None);
    }

    #[test]
    fn palette_entry_roundtrips_through_block_state_resolver() {
        let entry = PaletteEntry::with_properties(
            "minecraft:oak_stairs".to_string(),
            vec![
                ("facing".to_string(), "north".to_string()),
                ("half".to_string(), "bottom".to_string()),
            ],
        );
        let state = pumpkin_world::generation::structure::template::BlockStateResolver::resolve(
            &entry,
            pumpkin_data::Rotation::None,
            pumpkin_data::Mirror::None,
        )
        .expect("resolve should succeed");

        let recaptured = palette_entry_from_state_id(state.id);
        assert_eq!(recaptured.name, "minecraft:oak_stairs");
        assert!(
            recaptured
                .properties
                .contains(&("facing".to_string(), "north".to_string()))
        );
        assert!(
            recaptured
                .properties
                .contains(&("half".to_string(), "bottom".to_string()))
        );
    }
}
