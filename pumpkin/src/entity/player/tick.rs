use super::Player;
use super::statistics;
use crate::block;
use crate::entity::EntityBase;
use crate::net::ClientPlatform;
use crate::net::DisconnectReason;
use crate::server::Server;
use crate::world::World;
use pumpkin_data::Block;
use pumpkin_data::BlockState;
use pumpkin_data::translation;
use pumpkin_protocol::bedrock::client::play_status::CPlayStatus;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::CMapItemData;
use pumpkin_protocol::java::client::play::CSetCamera;
use pumpkin_protocol::java::client::play::MapIcon;
use pumpkin_protocol::java::client::play::MapPatch;
use pumpkin_util::Hand;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

impl Player {
    #[expect(clippy::too_many_lines)]
    pub async fn tick(self: &Arc<Self>, server: &Server) {
        if let Some(camera_id) = self.camera_target_id.load() {
            if camera_id == self.entity_id() {
                self.camera_target_id.store(None);
            } else {
                let world = self.world();
                let target = world
                    .get_player_by_id(camera_id)
                    .map(|p| Arc::clone(&p) as Arc<dyn EntityBase>)
                    .or_else(|| world.get_entity_by_id(camera_id));
                if let Some(target) = target {
                    let target_pos = target.get_entity().pos.load();
                    let player_pos = self.living_entity.entity.pos.load();
                    if player_pos != target_pos {
                        self.living_entity.entity.set_pos(target_pos);
                        crate::world::chunker::update_position(self).await;
                    }
                } else {
                    // Target no longer exists, reset camera back to player
                    self.camera_target_id.store(None);
                    self.client
                        .send_packet_now(&CSetCamera::new(self.entity_id().into()))
                        .await;
                }
            }
        }

        self.current_screen_handler
            .lock()
            .await
            .lock()
            .await
            .send_content_updates()
            .await;

        // if self.client.closed.load(Ordering::Relaxed) {
        //     return;
        // }

        // Statistics updates
        {
            let mut stats = self.stats.lock().await;
            stats.increment_custom(statistics::CustomStatistic::PlayTime, 1);
            stats.increment_custom(statistics::CustomStatistic::TotalWorldTime, 1);
            stats.increment_custom(statistics::CustomStatistic::TimeSinceDeath, 1);
            stats.increment_custom(statistics::CustomStatistic::TimeSinceRest, 1);
            if self.living_entity.entity.sneaking.load(Ordering::Relaxed) {
                stats.increment_custom(statistics::CustomStatistic::SneakTime, 1);
            }
        }

        {
            let mut xp = self.experience_pick_up_delay.lock().await;
            if *xp > 0 {
                *xp -= 1;
            }
        }
        let (chunk_of_chunks, total_sent_chunks) = {
            let mut chunk_manager = self.chunk_manager.lock().await;
            chunk_manager.pull_new_chunks();
            let chunks = if let ClientPlatform::Java(_) = self.client.as_ref() {
                // Java clients can only send a limited amount of chunks per tick.
                // If we have sent too many chunks without receiving an ack, we stop sending chunks.
                chunk_manager
                    .can_send_chunk()
                    .then(|| chunk_manager.next_chunk())
            } else {
                Some(chunk_manager.next_chunk())
            };
            (chunks, chunk_manager.sent_chunks_count())
        };
        if let Some(chunk_of_chunks) = chunk_of_chunks {
            let client = self.client.clone();
            tokio::spawn(async move {
                client.send_chunks(&chunk_of_chunks).await;
            });
            if let ClientPlatform::Bedrock(bedrock_client) = self.client.as_ref()
                && !self.bedrock_spawned.load(Ordering::Relaxed)
                && total_sent_chunks > 4
            {
                bedrock_client
                    .enqueue_packet(&CPlayStatus::PlayerSpawn)
                    .await;
                self.bedrock_spawned.store(true, Ordering::Relaxed);
            }
        }
        self.tick_counter.fetch_add(1, Ordering::Relaxed);
        self.living_entity
            .entity
            .age
            .fetch_add(1, Ordering::Relaxed);
        if let Some(sleeping_since) = self.sleeping_since.load()
            && sleeping_since < 101
        {
            self.sleeping_since.store(Some(sleeping_since + 1));
        }

        if self.mining.load(Ordering::Relaxed) {
            let pos = self.mining_pos.lock().await;
            let world = self.world();
            let state = world.get_block_state(&pos);
            // Is the block broken?
            if state.is_air() {
                world
                    .set_block_breaking(&self.living_entity.entity, *pos, -1)
                    .await;
                self.current_block_destroy_stage
                    .store(-1, Ordering::Relaxed);
                self.mining.store(false, Ordering::Relaxed);
            } else {
                self.continue_mining(
                    *pos,
                    &world,
                    state,
                    self.start_mining_time.load(Ordering::Relaxed),
                )
                .await;
            }
        }
        self.last_attacked_ticks.fetch_add(1, Ordering::Relaxed);

        let caller: Arc<dyn EntityBase> = self.clone();
        self.living_entity.tick(&caller, server).await;
        // Vanilla updates pose in PlayerEntity#tick after super.tick().
        self.update_player_pose().await;
        self.breath_manager.tick(self).await;
        self.hunger_manager.tick(self).await;
        self.check_inventory_advancements().await;
        self.advancements.lock().await.flush_dirty(self, true);

        // experience handling
        self.tick_experience().await;
        self.tick_health().await;
        self.tick_maps(server).await;

        // Timeout/keep alive handling
        self.tick_client_load_timeout();
        // Idle timeout handling
        let now = Instant::now();
        let idle_timeout_minutes = server.player_idle_timeout.load(Ordering::Relaxed);
        if idle_timeout_minutes > 0 {
            let idle_duration = now.duration_since(self.last_action_time.load());
            if idle_duration >= Duration::from_secs(idle_timeout_minutes as u64 * 60) {
                self.kick(
                    DisconnectReason::KickedForIdle,
                    TextComponent::translate_cross(
                        translation::java::MULTIPLAYER_DISCONNECT_IDLING,
                        translation::java::MULTIPLAYER_DISCONNECT_IDLING,
                        [],
                    ),
                )
                .await;
            }
        }
    }

    async fn continue_mining(
        &self,
        location: BlockPos,
        world: &World,
        state: &BlockState,
        starting_time: i32,
    ) {
        let time = self.tick_counter.load(Ordering::Relaxed) - starting_time;
        let speed = block::calc_block_breaking(self, state, Block::from_state_id(state.id)).await
            * (time + 1) as f32;
        let progress = (speed * 10.0) as i32;
        if progress != self.current_block_destroy_stage.load(Ordering::Relaxed) {
            world
                .set_block_breaking(&self.living_entity.entity, location, progress)
                .await;
            self.current_block_destroy_stage
                .store(progress, Ordering::Relaxed);
        }
    }

    pub fn tick_client_load_timeout(&self) {
        if !self.client_loaded.load(Ordering::Relaxed) {
            let timeout = self.client_loaded_timeout.load(Ordering::Relaxed);
            self.client_loaded_timeout
                .store(timeout.saturating_sub(1), Ordering::Relaxed);
        }
    }

    pub async fn tick_maps(&self, server: &Server) {
        use pumpkin_data::data_component_impl::MapIdImpl;
        use pumpkin_data::item::Item;

        for hand in Hand::all() {
            let item_in_hand = self.inventory.get_stack_in_hand(hand).await;

            let stack = item_in_hand.lock().await;
            if stack.item.id == Item::FILLED_MAP.id
                && let Some(map_id_comp) = stack.get_data_component::<MapIdImpl>()
            {
                let map_id = map_id_comp.id;
                if let Some(map_data_arc) = server.map_manager.get_map(map_id) {
                    let mut map_data = map_data_arc.lock().await;
                    map_data.update(self);

                    let tick_count = self.tick_counter.load(Ordering::Relaxed);
                    if map_data.dirty || tick_count % 10 == 0 {
                        let scale = 1 << map_data.scale;
                        let pos = self.position();
                        let dx = pos.x - map_data.center_x as f64;
                        let dz = pos.z - map_data.center_z as f64;

                        let icon_x = (dx / scale as f64 * 2.0).clamp(-128.0, 127.0) as i8;
                        let icon_z = (dz / scale as f64 * 2.0).clamp(-128.0, 127.0) as i8;

                        let yaw = self.living_entity.entity.yaw.load();
                        let icon_direction =
                            ((((yaw * 16.0 / 360.0).round() as i32 + 8) % 16 + 16) % 16) as i8;

                        let icons = [MapIcon {
                            icon_type: VarInt(0), // White pointer
                            x: icon_x,
                            z: icon_z,
                            direction: icon_direction,
                            display_name: None,
                        }];

                        let data = map_data.dirty.then(|| MapPatch {
                            columns: 128,
                            rows: 128,
                            x: 0,
                            z: 0,
                            data: &*map_data.colors,
                        });

                        self.client
                            .enqueue_packet(&CMapItemData {
                                map_id: VarInt(map_id),
                                scale: map_data.scale,
                                locked: map_data.locked,
                                icons: Some(&icons),
                                data,
                            })
                            .await;
                        map_data.dirty = false;
                    }
                }
            }
        }
    }
}
