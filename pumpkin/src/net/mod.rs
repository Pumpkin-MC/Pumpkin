use std::{
    net::SocketAddr,
    num::NonZeroU8,
    sync::{Arc, atomic::Ordering},
};

use crate::{
    data::{
        banned_ip_data::BANNED_IP_LIST, banned_player_data::BANNED_PLAYER_LIST,
        op_data::OPERATOR_CONFIG, whitelist_data::WHITELIST_CONFIG,
    },
    entity::player::{ChatMode, Hand},
    net::{bedrock::BedrockClientPlatform, java::JavaClientPlatform},
    server::Server,
};

use pumpkin_protocol::{ClientPacket, Property};
use pumpkin_util::{ProfileAction, text::TextComponent};
use serde::Deserialize;
use sha1::Digest;
use sha2::Sha256;
use simplelog::FormatItem;
use tokio::task::JoinHandle;

use thiserror::Error;
use uuid::Uuid;
pub mod authentication;
pub mod bedrock;
pub mod java;
pub mod lan_broadcast;
mod proxy;
pub mod query;
pub mod rcon;

#[derive(Deserialize, Clone, Debug)]
pub struct GameProfile {
    pub id: Uuid,
    pub name: String,
    pub properties: Vec<Property>,
    #[serde(rename = "profileActions")]
    pub profile_actions: Option<Vec<ProfileAction>>,
}

pub fn offline_uuid(username: &str) -> Result<Uuid, uuid::Error> {
    Uuid::from_slice(&Sha256::digest(username)[..16])
}

/// Represents a player's configuration settings.
///
/// This struct contains various options that can be customized by the player, affecting their gameplay experience.
///
/// **Usage:**
///
/// This struct is typically used to store and manage a player's preferences. It can be sent to the server when a player joins or when they change their settings.
#[derive(Clone)]
pub struct PlayerConfig {
    /// The player's preferred language.
    pub locale: String, // 16
    /// The maximum distance at which chunks are rendered.
    pub view_distance: NonZeroU8,
    /// The player's chat mode settings
    pub chat_mode: ChatMode,
    /// Whether chat colors are enabled.
    pub chat_colors: bool,
    /// The player's skin configuration options.
    pub skin_parts: u8,
    /// The player's dominant hand (left or right).
    pub main_hand: Hand,
    /// Whether text filtering is enabled.
    pub text_filtering: bool,
    /// Whether the player wants to appear in the server list.
    pub server_listing: bool,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            locale: "en_us".to_string(),
            view_distance: NonZeroU8::new(10).unwrap(),
            chat_mode: ChatMode::Enabled,
            chat_colors: true,
            skin_parts: 0,
            main_hand: Hand::Right,
            text_filtering: false,
            server_listing: false,
        }
    }
}

pub enum PacketHandlerState {
    PacketReady,
    Stop,
}

/// This is just a Wrapper for both Java & Bedrock connections
pub enum ClientPlatform {
    Java(Arc<JavaClientPlatform>),
    Bedrock(Arc<BedrockClientPlatform>),
}

impl ClientPlatform {
    pub async fn address(&self) -> SocketAddr {
        match self {
            Self::Java(java) => *java.address.lock().await,
            Self::Bedrock(bedrock) => bedrock.address,
        }
    }

    #[must_use]
    pub fn closed(&self) -> bool {
        match self {
            Self::Java(java) => java.closed.load(Ordering::Relaxed),
            Self::Bedrock(bedrock) => bedrock.closed.load(Ordering::Relaxed),
        }
    }

    #[must_use]
    pub fn protocol_version(&self) -> i32 {
        match self {
            Self::Java(java) => java.protocol_version.load(Ordering::Relaxed),
            Self::Bedrock(bedrock) => bedrock.protocol_version.load(Ordering::Relaxed),
        }
    }

    pub async fn await_close_interrupt(&self) {
        match self {
            Self::Java(java) => java.await_close_interrupt().await,
            Self::Bedrock(bedrock) => bedrock.await_close_interrupt().await,
        }
    }

    pub fn spawn_task<F>(&self, task: F) -> Option<JoinHandle<F::Output>>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        match self {
            Self::Java(java) => java.spawn_task(task),
            Self::Bedrock(bedrock) => bedrock.spawn_task(task),
        }
    }

    pub async fn enqueue_packet<P>(&self, packet: &P)
    where
        P: ClientPacket,
    {
        match self {
            Self::Java(java) => java.enqueue_packet(packet).await,
            Self::Bedrock(bedrock) => bedrock.enqueue_packet(packet).await,
        }
    }

    pub async fn send_packet_now<P: ClientPacket>(&self, packet: &P) {
        match self {
            Self::Java(java) => java.send_packet_now(packet).await,
            Self::Bedrock(_bedrock) => {
                // bedrock
                //     .send_game_packet(packet, pumpkin_protocol::bedrock::RakReliability::Reliable)
                //     .await;
            }
        }
    }

    pub async fn kick(&self, reason: TextComponent) {
        match self {
            Self::Java(java) => java.kick(reason).await,
            Self::Bedrock(bedrock) => bedrock.kick(reason).await,
        }
    }
}

pub async fn can_not_join(
    profile: &GameProfile,
    address: &SocketAddr,
    server: &Server,
) -> Option<TextComponent> {
    const FORMAT_DESCRIPTION: &[FormatItem<'_>] = time::macros::format_description!(
        "[year]-[month]-[day] at [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute]"
    );

    let mut banned_players = BANNED_PLAYER_LIST.write().await;
    if let Some(entry) = banned_players.get_entry(profile) {
        let text = TextComponent::translate(
            "multiplayer.disconnect.banned.reason",
            [TextComponent::text(entry.reason.clone())],
        );
        return Some(match entry.expires {
            Some(expires) => text.add_child(TextComponent::translate(
                "multiplayer.disconnect.banned.expiration",
                [TextComponent::text(
                    expires.format(FORMAT_DESCRIPTION).unwrap().to_string(),
                )],
            )),
            None => text,
        });
    }
    drop(banned_players);

    if server.white_list.load(Ordering::Relaxed) {
        let ops = OPERATOR_CONFIG.read().await;
        let whitelist = WHITELIST_CONFIG.read().await;

        if ops.get_entry(&profile.id).is_none() && !whitelist.is_whitelisted(profile) {
            return Some(TextComponent::translate(
                "multiplayer.disconnect.not_whitelisted",
                &[],
            ));
        }
    }

    let mut banned_ips = BANNED_IP_LIST.write().await;
    if let Some(entry) = banned_ips.get_entry(&address.ip()) {
        let text = TextComponent::translate(
            "multiplayer.disconnect.banned_ip.reason",
            [TextComponent::text(entry.reason.clone())],
        );
        return Some(match entry.expires {
            Some(expires) => text.add_child(TextComponent::translate(
                "multiplayer.disconnect.banned_ip.expiration",
                [TextComponent::text(
                    expires.format(FORMAT_DESCRIPTION).unwrap().to_string(),
                )],
            )),
            None => text,
        });
    }

    None
}

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("failed to decrypt shared secret")]
    FailedDecrypt,
    #[error("shared secret has the wrong length")]
    SharedWrongLength,
}

fn is_valid_player_name(name: &str) -> bool {
    name.len() <= 16 && name.chars().all(|c| c > 32u8 as char && c < 127u8 as char)
}
