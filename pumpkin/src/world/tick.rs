use crate::block::entities::BlockEntity;
use crate::block::{OnScheduledTickArgs, RandomTickArgs};
use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::server::Server;
use crate::world::natural_spawner::{SpawnState, spawn_for_chunk};
use crate::world::{World, dragon_fight, natural_spawner};
use pumpkin_data::Block;
use pumpkin_data::entity::{EntityType, MobCategory};
use pumpkin_util::Difficulty;
use pumpkin_util::math::{get_section_cord, vector2::Vector2, vector3::Vector3};
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::chunk::ChunkHeightmapType::MotionBlocking;
use rand::seq::SliceRandom;
use rand::{RngExt, rng};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::atomic::Ordering::Relaxed;
use tracing::{debug, error, info, warn};

impl World {
    #[expect(clippy::too_many_lines)]
    pub async fn tick(self: &Arc<Self>, server: Arc<Server>) {
        let start = tokio::time::Instant::now();

        self.flush_block_updates().await;
        self.flush_synced_block_events().await;
        self.tick_pending_vibrations().await;
        self.update_active_chunks();
        self.tick_environment().await;

        // Vanilla `ServerLevel.tick`: the "raid" profiler section runs after the
        // pending block/fluid ticks and **before** `chunkSource.tick`, which is
        // what drives the custom spawners (`ServerLevel.java:371-375`). Raids are
        // ticked here for the same ordering: a raid that ends this tick has already
        // released its raiders before entity ticking looks at them.
        self.raids.tick(self).await;

        let world_for_chunks = self.clone();
        let chunk_future = async move {
            let t = tokio::time::Instant::now();
            world_for_chunks.tick_chunks().await;
            t.elapsed()
        };

        let players = self.players.load();
        let player_count = players.len();
        let players_cache = Arc::new(
            players
                .iter()
                .map(|player| {
                    let entity = player.get_entity();
                    let pos = entity.pos.load();
                    let bb = entity.bounding_box.load().expand(1.0, 0.5, 1.0);
                    (player.clone(), pos, bb)
                })
                .collect::<Vec<_>>(),
        );

        let server_for_players = server.clone();
        let player_future = async move {
            let t = tokio::time::Instant::now();
            let mut tasks = tokio::task::JoinSet::new();
            for player in players.iter() {
                let p_clone = player.clone();
                let s_clone = server_for_players.clone();
                tasks.spawn(async move {
                    p_clone.tick(&s_clone).await;
                });
            }
            while let Some(res) = tasks.join_next().await {
                if let Err(e) = res {
                    error!("Player tick panicked: {:?}", e);
                }
            }
            t.elapsed()
        };

        let entities_to_tick = self.entities.load();
        let entity_count = entities_to_tick.len();
        let server_for_entities = server.clone();
        let active_chunks = self.active_chunks.load();

        let entity_future = async move {
            let t = tokio::time::Instant::now();
            let mut tasks = tokio::task::JoinSet::new();
            for entity in entities_to_tick.iter() {
                // Only tick entities that sit in an active (ticking) chunk — the
                // same set block-entity ticking and mob spawning already use, and
                // like vanilla, which ticks entities only within the simulation
                // distance. Use the live position: fast movers such as minecarts
                // and projectiles write `pos` directly and leave the cached
                // chunk_pos stale.
                let entity_pos = entity.get_entity().pos.load();
                let entity_chunk = Vector2::new(
                    get_section_cord(entity_pos.x.floor() as i32),
                    get_section_cord(entity_pos.z.floor() as i32),
                );
                if !active_chunks.contains(&entity_chunk) {
                    continue;
                }

                // Skip entities already removed mid-tick (despawn / death cleanup
                // from a concurrent task). The tick snapshot still holds the Arc.
                let base = entity.get_entity();
                if base.removed.load(Ordering::Relaxed) || base.is_removed() {
                    continue;
                }

                let e_clone = entity.clone();
                let s_clone = server_for_entities.clone();
                let p_cache = players_cache.clone();

                tasks.spawn(async move {
                    let inner = e_clone.get_entity();
                    // Re-check after spawn: another task may have removed us.
                    if inner.removed.load(Ordering::Relaxed) || inner.is_removed() {
                        return;
                    }
                    inner.age.fetch_add(1, Relaxed);
                    e_clone.tick(&e_clone, &s_clone).await;

                    let entity_inner = e_clone.get_entity();
                    let entity_pos = entity_inner.pos.load();
                    let entity_bb = entity_inner.bounding_box.load();

                    for (player, player_pos, player_bb) in p_cache.iter() {
                        if (player_pos.x - entity_pos.x).abs() < 5.0
                            && (player_pos.y - entity_pos.y).abs() < 5.0
                            && (player_pos.z - entity_pos.z).abs() < 5.0
                            && player_bb.intersects(&entity_bb)
                        {
                            e_clone.on_player_collision(player).await;
                            break;
                        }
                    }
                });
            }
            while let Some(res) = tasks.join_next().await {
                if let Err(e) = res {
                    error!("Entity tick panicked: {:?}", e);
                }
            }
            t.elapsed()
        };

        let active_chunks = self.active_chunks.load();
        let mut block_entities: Vec<Arc<dyn BlockEntity>> = Vec::new();
        for chunk_pos in active_chunks.iter() {
            if let Some(chunk_block_entities) = self.block_entities.get(chunk_pos) {
                block_entities.extend(chunk_block_entities.values().cloned());
            }
        }
        let block_entity_count = block_entities.len();

        let world_for_be = self.clone();
        let block_entity_future = async move {
            let t = tokio::time::Instant::now();
            let mut tasks = tokio::task::JoinSet::new();
            for be in block_entities {
                let be_clone = be.clone();
                let w_clone = world_for_be.clone();
                tasks.spawn(async move {
                    be_clone.tick(&w_clone).await;
                });
            }
            while let Some(res) = tasks.join_next().await {
                if let Err(e) = res {
                    error!("Block entity panicked: {:?}", e);
                }
            }
            t.elapsed()
        };

        // Chunk ticking owns the natural-spawn snapshot and mutates its caps as
        // packs are created. Run it before entity/block-entity tasks so removals,
        // mob-spawner additions, and player movement cannot race that snapshot.
        // Vanilla likewise performs natural spawning from the chunk-source tick
        // before ticking entities.
        let chunk_elapsed = chunk_future.await;
        let (player_elapsed, entity_elapsed, block_entity_elapsed) =
            tokio::join!(player_future, entity_future, block_entity_future);

        self.level.chunk_loading.lock().unwrap().send_change();

        if let Some(ref fight_mutex) = self.dragon_fight {
            dragon_fight::DragonFight::tick(fight_mutex, self).await;
        }

        let total_elapsed = start.elapsed();
        let total_ms = total_elapsed.as_millis();
        // Verbose lag breakdown only in logging.development mode.
        // Production keeps the previous quiet debug-only path.
        if pumpkin_config::development_mode() {
            if total_ms > 200 {
                warn!(
                    "Very slow tick [{}ms]: chunks={:?} players({})={:?} entities({})={:?} block_entities({})={:?}",
                    total_ms,
                    chunk_elapsed,
                    player_count,
                    player_elapsed,
                    entity_count,
                    entity_elapsed,
                    block_entity_count,
                    block_entity_elapsed,
                );
            } else if total_ms > 50 {
                info!(
                    "Slow tick [{}ms]: chunks={:?} players({})={:?} entities({})={:?} block_entities({})={:?}",
                    total_ms,
                    chunk_elapsed,
                    player_count,
                    player_elapsed,
                    entity_count,
                    entity_elapsed,
                    block_entity_count,
                    block_entity_elapsed,
                );
            } else if entity_elapsed.as_millis() > 40 {
                info!(
                    "Entity tick heavy [{}ms] for {} entities (total tick {}ms)",
                    entity_elapsed.as_millis(),
                    entity_count,
                    total_ms,
                );
            }
        } else if total_ms > 50 {
            debug!(
                "Slow Tick [{}ms]: Chunks: {:?} | Players({}): {:?} | Entities({}): {:?} | Block Entities({}): {:?}",
                total_ms,
                chunk_elapsed,
                player_count,
                player_elapsed,
                entity_count,
                entity_elapsed,
                block_entity_count,
                block_entity_elapsed,
            );
        }

        // Vanilla broadcasts only once mid-tick (in chunkSource.tick). Changes after
        // that (entity/player phase) wait until the next tick's broadcast. We flush
        // once more at tick end so player/entity-driven setBlock still syncs without
        // waiting a full extra 50ms — still one batch, not per-block.
        self.flush_block_updates().await;
    }

    async fn tick_environment(&self) {
        let (world_age, is_night, time_of_day) = {
            let mut level_time = self.level_time.lock().await;
            let (advance_time, advance_weather) = {
                let lock = self.level_info.load();
                (
                    lock.game_rules.advance_time,
                    lock.game_rules.advance_weather,
                )
            };
            level_time.tick_time(advance_time, advance_weather);

            // Auto-save logic
            if level_time.world_age % 100 == 0 {
                self.level.should_unload.store(true, Relaxed);
                let cleaned_chunks = self.level.clean_memory();
                if !cleaned_chunks.is_empty() {
                    self.remove_entities_in_chunks(&cleaned_chunks).await;
                    self.level.clean_entity_chunks(&cleaned_chunks);
                }
                // If autosave is configured and this tick will trigger an autosave, don't double notify
                if self.level.autosave_ticks == 0 {
                    self.level.level_channel.notify();
                } else {
                    let autosave = self.level.autosave_ticks as i64;
                    if autosave == 0 || level_time.world_age % autosave != 0 {
                        self.level.level_channel.notify();
                    }
                }
            }
            if self.level.autosave_ticks > 0 && self.level.save_enabled.load(Relaxed) {
                let autosave = self.level.autosave_ticks as i64;
                if autosave > 0 && level_time.world_age % autosave == 0 {
                    self.level.should_save.store(true, Relaxed);
                    self.level.level_channel.notify();
                }
            }
            (
                level_time.world_age,
                level_time.is_night(),
                level_time.time_of_day,
            )
        };

        let mut weather = self.weather.lock().await;
        weather.tick_weather(self);

        // Cache sky darken for spawn / brightness (vanilla ambientDarkness).
        let sky_darken =
            Self::calculate_sky_darken(time_of_day, weather.rain_level, weather.thunder_level);
        self.sky_darken.store(sky_darken, Relaxed);

        if self.should_skip_night() && is_night {
            let mut level_time = self.level_time.lock().await;
            let time = time_of_day + 24000;
            level_time.set_time(time - time % 24000);
            level_time.send_time(self).await;
            drop(level_time);

            for player in self.players.load().iter() {
                player.wake_up().await;
            }

            if weather.weather_cycle_enabled && (weather.raining || weather.thundering) {
                weather.reset_weather_cycle(self);
            }
        } else if world_age % 20 == 0 {
            let level_time = self.level_time.lock().await;
            level_time.send_time(self).await;
        }
    }

    #[expect(clippy::too_many_lines)]
    pub async fn tick_chunks(self: &Arc<Self>) {
        let active_chunks = self.active_chunks.load();
        let tick_data = self.level.get_tick_data(&active_chunks);

        // Vanilla LevelTicks.tick: scheduled block/fluid ticks (NTE) run **sequentially**
        // in sub-tick order — not in parallel. Parallel JoinSet races break falling-block
        // chains (top sand sees bottom still present and never re-schedules).
        //
        // 1. Block scheduled ticks (FallingBlock, repeater, etc.)
        for scheduled_tick in tick_data.block_ticks {
            let pos = scheduled_tick.position;
            let block = self.get_block(&pos);
            if let Some(pumpkin_block) = self.block_registry.get_pumpkin_block(block.id) {
                pumpkin_block
                    .on_scheduled_tick(OnScheduledTickArgs {
                        world: self,
                        block,
                        position: &pos,
                    })
                    .await;
            }
        }

        // 2. Fluid scheduled ticks (water/lava flow)
        for scheduled_tick in tick_data.fluid_ticks {
            let pos = scheduled_tick.position;
            let fluid = self.get_fluid(&pos);
            if let Some(pumpkin_fluid) = self.block_registry.get_pumpkin_fluid(fluid.id) {
                pumpkin_fluid.on_scheduled_tick(self, fluid, &pos).await;
            }
        }

        // Vanilla: after block/fluid scheduled ticks, chunkSource.tick →
        // broadcastChangedChunks (batch dirty sections). We flush once here so NTE
        // results (sand, water, wire) reach clients this game tick — not per setBlock.
        self.flush_block_updates().await;

        // Random ticks can run independently. Natural spawning stays sequential below:
        // SpawnState updates local caps and spawn potential as each pack succeeds.
        let mut chunk_tasks = tokio::task::JoinSet::new();

        // 3. Spawn Random Ticks
        for scheduled_tick in tick_data.random_ticks {
            let world = self.clone();
            let pos = scheduled_tick.position;
            let tick_block = scheduled_tick.tick_block;
            let tick_fluid = scheduled_tick.tick_fluid;

            chunk_tasks.spawn(async move {
                let (block, fluid) = match (tick_block, tick_fluid) {
                    (true, true) => {
                        let (b, f) = world.get_block_and_fluid(&pos);
                        (Some(b), Some(f))
                    }
                    (true, false) => (Some(world.get_block(&pos)), None),
                    (false, true) => (None, Some(world.get_fluid(&pos))),
                    (false, false) => (None, None),
                };

                if let Some(block) = block
                    && let Some(pumpkin_block) = world.block_registry.get_pumpkin_block(block.id)
                {
                    pumpkin_block
                        .random_tick(RandomTickArgs {
                            world: &world,
                            block,
                            position: &pos,
                        })
                        .await;
                }

                if let Some(fluid) = fluid
                    && let Some(pumpkin_fluid) = world.block_registry.get_pumpkin_fluid(fluid.id)
                {
                    pumpkin_fluid.random_tick(fluid, &world, &pos).await;
                }
            });
        }

        // 4. Calculate Spawn List (Sequential setup)
        let spawn_state = self.spawn_state.load();
        let (spawn_mobs, spawn_monsters, peaceful) = {
            let lock = self.level_info.load();
            (
                lock.game_rules.spawn_mobs,
                lock.game_rules.spawn_monsters,
                lock.difficulty == Difficulty::Peaceful,
            )
        };
        // Vanilla ServerChunkCache: animals/persistent every 400 game ticks;
        // monsters when doMobSpawning && !peaceful && spawnMonsters.
        let spawn_persistent = self.level_time.lock().await.world_age % 400 == 0;
        let spawn_enemies = !peaceful && spawn_monsters && spawn_mobs;

        // Vanilla only builds the category list when doMobSpawning is true.
        let spawn_list = if spawn_mobs {
            Arc::new(natural_spawner::get_filtered_spawning_categories(
                &spawn_state,
                spawn_enemies,
                spawn_persistent,
            ))
        } else {
            Arc::new(Vec::new())
        };
        if pumpkin_config::development_mode() && spawn_persistent {
            tracing::debug!(
                "vanilla persistent spawn tick: categories={:?} spawnable_chunks={}",
                spawn_list.iter().map(|c| c.id).collect::<Vec<_>>(),
                spawn_state.spawnable_chunk_count()
            );
        }

        // 5. Spawn chunks in the shuffled vanilla order.
        // Vanilla collectSpawningChunks: natural-spawn candidates (radius 8)
        // ∩ entity-ticking (simulation distance / active_chunks)
        // ∩ anyPlayerCloseEnoughForSpawning (< 128 blocks).
        if !spawn_list.is_empty() {
            let mut spawning_chunks = Vec::new();
            for pos in active_chunks.iter() {
                if !natural_spawner::is_natural_spawn_candidate(self, *pos) {
                    continue;
                }
                if !natural_spawner::any_player_close_enough_for_spawning(self, *pos) {
                    continue;
                }
                if let Some(chunk) = self.level.read_chunk_sync(pos, std::clone::Clone::clone) {
                    spawning_chunks.push((*pos, chunk));
                }
            }

            spawning_chunks.shuffle(&mut rng());

            for (pos, chunk) in spawning_chunks {
                // NaturalSpawner mutates the shared SpawnState after each successful
                // pack. Running these tasks concurrently lets several chunks all pass
                // the same local-cap check before any of them increments it.
                self.tick_spawning_chunk(pos, &chunk, &spawn_list, &spawn_state)
                    .await;
            }
        }

        while let Some(res) = chunk_tasks.join_next().await {
            if let Err(e) = res {
                error!("Chunk task panicked: {:?}", e);
            }
        }

        // Vanilla ServerLevel.tickCustomSpawners (ServerLevel.java:454-458), driven
        // from ServerChunkCache.tick (ServerChunkCache.java:386-387). The overworld
        // spawner list is PhantomSpawner, PatrolSpawner, CatSpawner, VillageSiege,
        // WanderingTraderSpawner (MinecraftServer.java:460); Pumpkin implements the
        // first two.
        self.phantom_spawner.tick(self, spawn_enemies).await;
        self.patrol_spawner.tick(self, spawn_enemies).await;

        // Update chunk inhabited time for active chunks
        let loaded_chunks = self.level.loaded_chunks.clone();
        for pos in active_chunks.iter() {
            if let Some(chunk) = loaded_chunks.get(pos) {
                chunk.inhabited_time.fetch_add(1, Relaxed);
                chunk.dirty.store(true, Relaxed);
            }
        }
    }

    pub async fn tick_spawning_chunk(
        self: &Arc<Self>,
        chunk_pos: Vector2<i32>,
        chunk: &Arc<ChunkData>,
        spawn_list: &Vec<&'static MobCategory>,
        spawn_state: &Arc<SpawnState>,
    ) {
        // this.level.tickThunder(chunk);
        // Simulation-distance gate: callers only pass active (ticking) chunks.
        let (is_raining, is_thundering) = {
            let weather = self.weather.lock().await;
            (weather.raining, weather.thundering)
        };

        if is_raining && is_thundering && rng().random_range(0..100_000) == 0 {
            let rand_value = rng().random::<i32>() >> 2;
            let delta = Vector3::new(rand_value & 15, rand_value >> 16 & 15, rand_value >> 8 & 15);
            let random_pos = Vector3::new(
                chunk_pos.x << 4,
                chunk.heightmap.lock().unwrap().get(
                    MotionBlocking,
                    chunk_pos.x << 4,
                    chunk_pos.y << 4,
                    self.min_y,
                ),
                chunk_pos.y << 4,
            )
            .add(&delta);
            // TODO this.getBrightness(LightLayer.SKY, blockPos) >= 15;
            // TODO heightmap

            // TODO findLightningRod(blockPos)
            // TODO encapsulatingFullBlocks
            if true {
                // TODO biome.getPrecipitationAt(pos, this.getSeaLevel()) == Biome.Precipitation.RAIN
                // TODO this.getCurrentDifficultyAt(blockPos);
                if rng().random::<f32>() < 0.0675
                    && self.get_block(&random_pos.to_block_pos().down()) != &Block::LIGHTNING_ROD
                {
                    let entity = Entity::new(
                        self.clone(),
                        random_pos.to_f64(),
                        &EntityType::SKELETON_HORSE,
                    );
                    self.spawn_entity(Arc::new(entity)).await;
                }
                let entity = Entity::new(
                    self.clone(),
                    random_pos.to_f64().add_raw(0.5, 0., 0.5),
                    &EntityType::LIGHTNING_BOLT,
                );
                self.spawn_entity(Arc::new(entity)).await;
            }
        }

        if spawn_list.is_empty() {
            return;
        }
        // TODO this.level.canSpawnEntitiesInChunk(chunkPos)
        let entities = spawn_for_chunk(
            self,
            chunk_pos,
            chunk,
            spawn_state,
            spawn_list,
            is_thundering,
        );
        for entity in entities {
            self.spawn_natural_entity(entity.clone()).await;
            crate::world::natural_spawner::try_spawn_chicken_jockey(self, &entity).await;
        }
    }

    /// Returns true if enough players are sleeping and we should skip the night.
    pub fn should_skip_night(&self) -> bool {
        let players = self.players.load();

        let player_count = players.len();
        let sleeping_player_count = players
            .iter()
            .filter(|player| {
                player
                    .sleeping_since
                    .load()
                    .is_some_and(|since| since >= 100)
            })
            .count();
        drop(players);

        if player_count == 0 {
            return false;
        }

        let sleep_percentage = self
            .level_info
            .load()
            .game_rules
            .players_sleeping_percentage
            .clamp(0, 100);
        let required_sleeping =
            ((player_count as f64 * sleep_percentage as f64) / 100.0).ceil() as usize;
        let required_sleeping = required_sleeping.max(1);

        sleeping_player_count >= required_sleeping
    }
}
