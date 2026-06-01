use crate::serial::PacketWrite;
use std::io::{Error, Write};

use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::codec::var_int::VarInt;

#[derive(Debug, PacketWrite)]
#[packet(27)]
pub struct SActorEvent {
    pub entity_runtime_id: VarInt,
    pub event_type: ActorEventType,
    pub event_data: VarInt,
    pub fire_at_position: Option<Vector3<f32>>,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ActorEventType {
    Jump = 1,
    Hurt = 2,
    Death = 3,
    StartAttacking = 4,
    StopAttacking = 5,
    TamingFailed = 6,
    TamingSucceeded = 7,
    ShakeWetness = 8,
    UseItem = 9,
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
    BabyEat = 60,
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
    LandedOnGround = 75,
    ActorGrowUp = 76,
    VibrationDetected = 77,
    DrinkMilk = 78,
    WetnessStop = 79,
    KineticDamageDealt = 80,
    HurtWithoutReceivingDamage = 81,
}

impl PacketWrite for ActorEventType {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (*self as u8).write(writer)
    }
}
