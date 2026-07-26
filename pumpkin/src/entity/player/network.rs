use super::Player;
use super::statistics;
use crate::data::SaveJSONConfiguration;
use crate::entity::EntityBase;
use crate::net::ClientPlatform;
use crate::net::DisconnectReason;
use crate::plugin::server::packet::PacketSentEvent;
use crate::server::Server;
use crate::world::World;
use bytes::Bytes;
use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::Taggable;
use pumpkin_data::translation;
use pumpkin_protocol::bedrock::client::set_time::CSetTime;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::CAwardStats;
use pumpkin_protocol::java::client::play::CChangeDifficulty;
use pumpkin_protocol::java::client::play::CCustomPayload;
use pumpkin_protocol::java::client::play::CUpdateTime;
use pumpkin_protocol::java::client::play::Statistic;
use pumpkin_util::text::TextComponent;
use std::sync::Arc;
use std::sync::atomic::Ordering;

impl Player {
    pub async fn fire_packet_sent<P: Send + Sync + std::any::Any>(
        self: &Arc<Self>,
        packet: P,
        packet_id: i32,
        payload: Bytes,
    ) -> bool {
        if let Some(server) = self.world().server.upgrade() {
            let event = PacketSentEvent::new(self.clone(), packet_id, payload, Arc::new(packet));
            let event = server.plugin_manager.fire(event).await;
            return event.cancelled;
        }
        false
    }

    pub async fn fire_packet_sent_no_obj(self: &Arc<Self>, packet_id: i32, payload: Bytes) -> bool {
        if let Some(server) = self.world().server.upgrade() {
            // This is a dummy object to satisfy the non-optional requirement in WIT
            // In the future we should make all packets 'static or have a way to represent raw packets in WIT
            struct RawPacket;
            let event = PacketSentEvent::new(self.clone(), packet_id, payload, Arc::new(RawPacket));
            let event = server.plugin_manager.fire(event).await;
            return event.cancelled;
        }
        false
    }

    pub async fn send_stats(&self) {
        if let ClientPlatform::Java(java) = self.client.as_ref() {
            let stats_guard = self.stats.lock().await;
            let packet_stats: Vec<Statistic> = stats_guard
                .stats
                .iter()
                .map(|((category, stat), value)| Statistic {
                    category_id: VarInt(*category),
                    statistic_id: VarInt(*stat),
                    value: VarInt(*value),
                })
                .collect();

            java.enqueue_packet(&CAwardStats {
                stats: &packet_stats,
            })
            .await;
        }
    }

    pub async fn increment_stat(
        &self,
        category: statistics::StatisticCategory,
        stat: i32,
        amount: i32,
    ) {
        self.stats.lock().await.increment(category, stat, amount);
    }

    pub async fn set_stat(&self, category: statistics::StatisticCategory, stat: i32, value: i32) {
        self.stats.lock().await.set(category, stat, value);
    }

    pub async fn get_movement_statistic(&self) -> statistics::CustomStatistic {
        let entity = self.get_entity();
        if entity.has_vehicle().await {
            let vehicle = entity.vehicle.lock().await;
            if let Some(vehicle) = vehicle.as_ref() {
                let entity_type = vehicle.get_entity().entity_type;
                if entity_type == &EntityType::OAK_BOAT
                    || entity_type == &EntityType::SPRUCE_BOAT
                    || entity_type == &EntityType::BIRCH_BOAT
                    || entity_type == &EntityType::JUNGLE_BOAT
                    || entity_type == &EntityType::ACACIA_BOAT
                    || entity_type == &EntityType::DARK_OAK_BOAT
                    || entity_type == &EntityType::MANGROVE_BOAT
                    || entity_type == &EntityType::CHERRY_BOAT
                    || entity_type == &EntityType::BAMBOO_RAFT
                {
                    return statistics::CustomStatistic::BoatOneCm;
                }
                if entity_type == &EntityType::MINECART
                    || entity_type == &EntityType::CHEST_MINECART
                    || entity_type == &EntityType::FURNACE_MINECART
                    || entity_type == &EntityType::TNT_MINECART
                    || entity_type == &EntityType::HOPPER_MINECART
                    || entity_type == &EntityType::COMMAND_BLOCK_MINECART
                    || entity_type == &EntityType::SPAWNER_MINECART
                {
                    return statistics::CustomStatistic::MinecartOneCm;
                }
                if entity_type == &EntityType::HORSE
                    || entity_type == &EntityType::DONKEY
                    || entity_type == &EntityType::MULE
                    || entity_type == &EntityType::SKELETON_HORSE
                    || entity_type == &EntityType::ZOMBIE_HORSE
                {
                    return statistics::CustomStatistic::HorseOneCm;
                }
                if entity_type == &EntityType::PIG {
                    return statistics::CustomStatistic::PigOneCm;
                }
                if entity_type == &EntityType::STRIDER {
                    return statistics::CustomStatistic::StriderOneCm;
                }
            }
        }

        if self.is_flying().await {
            return statistics::CustomStatistic::FlyOneCm;
        }

        if entity.fall_flying.load(Ordering::Relaxed) {
            return statistics::CustomStatistic::AviateOneCm;
        }

        if entity.swimming.load(Ordering::Relaxed) {
            return statistics::CustomStatistic::SwimOneCm;
        }

        let pos = entity.block_pos.load();
        let world = entity.world.load_full();
        let block = world.get_block(&pos);
        if block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_CLIMBABLE) {
            return statistics::CustomStatistic::ClimbOneCm;
        }

        if entity.touching_water.load(Ordering::Relaxed) {
            return statistics::CustomStatistic::WalkUnderWaterOneCm;
        }

        if entity.sneaking.load(Ordering::Relaxed) {
            return statistics::CustomStatistic::CrouchOneCm;
        }

        if entity.sprinting.load(Ordering::Relaxed) {
            return statistics::CustomStatistic::SprintOneCm;
        }

        if !entity.on_ground.load(Ordering::Relaxed) && entity.velocity.load().y < -0.005 {
            return statistics::CustomStatistic::FallOneCm;
        }

        statistics::CustomStatistic::WalkOneCm
    }

    /// Sets the player's difficulty level.
    pub async fn send_difficulty_update(&self) {
        let world = self.world();
        let level_info = world.level_info.load();
        self.client
            .enqueue_packet_editioned(
                &CChangeDifficulty::new(level_info.difficulty as u8, level_info.difficulty_locked),
                &pumpkin_protocol::bedrock::client::CSetDifficulty::new(
                    level_info.difficulty as u32,
                ),
            )
            .await;
    }

    /// Sends the world time to only this player.
    pub async fn send_time(&self, world: &World) {
        let l_world = world.level_time.lock().await;
        self.client
            .enqueue_packet_editioned(
                &CUpdateTime::new(l_world.world_age, l_world.time_of_day, true),
                &CSetTime::new(l_world.query_daytime() as _),
            )
            .await;
    }

    pub async fn kick(&self, reason: DisconnectReason, message: TextComponent) {
        self.client.kick(reason, message).await;
    }

    pub async fn ban(&self, server: &Server, reason: Option<TextComponent>) {
        let mut banned_players = server.data.banned_player_list.write().await;
        let string_reason = reason.clone().map_or_else(
            || "Banned by an operator.".to_string(),
            pumpkin_util::text::TextComponent::get_text,
        );

        if banned_players
            .banned_players
            .iter()
            .any(|entry| entry.uuid == self.gameprofile.id)
        {
            return;
        }

        banned_players.banned_players.push(
            crate::data::banlist_serializer::BannedPlayerEntry::new(
                &self.gameprofile,
                "Plugin".to_string(),
                None,
                string_reason,
            ),
        );

        banned_players.save();
        drop(banned_players);

        let kick_reason = reason.unwrap_or_else(|| {
            TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_BANNED,
                translation::bedrock::DISCONNECTIONSCREEN_TITLE_BANNEDBYHOST,
                [],
            )
        });

        self.kick(DisconnectReason::Kicked, kick_reason).await;
    }

    pub async fn ban_ip(&self, server: &Server, reason: Option<TextComponent>) {
        let mut banned_ips = server.data.banned_ip_list.write().await;
        let string_reason = reason.clone().map_or_else(
            || "Banned by an operator.".to_string(),
            pumpkin_util::text::TextComponent::get_text,
        );
        let target_ip = self.client.address().await.ip();

        if banned_ips.get_entry(&target_ip).is_some() {
            return;
        }

        banned_ips
            .banned_ips
            .push(crate::data::banlist_serializer::BannedIpEntry::new(
                target_ip,
                "Plugin".to_string(),
                None,
                string_reason,
            ));

        banned_ips.save();
        drop(banned_ips);

        let kick_reason = reason.unwrap_or_else(|| {
            TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_IP_BANNED,
                translation::java::MULTIPLAYER_DISCONNECT_IP_BANNED,
                [],
            )
        });

        let affected = server.get_players_by_ip(target_ip).await;
        for target in affected {
            target
                .kick(DisconnectReason::Kicked, kick_reason.clone())
                .await;
        }
    }

    /// Sends a custom payload packet to this player (Java edition only).
    pub async fn send_custom_payload(&self, channel: &str, data: &[u8]) {
        if let ClientPlatform::Java(java) = self.client.as_ref() {
            java.enqueue_packet(&CCustomPayload::new(channel, data))
                .await;
        }
    }
}
