use pumpkin_util::{identifier::Identifier, version::MinecraftVersion};
use thiserror::Error;

use crate::mapping::NetworkId;

#[derive(Debug, Error)]
pub enum RegistryInsertError {
    #[error("registry entry `{0}` is already registered")]
    AlreadyRegistered(Identifier),
}

#[derive(Debug, Error)]
pub enum VersionMappingError {
    #[error("cannot create version mapping for unknown registry entry `{0}`")]
    UnknownEntry(Identifier),

    #[error(
        "registry entry `{identifier}` is already mapped to network ID \
         {existing_network_id} for Minecraft version {version}, but mapping \
         to network ID {requested_network_id} was requested"
    )]
    IdentifierAlreadyMapped {
        version: MinecraftVersion,
        identifier: Identifier,
        existing_network_id: NetworkId,
        requested_network_id: NetworkId,
    },

    #[error(
        "network ID {network_id} is already mapped to registry entry \
         `{existing_identifier}` for Minecraft version {version}, but mapping \
         it to `{requested_identifier}` was requested"
    )]
    NetworkIdAlreadyMapped {
        version: MinecraftVersion,
        network_id: NetworkId,
        existing_identifier: Identifier,
        requested_identifier: Identifier,
    },
}

#[derive(Debug, Error)]
pub enum RegistryGetError {
    #[error("registry path cannot be empty")]
    EmptyPath,

    #[error("registry entry `{0}` was not found")]
    NotFound(Identifier),

    #[error("registry entry `{0}` is not a nested registry")]
    ExpectedRegistry(Identifier),

    #[error("registry `{identifier}` has the wrong entry type; expected `{expected}`")]
    TypeMismatch {
        identifier: Identifier,
        expected: &'static str,
    },
}
