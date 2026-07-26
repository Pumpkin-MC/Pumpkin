use crate::block;
use crate::block::BlockEvent;
use crate::entity::EntityBase;
use crate::entity::{Entity, player::Player};
use crate::net::{ClientPlatform, java::JavaClient};
use crate::plugin::block::block_break::BlockBreakEvent;
use crate::world::World;
use crate::world::chunker::{get_view_distance, is_within_view_distance};
use crate::world::explosion::Explosion;
use crate::world::loot::LootContextParameters;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::block_properties::is_air;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::Sound;
use pumpkin_data::sound_id_remap::remap_sound_id_for_version;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockDirection, BlockState, BlockStateId};
use pumpkin_nbt::to_bytes_unnamed;
use pumpkin_protocol::bedrock::client::level_event::{CLevelEvent, LevelEvent};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{
    CBlockEntityData, CBlockEvent, CBlockUpdate, CExplosion, CMultiBlockUpdate,
    CSetBlockDestroyStage, CWorldEvent,
};
use pumpkin_protocol::{IdOr, SoundEvent};
use pumpkin_util::math::position::chunk_section_from_pos;
use pumpkin_util::math::{position::BlockPos, vector2::Vector2, vector3::Vector3};
use pumpkin_world::chunk::io::Dirtiable;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;
use rustc_hash::FxHashSet;
use std::collections::HashMap;
use std::sync::Arc;

impl World {
    pub async fn add_synced_block_event(&self, pos: BlockPos, r#type: u8, data: u8) {
        let block_id = self.get_block(&pos).id.as_u16();
        let mut queue = self.synced_block_event_queue.lock().await;
        queue.insert(BlockEvent {
            pos,
            block_id,
            r#type,
            data,
        });
    }

    pub async fn flush_synced_block_events(self: &Arc<Self>) {
        let _flush_guard = self.synced_block_event_flush_lock.lock().await;

        // Vanilla broadcasts changed chunks before processing block events. Enqueue
        // those state packets first so the event cannot animate an older client state.
        self.flush_block_updates().await;

        // THIS IS IMPORTANT
        // it prevents deadlocks and also removes the need to wait for a lock when adding a new synced block
        let events = {
            let mut queue = self.synced_block_event_queue.lock().await;
            std::mem::take(&mut *queue)
        };

        for event in events {
            let block = self.get_block(&event.pos);
            // `ServerLevel.doBlockEvent` only runs an event while its original
            // block type is still present. Do not let a stale piston/chest event
            // act on a replacement block after a rapid state change.
            if block.id.as_u16() != event.block_id {
                continue;
            }
            if !self
                .block_registry
                .on_synced_block_event(block, self, &event.pos, event.r#type, event.data)
                .await
            {
                continue;
            }
            let packet = CBlockEvent::new(
                event.pos,
                event.r#type,
                event.data,
                VarInt(block.id.as_u16() as i32),
            );
            self.enqueue_synced_block_event(event.pos, &packet).await;
        }
    }

    /// Enqueues block events reliably on the same FIFO queue as block state updates.
    async fn enqueue_synced_block_event(&self, pos: BlockPos, packet: &CBlockEvent) {
        let pos = Vector3::new(f64::from(pos.0.x), f64::from(pos.0.y), f64::from(pos.0.z));
        let recipients = self
            .players
            .load()
            .iter()
            // Vanilla `PlayerList.broadcast` sends block events to everyone in
            // the same dimension within a 64-block radius, not every chunk watcher.
            .filter(|player| player.position().squared_distance_to_vec(&pos) <= 4096.0)
            .cloned()
            .collect::<Vec<_>>();

        for player in recipients {
            if let ClientPlatform::Java(client) = player.client.as_ref() {
                client.enqueue_packet(packet).await;
            }
        }
    }

    pub async fn register_block_change(&self, position: BlockPos, _block_state_id: BlockStateId) {
        self.unsent_block_changes.lock().await.insert(position);
    }

    /// Send pending block state changes to clients.
    ///
    /// Vanilla analogue: `ServerChunkCache.blockChanged` only dirties a
    /// `ChunkHolder`; `broadcastChangedChunks` later emits
    /// `ClientboundBlockUpdatePacket` / `ClientboundSectionBlocksUpdatePacket`
    /// once per tick. Call sites must batch (after NTE / end of tick), never
    /// flush on every single `setBlock` (e.g. each redstone dust power change).
    pub async fn flush_block_updates(&self) {
        let _flush_guard = self.block_update_flush_lock.lock().await;
        let mut block_state_updates_by_chunk_section: HashMap<
            Vector3<i32>,
            Vec<(BlockPos, BlockStateId)>,
        > = HashMap::new();
        let changes = {
            let mut guard = self.unsent_block_changes.lock().await;
            std::mem::take(&mut *guard)
        };
        for position in changes {
            // `set_block_state` mutates the chunk before it can await the dirty
            // set lock. Re-read here so racing water/redstone/player updates send
            // the final state, rather than whichever caller acquired that lock last.
            let block_state_id = self.get_block_state_id(&position);
            let chunk_section = chunk_section_from_pos(&position);
            block_state_updates_by_chunk_section
                .entry(chunk_section)
                .or_default()
                .push((position, block_state_id));
        }

        for (chunk_section, updates) in block_state_updates_by_chunk_section {
            if updates.is_empty() {
                continue;
            }
            let chunk_pos = Vector2::new(chunk_section.x, chunk_section.z);
            self.enqueue_block_updates(chunk_pos, &updates).await;
        }

        let block_entity_updates = {
            let mut guard = self.unsent_block_entity_updates.lock().unwrap();
            std::mem::take(&mut *guard)
        };
        self.enqueue_block_entity_updates(&block_entity_updates)
            .await;
    }

    /// Enqueues Java corrections in the same order domain as normal block broadcasts.
    /// Writes during the correction remain dirty and are broadcast afterwards.
    pub async fn enqueue_block_state_corrections(
        &self,
        client: &JavaClient,
        positions: &[BlockPos],
    ) {
        let _flush_guard = self.block_update_flush_lock.lock().await;
        for position in positions {
            let state_id = self.get_block_state_id(position);
            client
                .enqueue_packet(&CBlockUpdate::new(
                    *position,
                    VarInt(i32::from(state_id.as_u16())),
                ))
                .await;
        }
    }

    /// Queues authoritative block updates on each client's ordered normal queue.
    ///
    /// World state packets must stay on the ordered, non-lossy queue so rapid
    /// redstone and fluid cascades cannot leave a stale client snapshot.
    async fn enqueue_block_updates(
        &self,
        chunk_pos: Vector2<i32>,
        updates: &[(BlockPos, BlockStateId)],
    ) {
        let recipients = self
            .players
            .load()
            .iter()
            .filter(|player| {
                let center = player.get_entity().chunk_pos.load();
                let view_distance = get_view_distance(player).get() as i32;
                is_within_view_distance(chunk_pos, center, view_distance)
            })
            .cloned()
            .collect::<Vec<_>>();

        if updates.len() == 1 {
            let (block_pos, block_state_id) = updates[0];
            let java_packet =
                CBlockUpdate::new(block_pos, i32::from(block_state_id.as_u16()).into());
            let bedrock_packet = pumpkin_protocol::bedrock::client::CUpdateBlock::new(
                block_pos,
                BlockState::to_be_network_id(block_state_id) as u32,
            );

            for player in recipients {
                match player.client.as_ref() {
                    ClientPlatform::Java(client) => client.enqueue_packet(&java_packet).await,
                    ClientPlatform::Bedrock(client) => client.enqueue_packet(&bedrock_packet).await,
                }
            }
            return;
        }

        let java_packet = CMultiBlockUpdate::new(updates);
        for player in recipients {
            match player.client.as_ref() {
                ClientPlatform::Java(client) => client.enqueue_packet(&java_packet).await,
                ClientPlatform::Bedrock(client) => {
                    for (block_pos, block_state_id) in updates {
                        let bedrock_packet = pumpkin_protocol::bedrock::client::CUpdateBlock::new(
                            *block_pos,
                            BlockState::to_be_network_id(*block_state_id) as u32,
                        );
                        client.enqueue_packet(&bedrock_packet).await;
                    }
                }
            }
        }
    }

    async fn enqueue_block_entity_updates(&self, updates: &FxHashSet<BlockPos>) {
        for position in updates {
            let Some(block_entity) = self.get_block_entity(position) else {
                continue;
            };
            let state_id = self.get_block_state_id(position);
            if BlockState::from_id(state_id).block_entity_type == u16::MAX {
                continue;
            }
            let Some(nbt) = block_entity.chunk_data_nbt() else {
                continue;
            };

            let mut bytes = Vec::new();
            to_bytes_unnamed(&nbt, &mut bytes).unwrap();

            let state_packet = CBlockUpdate::new(*position, VarInt(i32::from(state_id.as_u16())));
            let block_entity_packet = CBlockEntityData::new(
                *position,
                VarInt(block_entity.get_id() as i32),
                bytes.into_boxed_slice(),
            );
            let chunk_pos = position.chunk_position();
            let recipients = self
                .players
                .load()
                .iter()
                .filter(|player| {
                    let center = player.get_entity().chunk_pos.load();
                    let view_distance = get_view_distance(player).get() as i32;
                    is_within_view_distance(chunk_pos, center, view_distance)
                })
                .cloned()
                .collect::<Vec<_>>();

            for player in recipients {
                if let ClientPlatform::Java(client) = player.client.as_ref() {
                    client.enqueue_packet(&state_packet).await;
                    client.enqueue_packet(&block_entity_packet).await;
                }
            }
        }
    }

    pub async fn explode(self: &Arc<Self>, position: Vector3<f64>, power: f32) {
        self.emit_vibration(crate::world::vibrations::Vibration::Explode, position)
            .await;
        let explosion = Explosion::new(power, position);
        let block_count = explosion.explode(self).await;
        let particle = if power < 2.0 {
            Particle::Explosion
        } else {
            Particle::ExplosionEmitter
        };
        for player in self.players.load().iter() {
            let mut sound_id = Sound::EntityGenericExplode as u16;
            if let ClientPlatform::Java(java_client) = player.client.as_ref() {
                sound_id = remap_sound_id_for_version(sound_id, java_client.version.load());
            }
            let sound = IdOr::<SoundEvent>::Id(sound_id);
            if player.position().squared_distance_to_vec(&position) > 4096.0 {
                continue;
            }
            player
                .client
                .enqueue_packet(&CExplosion::new(
                    position,
                    power,
                    block_count as i32,
                    None,
                    VarInt(particle as i32),
                    sound.clone(),
                ))
                .await;
        }
    }

    pub async fn set_block_breaking(&self, from: &Entity, location: BlockPos, progress: i32) {
        let chunk_pos = location.chunk_position(); // pumpkin's BlockPos already has this method
        let je_packet = CSetBlockDestroyStage::new(from.entity_id.into(), location, progress as i8);

        let (event_id, data) = match progress {
            -1 => (LevelEvent::BlockStopBreak, 0),
            0 => (LevelEvent::BlockStartBreak, 0),
            _ => (LevelEvent::BlockUpdateBreak, progress),
        };

        let be_packet = CLevelEvent {
            event_id: VarInt(event_id as i32),
            position: Vector3::new(
                location.0.x as f32,
                location.0.y as f32,
                location.0.z as f32,
            ),
            data: VarInt(data),
        };

        self.broadcast_to_chunk_except_editioned(
            chunk_pos,
            &[from.entity_uuid],
            &je_packet,
            &be_packet,
        )
        .await;
    }

    /// Sets a block and returns the old block id
    #[expect(clippy::too_many_lines)]
    pub async fn set_block_state(
        self: &Arc<Self>,
        position: &BlockPos,
        block_state_id: BlockStateId,
        flags: BlockFlags,
    ) -> BlockStateId {
        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        let replaced_block_state_id = self
            .level
            .read_chunk_sync(&chunk_coordinate, |chunk| {
                let replaced_block_state_id = chunk.set_block_absolute_y(
                    relative.x as usize,
                    relative.y,
                    relative.z as usize,
                    block_state_id,
                );
                // Mark chunk dirty if it isn't already
                if replaced_block_state_id != block_state_id && !chunk.is_dirty() {
                    chunk.mark_dirty(true);
                }
                replaced_block_state_id
            })
            .unwrap_or(Block::AIR.default_state.id);

        if replaced_block_state_id == block_state_id {
            return block_state_id;
        }

        self.unsent_block_changes.lock().await.insert(*position);

        let old_block = Block::from_state_id(replaced_block_state_id);
        let new_block = Block::from_state_id(block_state_id);

        let block_moved = flags.contains(BlockFlags::MOVED);

        let is_new_block = old_block != new_block;

        // WorldChunk.java line 305-314
        if is_new_block
            && old_block.default_state.block_entity_type != u16::MAX
            && let Some(entity) = self.get_block_entity(position)
        {
            entity.on_block_replaced(self.clone(), *position).await;
            self.remove_block_entity(position);
        }

        // WorldChunk.java line 317
        if is_new_block && (flags.contains(BlockFlags::NOTIFY_NEIGHBORS) || block_moved) {
            self.block_registry
                .on_state_replaced(
                    self,
                    old_block,
                    position,
                    replaced_block_state_id,
                    block_moved,
                )
                .await;
        }

        // WorldChunk.java line 318
        if !flags.contains(BlockFlags::SKIP_BLOCK_ADDED_CALLBACK) && new_block != old_block {
            self.block_registry
                .on_placed(
                    self,
                    new_block,
                    block_state_id,
                    position,
                    replaced_block_state_id,
                    block_moved,
                )
                .await;
            let new_fluid = self.get_fluid(position);
            self.block_registry
                .on_placed_fluid(
                    self,
                    new_fluid,
                    block_state_id,
                    position,
                    replaced_block_state_id,
                    block_moved,
                )
                .await;
        }

        // Ig they do this cause it could be modified in chunkPos.setBlockState?
        if self.get_block_state_id(position) == block_state_id {
            if flags.contains(BlockFlags::NOTIFY_LISTENERS) {
                // Mob AI update
            }

            if flags.contains(BlockFlags::NOTIFY_NEIGHBORS) {
                self.update_neighbors(position, None).await;
                // TODO: updateComparators
            }

            if !flags.contains(BlockFlags::FORCE_STATE) {
                let mut new_flags = flags;
                new_flags.remove(BlockFlags::NOTIFY_NEIGHBORS);
                new_flags.remove(BlockFlags::NOTIFY_LISTENERS);
                self.block_registry
                    .prepare(
                        self,
                        position,
                        Block::from_state_id(replaced_block_state_id),
                        replaced_block_state_id,
                        new_flags,
                    )
                    .await;
                self.block_registry
                    .update_neighbors(
                        self,
                        position,
                        Block::from_state_id(block_state_id),
                        new_flags,
                    )
                    .await;
                self.block_registry
                    .prepare(
                        self,
                        position,
                        Block::from_state_id(block_state_id),
                        block_state_id,
                        new_flags,
                    )
                    .await;
            }
        }

        let (_chunk_coordinate, _) = position.chunk_and_chunk_relative_position();

        self.level
            .light_engine
            .update_lighting_at(&self.level, *position);

        replaced_block_state_id
    }

    pub fn schedule_block_tick(
        &self,
        block: &Block,
        block_pos: BlockPos,
        delay: u8,
        priority: TickPriority,
    ) {
        self.level
            .schedule_block_tick(block, block_pos, delay, priority);
    }

    pub fn schedule_fluid_tick(
        &self,
        fluid: &Fluid,
        block_pos: BlockPos,
        delay: u8,
        priority: TickPriority,
    ) {
        self.level
            .schedule_fluid_tick(fluid, block_pos, delay, priority);
    }

    pub fn is_block_tick_scheduled(&self, block_pos: &BlockPos, block: &Block) -> bool {
        self.level.is_block_tick_scheduled(block_pos, block)
    }

    pub fn is_fluid_tick_scheduled(&self, block_pos: &BlockPos, fluid: &Fluid) -> bool {
        self.level.is_fluid_tick_scheduled(block_pos, fluid)
    }

    // Return new state
    #[allow(clippy::too_many_lines)]
    pub async fn break_block(
        self: &Arc<Self>,
        position: &BlockPos,
        cause: Option<Arc<Player>>,
        flags: BlockFlags,
    ) -> Option<BlockStateId> {
        let (broken_block, broken_block_state) = self.get_block_and_state_id(position);
        if is_air(broken_block_state) {
            return None;
        }
        let event = BlockBreakEvent::new(
            cause.clone(),
            broken_block,
            *position,
            0,
            !flags.contains(BlockFlags::SKIP_DROPS),
        );

        let event = self
            .server
            .upgrade()
            .unwrap()
            .plugin_manager
            .fire::<BlockBreakEvent>(event)
            .await;

        if !event.cancelled {
            let mut flags = flags;
            if event.drop {
                flags.remove(BlockFlags::SKIP_DROPS);
            } else {
                flags.insert(BlockFlags::SKIP_DROPS);
            }
            let new_state_id = if broken_block
                .properties(broken_block_state)
                .and_then(|properties| {
                    properties
                        .to_props()
                        .into_iter()
                        .find(|p| p.0 == "waterlogged")
                        .map(|(_, value)| value == "true")
                })
                .unwrap_or(false)
            {
                // Vanilla source water is LiquidBlock level 0 = Block::WATER default (state 86).
                // Do not use FlowingWaterLikeFluidProperties::to_state_id — generated table is inverted.
                Block::WATER.default_state.id
            } else {
                BlockStateId::AIR
            };

            let broken_state_id = self.set_block_state(position, new_state_id, flags).await;

            self.emit_vibration(
                crate::world::vibrations::Vibration::BlockDestroy,
                position.to_centered_f64(),
            )
            .await;

            // Close container screens for any players viewing this block
            self.close_container_screens_at(position).await;

            let luck = cause.as_ref().map_or(0.0, |player| {
                player.living_entity.get_attribute_value(&Attributes::LUCK) as f32
            });

            if Block::from_state_id(broken_state_id) != &Block::FIRE {
                let particles_packet = CWorldEvent::new(
                    WorldEvent::ParticlesDestroyBlock as i32,
                    *position,
                    broken_state_id.as_u16().into(),
                    false,
                );
                let chunk_pos = position.chunk_position();
                match &cause {
                    Some(player) => {
                        self.broadcast_to_chunk_except(
                            chunk_pos,
                            &[player.get_entity().entity_uuid],
                            &particles_packet,
                        );
                    }
                    None => self.broadcast_to_chunk(chunk_pos, &particles_packet),
                }
            }
            if !flags.contains(BlockFlags::SKIP_DROPS) {
                let tool = if let Some(player) = &cause {
                    let hand_stack = player
                        .inventory
                        .get_stack_in_hand(pumpkin_util::Hand::Right)
                        .await;
                    let stack_guard = hand_stack.lock().await;
                    (stack_guard.item_count > 0).then(|| stack_guard.clone())
                } else {
                    None
                };

                let is_raining = self.is_raining().await;
                let is_thundering = self.is_thundering().await;

                let params = LootContextParameters {
                    block_state: Some(BlockState::from_id(broken_state_id)),
                    luck,
                    position: Some(pumpkin_util::math::vector3::Vector3::new(
                        position.0.x as f64,
                        position.0.y as f64,
                        position.0.z as f64,
                    )),
                    world_time: self.level_info.load().day_time as u64,
                    tool,
                    is_raining: Some(is_raining),
                    is_thundering: Some(is_thundering),
                    ..Default::default()
                };
                block::drop_loot(self, broken_block, position, true, params).await;
            }
            return Some(new_state_id);
        }
        None
    }

    /// Close container screens for all players who have a container open at the given block position.
    pub async fn close_container_screens_at(&self, position: &BlockPos) {
        let players = self.players.load();
        for player in players.iter() {
            if player.open_container_pos.load() == Some(*position) {
                player.close_handled_screen().await;
            }
        }
    }

    /// Updates neighboring blocks of a block.
    ///
    /// Vanilla `Level.updateNeighborsAt` /
    /// `CollectingNeighborUpdater.updateNeighborsAtExceptFromFacing`
    /// (order W/E/D/U/N/S, re-entrant queue).
    ///
    /// Boxed to break async recursion through neighbor handlers / `set_block_state`.
    pub fn update_neighbors<'a>(
        self: &'a Arc<Self>,
        block_pos: &'a BlockPos,
        except: Option<BlockDirection>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        let source_block = self.get_block(block_pos);
        let pos = *block_pos;
        Box::pin(async move {
            self.neighbor_updater
                .update_neighbors_at_except(self, pos, source_block, except, None)
                .await;
        })
    }

    /// Vanilla `Level.neighborChanged` / `CollectingNeighborUpdater.neighborChanged`.
    pub fn update_neighbor<'a>(
        self: &'a Arc<Self>,
        neighbor_block_pos: &'a BlockPos,
        source_block: &'a Block,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        // Re-resolve so the collector can store a `'static` registry reference.
        let source = Block::from_id(source_block.id);
        let pos = *neighbor_block_pos;
        Box::pin(async move {
            self.neighbor_updater
                .neighbor_changed(self, pos, source, None)
                .await;
        })
    }

    pub async fn update_from_neighbor_shapes(
        self: &Arc<Self>,
        state_id: BlockStateId,
        pos: &BlockPos,
    ) -> BlockStateId {
        let mut current_state_id = state_id;
        let block = Block::from_state_id(state_id);
        for direction in BlockDirection::all() {
            let neighbor_pos = pos.offset(direction.to_offset());
            let neighbor_state_id = self.get_block_state_id(&neighbor_pos);
            current_state_id = self
                .block_registry
                .get_state_for_neighbor_update(
                    self,
                    block,
                    current_state_id,
                    pos,
                    direction,
                    &neighbor_pos,
                    neighbor_state_id,
                )
                .await;
        }
        current_state_id
    }

    pub async fn replace_with_state_for_neighbor_update(
        self: &Arc<Self>,
        block_pos: &BlockPos,
        direction: BlockDirection,
        flags: BlockFlags,
    ) {
        let (block, block_state_id) = self.get_block_and_state_id(block_pos);

        if flags.contains(BlockFlags::SKIP_REDSTONE_WIRE_STATE_REPLACEMENT)
            && *block == Block::REDSTONE_WIRE
        {
            return;
        }

        let neighbor_pos = block_pos.offset(direction.to_offset());
        let neighbor_state_id = self.get_block_state_id(&neighbor_pos);

        let new_state_id = self
            .block_registry
            .get_state_for_neighbor_update(
                self,
                block,
                block_state_id,
                block_pos,
                direction,
                &neighbor_pos,
                neighbor_state_id,
            )
            .await;

        if new_state_id != block_state_id {
            // Vanilla `Block.updateOrDestroy`: the *result* is applied with notify flags so
            // clients and cascading neighbours see the change. Incoming `flags` often have
            // NOTIFY stripped to prevent shape-update recursion storms during setBlock; that
            // must not leave drops invisible or attachment chains stuck until random ticks.
            let apply_flags = flags | BlockFlags::NOTIFY_NEIGHBORS | BlockFlags::NOTIFY_LISTENERS;
            if is_air(new_state_id) {
                self.break_block(block_pos, None, apply_flags).await;
            } else {
                self.set_block_state(block_pos, new_state_id, apply_flags)
                    .await;
            }
        }
    }
}
