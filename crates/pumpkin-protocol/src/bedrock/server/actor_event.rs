use std::io::{Error, Read, Write};

use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{
    codec::{var_int::VarInt, var_ulong::VarULong},
    serial::{PacketRead, PacketWrite},
};

#[derive(Debug, PacketRead, PacketWrite)]
#[packet(27)]
pub struct SActorEvent {
    pub entity_runtime_id: VarULong,
    pub event_type: ActorEventType,
    pub event_data: VarInt,
    pub fire_at_position: Option<Vector3<f32>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ActorEventType {
    None = 0,
    Jump = 1,
    Hurt = 2,
    Death = 3,
    StartAttacking = 4,
    StopAttacking = 5,
    TamingFailed = 6,
    TamingSucceeded = 7,
    ShakeWetness = 8,
    EatGrass = 10,
    FishhookBubble = 11,
    FishhookFishPosition = 12,
    FishhookHookTime = 13,
    FishhookTease = 14,
    SquidFleeing = 15,
    ZombieConverting = 16,
    PlayAmbient = 17,
    SpawnAlive = 18,
    StartOfferFlower = 19,
    StopOfferFlower = 20,
    LoveHearts = 21,
    VillagerAngry = 22,
    VillagerHappy = 23,
    WitchHatMagic = 24,
    FireworksExplode = 25,
    InLoveHearts = 26,
    SilverfishMergeAnimation = 27,
    GuardianAttackSound = 28,
    DrinkPotion = 29,
    ThrowPotion = 30,
    CartWithPrimeTNT = 31,
    PrimeCreeper = 32,
    AirSupply = 33,
    AddPlayerLevels = 34,
    GuardianMiningFatigue = 35,
    AgentSwingArm = 36,
    DragonStartDeathAnim = 37,
    GroundDust = 38,
    Shake = 39,
    Feed = 57,
    BabyAge = 60,
    InstantDeath = 61,
    NotifyTrade = 62,
    LeashDestroyed = 63,
    CaravanUpdated = 64,
    TalismanActivate = 65,
    UpdateStructureFeature = 66,
    PlayerSpawnedMob = 67,
    Puke = 68,
    UpdateStackSize = 69,
    StartSwimming = 70,
    BalloonPop = 71,
    TreasureHunt = 72,
    SummonAgent = 73,
    FinishedChargingItem = 74,
    ActorGrowUp = 76,
    VibrationDetected = 77,
    DrinkMilk = 78,
    ShakeWetnessStop = 79,
    KineticDamageDealt = 80,
    HurtWithoutReceivingDamage = 81,
}

impl PacketRead for ActorEventType {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(match u8::read(reader)? {
            0 => Self::None,
            1 => Self::Jump,
            2 => Self::Hurt,
            3 => Self::Death,
            4 => Self::StartAttacking,
            5 => Self::StopAttacking,
            6 => Self::TamingFailed,
            7 => Self::TamingSucceeded,
            8 => Self::ShakeWetness,
            10 => Self::EatGrass,
            11 => Self::FishhookBubble,
            12 => Self::FishhookFishPosition,
            13 => Self::FishhookHookTime,
            14 => Self::FishhookTease,
            15 => Self::SquidFleeing,
            16 => Self::ZombieConverting,
            17 => Self::PlayAmbient,
            18 => Self::SpawnAlive,
            19 => Self::StartOfferFlower,
            20 => Self::StopOfferFlower,
            21 => Self::LoveHearts,
            22 => Self::VillagerAngry,
            23 => Self::VillagerHappy,
            24 => Self::WitchHatMagic,
            25 => Self::FireworksExplode,
            26 => Self::InLoveHearts,
            27 => Self::SilverfishMergeAnimation,
            28 => Self::GuardianAttackSound,
            29 => Self::DrinkPotion,
            30 => Self::ThrowPotion,
            31 => Self::CartWithPrimeTNT,
            32 => Self::PrimeCreeper,
            33 => Self::AirSupply,
            34 => Self::AddPlayerLevels,
            35 => Self::GuardianMiningFatigue,
            36 => Self::AgentSwingArm,
            37 => Self::DragonStartDeathAnim,
            38 => Self::GroundDust,
            39 => Self::Shake,
            57 => Self::Feed,
            60 => Self::BabyAge,
            61 => Self::InstantDeath,
            62 => Self::NotifyTrade,
            63 => Self::LeashDestroyed,
            64 => Self::CaravanUpdated,
            65 => Self::TalismanActivate,
            66 => Self::UpdateStructureFeature,
            67 => Self::PlayerSpawnedMob,
            68 => Self::Puke,
            69 => Self::UpdateStackSize,
            70 => Self::StartSwimming,
            71 => Self::BalloonPop,
            72 => Self::TreasureHunt,
            73 => Self::SummonAgent,
            74 => Self::FinishedChargingItem,
            76 => Self::ActorGrowUp,
            77 => Self::VibrationDetected,
            78 => Self::DrinkMilk,
            79 => Self::ShakeWetnessStop,
            80 => Self::KineticDamageDealt,
            81 => Self::HurtWithoutReceivingDamage,
            event => return Err(Error::other(format!("Invalid actor event ID: {event}"))),
        })
    }
}

impl PacketWrite for ActorEventType {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (*self as u8).write(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_feed_event() {
        let packet = SActorEvent::read(&mut b"\x019\x80\x80\x90\x11\0".as_slice()).unwrap();

        assert_eq!(packet.entity_runtime_id, VarULong(1));
        assert_eq!(packet.event_type, ActorEventType::Feed);
        assert_eq!(packet.event_data, VarInt(17_956_864));
        assert_eq!(packet.fire_at_position, None);
    }

    #[test]
    fn feed_event_wire_value_is_bidirectional() {
        let mut encoded = Vec::new();
        ActorEventType::Feed.write(&mut encoded).unwrap();

        assert_eq!(encoded, [57]);
        assert_eq!(
            ActorEventType::read(&mut encoded.as_slice()).unwrap(),
            ActorEventType::Feed
        );
    }
}
