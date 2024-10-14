use std::{fs::File, path::Path};

use base64::{engine::general_purpose, Engine as _};
use pumpkin_config::{BasicConfiguration, BASIC_CONFIG};
use pumpkin_protocol::{
    client::{config::CPluginMessage, status::CStatusResponse},
    Players, Sample, StatusResponse, VarInt, Version, CURRENT_MC_PROTOCOL,
};

use super::CURRENT_MC_VERSION;

pub struct CachedStatus {
    _status_response: StatusResponse,
    // We cache the json response here so we don't parse it every time someone makes a Status request.
    // Keep in mind that we must parse this again, when the StatusResponse changes which usually happen when a player joins or leaves
    status_response_json: String,
}

pub struct CachedBranding {
    /// Cached Server brand buffer so we don't have to rebuild them every time a player joins
    cached_server_brand: Vec<u8>,
}

impl CachedBranding {
    pub fn new() -> Self {
        let cached_server_brand = Self::build_brand();
        Self {
            cached_server_brand,
        }
    }
    pub fn get_branding(&self) -> CPluginMessage {
        CPluginMessage::new("minecraft:brand", &self.cached_server_brand)
    }
    fn build_brand() -> Vec<u8> {
        let brand = "Pumpkin";
        let mut buf = vec![];
        let _ = VarInt(brand.len() as i32).encode(&mut buf);
        buf.extend_from_slice(brand.as_bytes());
        buf
    }
}

impl CachedStatus {
    pub fn new() -> Self {
        let status_response = Self::build_response(&BASIC_CONFIG);
        let status_response_json = serde_json::to_string(&status_response)
            .expect("Failed to parse Status response into JSON");

        Self {
            _status_response: status_response,
            status_response_json,
        }
    }

    pub fn get_status(&self) -> CStatusResponse<'_> {
        CStatusResponse::new(&self.status_response_json)
    }

    pub fn build_response(config: &BasicConfiguration) -> StatusResponse {
        let icon_path = "/icon.png";
        let icon = if Path::new(icon_path).exists() {
            Some(Self::load_icon(icon_path))
        } else {
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
                sample: vec![Sample {
                    name: "".into(),
                    id: "".into(),
                }],
            }),
            description: config.motd.clone(),
            favicon: icon,
            enforce_secure_chat: false,
        }
    }

    fn load_icon<P: AsRef<Path>>(path: P) -> String {
        let icon = png::Decoder::new(File::open(path).expect("Failed to load icon"));
        let mut reader = icon.read_info().unwrap();
        let info = reader.info();
        assert!(info.width == 64, "Icon width must be 64");
        assert!(info.height == 64, "Icon height must be 64");
        // Allocate the output buffer.
        let mut buf = vec![0; reader.output_buffer_size()];
        // Read the next frame. An APNG might contain multiple frames.
        let info = reader.next_frame(&mut buf).unwrap();
        // Grab the bytes of the image.
        let bytes = &buf[..info.buffer_size()];
        let mut result = "data:image/png;base64,".to_owned();
        general_purpose::STANDARD.encode_string(bytes, &mut result);
        result
    }
}
