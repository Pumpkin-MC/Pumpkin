use crate::entity::EntityBase;
use crate::entity::{Entity, player::Player};
use crate::net::{ClientPlatform, java::JavaClient};
use crate::world::World;
use crate::world::chunker::{self, get_view_distance, is_within_view_distance};
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::world::{RAW, WorldEvent};
use pumpkin_protocol::bedrock::server::text::SText;
use pumpkin_protocol::codec::data_component::data_to_proto_sound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::codec::var_long::VarLong;
use pumpkin_protocol::java::client::play::{
    CDisguisedChatMessage, CEntityStatus, CGameEvent, CPlayerChatMessage, CRemoveMobEffect,
    CSoundEffect, CSpawnEntity, CSystemChatMessage, CUpdateMobEffect, CWorldEvent, FilterType,
    GameEvent,
};
use pumpkin_protocol::java::server::play::SChatMessage;
use pumpkin_protocol::{BClientPacket, ClientPacket, IdOr};
use pumpkin_util::Difficulty;
use pumpkin_util::math::{position::BlockPos, vector2::Vector2, vector3::Vector3};
use pumpkin_util::text::TextComponent;
use pumpkin_util::version::JavaMinecraftVersion;
use rand::{RngExt, rng};
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::error;

impl World {
    /// Sends an entity status update to all players tracking the specified entity.
    ///
    /// Java: `ClientboundEntityEventPacket` (entity event / status byte).
    /// Bedrock: `ActorEvent` with the same numeric codes where they align
    /// (e.g. iron golem arm raise = 4 = `StartAttacking`).
    pub fn send_entity_status(&self, entity: &Entity, status: EntityStatus) {
        use pumpkin_protocol::bedrock::server::actor_event::{ActorEventType, SActorEvent};

        let chunk_pos = entity.chunk_pos.load();
        let status_byte = status as i8;
        let je = CEntityStatus::new(entity.entity_id, status_byte);

        // Map shared status codes used by both editions. Unknowns fall back to
        // Hurt so Bedrock still receives *something* rather than a silent drop.
        let be_type = match status_byte {
            1 => ActorEventType::Jump,
            3 => ActorEventType::Death,
            4 => ActorEventType::StartAttacking,
            5 => ActorEventType::StopAttacking,
            6 => ActorEventType::TamingFailed,
            7 => ActorEventType::TamingSucceeded,
            8 => ActorEventType::ShakeWetness,
            9 => ActorEventType::UseItem,
            10 => ActorEventType::EatGrass,
            11 => ActorEventType::StartOfferFlower,
            12 | 18 => ActorEventType::LoveHearts,
            13 => ActorEventType::VillagerAngry,
            14 => ActorEventType::VillagerHappy,
            15 => ActorEventType::WitchHatMagic,
            16 => ActorEventType::ZombieConverting,
            17 => ActorEventType::FireworksExplode,
            21 => ActorEventType::GuardianAttackSound,
            34 => ActorEventType::StopOfferFlower,
            _ => ActorEventType::Hurt,
        };
        let be = SActorEvent {
            entity_runtime_id: VarLong(i64::from(entity.entity_id)),
            event_type: be_type,
            event_data: VarInt(0),
            fire_at_position: None,
        };
        self.broadcast_to_chunk_editioned_sync(chunk_pos, &je, &be);
    }

    pub fn send_remove_mob_effect(&self, entity: &Entity, effect_type: &'static StatusEffect) {
        let chunk_pos = entity.chunk_pos.load();
        self.broadcast_to_chunk(
            chunk_pos,
            &CRemoveMobEffect::new(entity.entity_id.into(), VarInt(i32::from(effect_type.id))),
        );
    }

    pub fn send_add_mob_effect(&self, entity: &Entity, effect: &pumpkin_data::potion::Effect) {
        // TODO: only nearby
        let mut flags: i8 = 0;
        if effect.ambient {
            flags |= 0x01;
        }
        if effect.show_particles {
            flags |= 0x02;
        }
        if effect.show_icon {
            flags |= 0x04;
        }

        self.broadcast_packet_all(&CUpdateMobEffect::new(
            VarInt(entity.entity_id),
            VarInt(i32::from(effect.effect_type.id)),
            VarInt(i32::from(effect.amplifier)),
            VarInt(effect.duration),
            flags,
        ));
    }

    pub fn set_difficulty(&self, difficulty: Difficulty) {
        let current_info = self.level_info.load();
        let mut new_info = (**current_info).clone();
        new_info.difficulty = difficulty;
        self.level_info.store(Arc::new(new_info));
    }

    fn collect_java_recipients_by_version<'a>(
        players: impl Iterator<Item = &'a Arc<Player>>,
    ) -> BTreeMap<JavaMinecraftVersion, Vec<&'a JavaClient>> {
        let mut recipients_by_version: BTreeMap<JavaMinecraftVersion, Vec<&'a JavaClient>> =
            BTreeMap::new();
        for player in players {
            if let ClientPlatform::Java(java_client) = player.client.as_ref() {
                recipients_by_version
                    .entry(java_client.version.load())
                    .or_default()
                    .push(java_client);
            }
        }
        recipients_by_version
    }

    fn broadcast_java_grouped<P: ClientPacket>(
        packet: &P,
        recipients_by_version: BTreeMap<JavaMinecraftVersion, Vec<&JavaClient>>,
    ) {
        for (version, recipients) in recipients_by_version {
            let packet_data = match JavaClient::serialize_packet_for_version(packet, version) {
                Ok(packet_data) => packet_data,
                Err(err) => {
                    error!(
                        "Failed to serialize packet {} for version {:?}: {}",
                        std::any::type_name::<P>(),
                        version,
                        err
                    );
                    continue;
                }
            };

            for recipient in recipients {
                recipient.try_enqueue_packet_data(packet_data.clone());
            }
        }
    }

    /// Broadcasts a packet to all connected players within the world.
    /// Please avoid this as we want to replace it with `broadcast_editioned`
    ///
    /// Sends the specified packet to every player currently logged in to the world.
    ///
    /// **Note:** This function acquires a lock on the `current_players` map, ensuring thread safety.
    pub fn broadcast_packet_all<P: ClientPacket>(&self, packet: &P) {
        let players = self.players.load();
        let recipients_by_version = Self::collect_java_recipients_by_version(players.iter());
        Self::broadcast_java_grouped(packet, recipients_by_version);
    }

    pub fn broadcast_packet_all_sync<P: ClientPacket>(&self, packet: &P) {
        let players = self.players.load();
        for player in players.iter() {
            match player.client.as_ref() {
                ClientPlatform::Java(java) => {
                    if let Ok(data) =
                        JavaClient::serialize_packet_for_version(packet, java.version.load())
                    {
                        java.try_enqueue_packet_data(data);
                    }
                }
                ClientPlatform::Bedrock(_) => {
                    // TODO
                }
            }
        }
    }

    pub async fn broadcast_system_message(&self, message: &TextComponent, overlay: bool) {
        let je_packet = CSystemChatMessage::new(message, overlay);
        let be_packet = Self::component_to_bedrock_text(message);
        self.broadcast_editioned(&je_packet, &be_packet).await;
    }

    fn component_to_bedrock_text(message: &TextComponent) -> SText<'static> {
        match &*message.0.content {
            pumpkin_util::text::TextContent::Translate {
                translate,
                bedrock_translate,
                with,
            } => {
                let key = bedrock_translate.as_deref().unwrap_or(translate.as_ref());
                let parameters = with
                    .iter()
                    .map(pumpkin_util::text::TextComponentBase::to_bedrock_string)
                    .collect();
                SText::translation(key.to_string(), parameters)
            }
            _ => SText::system_message(
                message
                    .0
                    .to_bedrock_legacy(pumpkin_util::translation::Locale::EnUs),
            ),
        }
    }

    pub async fn broadcast_message(
        &self,
        message: &TextComponent,
        sender_name: &TextComponent,
        chat_type: u8,
        target_name: Option<&TextComponent>,
    ) {
        let be_packet = SText::new(message.clone().get_text(), sender_name.clone().get_text());
        let je_packet =
            CDisguisedChatMessage::new(message, (chat_type + 1).into(), sender_name, target_name);

        self.broadcast_editioned(&je_packet, &be_packet).await;
    }

    // This should replace broadcast_packet_all at some point
    pub async fn broadcast_editioned<J: ClientPacket, B: BClientPacket>(
        &self,
        je_packet: &J,
        be_packet: &B,
    ) {
        let players = self.players.load();
        let je_recipients_by_version = Self::collect_java_recipients_by_version(players.iter());
        let mut be_recipients = Vec::new();

        for player in players.iter() {
            if let ClientPlatform::Bedrock(be_client) = player.client.as_ref() {
                be_recipients.push(be_client.clone());
            }
        }

        Self::broadcast_java_grouped(je_packet, je_recipients_by_version);

        for recipient in be_recipients {
            recipient.enqueue_packet(be_packet).await;
        }
    }

    pub async fn broadcast_secure_player_chat(
        &self,
        sender: &Arc<Player>,
        chat_message: &SChatMessage<'_>,
        decorated_message: &TextComponent,
    ) {
        let messages_sent: i32 = sender.chat_session.lock().await.messages_sent;
        let sender_last_seen = {
            let cache = sender.signature_cache.lock().await;
            cache.last_seen.clone()
        };

        for recipient in self.players.load().iter() {
            let messages_received: i32 = recipient.chat_session.lock().await.messages_received;
            let packet = &CPlayerChatMessage::new(
                VarInt(messages_received),
                sender.gameprofile.id,
                VarInt(messages_sent),
                chat_message.signature.map(std::convert::Into::into),
                chat_message.message.into(),
                chat_message.timestamp,
                chat_message.salt,
                sender_last_seen.indexed_for(recipient).await,
                Some(decorated_message.clone()),
                FilterType::PassThrough,
                (RAW + 1).into(), // Custom registry chat_type with no sender name
                TextComponent::empty(), // Not needed since we're injecting the name in the message for custom formatting
                None,
            );
            recipient.client.enqueue_packet(packet).await;

            if let Some(signature) = chat_message.signature {
                recipient
                    .signature_cache
                    .lock()
                    .await
                    .add_seen_signature(signature);
            }

            if recipient.gameprofile.id != sender.gameprofile.id {
                // Sender may update recipient on signatures recipient hasn't seen
                recipient
                    .signature_cache
                    .lock()
                    .await
                    .cache_signatures(sender_last_seen.as_ref());
            }
            recipient.chat_session.lock().await.messages_received += 1;
        }

        sender.chat_session.lock().await.messages_sent += 1;
    }

    pub fn broadcast_packet_except_editioned_sync<J: ClientPacket, B: BClientPacket>(
        &self,
        except: &[uuid::Uuid],
        je_packet: &J,
        be_packet: &B,
    ) {
        let players = self.players.load();
        let mut java_recipients = Vec::new();

        for p in players.iter() {
            if except.contains(&p.gameprofile.id) {
                continue;
            }
            match p.client.as_ref() {
                ClientPlatform::Java(_) => java_recipients.push(p),
                ClientPlatform::Bedrock(be_client) => be_client.try_enqueue_packet(be_packet),
            }
        }

        let recipients_by_version =
            Self::collect_java_recipients_by_version(java_recipients.into_iter());
        Self::broadcast_java_grouped(je_packet, recipients_by_version);
    }

    pub async fn broadcast_packet_except_editioned<J: ClientPacket, B: BClientPacket>(
        &self,
        except: &[uuid::Uuid],
        je_packet: &J,
        be_packet: &B,
    ) {
        let players = self.players.load();
        let mut java_recipients = Vec::new();
        let mut bedrock_recipients = Vec::new();

        for p in players.iter() {
            if except.contains(&p.gameprofile.id) {
                continue;
            }
            match p.client.as_ref() {
                ClientPlatform::Java(_) => java_recipients.push(p),
                ClientPlatform::Bedrock(be_client) => bedrock_recipients.push(be_client.clone()),
            }
        }

        let recipients_by_version =
            Self::collect_java_recipients_by_version(java_recipients.into_iter());
        Self::broadcast_java_grouped(je_packet, recipients_by_version);

        for be_client in bedrock_recipients {
            be_client.enqueue_packet(be_packet).await;
        }
    }

    /// Broadcasts a packet to all connected players within the world, excluding the specified players.
    ///
    /// Sends the specified packet to every player currently logged in to the world, excluding the players listed in the `except` parameter.
    ///
    /// **Note:** This function acquires a lock on the `current_players` map, ensuring thread safety.
    pub fn broadcast_packet_except<P: ClientPacket>(&self, except: &[uuid::Uuid], packet: &P) {
        let players = self.players.load();
        let recipients_by_version = Self::collect_java_recipients_by_version(
            players
                .iter()
                .filter(|candidate| !except.contains(&candidate.gameprofile.id)),
        );
        Self::broadcast_java_grouped(packet, recipients_by_version);
    }

    pub fn spawn_particle(
        &self,
        position: Vector3<f64>,
        offset: Vector3<f32>,
        max_speed: f32,
        particle_count: i32,
        particle: Particle,
    ) {
        for player in self.players.load().iter() {
            player.spawn_particle(position, offset, max_speed, particle_count, particle);
        }
    }

    pub fn play_sound(&self, sound: Sound, category: SoundCategory, position: &Vector3<f64>) {
        self.play_sound_raw(sound as u16, category, position, 1.0, 1.0);
    }

    pub fn play_sound_event(
        &self,
        sound: &pumpkin_data::data_component_impl::IdOr<
            pumpkin_data::data_component_impl::SoundEvent,
        >,
        category: SoundCategory,
        position: &Vector3<f64>,
    ) {
        // Same path as play_sound_raw so distance filtering matches vanilla.
        let seed = rng().random::<i64>();
        let packet = CSoundEffect::new(
            data_to_proto_sound(sound),
            category,
            position,
            1.0,
            1.0,
            seed,
        );
        self.broadcast_sound_packet(position, 1.0, &packet, None);
    }

    pub fn play_sound_fine(
        &self,
        sound: Sound,
        category: SoundCategory,
        position: &Vector3<f64>,
        volume: f32,
        pitch: f32,
    ) {
        self.play_sound_raw(sound as u16, category, position, volume, pitch);
    }

    pub fn play_sound_expect(
        &self,
        player: &Player,
        sound: Sound,
        category: SoundCategory,
        position: &Vector3<f64>,
    ) {
        self.play_sound_raw_expect(player, sound as u16, category, position, 1.0, 1.0);
    }

    /// Vanilla `PlayerManager.sendToAround` range for a sound with the given volume:
    /// `volume > 1.0 ? 16.0 * volume : 16.0` (euclidean blocks).
    #[must_use]
    pub fn sound_hear_distance(volume: f32) -> f64 {
        if volume > 1.0 {
            16.0 * f64::from(volume)
        } else {
            16.0
        }
    }

    fn broadcast_sound_packet(
        &self,
        position: &Vector3<f64>,
        volume: f32,
        packet: &CSoundEffect,
        except: Option<uuid::Uuid>,
    ) {
        let max_dist_sq = {
            let d = Self::sound_hear_distance(volume);
            d * d
        };
        let players = self.players.load();
        let recipients = players.iter().filter(|p| {
            if except.is_some_and(|id| p.gameprofile.id == id) {
                return false;
            }
            let player_pos = p.position();
            player_pos.squared_distance_to_vec(position) <= max_dist_sq
        });
        let recipients_by_version = Self::collect_java_recipients_by_version(recipients);
        Self::broadcast_java_grouped(packet, recipients_by_version);
    }

    pub fn play_sound_raw(
        &self,
        sound_id: u16,
        category: SoundCategory,
        position: &Vector3<f64>,
        volume: f32,
        pitch: f32,
    ) {
        // Vanilla uses ThreadLocalRandom.nextLong() for the sound seed.
        let seed = rand::rng().random::<i64>();
        let packet = CSoundEffect::new(IdOr::Id(sound_id), category, position, volume, pitch, seed);
        self.broadcast_sound_packet(position, volume, &packet, None);
    }

    pub fn play_sound_raw_expect(
        &self,
        player: &Player,
        sound_id: u16,
        category: SoundCategory,
        position: &Vector3<f64>,
        volume: f32,
        pitch: f32,
    ) {
        let seed = rand::rng().random::<i64>();
        let packet = CSoundEffect::new(IdOr::Id(sound_id), category, position, volume, pitch, seed);

        let max_dist_sq = {
            let d = Self::sound_hear_distance(volume);
            d * d
        };
        let players = self.players.load();
        let recipients = players.iter().filter(|p| {
            // Skip the expected player
            if p.gameprofile.id == player.gameprofile.id {
                return false;
            }

            let player_pos = p.position();
            player_pos.squared_distance_to_vec(position) <= max_dist_sq
        });

        let recipients_by_version = Self::collect_java_recipients_by_version(recipients);
        Self::broadcast_java_grouped(&packet, recipients_by_version);
    }

    pub fn play_block_sound(&self, sound: Sound, category: SoundCategory, position: BlockPos) {
        let new_vec = Vector3::new(
            f64::from(position.0.x) + 0.5,
            f64::from(position.0.y) + 0.5,
            f64::from(position.0.z) + 0.5,
        );
        self.play_sound(sound, category, &new_vec);
    }

    pub fn play_block_sound_expect(
        &self,
        player: &Player,
        sound: Sound,
        category: SoundCategory,
        position: BlockPos,
    ) {
        let new_vec = Vector3::new(
            f64::from(position.0.x) + 0.5,
            f64::from(position.0.y) + 0.5,
            f64::from(position.0.z) + 0.5,
        );
        self.play_sound_expect(player, sound, category, &new_vec);
    }

    pub async fn send_world_info(
        &self,
        player: &Arc<Player>,
        position: Vector3<f64>,
        yaw: f32,
        pitch: f32,
    ) {
        if let ClientPlatform::Java(client) = player.client.as_ref() {
            self.worldborder.lock().await.init_client(client).await;
        }

        // TODO: World spawn (compass stuff)

        player
            .client
            .enqueue_packet(&CGameEvent::new(GameEvent::StartWaitingChunks, 0.0))
            .await;

        let entity = &player.get_entity();

        self.broadcast_packet_except(
            &[player.gameprofile.id],
            // TODO: add velo
            &CSpawnEntity::new(
                entity.entity_id.into(),
                player.gameprofile.id,
                i32::from(EntityType::PLAYER.id).into(),
                position,
                pitch,
                yaw,
                yaw,
                0.into(),
                Vector3::new(0.0, 0.0, 0.0),
            ),
        );

        player.send_client_information();

        chunker::update_position(player).await;
        // Update commands

        player.set_health(20.0).await;
    }

    pub fn sync_world_event(&self, world_event: WorldEvent, position: BlockPos, data: i32) {
        let chunk_pos = position.chunk_position();
        self.broadcast_to_chunk(
            chunk_pos,
            &CWorldEvent::new(world_event as i32, position, data, false),
        );
    }

    /// Broadcasts a packet to all players who currently have the target chunk loaded.
    /// This uses highly optimized Chebyshev distance math (Chunk Grid) instead of floating point distance checks.
    pub fn broadcast_to_chunk<P: ClientPacket>(&self, chunk_pos: Vector2<i32>, packet: &P) {
        let players = self.players.load();

        let recipients = players.iter().filter(|p| {
            let center = p.get_entity().chunk_pos.load();
            let view_distance = get_view_distance(p).get() as i32;

            // Chebyshev distance (Minecraft's chunk loading shape)
            is_within_view_distance(chunk_pos, center, view_distance)
        });

        let recipients_by_version = Self::collect_java_recipients_by_version(recipients);
        Self::broadcast_java_grouped(packet, recipients_by_version);
    }

    pub fn broadcast_to_chunk_editioned_sync<J: ClientPacket, B: BClientPacket>(
        &self,
        chunk_pos: Vector2<i32>,
        je_packet: &J,
        be_packet: &B,
    ) {
        let players = self.players.load();
        let mut java_recipients = Vec::new();

        let recipients = players.iter().filter(|p| {
            let center = p.get_entity().chunk_pos.load();
            let view_distance = get_view_distance(p).get() as i32;
            is_within_view_distance(chunk_pos, center, view_distance)
        });

        for p in recipients {
            match p.client.as_ref() {
                ClientPlatform::Java(_) => java_recipients.push(p),
                ClientPlatform::Bedrock(be_client) => be_client.try_enqueue_packet(be_packet),
            }
        }

        let recipients_by_version =
            Self::collect_java_recipients_by_version(java_recipients.into_iter());
        Self::broadcast_java_grouped(je_packet, recipients_by_version);
    }

    /// Broadcasts a packet to chunk watchers, excluding specific players.
    pub fn broadcast_to_chunk_except<P: ClientPacket>(
        &self,
        chunk_pos: Vector2<i32>,
        except: &[uuid::Uuid],
        packet: &P,
    ) {
        let players = self.players.load();

        let recipients = players.iter().filter(|p| {
            if except.contains(&p.get_entity().entity_uuid) {
                return false;
            }
            let center = p.get_entity().chunk_pos.load();
            let view_distance = get_view_distance(p).get() as i32;

            is_within_view_distance(chunk_pos, center, view_distance)
        });

        let recipients_by_version = Self::collect_java_recipients_by_version(recipients);
        Self::broadcast_java_grouped(packet, recipients_by_version);
    }

    pub async fn broadcast_to_chunk_except_editioned<J: ClientPacket, B: BClientPacket>(
        &self,
        chunk_pos: Vector2<i32>,
        except: &[uuid::Uuid],
        je_packet: &J,
        be_packet: &B,
    ) {
        let players = self.players.load();
        let recipients = players.iter().filter(|p| {
            if except.contains(&p.get_entity().entity_uuid) {
                return false;
            }
            let center = p.get_entity().chunk_pos.load();
            let view_distance = get_view_distance(p).get() as i32;

            is_within_view_distance(chunk_pos, center, view_distance)
        });

        let mut java_recipients = Vec::new();
        let mut bedrock_recipients = Vec::new();

        for p in recipients {
            match p.client.as_ref() {
                ClientPlatform::Java(_) => java_recipients.push(p),
                ClientPlatform::Bedrock(be_client) => bedrock_recipients.push(be_client.clone()),
            }
        }

        let je_recipients_by_version =
            Self::collect_java_recipients_by_version(java_recipients.into_iter());
        Self::broadcast_java_grouped(je_packet, je_recipients_by_version);

        for recipient in bedrock_recipients {
            recipient.enqueue_packet(be_packet).await;
        }
    }
}
