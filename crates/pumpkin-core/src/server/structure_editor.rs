use std::collections::HashMap;
use std::sync::Arc;

use pumpkin_data::block_rotation::{Mirror, Rotation};
use pumpkin_data::{Block, BlockState, BlockStateId};
use pumpkin_gametest::{
    GameTestResult, GameTestRotation, GameTestStructureTemplate, GameTestWorld,
};
use pumpkin_nbt::{NbtCompound, tag::NbtTag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::CURRENT_MC_DATA_VERSION;
use pumpkin_world::world::BlockFlags;
use rand::{RngExt, SeedableRng, rngs::StdRng};

use crate::server::server_test_manager::ServerGameTestWorld;
use crate::world::World;

fn transform_block(pos: Vector3<i32>, mirror: Mirror, rotation: Rotation) -> Vector3<i32> {
    let x = if mirror == Mirror::FrontBack {
        -pos.x
    } else {
        pos.x
    };
    let z = if mirror == Mirror::LeftRight {
        -pos.z
    } else {
        pos.z
    };
    match rotation {
        Rotation::None => Vector3::new(x, pos.y, z),
        Rotation::Clockwise90 => Vector3::new(-z, pos.y, x),
        Rotation::Rotate180 => Vector3::new(-x, pos.y, -z),
        Rotation::CounterClockwise90 => Vector3::new(z, pos.y, -x),
    }
}

fn transform_entity(pos: Vector3<f64>, mirror: Mirror, rotation: Rotation) -> Vector3<f64> {
    let x = if mirror == Mirror::FrontBack {
        1.0 - pos.x
    } else {
        pos.x
    };
    let z = if mirror == Mirror::LeftRight {
        1.0 - pos.z
    } else {
        pos.z
    };
    match rotation {
        Rotation::None => Vector3::new(x, pos.y, z),
        Rotation::Clockwise90 => Vector3::new(1.0 - z, pos.y, x),
        Rotation::Rotate180 => Vector3::new(1.0 - x, pos.y, 1.0 - z),
        Rotation::CounterClockwise90 => Vector3::new(z, pos.y, 1.0 - x),
    }
}

/// Places a saved structure with the same template parser and world adapter used by `GameTests`.
#[expect(
    clippy::too_many_arguments,
    reason = "structure block settings come from one editor packet"
)]
#[expect(
    clippy::too_many_lines,
    reason = "structure placement applies every editor setting"
)]
pub async fn place_editor_structure(
    world: Arc<World>,
    template: &GameTestStructureTemplate,
    origin: BlockPos,
    rotation: GameTestRotation,
    mirror: Mirror,
    ignore_entities: bool,
    strict: bool,
    integrity: f32,
    seed: i64,
) -> GameTestResult<()> {
    let adapter = ServerGameTestWorld::new(world.clone());
    let block_rotation = rotation.as_block_rotation();
    let flags = if strict {
        BlockFlags::NOTIFY_LISTENERS
            | BlockFlags::MOVED
            | BlockFlags::SKIP_REDSTONE_WIRE_STATE_REPLACEMENT
            | BlockFlags::UPDATE_KNOWN_SHAPE
    } else {
        BlockFlags::NOTIFY_LISTENERS
    };
    let mut rng = StdRng::seed_from_u64(if seed == 0 {
        rand::random::<u64>()
    } else {
        seed as u64
    });
    for block in template.blocks() {
        if integrity < 1.0 && rng.random::<f32>() > integrity.clamp(0.0, 1.0) {
            continue;
        }
        if block.state == Block::STRUCTURE_VOID.default_state.id {
            continue;
        }
        let relative = transform_block(
            Vector3::new(block.position[0], block.position[1], block.position[2]),
            mirror,
            block_rotation,
        );
        let position = BlockPos::new(
            origin.0.x + relative.x,
            origin.0.y + relative.y,
            origin.0.z + relative.z,
        );
        let (block_type, _) = BlockState::from_id_with_block(block.state);
        let mirrored = world
            .block_registry
            .mirror(block_type, block.state, mirror)
            .id;
        let state = adapter.rotate_block_state(mirrored, rotation).await?;
        adapter.set_block_state(&position, state, flags).await?;
        if let Some(nbt) = &block.nbt {
            adapter.set_block_entity_nbt(&position, nbt).await?;
        } else if let Some(mode) = block.test_mode {
            let mut nbt = NbtCompound::new();
            nbt.put_string("id", "minecraft:test_block".to_string());
            nbt.put_string("mode", mode.serialized_name().to_string());
            nbt.put_string("message", String::new());
            nbt.put_bool("powered", false);
            adapter.set_block_entity_nbt(&position, &nbt).await?;
        }
    }
    if !ignore_entities {
        for entity in template.entities() {
            let relative = transform_entity(entity.position, mirror, block_rotation);
            let block_relative = transform_block(
                Vector3::new(
                    entity.block_position[0],
                    entity.block_position[1],
                    entity.block_position[2],
                ),
                mirror,
                block_rotation,
            );
            let mut nbt = entity.nbt.clone();
            nbt.put(
                "Pos",
                NbtTag::List(vec![
                    (f64::from(origin.0.x) + relative.x).into(),
                    (f64::from(origin.0.y) + relative.y).into(),
                    (f64::from(origin.0.z) + relative.z).into(),
                ]),
            );
            nbt.child_tags.remove("UUID");
            if nbt.get("block_pos").is_some() {
                nbt.put(
                    "block_pos",
                    NbtTag::IntArray(vec![
                        origin.0.x + block_relative.x,
                        origin.0.y + block_relative.y,
                        origin.0.z + block_relative.z,
                    ]),
                );
            }
            let angles = nbt.get_list("Rotation").unwrap_or_default();
            let yaw = angles
                .first()
                .and_then(NbtTag::extract_float)
                .unwrap_or(0.0);
            let pitch = angles.get(1).and_then(NbtTag::extract_float).unwrap_or(0.0);
            let angle = pumpkin_util::math::wrap_degrees(yaw);
            let mirrored = match mirror {
                Mirror::None => angle,
                Mirror::LeftRight => 180.0 - angle,
                Mirror::FrontBack => -angle,
            };
            let rotated = angle
                + match block_rotation {
                    Rotation::None => 0.0,
                    Rotation::Clockwise90 => 90.0,
                    Rotation::Rotate180 => 180.0,
                    Rotation::CounterClockwise90 => 270.0,
                };
            let yaw = rotated + mirrored - angle;
            nbt.put("Rotation", NbtTag::List(vec![yaw.into(), pitch.into()]));
            adapter.spawn_structure_entity(&nbt).await?;
        }
    }
    Ok(())
}

fn update_capture_tickets(world: &World, chunks: &[Vector2<i32>], add: bool) {
    let mut loading = world
        .level
        .chunk_loading
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for &chunk in chunks {
        if add {
            loading.add_force_ticket(chunk);
        } else {
            loading.remove_force_ticket(chunk);
        }
    }
    loading.send_change();
}

/// Captures a loaded world region in the 26.3 structure template format.
#[expect(
    clippy::too_many_lines,
    reason = "structure capture writes blocks, block entities, and entities"
)]
pub async fn capture_structure(
    world: &Arc<World>,
    origin: BlockPos,
    size: [i32; 3],
    include_entities: bool,
    ignore_air: bool,
) -> Result<NbtCompound, String> {
    let volume = size
        .iter()
        .try_fold(1i64, |volume, axis| volume.checked_mul(i64::from(*axis)))
        .ok_or_else(|| "Structure size is too large".to_string())?;
    if size.iter().any(|axis| *axis <= 0 || *axis > 256) || volume > 1_000_000 {
        return Err(format!("Invalid structure size {size:?}"));
    }
    let max = BlockPos::new(
        origin
            .0
            .x
            .checked_add(size[0] - 1)
            .ok_or("Structure X exceeds world limits")?,
        origin
            .0
            .y
            .checked_add(size[1] - 1)
            .ok_or("Structure Y exceeds world limits")?,
        origin
            .0
            .z
            .checked_add(size[2] - 1)
            .ok_or("Structure Z exceeds world limits")?,
    );
    let mut chunks = Vec::new();
    for x in (origin.0.x >> 4)..=(max.0.x >> 4) {
        for z in (origin.0.z >> 4)..=(max.0.z >> 4) {
            chunks.push(Vector2::new(x, z));
        }
    }
    update_capture_tickets(world, &chunks, true);
    for &chunk in &chunks {
        world.level.get_or_fetch_chunk(chunk, |_| ()).await;
    }

    let mut palette_ids: HashMap<BlockStateId, i32> = HashMap::new();
    let mut palette = Vec::new();
    let mut blocks = Vec::new();
    for y in 0..size[1] {
        for x in 0..size[0] {
            for z in 0..size[2] {
                let pos = BlockPos::new(origin.0.x + x, origin.0.y + y, origin.0.z + z);
                let block = world.get_block(&pos);
                if block == &Block::STRUCTURE_VOID || (ignore_air && block == &Block::AIR) {
                    continue;
                }
                let state_id = world.get_block_state_id(&pos);
                let state = *palette_ids.entry(state_id).or_insert_with(|| {
                    let mut entry = NbtCompound::new();
                    entry.put_string("id", block.name.to_string());
                    if let Some(properties) = block.properties(state_id) {
                        let values = properties.to_props();
                        if !values.is_empty() {
                            let mut tags = NbtCompound::new();
                            for property in values {
                                tags.put_string(
                                    property.key().to_str(),
                                    property.value_str().to_string(),
                                );
                            }
                            entry.put_compound("properties", tags);
                        }
                    }
                    let index = palette.len() as i32;
                    palette.push(NbtTag::Compound(entry));
                    index
                });
                let mut entry = NbtCompound::new();
                entry.put_list("pos", vec![NbtTag::Int(x), NbtTag::Int(y), NbtTag::Int(z)]);
                entry.put_int("state", state);
                if let Some(block_entity) = world.get_block_entity(&pos) {
                    let mut nbt = NbtCompound::new();
                    block_entity.write_internal(&mut nbt);
                    entry.put_compound("nbt", nbt);
                }
                blocks.push(NbtTag::Compound(entry));
            }
        }
    }

    let mut entities = Vec::new();
    if include_entities {
        for entity in world.entities.load().iter() {
            if entity.get_player().is_some() {
                continue;
            }
            let pos = entity.get_entity().pos.load();
            if pos.x < f64::from(origin.0.x)
                || pos.y < f64::from(origin.0.y)
                || pos.z < f64::from(origin.0.z)
                || pos.x >= f64::from(origin.0.x) + f64::from(size[0])
                || pos.y >= f64::from(origin.0.y) + f64::from(size[1])
                || pos.z >= f64::from(origin.0.z) + f64::from(size[2])
            {
                continue;
            }
            let relative = [
                pos.x - f64::from(origin.0.x),
                pos.y - f64::from(origin.0.y),
                pos.z - f64::from(origin.0.z),
            ];
            let mut nbt = NbtCompound::new();
            entity.write_nbt(&mut nbt);
            let mut entry = NbtCompound::new();
            entry.put_list("pos", relative.into_iter().map(NbtTag::Double).collect());
            entry.put_list(
                "blockPos",
                relative
                    .into_iter()
                    .map(|value| NbtTag::Int(value.floor() as i32))
                    .collect(),
            );
            entry.put_compound("nbt", nbt);
            entities.push(NbtTag::Compound(entry));
        }
    }

    let mut root = NbtCompound::new();
    root.put_int("DataVersion", CURRENT_MC_DATA_VERSION);
    root.put_list("size", size.into_iter().map(NbtTag::Int).collect());
    root.put_list("palette", palette);
    root.put_list("blocks", blocks);
    root.put_list("entities", entities);
    update_capture_tickets(world, &chunks, false);
    Ok(root)
}
