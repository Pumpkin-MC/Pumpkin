use std::{num::NonZero, sync::Arc, sync::atomic::Ordering};

use crate::{
    entity::player::ChatMode,
    net::{
        PlayerConfig, can_not_join,
        java::{JavaClient, PacketHandlerResult},
    },
    server::Server,
};
use core::str;
use pumpkin_data::registry::Registry;
use pumpkin_protocol::{
    ConnectionState, KnownPack,
    java::{
        client::config::{CFeatureFlags, CFinishConfig, CKnownPacks, CRegistryData, CUpdateTags},
        server::config::{
            ResourcePackResponseResult, SClientInformationConfig, SConfigCookieResponse,
            SConfigResourcePack, SKeepAlive, SPluginMessage,
        },
    },
};
use pumpkin_util::{Hand, text::TextComponent};
use tracing::{debug, trace, warn};

const BRAND_CHANNEL_PREFIX: &str = "minecraft:brand";

pub mod client_information;
pub mod config_acknowledged;
pub(super) use config_acknowledged::build_dimension_nbt;
pub mod cookie_response;
pub mod keep_alive;
pub mod known_packs;
pub mod plugin_message;
pub mod resource_pack;

impl JavaClient {
    pub async fn handle_config_packet(
        &self,
        server: &Arc<Server>,
        packet: &pumpkin_protocol::RawPacket,
    ) -> Result<(), pumpkin_protocol::ser::ReadingError> {
        use pumpkin_protocol::java::server::config::{
            SAcceptCodeOfConduct, SAcknowledgeFinishConfig, SConfigPong, SKnownPacks,
        };
        use pumpkin_protocol::{ServerPacket, packet::MultiVersionJavaPacket};
        let version = self.version.load();
        let mut payload = packet.payload.as_ref();
        match packet.id {
            id if id == SClientInformationConfig::to_id(version) => {
                self.handle_client_information_config(SClientInformationConfig::read(
                    &mut payload,
                    &version,
                )?)
                .await;
            }
            id if id == SPluginMessage::to_id(version) => {
                self.handle_plugin_message(SPluginMessage::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SKeepAlive::to_id(version) => {
                self.handle_config_keep_alive(&SKeepAlive::read(&mut payload, &version)?);
            }
            id if id == SAcknowledgeFinishConfig::to_id(version) => {
                let _ = SAcknowledgeFinishConfig::read(&mut payload, &version)?;
                if !self.user.acknowledge_state(ConnectionState::Play) {
                    return Err(pumpkin_protocol::ser::ReadingError::Message(
                        "Unexpected configuration acknowledgement".into(),
                    ));
                }
                if let PacketHandlerResult::ReadyToPlay(_, config) =
                    self.handle_config_acknowledged(server).await
                    && let Some(player) = self.user.player()
                {
                    player.config.store(Arc::new(config));
                }
            }
            id if id == SKnownPacks::to_id(version) => {
                let _ = SKnownPacks::read(&mut payload, &version)?;
                let _ = self.handle_known_packs(server).await;
            }
            id if id == SConfigResourcePack::to_id(version) => {
                self.handle_resource_pack_response(
                    server,
                    SConfigResourcePack::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SConfigCookieResponse::to_id(version) => {
                self.handle_config_cookie_response(&SConfigCookieResponse::read(
                    &mut payload,
                    &version,
                )?);
            }
            id if id == SConfigPong::to_id(version) => {
                let _ = SConfigPong::read(&mut payload, &version)?;
            }
            id if id == SAcceptCodeOfConduct::to_id(version) => {
                let _ = SAcceptCodeOfConduct::read(&mut payload, &version)?;
            }
            _ => {
                return Err(pumpkin_protocol::ser::ReadingError::Message(format!(
                    "Unknown configuration packet {}",
                    packet.id
                )));
            }
        }
        Ok(())
    }
}
