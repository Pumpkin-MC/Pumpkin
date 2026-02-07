use std::io::Read;

use pumpkin_data::packet::serverbound::CONFIG_COOKIE_RESPONSE;
use pumpkin_macros::java_packet;
use pumpkin_util::resource_location::ResourceLocation;

use crate::{ReadingError, ServerPacket, ser::NetworkReadExt};

/// The maximum allowed size for a cookie payload (5 KiB).
const MAX_COOKIE_LENGTH: usize = 5120;

/// Response to a `CCookieRequest` from the server during the configuration phase
///
/// Cookies allow servers to store small amounts of data on the client side,
/// which can be retrieved later (e.g., for session tracking or preferences)
#[java_packet(CONFIG_COOKIE_RESPONSE)]
pub struct SConfigCookieResponse {
    /// The unique identifier for the cookie being returned
    pub key: ResourceLocation,
    /// Indicates whether a payload is attached to this response
    pub has_payload: bool,
    /// The actual data stored in the cookie. Limited to 5120 bytes
    pub payload: Option<Box<[u8]>>,
}

impl ServerPacket for SConfigCookieResponse {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;
        let key = read.get_string()?;
        let has_payload = read.get_bool()?;

        if !has_payload {
            return Ok(Self {
                key,
                has_payload,
                payload: None,
            });
        }

        let payload_length = read.get_var_int()?.0 as usize;
        if payload_length > MAX_COOKIE_LENGTH {
            return Err(ReadingError::TooLarge("SConfigCookieResponse".to_string()));
        }

        let payload = read.read_boxed_slice(payload_length)?;
        Ok(Self {
            key,
            has_payload,
            payload: Some(payload),
        })
    }
}
