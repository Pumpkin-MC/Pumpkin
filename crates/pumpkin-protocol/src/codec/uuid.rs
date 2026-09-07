use std::io::{Error, Write};

use uuid::Uuid;

use crate::serial::PacketWrite;

impl PacketWrite for Uuid {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        // Bedrock writes the two UUID halves as little-endian integers.
        // Java uses the separate NetworkWriteExt::write_uuid implementation.
        let (most, least) = self.as_u64_pair();
        writer.write_all(&most.to_le_bytes())?;
        writer.write_all(&least.to_le_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bedrock::{
            client::{
                GameType,
                add_player::CAddPlayer,
                common::{
                    BuildPlatform, CommandPermissionLevel, PlayerPermissionLevel,
                    SerializedAbilitiesData,
                },
                player_list::{CPlayerList, PlayerListEntry, Skin},
                player_skin::CPlayerSkin,
                set_actor_data::{PropertySyncData, SyncedActorDataList},
            },
            network_item::NetworkItemStackDescriptor,
            server::player_skin::SPlayerSkin,
        },
        codec::{var_long::VarLong, var_ulong::VarULong},
        ser::{NetworkReadExt, NetworkWriteExt},
        serial::{PacketRead, PacketReadSlice},
    };
    use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};

    const UUID: Uuid = Uuid::from_u128(0x0011_2233_4455_4677_8899_aabb_ccdd_eeff);
    const BEDROCK_BYTES: [u8; 16] = [
        0x77, 0x46, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00, 0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99,
        0x88,
    ];
    const JAVA_BYTES: [u8; 16] = [
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x46, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
        0xff,
    ];

    #[test]
    fn bedrock_uuid_write_uses_little_endian_halves() {
        let mut encoded = Vec::new();
        UUID.write(&mut encoded).unwrap();
        assert_eq!(encoded, BEDROCK_BYTES);
    }

    #[test]
    fn bedrock_uuid_read_accepts_known_wire_bytes() {
        assert_eq!(Uuid::read(&mut BEDROCK_BYTES.as_slice()).unwrap(), UUID);
    }

    #[test]
    fn bedrock_uuid_read_slice_accepts_known_wire_bytes() {
        let bytes = [BEDROCK_BYTES.as_slice(), &[0xaa]].concat();
        let mut remaining = bytes.as_slice();
        assert_eq!(Uuid::read_slice(&mut remaining).unwrap(), UUID);
        assert_eq!(remaining, &[0xaa]);
    }

    #[test]
    fn bedrock_uuid_rejects_truncated_input() {
        for length in 0..BEDROCK_BYTES.len() {
            let bytes = &BEDROCK_BYTES[..length];
            let mut reader = bytes;
            let error = Uuid::read(&mut reader).unwrap_err();
            assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);

            let mut remaining = bytes;
            let error = Uuid::read_slice(&mut remaining).unwrap_err();
            assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
            assert_eq!(remaining, bytes);
        }
    }

    #[test]
    fn java_uuid_keeps_big_endian_halves() {
        let mut encoded = Vec::new();
        encoded.write_uuid(&UUID).unwrap();
        assert_eq!(encoded, JAVA_BYTES);
        assert_eq!(JAVA_BYTES.as_slice().get_uuid().unwrap(), UUID);
    }

    #[test]
    fn bedrock_player_list_actions_use_known_uuid_bytes() {
        let mut packet = CPlayerList {
            action: CPlayerList::ACTION_ADD,
            entries: vec![PlayerListEntry {
                uuid: UUID,
                entity_unique_id: VarLong(1),
                username: "player".to_string(),
                xuid: String::new(),
                platform_chat_id: String::new(),
                build_platform: BuildPlatform::Unknown,
                skin: Skin::steve(),
                is_teacher: false,
                is_host: false,
                is_sub_client: false,
                player_color: [0; 4],
            }],
        };
        let mut encoded = Vec::new();
        packet.write(&mut encoded).unwrap();
        assert_eq!(&encoded[..3], &[1, 1, 0]);
        assert_eq!(&encoded[3..19], &BEDROCK_BYTES);

        packet.action = CPlayerList::ACTION_REMOVE;
        encoded.clear();
        packet.write(&mut encoded).unwrap();
        assert_eq!(&encoded[..3], &[1, 0, 1]);
        assert_eq!(&encoded[3..], &BEDROCK_BYTES);
    }

    #[test]
    fn bedrock_add_player_uses_known_uuid_bytes() {
        let packet = CAddPlayer {
            uuid: UUID,
            player_name: "player".to_string(),
            target_runtime_id: VarULong(1),
            platform_chat_id: String::new(),
            position: Vector3::default(),
            velocity: Vector3::default(),
            rotation: Vector2::default(),
            y_head_rotation: 0.0,
            carried_item: NetworkItemStackDescriptor::default(),
            player_game_type: GameType::Survival,
            entity_data: SyncedActorDataList::default(),
            synced_properties: PropertySyncData::default(),
            abilities_data: SerializedAbilitiesData {
                target_player_raw_id: 1,
                player_permissions: PlayerPermissionLevel::Member,
                command_permissions: CommandPermissionLevel::Any,
                layers: Vec::new(),
            },
            actor_links: Vec::new(),
            device_id: String::new(),
            build_platform: BuildPlatform::Unknown,
        };
        let mut encoded = Vec::new();
        packet.write(&mut encoded).unwrap();
        assert_eq!(&encoded[..16], &BEDROCK_BYTES);
    }

    #[test]
    fn bedrock_player_skin_uses_known_uuid_bytes_in_both_directions() {
        let skin = Skin::steve();
        let packet = CPlayerSkin {
            uuid: UUID,
            skin: &skin,
            new_skin_name: "new",
            old_skin_name: "old",
        };
        let mut encoded = Vec::new();
        packet.write(&mut encoded).unwrap();
        assert_eq!(&encoded[..16], &BEDROCK_BYTES);

        let mut incoming = BEDROCK_BYTES.to_vec();
        skin.write(&mut incoming).unwrap();
        "new".write(&mut incoming).unwrap();
        "old".write(&mut incoming).unwrap();
        let decoded = SPlayerSkin::read(&mut incoming.as_slice()).unwrap();
        assert_eq!(decoded.uuid, UUID);
        assert_eq!(decoded.new_skin_name, "new");
        assert_eq!(decoded.old_skin_name, "old");
    }
}
