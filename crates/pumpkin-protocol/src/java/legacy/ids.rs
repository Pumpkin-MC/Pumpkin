//! Packet ids of every supported Java version.

use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::ConnectionState;

#[path = "generated_ids.rs"]
mod generated;

pub use generated::{PacketId, VERSIONS, clientbound, serverbound};

// The generated columns must follow `JavaMinecraftVersion` exactly.
const _: () = {
    assert!(JavaMinecraftVersion::KNOWN.len() == VERSIONS);
    assert!(JavaMinecraftVersion::KNOWN[VERSIONS - 1] as usize == CURRENT_MC_VERSION as usize);
    let mut i = 0;
    while i < VERSIONS {
        assert!(JavaMinecraftVersion::KNOWN[i] as usize == i);
        i += 1;
    }
};

impl PacketId {
    /// The id in `version`, or `-1` when that version doesn't have the packet. Unknown
    /// versions get the current id.
    #[must_use]
    pub const fn to_id(&self, version: JavaMinecraftVersion) -> i32 {
        let column = version as usize;
        if column < VERSIONS {
            self.0[column] as i32
        } else {
            self.current()
        }
    }

    /// The id in the current version.
    #[must_use]
    pub const fn current(&self) -> i32 {
        self.0[VERSIONS - 1] as i32
    }
}

const fn state_index(state: ConnectionState) -> usize {
    match state {
        ConnectionState::HandShake => 0,
        ConnectionState::Status => 1,
        ConnectionState::Login | ConnectionState::Transfer => 2,
        ConnectionState::Config => 3,
        ConnectionState::Play => 4,
    }
}

fn lookup(
    tables: &[[&[i16]; VERSIONS]; 5],
    state: ConnectionState,
    id: i32,
    version: JavaMinecraftVersion,
) -> Option<i32> {
    let table = tables[state_index(state)].get(version as usize)?;
    let id = *table.get(usize::try_from(id).ok()?)?;
    (id != -1).then_some(i32::from(id))
}

/// The current id of a packet the client sent as `client_id`.
#[must_use]
pub fn serverbound_to_current(
    state: ConnectionState,
    client_id: i32,
    version: JavaMinecraftVersion,
) -> Option<i32> {
    lookup(
        &generated::SERVERBOUND_TO_CURRENT,
        state,
        client_id,
        version,
    )
}

/// The client's id of the current packet `current_id`; `None` when the client has no such packet.
#[must_use]
pub fn clientbound_id(
    state: ConnectionState,
    current_id: i32,
    version: JavaMinecraftVersion,
) -> Option<i32> {
    lookup(
        &generated::CLIENTBOUND_FROM_CURRENT,
        state,
        current_id,
        version,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_version_maps_to_itself() {
        let id = serverbound::play::CHAT.current();
        assert_eq!(
            serverbound_to_current(ConnectionState::Play, id, CURRENT_MC_VERSION),
            Some(id)
        );
        assert_eq!(
            clientbound_id(ConnectionState::Play, id, CURRENT_MC_VERSION),
            Some(id)
        );
    }

    #[test]
    fn renamed_packets_use_the_older_row() {
        let version = JavaMinecraftVersion::V_1_21_11;
        let swing = serverbound::play::SWING.to_id(version);
        assert_eq!(
            serverbound_to_current(ConnectionState::Play, swing, version),
            Some(serverbound::play::PUNCH.current())
        );
    }

    #[test]
    fn missing_packets_have_no_client_id() {
        let version = JavaMinecraftVersion::V_1_8;
        let bundle = clientbound::play::BUNDLE_DELIMITER.current();
        assert_eq!(clientbound_id(ConnectionState::Play, bundle, version), None);
    }
}
