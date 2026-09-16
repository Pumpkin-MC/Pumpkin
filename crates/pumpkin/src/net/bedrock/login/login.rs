use pumpkin_protocol::bedrock::client::PackIdVersion;

#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_protocol::bedrock::client::handshake::CHandshake;
use rand::RngExt;
use sha2::{Digest, Sha256};

/// Builds the name a Bedrock player is known by on the server.
///
/// Bedrock gamertags may contain spaces, which cannot be typed as a command argument,
/// and may collide with a Java account name on a cross-play server. `prefix` is
/// prepended as-is (empty means no prefix) and, when `replace_spaces` is set, spaces
/// become underscores.
fn build_username(display_name: &str, prefix: &str, replace_spaces: bool) -> String {
    let mut name = String::with_capacity(prefix.len() + display_name.len());
    name.push_str(prefix);
    if replace_spaces {
        name.extend(display_name.chars().map(|c| if c == ' ' { '_' } else { c }));
    } else {
        name.push_str(display_name);
    }
    name
}

/// Starts the encryption handshake required by `RakNet`; `NetherNet` uses DTLS instead.
async fn begin_encryption(
    client: &BedrockClient,
    server: &Server,
    client_public_key: &pumpkin_auth::p384::PublicKey,
) -> Result<(), LoginError> {
    let server_key = server
        .bedrock_private_key
        .get_or_init(|| async {
            let mut rng = rand::rng();
            loop {
                let mut private_key_bytes = [0u8; 48];
                rng.fill(&mut private_key_bytes);
                if let Ok(key) =
                    pumpkin_auth::p384::ecdsa::SigningKey::from_slice(&private_key_bytes)
                {
                    break Arc::new(key);
                }
            }
        })
        .await
        .clone();

    let mut salt = [0u8; 16];
    rand::rng().fill(&mut salt);

    let handshake_jwt = pumpkin_auth::jwt::generate_handshake_jwt(&server_key, &salt)
        .map_err(LoginError::ChainValidationFailed)?;
    client.send_packet(&CHandshake::new(handshake_jwt)).await;

    let shared_secret = pumpkin_auth::jwt::compute_shared_secret(&server_key, client_public_key);
    let mut hasher = Sha256::new();
    hasher.update(salt);
    hasher.update(shared_secret);
    let key_bytes: [u8; 32] = hasher.finalize().into();

    client
        .enable_encryption(&key_bytes)
        .await
        .map_err(|error| AuthError::PublicKeyBuild(error.to_string()))?;

    Ok(())
}

impl BedrockClient {
    pub async fn handle_login(
        self: &Arc<Self>,
        packet: SLogin,
        server: &Server,
    ) -> Result<Option<PacketHandlerResult>, LoginError> {
        self.try_handle_login(packet, server).await
    }

    pub async fn try_handle_login(
        self: &Arc<Self>,
        packet: SLogin,
        server: &Server,
    ) -> Result<Option<PacketHandlerResult>, LoginError> {
        let auth_payload: AuthPayload = serde_json::from_slice(&packet.jwt)?;
        let player_data = if server.advanced_config.networking.bedrock.online_mode {
            match auth_payload.authentication_type {
                AuthenticationType::Full => {
                    verify_oidc_token_path(server, &auth_payload.token, false).await?
                }
                AuthenticationType::SelfSigned => {
                    if server
                        .advanced_config
                        .networking
                        .bedrock
                        .authentication
                        .enabled
                    {
                        return Err(LoginError::SelfSignedNotAllowed);
                    }

                    verify_oidc_token_path(server, &auth_payload.token, true).await?
                }
                AuthenticationType::Guest => {
                    return Err(LoginError::GuestUnimplemented);
                }
            }
        } else {
            pumpkin_auth::jwt::extract_oidc_token_player_claims(&auth_payload.token)?
        };

        let raw_token_str = std::str::from_utf8(&packet.raw_token).map_err(|_| {
            LoginError::InvalidTokenFormat(serde_json::Error::custom(
                "raw_token is not valid UTF-8",
            ))
        })?; // You'll need to add a string conversion error to LoginError, or handle it cleanly.

        let mut parts = raw_token_str.split('.');
        let _header = parts.next().ok_or(AuthError::InvalidTokenFormat)?;
        let payload_b64 = parts.next().ok_or(AuthError::InvalidTokenFormat)?;

        let payload_bytes = pumpkin_auth::jwt::decode_b64_url_nopad(payload_b64)
            .map_err(|_| LoginError::DecodeExtraError)?;
        let client_data: ClientData = serde_json::from_slice(&payload_bytes)?;

        let bedrock_config = &server.advanced_config.networking.bedrock;
        let name = build_username(
            &player_data.display_name,
            &bedrock_config.username_prefix,
            bedrock_config.replace_username_spaces,
        );

        let profile = GameProfile {
            id: Uuid::parse_str(&player_data.uuid).map_err(|_| LoginError::InvalidUuid)?,
            name,
            properties: ArcSwap::new(Arc::new(Vec::new())),
            profile_actions: None,
        };

        let login_public_key = pumpkin_auth::jwt::extract_cpk_from_token(&auth_payload.token)
            .map_err(LoginError::ChainValidationFailed)?;
        if self
            .nethernet_public_key()
            .is_some_and(|public_key| public_key != &login_public_key)
        {
            return Err(LoginError::ChainValidationFailed(
                AuthError::PublicKeyBuild(
                    "NetherNet and Bedrock login identities do not match".into(),
                ),
            ));
        }

        let new_config = PlayerConfig {
            locale: client_data.language_code.clone(),
            ..Default::default()
        };

        if !self.is_nethernet() {
            begin_encryption(self, server, &login_public_key).await?;

            self.client_data
                .store(std::sync::Arc::new(Some(std::sync::Arc::new(client_data))));
            self.pending_profile
                .store(std::sync::Arc::new(Some(std::sync::Arc::new((
                    profile, new_config,
                )))));
            return Ok(None);
        }

        self.send_login_success(server).await;
        self.client_data
            .store(Arc::new(Some(Arc::new(client_data))));

        Ok(Some(PacketHandlerResult::ReadyToPlay(profile, new_config)))
    }

    pub async fn send_login_success(&self, server: &Server) {
        self.enqueue_client_packet(&CPlayStatus::LoginSuccess).await;
        let br_config = &server.advanced_config.resource_pack.bedrock;

        let mut entries = Vec::new();
        if br_config.enabled {
            for pack in &br_config.packs {
                entries.push(PackInfoData {
                    pack_id_version: PackIdVersion::new(pack.uuid, pack.version.clone()),
                    pack_size: pack.size,
                    cdn_url: pack.download_url.clone(),
                    content_key: pack.content_key.clone(),
                    subpack_name: pack.sub_pack_name.clone(),
                    content_identity: pack.content_id.clone(),
                    has_scripts: pack.has_scripts,
                    is_addon_pack: pack.addon_pack,
                    is_ray_tracing_capable: pack.rtx_enabled,
                });
            }
        }

        let packs_info = CResourcePacksInfo {
            resource_pack_required: br_config.force,
            has_addon_packs: false,
            has_scripts: false,
            force_disable_vibrant_visuals: false,
            world_template_id_and_version: PackIdVersion::new(uuid::Uuid::nil(), String::new()),
            resource_packs: entries,
        };
        self.enqueue_client_packet(&packs_info).await;
    }
}

#[cfg(test)]
mod tests {
    use super::build_username;

    #[test]
    fn spaces_become_underscores_by_default() {
        assert_eq!(build_username("Some Player", "", true), "Some_Player");
    }

    #[test]
    fn spaces_are_kept_when_replacement_is_disabled() {
        assert_eq!(build_username("Some Player", "", false), "Some Player");
    }

    #[test]
    fn prefix_is_prepended() {
        assert_eq!(build_username("Some Player", ".", true), ".Some_Player");
    }

    #[test]
    fn empty_prefix_leaves_the_name_unchanged() {
        assert_eq!(build_username("Steve", "", true), "Steve");
    }
}
