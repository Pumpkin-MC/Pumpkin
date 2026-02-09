use crate::entity::player::Player;
use base64::{Engine as _, engine::general_purpose};
use core::error;
use pumpkin_config::BasicConfiguration;
use pumpkin_data::packet::CURRENT_MC_PROTOCOL;
use pumpkin_protocol::{
    Players, Sample, StatusResponse, Version,
    codec::var_int::VarInt,
    java::client::{config::CPluginMessage, status::CStatusResponse},
};
use pumpkin_world::CURRENT_MC_VERSION;
use std::{
    fs::{self},
    path::Path,
};
use uuid::Uuid;

const DEFAULT_ICON: &[u8] = include_bytes!("../../../assets/default_icon.png");
const MAX_SAMPLE_PLAYERS: usize = 12;

fn load_icon_from_file<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn error::Error>> {
    let buf = fs::read(path)?;
    if buf.len() >= 24 {
        let width = u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]);
        let height = u32::from_be_bytes([buf[20], buf[21], buf[22], buf[23]]);

        if width != 64 || height != 64 {
            return Err("Invalid favicon dimensions (must be 64x64)".into());
        }
    }
    Ok(load_icon_from_bytes(&buf))
}

fn load_icon_from_bytes(png_data: &[u8]) -> String {
    assert!(!png_data.is_empty(), "PNG data is empty");
    let mut result = "data:image/png;base64,".to_owned();
    general_purpose::STANDARD.encode_string(png_data, &mut result);
    result
}

pub struct CachedStatus {
    pub status_response: StatusResponse,
    // We cache the json response here so we don't parse it every time someone makes a status request.
    // Keep in mind that we must parse this again when the StatusResponse changes, which usually happen when a player joins or leaves.
    status_response_json: String,
    player_samples: Vec<(Uuid, String)>,
}

pub struct CachedBranding {
    /// Cached server brand buffer so we don't have to rebuild them every time a player joins
    cached_server_brand: Box<[u8]>,
}

impl<'a> CachedBranding {
    pub fn new() -> Self {
        let cached_server_brand = Self::build_brand();
        Self {
            cached_server_brand,
        }
    }
    pub fn get_branding(&self) -> CPluginMessage<'_> {
        CPluginMessage::new("minecraft:brand", &self.cached_server_brand)
    }
    const BRAND: &'a str = "Pumpkin";
    const BRAND_BYTES: &'a [u8] = Self::BRAND.as_bytes();

    fn build_brand() -> Box<[u8]> {
        let mut buf = Vec::new();
        VarInt(Self::BRAND.len() as i32).encode(&mut buf).unwrap();
        buf.extend_from_slice(Self::BRAND_BYTES);
        buf.into_boxed_slice()
    }
}

impl CachedStatus {
    #[must_use]
    pub fn new(config: &BasicConfiguration) -> Self {
        let status_response = Self::build_response(config);
        let status_response_json = serde_json::to_string(&status_response)
            .expect("Failed to parse status response into JSON");

        Self {
            status_response,
            status_response_json,
            player_samples: Vec::new(),
        }
    }

    pub fn get_status(&self) -> CStatusResponse {
        CStatusResponse::new(self.status_response_json.clone())
    }

    pub fn add_player(&mut self, player: &Player) {
        let status_response = &mut self.status_response;
        if let Some(players) = &mut status_response.players {
            players.online = players.online.saturating_add(1);

            let player_id = player.gameprofile.id;
            let player_name = player.gameprofile.name.clone();

            if !self.player_samples.iter().any(|(id, _)| *id == player_id) {
                self.player_samples.push((player_id, player_name));

                players.sample = self
                    .player_samples
                    .iter()
                    .take(MAX_SAMPLE_PLAYERS)
                    .map(|(id, name)| Sample {
                        name: name.clone(),
                        id: id.to_string(),
                    })
                    .collect();
            }
        }

        self.status_response_json = serde_json::to_string(&status_response)
            .expect("Failed to parse status response into JSON");
    }

    pub fn remove_player(&mut self, player: &Player) {
        let status_response = &mut self.status_response;
        if let Some(players) = &mut status_response.players {
            players.online = players.online.saturating_sub(1);

            let player_id = player.gameprofile.id;
            self.player_samples.retain(|(id, _)| *id != player_id);

            players.sample = self
                .player_samples
                .iter()
                .take(MAX_SAMPLE_PLAYERS)
                .map(|(id, name)| Sample {
                    name: name.clone(),
                    id: id.to_string(),
                })
                .collect();
        }

        self.status_response_json = serde_json::to_string(&status_response)
            .expect("Failed to parse status response into JSON");
    }

    pub fn build_response(config: &BasicConfiguration) -> StatusResponse {
        let favicon = if config.use_favicon {
            config.favicon_path.as_ref().map_or_else(
                || {
                    log::debug!("Loading default icon");

                    // Attempt to load default icon
                    Some(load_icon_from_bytes(DEFAULT_ICON))
                },
                |icon_path| {
                    if !std::path::Path::new(icon_path)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
                    {
                        log::warn!("Favicon is not a PNG-image, using default.");
                        return Some(load_icon_from_bytes(DEFAULT_ICON));
                    }
                    log::debug!("Attempting to load server favicon from '{icon_path}'");

                    match load_icon_from_file(icon_path) {
                        Ok(icon) => Some(icon),
                        Err(e) => {
                            let error_message = e.downcast_ref::<std::io::Error>().map_or_else(
                                || format!("other error: {e}; using default."),
                                |io_err| {
                                    if io_err.kind() == std::io::ErrorKind::NotFound {
                                        "not found; using default.".to_string()
                                    } else {
                                        format!("I/O error: {io_err}; using default.")
                                    }
                                },
                            );
                            log::warn!(
                                "Failed to load favicon from '{icon_path}': {error_message}"
                            );

                            Some(load_icon_from_bytes(DEFAULT_ICON))
                        }
                    }
                },
            )
        } else {
            log::info!("Favicon usage is disabled.");
            None
        };

        StatusResponse {
            version: Some(Version {
                name: CURRENT_MC_VERSION.into(),
                protocol: CURRENT_MC_PROTOCOL,
            }),
            players: Some(Players {
                max: config.max_players,
                online: 0,
                sample: vec![],
            }),
            description: config.motd.clone(),
            favicon,
            // This should stay true even when reports are disabled.
            // It prevents the annoying popup when joining the server.
            enforce_secure_chat: true,
        }
    }
}

impl Default for CachedStatus {
    fn default() -> Self {
        Self::new(&BasicConfiguration::default())
    }
}
