use arc_swap::ArcSwap;
use pumpkin_data::translation;
use pumpkin_protocol::{
    ConnectionState, KnownPack, Label, Link, LinkType,
    java::client::{
        config::{
            CConfigAddResourcePack, CConfigServerLinks, CFeatureFlags, CFinishConfig, CKnownPacks,
            CRegistryData, CUpdateTags,
        },
        login::{CLoginSuccess, CSetCompression},
    },
    java::server::config::SKnownPacks,
    java::server::login::{
        SEncryptionResponse, SLoginCookieResponse, SLoginPluginResponse, SLoginStart,
    },
};
use pumpkin_util::{text::TextComponent, version::JavaMinecraftVersion};
use std::sync::Arc;
use tracing::debug;
use uuid::Uuid;

use crate::{
    net::{
        GameProfile,
        authentication::{self, AuthError},
        is_valid_player_name,
        java::pending::PendingConnection,
        offline_uuid,
        proxy::{bungeecord, velocity},
    },
    server::Server,
};

impl PendingConnection {
    pub async fn handle_login_start(&mut self, server: &Server, login_start: SLoginStart) {
        debug!("login start");

        let max_players = server.advanced_config.networking.java.max_players;
        if max_players > 0 && server.get_player_count() >= max_players as usize {
            self.kick(TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_SERVER_FULL,
                translation::bedrock::DISCONNECTIONSCREEN_SERVERFULL,
                [],
            ))
            .await;
            return;
        }

        if !is_valid_player_name(&login_start.name) {
            self.kick(TextComponent::text("Invalid characters in username"))
                .await;
            return;
        }

        let proxy = &server.advanced_config.networking.proxy;
        if proxy.enabled {
            if proxy.velocity.enabled {
                velocity::velocity_login(self).await;
            } else if proxy.bungeecord.enabled {
                match bungeecord::bungeecord_login(
                    &self.address,
                    &self.server_address,
                    login_start.name.into_string(),
                ) {
                    Ok((_ip, profile)) => {
                        self.finish_login(&profile).await;
                        self.gameprofile = Some(profile);
                    }
                    Err(error) => self.kick(TextComponent::text(error.to_string())).await,
                }
            }
        } else {
            let id = if server.advanced_config.networking.java.online_mode {
                login_start.uuid
            } else {
                offline_uuid(&login_start.name).unwrap_or_else(|_| uuid::Uuid::nil())
            };

            let profile = GameProfile {
                id,
                name: login_start.name.into_string(),
                properties: ArcSwap::new(Arc::new(vec![])),
                profile_actions: None,
            };

            if server.advanced_config.networking.java.compression.enabled {
                self.enable_compression(server).await;
            }

            if server.advanced_config.networking.java.encryption {
                let verify_token: [u8; 4] = rand::random();
                self.send_packet_now(
                    &server
                        .encryption_request(
                            &verify_token,
                            server.advanced_config.networking.java.online_mode,
                        )
                        .await,
                )
                .await;
            } else {
                self.finish_login(&profile).await;
            }

            self.gameprofile = Some(profile);
        }
    }

    pub async fn handle_encryption_response(
        &mut self,
        server: &Server,
        encryption_response: SEncryptionResponse,
    ) {
        debug!("Handling encryption");
        let Ok(shared_secret) = server.decrypt(&encryption_response.shared_secret).await else {
            self.kick(TextComponent::text("Failed to decrypt shared secret"))
                .await;
            return;
        };

        if let Err(error) = self.set_encryption(&shared_secret) {
            self.kick(TextComponent::text(error.to_string())).await;
            return;
        }

        let profile_name = {
            let Some(profile) = self.gameprofile.as_ref() else {
                self.kick(TextComponent::text("No `GameProfile`")).await;
                return;
            };
            profile.name.clone()
        };

        if server.advanced_config.networking.java.online_mode {
            match self
                .authenticate(server, &shared_secret, &profile_name)
                .await
            {
                Ok(new_profile) => self.gameprofile = Some(new_profile),
                Err(error) => {
                    self.kick(match error {
                        AuthError::FailedResponse => TextComponent::translate_cross(
                            translation::java::MULTIPLAYER_DISCONNECT_AUTHSERVERS_DOWN,
                            translation::bedrock::DISCONNECT_LOGINFAILEDINFO_SERVERSUNAVAILABLE,
                            [],
                        ),
                        AuthError::UnverifiedUsername => TextComponent::translate_cross(
                            translation::java::MULTIPLAYER_DISCONNECT_UNVERIFIED_USERNAME,
                            translation::bedrock::DISCONNECT_LOGINFAILEDINFO_INVALIDSESSION,
                            [],
                        ),
                        e => TextComponent::text(e.to_string()),
                    })
                    .await;
                    return;
                }
            }
        }

        let Some(profile) = self.gameprofile.clone() else {
            return;
        };

        if let Some(online_player) = &server.get_player_by_uuid(profile.id) {
            debug!(
                "Player (IP '{}', username '{}') tried to log in with the same UUID ('{}') as an online player (username '{}')",
                &self.address, &profile.name, &profile.id, &online_player.gameprofile.name
            );
            self.kick(TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_DUPLICATE_LOGIN,
                translation::bedrock::DISCONNECTIONSCREEN_LOGGEDINOTHERLOCATION,
                [],
            ))
            .await;
            return;
        }

        if let Some(online_player) = &server.get_player_by_name(&profile.name) {
            debug!(
                "A player (IP '{}', attempted username '{}') tried to log in with the same username as an online player (UUID '{}', username '{}')",
                &self.address, &profile.name, &profile.id, &online_player.gameprofile.name
            );
            self.kick(TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_DUPLICATE_LOGIN,
                translation::bedrock::DISCONNECTIONSCREEN_LOGGEDINOTHERLOCATION,
                [],
            ))
            .await;
            return;
        }

        self.finish_login(&profile).await;
    }

    async fn enable_compression(&mut self, server: &Server) {
        if self.version.load() < JavaMinecraftVersion::V_1_8 {
            return;
        }
        let compression = server
            .advanced_config
            .networking
            .java
            .compression
            .info
            .clone();
        self.send_packet_now(&CSetCompression::new(
            pumpkin_protocol::codec::var_int::VarInt(compression.threshold as i32),
        ))
        .await;
        self.set_compression(&compression);
    }

    async fn finish_login(&mut self, profile: &GameProfile) {
        let props = profile.properties.load();
        let packet = CLoginSuccess::new(
            &profile.id,
            &profile.name,
            &props,
            false,
            uuid::Uuid::new_v4(),
        );
        self.send_packet_now(&packet).await;
        if self.version.load() < JavaMinecraftVersion::V_1_20_2 {
            self.connection_state.store(ConnectionState::Play);
        }
    }

    async fn authenticate(
        &self,
        server: &Server,
        shared_secret: &[u8],
        username: &str,
    ) -> Result<GameProfile, AuthError> {
        let hash = server.digest_secret(shared_secret).await;
        let ip = self.address.ip();
        let profile = authentication::authenticate(
            username,
            &hash,
            &ip,
            &server.advanced_config.networking.java.authentication,
        )?;

        if let Some(actions) = &profile.profile_actions {
            if server
                .advanced_config
                .networking
                .java
                .authentication
                .player_profile
                .allow_banned_players
            {
                for allowed in &server
                    .advanced_config
                    .networking
                    .java
                    .authentication
                    .player_profile
                    .allowed_actions
                {
                    if !actions.contains(allowed) {
                        return Err(AuthError::DisallowedAction);
                    }
                }
                if !actions.is_empty() {
                    return Err(AuthError::Banned);
                }
            } else if !actions.is_empty() {
                return Err(AuthError::Banned);
            }
        }
        for property in profile.properties.load().iter() {
            authentication::validate_textures(
                property,
                &server
                    .advanced_config
                    .networking
                    .java
                    .authentication
                    .textures,
            )
            .map_err(AuthError::TextureError)?;
        }
        Ok(profile)
    }

    pub fn handle_login_cookie_response(&self, packet: &SLoginCookieResponse<'_>) {
        debug!(
            "Received cookie_response[login]: key: \"{}\", payload_length: \"{:?}\"",
            packet.key,
            packet.payload.as_ref().map(|p| p.len())
        );
    }

    pub async fn handle_plugin_response(
        &mut self,
        server: &Server,
        plugin_response: SLoginPluginResponse,
    ) {
        debug!("Handling plugin");
        let velocity_config = &server.advanced_config.networking.proxy.velocity;
        if velocity_config.enabled {
            match velocity::receive_velocity_plugin_response(
                self.address.port(),
                velocity_config,
                plugin_response,
            ) {
                Ok((profile, new_address)) => {
                    self.finish_login(&profile).await;
                    self.gameprofile = Some(profile);
                    self.address = new_address;
                }
                Err(error) => self.kick(TextComponent::text(error.to_string())).await,
            }
        }
    }

    pub async fn handle_login_acknowledged(&mut self, server: &Server) {
        debug!("Handling login acknowledgement");
        self.connection_state.store(ConnectionState::Config);
        self.send_packet_now(&server.get_branding()).await;

        if server.advanced_config.server_links.enabled
            && self.version.load() >= JavaMinecraftVersion::V_1_21
        {
            let mut links: Vec<Link> = Vec::new();

            let bug_report = &server.advanced_config.server_links.bug_report;
            if !bug_report.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::BugReport), bug_report));
            }

            let support = &server.advanced_config.server_links.support;
            if !support.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::Support), support));
            }

            let status = &server.advanced_config.server_links.status;
            if !status.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::Status), status));
            }

            let feedback = &server.advanced_config.server_links.feedback;
            if !feedback.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::Feedback), feedback));
            }

            let community = &server.advanced_config.server_links.community;
            if !community.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::Community), community));
            }

            let website = &server.advanced_config.server_links.website;
            if !website.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::Website), website));
            }

            let forums = &server.advanced_config.server_links.forums;
            if !forums.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::Forums), forums));
            }

            let news = &server.advanced_config.server_links.news;
            if !news.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::News), news));
            }

            let announcements = &server.advanced_config.server_links.announcements;
            if !announcements.is_empty() {
                links.push(Link::new(
                    Label::BuiltIn(LinkType::Announcements),
                    announcements,
                ));
            }

            for (key, value) in &server.advanced_config.server_links.custom {
                links.push(Link::new(
                    Label::TextComponent(TextComponent::text(key.clone()).into()),
                    value,
                ));
            }

            self.send_packet_now(&CConfigServerLinks::new(&links)).await;
        }

        let resource_config = &server.advanced_config.resource_pack.java;
        if resource_config.enabled {
            let uuid = Uuid::new_v3(&uuid::Uuid::NAMESPACE_DNS, resource_config.url.as_bytes());
            let resource_pack = CConfigAddResourcePack::new(
                &uuid,
                &resource_config.url,
                &resource_config.sha1,
                resource_config.force,
                if resource_config.prompt_message.is_empty() {
                    None
                } else {
                    Some(TextComponent::text(resource_config.prompt_message.clone()))
                },
            );

            self.send_packet_now(&resource_pack).await;
        } else if self.version.load() >= JavaMinecraftVersion::V_1_20_5 {
            self.send_known_packs().await;
        } else {
            self.handle_known_packs(
                SKnownPacks {
                    known_packs: Vec::new(),
                },
                server,
            )
            .await;
        }
        debug!("login acknowledged");
    }

    pub async fn send_known_packs(&mut self) {
        let version_str = self.version.load().to_string();
        self.send_packet_now(&CKnownPacks::new(&[KnownPack {
            namespace: "minecraft",
            id: "core",
            version: &version_str,
        }]))
        .await;
    }

    pub async fn handle_known_packs(&mut self, _packet: SKnownPacks<'_>, _server: &Server) {
        let version = self.version.load();
        if version >= JavaMinecraftVersion::V_1_20_2 {
            self.send_packet_now(&CFeatureFlags::new(&["minecraft:vanilla".to_string()]))
                .await;
            let registry = pumpkin_data::registry::Registry::get_synced(version);
            for reg in &registry {
                self.send_packet_now(&CRegistryData::new(&reg.registry_id, &reg.registry_entries))
                    .await;
            }
        }
        let all_keys = [
            pumpkin_data::tag::RegistryKey::BannerPattern,
            pumpkin_data::tag::RegistryKey::Block,
            pumpkin_data::tag::RegistryKey::CatVariant,
            pumpkin_data::tag::RegistryKey::DamageType,
            pumpkin_data::tag::RegistryKey::Dialog,
            pumpkin_data::tag::RegistryKey::DimensionType,
            pumpkin_data::tag::RegistryKey::Enchantment,
            pumpkin_data::tag::RegistryKey::EntityType,
            pumpkin_data::tag::RegistryKey::Fluid,
            pumpkin_data::tag::RegistryKey::GameEvent,
            pumpkin_data::tag::RegistryKey::Instrument,
            pumpkin_data::tag::RegistryKey::Item,
            pumpkin_data::tag::RegistryKey::PaintingVariant,
            pumpkin_data::tag::RegistryKey::PointOfInterestType,
            pumpkin_data::tag::RegistryKey::Potion,
            pumpkin_data::tag::RegistryKey::Timeline,
            pumpkin_data::tag::RegistryKey::WorldgenBiome,
        ];

        let mut tags = Vec::new();
        for key in all_keys {
            if pumpkin_data::tag::get_registry_key_tags(version, key)
                .is_some_and(|map| !map.is_empty())
            {
                tags.push(key);
            }
        }
        self.send_packet_now(&CUpdateTags::new(&tags)).await;
        self.send_packet_now(&CFinishConfig).await;
    }
}
