use crate::entity::{EntityBase, player::Player, r#type::from_type};
use crate::world::chunker::get_simulation_distance;
use crate::world::natural_spawner::SpawnState;
use crate::world::{World, natural_spawner};
use pumpkin_data::block_properties::is_air;
use pumpkin_data::entity::EntityType;
use pumpkin_util::GameMode;
use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};
use pumpkin_world::chunk::ChunkHeightmapType;
use pumpkin_world::chunk::io::Dirtiable;
use rustc_hash::FxHashSet;
use std::sync::Arc;
use tracing::{debug, trace, warn};
use uuid::Uuid;

impl World {
    pub async fn get_heightmap_height_async(
        &self,
        height_map: ChunkHeightmapType,
        x: i32,
        z: i32,
    ) -> i32 {
        let chunk_pos = Vector2::new(x >> 4, z >> 4);
        self.level
            .get_or_fetch_chunk(chunk_pos, |chunk| {
                chunk
                    .heightmap
                    .lock()
                    .unwrap()
                    .get(height_map, x, z, self.min_y)
            })
            .await
    }

    pub fn update_active_chunks(self: &Arc<Self>) {
        // Vanilla: entity/block ticking uses simulation-distance; natural spawn
        // candidate set uses a fixed radius-8 tracker (independent of sim dist).
        let mut active_chunks = FxHashSet::default();
        let mut natural_spawn_chunks = FxHashSet::default();
        let spectators_generate_chunks =
            self.level_info.load().game_rules.spectators_generate_chunks;
        for player in self.players.load().iter() {
            // Match ChunkMap.skipPlayer: spectators only drive chunk loading and
            // the natural-spawn chunk tracker when this rule permits it.
            if player.gamemode.load() == GameMode::Spectator && !spectators_generate_chunks {
                continue;
            }
            let center = player.get_entity().chunk_pos.load();
            let simulation_distance =
                std::num::NonZeroI32::from(get_simulation_distance(player)).get();
            for dx in -simulation_distance..=simulation_distance {
                for dy in -simulation_distance..=simulation_distance {
                    active_chunks.insert(center.add_raw(dx, dy));
                }
            }
            // Vanilla DistanceManager.naturalSpawnChunkCounter range = 8.
            for dx in -natural_spawner::NATURAL_SPAWN_CHUNK_RANGE
                ..=natural_spawner::NATURAL_SPAWN_CHUNK_RANGE
            {
                for dy in -natural_spawner::NATURAL_SPAWN_CHUNK_RANGE
                    ..=natural_spawner::NATURAL_SPAWN_CHUNK_RANGE
                {
                    natural_spawn_chunks.insert(center.add_raw(dx, dy));
                }
            }
        }
        if let Ok(forced) = self.forced_chunks.lock() {
            active_chunks.extend(forced.iter().copied());
        }

        // Vanilla getNaturalSpawnChunkCount: size of the radius-8 tracker, not
        // the simulation-distance ticking set.
        let spawnable_chunks = natural_spawn_chunks.len() as i32;

        self.active_chunks.store(Arc::new(active_chunks));

        self.spawn_state.store(Arc::new(SpawnState::new(
            spawnable_chunks,
            &self.entities,
            self,
        )));
    }

    /// Gets the y position of the first non air block from the top down
    pub fn get_top_block(&self, position: Vector2<i32>) -> i32 {
        let chunk_pos = Vector2::new(position.x >> 4, position.y >> 4);
        let relative_x = (position.x & 15) as usize;
        let relative_z = (position.y & 15) as usize;

        self.level
            .read_chunk_sync(&chunk_pos, |chunk| {
                let height = chunk.heightmap.lock().unwrap().get(
                    ChunkHeightmapType::WorldSurface,
                    position.x,
                    position.y,
                    self.dimension.min_y,
                );

                if height >= self.dimension.min_y {
                    return height;
                }

                for y in (self.dimension.min_y..self.dimension.min_y + self.dimension.height).rev()
                {
                    if let Some(block_id) = chunk
                        .section
                        .get_block_absolute_y(relative_x, y, relative_z)
                        && !is_air(block_id)
                    {
                        return y;
                    }
                }
                self.dimension.min_y
            })
            .unwrap_or(self.dimension.min_y)
    }

    pub fn get_heightmap_height(&self, height_map: ChunkHeightmapType, x: i32, z: i32) -> i32 {
        let chunk_pos = Vector2::new(x >> 4, z >> 4);
        self.level
            .read_chunk_sync(&chunk_pos, |chunk| {
                chunk
                    .heightmap
                    .lock()
                    .unwrap()
                    .get(height_map, x, z, self.min_y)
            })
            .unwrap_or(self.min_y)
    }

    // NOTE: This function doesn't actually await on anything, it just spawns two tokio tasks
    /// IMPORTANT: Chunks have to be non-empty
    pub(super) fn spawn_world_entity_chunks(
        self: &Arc<Self>,
        player: Arc<Player>,
        chunks: Vec<Vector2<i32>>,
        center_chunk: Vector2<i32>,
    ) {
        #[cfg(debug_assertions)]
        let inst = std::time::Instant::now();

        // Sort such that the first chunks are closest to the center.
        let mut chunks = chunks;
        chunks.sort_unstable_by_key(|pos| {
            let rel_x = pos.x - center_chunk.x;
            let rel_z = pos.y - center_chunk.y;
            rel_x * rel_x + rel_z * rel_z
        });

        let level = self.level.clone();
        let world = self.clone();

        player.clone().spawn_task(async move {
            let mut entity_receiver = level.receive_entity_chunks(chunks);
            'main: loop {
                let recv_result = tokio::select! {
                    () = player.client.await_close_interrupt() => {
                        debug!("Canceling player packet processing");
                        None
                    },
                    recv_result = entity_receiver.recv() => {
                        recv_result
                    }
                };

                let Some((chunk_weak, first_load)) = recv_result else {
                    break;
                };

                let Some(chunk) = chunk_weak.upgrade() else {
                    continue;
                };

                let position = Vector2::new(chunk.x, chunk.z);

                if !level.is_chunk_watched(&position) {
                    // No longer watched: don't make its entities live. Leave the
                    // serialized data untouched so the normal unload path persists
                    // it as-is (nothing went live, so there is nothing to save).
                    trace!(
                        "Received entity chunk {:?}, but it is no longer watched; leaving it for the unload path",
                        &position
                    );
                    continue 'main;
                }

                if first_load {
                    // Structure templates are applied while the block chunk reaches Full.
                    // Ensure this one chunk is ready before consuming its pending template
                    // entities; waiting for the whole view distance here stalls entity loads.
                    level.get_or_fetch_chunk(position, |_| ()).await;
                    if !level.is_chunk_watched(&position) {
                        continue 'main;
                    }

                    // First watcher: consume the serialized entities and make them
                    // live. The live entity list becomes the single source of
                    // truth, so the chunk's NBT is taken (cleared) to avoid keeping
                    // a duplicate copy that would be re-appended on the next unload
                    // and doubled on every reload.
                    let mut entity_nbts = std::mem::take(&mut *chunk.data.lock().await);
                    let structure_entities = level
                        .read_chunk_sync(&position, |block_chunk| {
                            let mut pending =
                                block_chunk.pending_structure_entities.lock().unwrap();
                            let entities = std::mem::take(&mut *pending);
                            if !entities.is_empty() {
                                // Persist the drained queue so a later restart does not
                                // recreate entities that have already become live.
                                block_chunk.mark_dirty(true);
                            }
                            entities
                        })
                        .unwrap_or_default();
                    entity_nbts.extend(structure_entities);
                    let mut entities_to_add: Vec<Arc<dyn EntityBase>> =
                        Vec::with_capacity(entity_nbts.len());
                    for entity_nbt in &entity_nbts {
                        let Some(id) = entity_nbt.get_string("id") else {
                            debug!("Entity has no ID");
                            continue;
                        };
                        let Some(entity_type) =
                            EntityType::from_name(id.strip_prefix("minecraft:").unwrap_or(id))
                        else {
                            warn!("Entity has no valid Entity Type {id}");
                            continue;
                        };

                        // Keep the persisted UUID so the entity keeps its identity
                        // across reloads (matching vanilla); only fall back to a
                        // fresh one if it is missing/corrupt.
                        let uuid = entity_nbt.get_uuid("UUID").unwrap_or_else(Uuid::new_v4);
                        // Pos is zero since it will be read from nbt.
                        let entity =
                            from_type(entity_type, Vector3::new(0.0, 0.0, 0.0), &world, uuid);
                        entity.read_nbt_non_mut(entity_nbt).await;
                        entity.init_data_tracker().await;

                        let base_entity = entity.get_entity();
                        // Clear velocity so the client does not replay the drop
                        // animation; residual velocity from the original drop is
                        // stale data.
                        base_entity.velocity.store(Vector3::default());

                        // Spawn + equipment (axe/armor) so weapons render on the client.
                        player.client.try_enqueue_spawn_packet(&entity);
                        entities_to_add.push(entity);
                    }

                    if !entities_to_add.is_empty() {
                        world.entities.extend(entities_to_add.iter().cloned());
                    }
                } else {
                    // The chunk's entities are already live (another watcher loaded
                    // them). Just send this player the spawn packets for the live
                    // entities currently in this chunk.
                    for entity in world.entities.load().iter() {
                        let base_entity = entity.get_entity();
                        if base_entity.chunk_pos.load() == position {
                            player.client.try_enqueue_spawn_packet(entity);
                        }
                    }
                }
            }

            #[cfg(debug_assertions)]
            debug!("Chunks queued after {}ms", inst.elapsed().as_millis());
        });
    }
}
