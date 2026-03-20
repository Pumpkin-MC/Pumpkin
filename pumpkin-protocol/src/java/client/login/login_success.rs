use pumpkin_data::packet::clientbound::{LOGIN_GAME_PROFILE, LOGIN_LOGIN_FINISHED};
use pumpkin_util::version::MinecraftVersion;
use serde::Serialize;

use crate::{ClientPacket, Property, packet::MultiVersionJavaPacket, ser::NetworkWriteExt};

/// Sent by the server to signal a successful login and transition to the configuration phase
///
/// This packet provides the client with its official UUID and username as
/// recognized by the server, along with any associated skin or cape properties.
pub struct CLoginSuccess<'a> {
    /// The unique identifier assigned to the player.
    pub uuid: &'a uuid::Uuid,
    /// The player's verified username.
    pub username: &'a str,
    /// A list of properties for the player's profile, such as skin data and signatures.
    /// This is typically retrieved from the Mojang authentication servers.
    pub properties: &'a [Property],
    /// (<1.21.2) Whether strict error handling should be enabled.
    pub strict_error_handling: bool,
}

impl<'a> CLoginSuccess<'a> {
    #[must_use]
    pub const fn new(
        uuid: &'a uuid::Uuid,
        username: &'a str,
        properties: &'a [Property],
        strict_error_handling: bool,
    ) -> Self {
        Self {
            uuid,
            username,
            properties,
            strict_error_handling,
        }
    }
}

impl MultiVersionJavaPacket for CLoginSuccess<'_> {
    fn to_id(version: MinecraftVersion) -> i32 {
        if version >= MinecraftVersion::V_1_21_2 {
            LOGIN_LOGIN_FINISHED.to_id(version)
        } else {
            LOGIN_GAME_PROFILE.to_id(version)
        }
    }
}

impl ClientPacket for CLoginSuccess<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &MinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_uuid(self.uuid)?;
        write.write_string(self.username)?;
        write.write_list(self.properties, |write, property| {
            let mut serializer = crate::ser::serializer::Serializer::new(write);
            property.serialize(&mut serializer)
        })?;
        if version < &MinecraftVersion::V_1_21_2 {
            write.write_bool(self.strict_error_handling)?;
        }
        Ok(())
    }
}
