use pumpkin_data::packet::clientbound::LOGIN_LOGIN_FINISHED;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::Property;

/// Sent by the server to signal a successful login and transition to the configuration phase
///
/// This packet provides the client with its official UUID and username as
/// recognized by the server, along with any associated skin or cape properties.
#[derive(Serialize)]
#[java_packet(LOGIN_LOGIN_FINISHED)]
pub struct CLoginSuccess<'a> {
    /// The unique identifier assigned to the player.
    pub uuid: &'a uuid::Uuid,
    /// The player's verified username.
    pub username: &'a str,
    /// A list of properties for the player's profile, such as skin data and signatures.
    /// This is typically retrieved from the Mojang authentication servers.
    pub properties: &'a [Property],
}

impl<'a> CLoginSuccess<'a> {
    #[must_use]
    pub fn new(uuid: &'a uuid::Uuid, username: &'a str, properties: &'a [Property]) -> Self {
        Self {
            uuid,
            username,
            properties,
        }
    }
}
