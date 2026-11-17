use pumpkin_data::packet::serverbound::play::SPECTATOR_ACTION;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket, VarInt,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(SPECTATOR_ACTION)]
pub struct SSpectateEntity {
    /// Vanilla `OPTIONAL_VAR_INT`: 0 for no target, otherwise the entity ID plus one.
    pub target: VarInt,
}

impl SSpectateEntity {
    #[must_use]
    pub const fn target_entity_id(&self) -> Option<i32> {
        if self.target.0 == 0 {
            None
        } else {
            Some(self.target.0.wrapping_sub(1))
        }
    }
}

impl<'a> ServerPacket<'a> for SSpectateEntity {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            target: bytebuf.get_var_int()?,
        })
    }
}

impl crate::ClientPacket for SSpectateEntity {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&self.target)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ClientPacket, java::packet_decoder::TCPNetworkDecoder, packet::MultiVersionJavaPacket,
    };

    #[test]
    fn spectator_click_without_target() -> Result<(), ReadingError> {
        // 26.3 sends OPTIONAL_VAR_INT(0) when a spectator clicks without an entity target.
        let mut payload = [0x00].as_slice();
        let packet = SSpectateEntity::read(&mut payload, &JavaMinecraftVersion::V_26_3)?;
        assert_eq!(packet.target_entity_id(), None);
        assert!(payload.is_empty());
        Ok(())
    }

    #[test]
    fn spectator_targets_use_optional_var_int() -> Result<(), Box<dyn std::error::Error>> {
        let fixtures: &[(&[u8], Option<i32>)] = &[
            (&[0x00], None),
            (&[0x01], Some(0)),
            (&[0x2b], Some(42)),
            (&[0x7f], Some(126)),
            (&[0x80, 0x01], Some(127)),
            (&[0x81, 0x01], Some(128)),
            (&[0x80, 0x80, 0x80, 0x80, 0x08], Some(i32::MAX)),
        ];

        for &(bytes, target) in fixtures {
            let mut payload = bytes;
            let packet = SSpectateEntity::read(&mut payload, &JavaMinecraftVersion::V_26_3)?;
            assert_eq!(packet.target_entity_id(), target);
            assert!(payload.is_empty());

            let mut encoded = Vec::new();
            packet.write_packet_data(&mut encoded, &JavaMinecraftVersion::V_26_3)?;
            assert_eq!(encoded, bytes);
        }
        Ok(())
    }

    #[test]
    fn spectator_action_rejects_malformed_var_int() {
        for bytes in [&[][..], &[0x80], &[0x80, 0x80, 0x80, 0x80, 0x80, 0x00]] {
            let mut payload = bytes;
            assert!(SSpectateEntity::read(&mut payload, &JavaMinecraftVersion::V_26_3).is_err());
        }
    }

    #[tokio::test]
    async fn consecutive_spectator_actions_preserve_packet_boundaries()
    -> Result<(), Box<dyn std::error::Error>> {
        // Complete uncompressed frames: click air, entity 42, then entity 127.
        let frames = [0x02, 0x3f, 0x00, 0x02, 0x3f, 0x2b, 0x03, 0x3f, 0x80, 0x01];
        let mut decoder = TCPNetworkDecoder::new(frames.as_slice());
        assert_eq!(SSpectateEntity::to_id(JavaMinecraftVersion::V_26_3), 0x3f);

        for target in [None, Some(42), Some(127)] {
            let raw = decoder.get_raw_packet().await?;
            assert_eq!(raw.id, 0x3f);
            let mut payload = raw.payload.as_ref();
            let packet = SSpectateEntity::read(&mut payload, &JavaMinecraftVersion::V_26_3)?;
            assert_eq!(packet.target_entity_id(), target);
            assert!(payload.is_empty());
        }
        Ok(())
    }
}
