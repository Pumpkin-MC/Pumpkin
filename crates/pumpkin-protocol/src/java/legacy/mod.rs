//! Java packet layouts for versions older than the current one.
//!
//! Compiled only with the `java-legacy` feature; the server never enables it. Nothing outside
//! this module depends on it, so it can be removed as one unit.
//!
//! # Calling it
//! - `packet.write_legacy(&mut buf, &version)` / `P::read_legacy(&mut bytes, &version)`
//!   write or read one payload in an older layout.
//! - [`encode`] frames id and payload for any version, current or older.
//! - [`ids`] maps packet ids between the current version and older ones.
//!
//! # Upgrading the current version
//! 1. Before changing a core packet, the golden fixtures hold its current bytes, rename them
//!    to the version being left.
//! 2. Every packet whose encoding changed now fails its fixture: add a branch for the old
//!    version to its [`LegacyWrite`] impl.
//! 3. Add the new `*_packets.json` to `assets/legacy_packets` and rerun `pumpkin-codegen`.
//! 4. Add the version to `JavaMinecraftVersion`.

#[cfg(test)]
mod fixtures;
pub mod ids;
pub mod packets;
pub mod removed;
pub mod text;

use std::io::Write;

use bytes::Bytes;

use crate::{
    VarInt,
    java::client::{
        config::CConfigAddResourcePack,
        login::{CEncryptionRequest, CLoginSuccess},
        play::{
            CEntityAnimation, CEntityPositionSync, CEntitySoundEffect, CEntityVelocity, CExplosion,
            CLogin, CParticle, CPlayerInfoUpdate, CPlayerPosition, CPlayerRotation,
            CPlayerSpawnPosition, CRespawn, CSetPlayerTeam, CSoundEffect, CSpawnEntity, CSwingArm,
            CUpdateAttributes, CUpdateEntityPos, CUpdateEntityPosRot, CUpdateEntityRot,
            CUpdateTime,
        },
    },
    java::server::{login::SEncryptionResponse, play::SInteract},
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};
use ids::{clientbound, serverbound};
use pumpkin_util::{math::position::BlockPos, text::TextComponent, version::JavaMinecraftVersion};

/// Where an older client expects a packet.
pub trait LegacyPacket {
    /// The packet's id in `version`; `None` when that version doesn't have it.
    fn legacy_id(version: JavaMinecraftVersion) -> Option<i32>;
}

/// Implements [`LegacyPacket`] from one row of [`ids`].
macro_rules! legacy_ids {
    ($($packet:ty => $row:expr),* $(,)?) => {$(
        impl LegacyPacket for $packet {
            fn legacy_id(version: JavaMinecraftVersion) -> Option<i32> {
                let id = $row.to_id(version);
                (id != -1).then_some(id)
            }
        }
    )*};
}
pub(crate) use legacy_ids;

legacy_ids! {
    CConfigAddResourcePack<'_> => clientbound::config::RESOURCE_PACK_PUSH,
    CEncryptionRequest<'_> => clientbound::login::HELLO,
    CLoginSuccess<'_> => clientbound::login::LOGIN_FINISHED,
    CEntityAnimation => clientbound::play::ANIMATE,
    // A swing was an ANIMATE animation before 26.3
    CSwingArm => clientbound::play::ANIMATE,
    CEntitySoundEffect => clientbound::play::SOUND_ENTITY,
    CEntityVelocity => clientbound::play::SET_ENTITY_MOTION,
    CExplosion => clientbound::play::EXPLODE,
    CLogin<'_> => clientbound::play::LOGIN,
    CParticle<'_> => clientbound::play::LEVEL_PARTICLES,
    CPlayerInfoUpdate<'_> => clientbound::play::PLAYER_INFO_UPDATE,
    CPlayerPosition => clientbound::play::PLAYER_POSITION,
    CPlayerRotation => clientbound::play::PLAYER_ROTATION,
    CPlayerSpawnPosition => clientbound::play::SET_DEFAULT_SPAWN_POSITION,
    CRespawn => clientbound::play::RESPAWN,
    CSetPlayerTeam<'_> => clientbound::play::SET_PLAYER_TEAM,
    CSoundEffect => clientbound::play::SOUND,
    CSpawnEntity => clientbound::play::ADD_ENTITY,
    CUpdateAttributes => clientbound::play::UPDATE_ATTRIBUTES,
    CUpdateEntityPos => clientbound::play::MOVE_ENTITY_POS,
    CUpdateEntityPosRot => clientbound::play::MOVE_ENTITY_POS_ROT,
    CUpdateEntityRot => clientbound::play::MOVE_ENTITY_ROT,
    CUpdateTime => clientbound::play::SET_TIME,
    SEncryptionResponse => serverbound::login::KEY,
    SInteract => serverbound::play::INTERACT,
}

impl LegacyPacket for CEntityPositionSync {
    /// `TELEPORT_ENTITY` before 1.21.2.
    fn legacy_id(version: JavaMinecraftVersion) -> Option<i32> {
        let row = if version >= JavaMinecraftVersion::V_1_21_2 {
            clientbound::play::ENTITY_POSITION_SYNC
        } else {
            clientbound::play::TELEPORT_ENTITY
        };
        let id = row.to_id(version);
        (id != -1).then_some(id)
    }
}

/// The id of `P` in `version`; `None` when that version doesn't have it.
#[must_use]
pub fn id<P: LegacyPacket>(version: JavaMinecraftVersion) -> Option<i32> {
    P::legacy_id(version)
}

/// Id and payload of `packet` for a client older than the current version. Current clients use
/// core's [`ClientPacket::serialize_packet`](crate::ClientPacket::serialize_packet).
///
/// # Errors
/// [`WritingError::UnsupportedVersion`] when `version` has no such packet, or the payload's error.
pub fn encode<P: LegacyPacket + LegacyWrite>(
    packet: &P,
    version: JavaMinecraftVersion,
) -> Result<Bytes, WritingError> {
    let id = P::legacy_id(version).ok_or(WritingError::UnsupportedVersion(version))?;
    let mut buf = Vec::new();
    buf.write_var_int(&VarInt(id))?;
    packet.write_legacy(&mut buf, &version)?;
    Ok(buf.into())
}

/// A core packet written for an older client.
#[diagnostic::on_unimplemented(
    message = "`{Self}` has no older layout in `pumpkin_protocol::java::legacy`",
    note = "add an impl in `java/legacy/packets/` with a branch per older version"
)]
pub trait LegacyWrite {
    fn write_legacy(
        &self,
        write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError>;
}

/// Registry NBT that older login and respawn packets embed; the caller supplies it for the
/// client's version.
#[derive(Clone, Copy, Debug, Default)]
pub struct LegacyRegistryNbt<'a> {
    /// The registry codec (1.16 - 1.20.1).
    pub login_codec: Option<&'a [u8]>,
    /// The dimension type itself (1.16.2 - 1.18.2); other versions send its name.
    pub dimension_type: Option<&'a [u8]>,
}

/// A core packet written for an older client, with data the caller supplies.
pub trait LegacyWriteWith<C> {
    fn write_legacy_with(
        &self,
        write: impl Write,
        version: &JavaMinecraftVersion,
        context: &C,
    ) -> Result<(), WritingError>;
}

/// A core packet read from an older client.
pub trait LegacyRead<'a>: Sized {
    fn read_legacy(
        read: &mut &'a [u8],
        version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError>;
}

/// Field encodings that changed before 26.3.
pub trait LegacyWriteExt: NetworkWriteExt {
    /// JSON text before 1.20.3, NBT since.
    fn write_component_legacy(
        &mut self,
        component: &TextComponent,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version < JavaMinecraftVersion::V_1_20_3 {
            let json = text::to_json(component, version);
            let max_len = if *version >= JavaMinecraftVersion::V_1_13 {
                262_144
            } else {
                32767
            };
            self.write_string_bounded(&json, max_len)
        } else {
            self.write_slice(&text::encode(component, version))
        }
    }

    /// X, Y, Z before 1.14, X, Z, Y since.
    fn write_block_pos_legacy(
        &mut self,
        pos: &BlockPos,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        self.write_i64_be(block_pos_to_version(pos.as_long(), version))
    }

    /// A byte before 1.21.2, a var int since.
    fn write_container_id_legacy(
        &mut self,
        container_id: &VarInt,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version >= JavaMinecraftVersion::V_1_21_2 {
            self.write_var_int(container_id)
        } else {
            self.write_u8(container_id.0 as u8)
        }
    }
}

impl<W: Write> LegacyWriteExt for W {}

pub trait LegacyReadExt: NetworkReadExt {
    /// A byte before 1.21.2, a var int since.
    fn get_container_id_legacy(
        &mut self,
        version: &JavaMinecraftVersion,
    ) -> Result<VarInt, ReadingError> {
        if *version >= JavaMinecraftVersion::V_1_21_2 {
            self.get_var_int()
        } else {
            Ok(VarInt(i32::from(self.get_u8()?)))
        }
    }

    /// X, Y, Z before 1.14, X, Z, Y since.
    fn get_block_pos_legacy(
        &mut self,
        version: &JavaMinecraftVersion,
    ) -> Result<BlockPos, ReadingError> {
        Ok(BlockPos::from_i64(block_pos_to_current(
            self.get_i64_be()?,
            version,
        )))
    }
}

impl<R: NetworkReadExt> LegacyReadExt for R {}

/// Repacks a current packed block position for the client.
#[must_use]
pub const fn block_pos_to_version(packed: i64, version: &JavaMinecraftVersion) -> i64 {
    if (*version as u8) >= (JavaMinecraftVersion::V_1_14 as u8) {
        return packed;
    }
    let (x, y, z) = (packed >> 38, packed << 52 >> 52, packed << 26 >> 38);
    ((x & 0x03FF_FFFF) << 38) | ((y & 0xFFF) << 26) | (z & 0x03FF_FFFF)
}

/// Repacks a client's packed block position as current.
#[must_use]
pub const fn block_pos_to_current(packed: i64, version: &JavaMinecraftVersion) -> i64 {
    if (*version as u8) >= (JavaMinecraftVersion::V_1_14 as u8) {
        return packed;
    }
    let (x, y, z) = (packed >> 38, packed << 26 >> 52, packed << 38 >> 38);
    ((x & 0x03FF_FFFF) << 38) | ((z & 0x03FF_FFFF) << 12) | (y & 0xFFF)
}

/// Equipment slot ids until 1.8: no off hand, so every armour slot is one lower.
#[must_use]
pub const fn slot_to_version(slot: i8, version: &JavaMinecraftVersion) -> i8 {
    if (*version as u8) <= (JavaMinecraftVersion::V_1_8 as u8) {
        match slot {
            0 => 0,
            2 => 1,
            3 => 2,
            4 => 3,
            5 => 4,
            _ => slot,
        }
    } else {
        slot
    }
}

/// Bit sets were a var int count of big endian longs before 26.3.
pub fn write_bit_set_legacy(
    mut write: impl Write,
    bit_set: &crate::codec::bit_set::BitSet,
    version: &JavaMinecraftVersion,
) -> Result<(), WritingError> {
    if *version >= JavaMinecraftVersion::V_26_3 {
        return bit_set.encode(&mut write);
    }
    write.write_var_int(&VarInt(bit_set.0.len() as i32))?;
    for word in &bit_set.0 {
        write.write_i64_be(*word)?;
    }
    Ok(())
}

const LEGACY_VELOCITY_CLAMP: f64 = 3.9;
const LEGACY_VELOCITY_SCALE: f64 = 8000.0;

/// Velocity component before 1.21.9: clamped, times 8000, as a short.
#[must_use]
pub fn encode_legacy_velocity_component(component: f64) -> i16 {
    (component.clamp(-LEGACY_VELOCITY_CLAMP, LEGACY_VELOCITY_CLAMP) * LEGACY_VELOCITY_SCALE) as i16
}

/// Velocity before 1.21.9: three shorts.
pub fn write_legacy_velocity(
    mut write: impl Write,
    velocity: &pumpkin_util::math::vector3::Vector3<f64>,
) -> Result<(), WritingError> {
    write.write_i16_be(encode_legacy_velocity_component(velocity.x))?;
    write.write_i16_be(encode_legacy_velocity_component(velocity.y))?;
    write.write_i16_be(encode_legacy_velocity_component(velocity.z))
}

#[cfg(test)]
mod tests {
    use pumpkin_util::math::position::BlockPos;

    use super::*;

    #[test]
    fn block_pos_is_x_y_z_before_1_14() {
        let pos = BlockPos::new(-3, 70, 12);
        let old = ((-3i64 & 0x03FF_FFFF) << 38) | (0x46 << 26) | 0xC;
        let mut out = Vec::new();
        out.write_block_pos_legacy(&pos, &JavaMinecraftVersion::V_1_13_2)
            .unwrap();
        assert_eq!(out, old.to_be_bytes());
        let mut read = &out[..];
        assert_eq!(
            read.get_block_pos_legacy(&JavaMinecraftVersion::V_1_13_2)
                .unwrap(),
            pos
        );
        assert_eq!(
            block_pos_to_version(pos.as_long(), &JavaMinecraftVersion::V_1_14),
            pos.as_long()
        );
    }
}
