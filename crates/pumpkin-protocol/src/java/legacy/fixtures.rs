//! Golden bytes of every older layout, and a check that each impl's newest branch is core's
//! encoding. When core moves to a new version, the second check keeps the version it left
//! reproducible. `PUMPKIN_BLESS_FIXTURES=1` rewrites the files in `fixtures/`.

use std::fmt::Write as _;
use std::path::PathBuf;

use pumpkin_data::{packet::CURRENT_MC_VERSION, sound::SoundCategory};
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    text::TextComponent,
    version::JavaMinecraftVersion,
};

use super::LegacyWrite;
use crate::{
    ClientPacket, IdOr, PositionFlag, VarInt,
    java::client::{
        config::CConfigAddResourcePack,
        login::{CEncryptionRequest, CLoginSuccess},
        play::{
            Animation, CEntityAnimation, CEntityPositionSync, CEntitySoundEffect, CEntityVelocity,
            CExplosion, CParticle, CPlayerPosition, CPlayerRotation, CPlayerSpawnPosition,
            CSoundEffect, CSpawnEntity, CSwingArm, CUpdateAttributes, CUpdateEntityPos,
            CUpdateEntityPosRot, CUpdateEntityRot, CUpdateTime, Property,
        },
    },
};

/// Every older version's bytes, consecutive versions with equal bytes on one line.
fn older_layouts(packet: &impl LegacyWrite) -> String {
    let mut runs: Vec<(JavaMinecraftVersion, JavaMinecraftVersion, String)> = Vec::new();
    for &version in JavaMinecraftVersion::KNOWN {
        if version >= CURRENT_MC_VERSION {
            break;
        }
        let mut bytes = Vec::new();
        let encoded = match packet.write_legacy(&mut bytes, &version) {
            Ok(()) => bytes.iter().fold(String::new(), |mut hex, b| {
                let _ = write!(hex, "{b:02x}");
                hex
            }),
            Err(error) => format!("error: {error}"),
        };
        match runs.last_mut() {
            Some((_, last, previous)) if *previous == encoded => *last = version,
            _ => runs.push((version, version, encoded)),
        }
    }
    runs.iter()
        .fold(String::new(), |mut out, (first, last, encoded)| {
            let _ = writeln!(out, "{first:?}..={last:?} {encoded}");
            out
        })
}

fn check<P: LegacyWrite + ClientPacket>(name: &str, packet: &P) {
    let mut core = Vec::new();
    packet.write_packet_data(&mut core).unwrap();
    let mut legacy = Vec::new();
    packet
        .write_legacy(&mut legacy, &CURRENT_MC_VERSION)
        .unwrap();
    assert_eq!(
        legacy, core,
        "{name}: the newest branch must be core's encoding"
    );

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/java/legacy/fixtures")
        .join(format!("{name}.txt"));
    let actual = older_layouts(packet);
    if std::env::var_os("PUMPKIN_BLESS_FIXTURES").is_some() {
        std::fs::write(&path, &actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("{name}: no fixture, run with PUMPKIN_BLESS_FIXTURES=1"));
    assert_eq!(actual, expected, "{name}: an older layout changed");
}

/// One test per packet, each building the packet with the given expression.
macro_rules! fixtures {
    ($($name:ident => $packet:expr),* $(,)?) => {$(
        #[test]
        fn $name() {
            check(stringify!($name), &$packet);
        }
    )*};
}

const POSITION: Vector3<f64> = Vector3::new(12.5, 64.0, -7.25);
const DELTA: Vector3<f64> = Vector3::new(0.25, -0.5, 1.0);
const UUID: uuid::Uuid = uuid::Uuid::from_u128(0x0123_4567_89ab_cdef_0123_4567_89ab_cdef);

fixtures! {
    add_resource_pack => CConfigAddResourcePack::new(
        &UUID,
        "https://example.com/pack.zip",
        "0123456789abcdef0123456789abcdef01234567",
        true,
        Some(TextComponent::text("Pack")),
    ),
    encryption_request => CEncryptionRequest::new("", &[1, 2, 3], &[4, 5, 6, 7], true),
    login_success => CLoginSuccess::new(&UUID, "Steve", &[], true, UUID),
    entity_animation => CEntityAnimation::new(VarInt(7), Animation::CriticalEffect),
    swing_arm => CSwingArm::new(VarInt(7), true),
    entity_position_sync => CEntityPositionSync::new(VarInt(7), POSITION, DELTA, 90.0, -10.0, true),
    entity_sound_effect => CEntitySoundEffect::new(
        IdOr::Id(3),
        SoundCategory::Hostile,
        VarInt(7),
        1.0,
        0.5,
        42,
    ),
    entity_velocity => CEntityVelocity::new(VarInt(7), DELTA),
    explosion => CExplosion::new(POSITION, 3.0, 5, Some(DELTA), VarInt(1), IdOr::Id(2)),
    particle => CParticle::new(false, true, POSITION, Vector3::new(0.5, 0.0, 0.5), 0.1, 4, VarInt(1), &[]),
    player_position => CPlayerPosition::new(
        VarInt(3),
        POSITION,
        DELTA,
        90.0,
        -10.0,
        vec![PositionFlag::X, PositionFlag::YRot],
    ),
    player_rotation => CPlayerRotation::new(45.0, 10.0),
    player_spawn_position => CPlayerSpawnPosition::new(
        BlockPos::new(1, 64, -2),
        90.0,
        0.0,
        "minecraft:overworld".to_string(),
    ),
    set_time => CUpdateTime::new(1000, 6000, true),
    sound_effect => CSoundEffect::new(IdOr::Id(3), SoundCategory::Blocks, &POSITION, 1.0, 0.5, 7),
    spawn_entity => CSpawnEntity::new(
        VarInt(7),
        UUID,
        VarInt(10),
        POSITION,
        -10.0,
        90.0,
        45.0,
        VarInt(0),
        DELTA,
    ),
    update_attributes => CUpdateAttributes::new(VarInt(7), vec![Property::new(VarInt(2), 0.1, vec![])]),
    update_entity_pos => CUpdateEntityPos::new(VarInt(7), Vector3::new(100, -50, 8), true),
    update_entity_pos_rot => CUpdateEntityPosRot::new(VarInt(7), Vector3::new(100, -50, 8), 64, 32, false),
    update_entity_rot => CUpdateEntityRot::new(VarInt(7), 64, 32, true),
}
