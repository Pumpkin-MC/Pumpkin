// Last verified for v2169

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
    pub target_runtime_id: VarULong,
    pub event_id: ActorEventID,
    pub data: VarInt,
    pub fire_at_position: Option<Vector3<f32>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ActorEventID {
    None,
    Jump,
    Hurt,
    Death,
    StartAttacking,
    StopAttacking,
    TamingFailed,
    TamingSucceeded,
    ShakeWetness,
    EatGrass = 10,
    FishhookBubble,
    FishhookFishPos,
    FishhookHookTime,
    FishhookTease,
    SquidFleeing,
    ZombieConverting,
    PlayAmbient,
    SpawnAlive,
    StartOfferFlower,
    StopOfferFlower,
    LoveHearts,
    VillagerAngry,
    VillagerHappy,
    WitchHatMagic,
    FireworksExplode,
    InLoveHearts,
    SilverfishMergeAnimation,
    GuardianAttackSound,
    DrinkPotion,
    ThrowPotion,
    PrimeTNTCart,
    PrimeCreeper,
    AirSupply,
    DeprecatedAddPlayerLevels,
    GuardianMiningFatigue,
    AgentSwingArm,
    DragonStartDeathAnim,
    GroundDust,
    Shake,
    Feed = 57,
    BabyAge = 60,
    InstantDeath,
    NotifyTrade,
    LeashDestroyed,
    CaravanUpdated,
    TalismanActivate,
    DeprecatedUpdateStructureFeature,
    PlayerSpawnedMob,
    Puke,
    UpdateStackSize,
    StartSwimming,
    BalloonPop,
    TreasureHunt,
    SummonAgent,
    FinishedChargingItem,
    ActorGrowUp = 76,
    VibrationDetected,
    DrinkMilk,
    ShakeWetnessStop,
    KineticDamageDealt,
    HurtWithoutReceivingDamage,
}

impl PacketRead for ActorEventID {
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
            12 => Self::FishhookFishPos,
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
            31 => Self::PrimeTNTCart,
            32 => Self::PrimeCreeper,
            33 => Self::AirSupply,
            34 => Self::DeprecatedAddPlayerLevels,
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
            66 => Self::DeprecatedUpdateStructureFeature,
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

impl PacketWrite for ActorEventID {
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

        assert_eq!(packet.target_runtime_id, VarULong(1));
        assert_eq!(packet.event_id, ActorEventID::Feed);
        assert_eq!(packet.data, VarInt(17_956_864));
        assert_eq!(packet.fire_at_position, None);
    }

    #[test]
    fn feed_event_wire_value_is_bidirectional() {
        let mut encoded = Vec::new();
        ActorEventID::Feed.write(&mut encoded).unwrap();

        assert_eq!(encoded, [57]);
        assert_eq!(
            ActorEventID::read(&mut encoded.as_slice()).unwrap(),
            ActorEventID::Feed
        );
    }
}
