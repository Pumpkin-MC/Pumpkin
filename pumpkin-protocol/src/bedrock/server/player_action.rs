use std::io::{Error, Read};

use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::{codec::var_int::VarInt, serial::PacketRead};

#[derive(Debug, PacketRead)]
#[packet(36)]
pub struct SPlayerAction {
    pub runtime_id: VarInt,
    pub action: Action,
    pub block_pos: BlockPos,
    pub result_pos: BlockPos,
    pub face: VarInt,
}

#[derive(Debug)]
#[repr(i32)]
pub enum Action {
    StartBreak = 0,
    AbortBreak = 1,
    StopBreak = 2,
    GetUpdatedBlock = 3,
    DropItem = 4,
    StartSleeping = 5,
    StopSleeping = 6,
    Respawn = 7,
    Jump = 8,
    StartSprint = 9,
    StopSprint = 10,
    StartSneak = 11,
    StopSneak = 12,
    CreativePlayerDestroyBlock = 13,
    DimensionChangeAck = 14,
    StartGlide = 15,
    StopGlide = 16,
    BuildDenied = 17,
    CrackBreak = 18,
    ChangeSkin = 19,
    SetEnchantmentSeed = 20,
    Swimming = 21,
    StopSwimming = 22,
    StartSpinAttack = 23,
    StopSpinAttack = 24,
    InteractBlock = 25,
    PredictDestroyBlock = 26,
    ContinueDestroyBlock = 27,
    StartItemUseOn = 28,
    StopItemUseOn = 29,
    HandledTeleport = 30,
    MissedSwing = 31,
    StartCrawling = 32,
    StopCrawling = 33,
    StartFlying = 34,
    StopFlying = 35,
    ClientAckServerData = 36,
}

impl PacketRead for Action {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let action = VarInt::read(reader)?;
        match action.0 {
            0 => Ok(Self::StartBreak),
            1 => Ok(Self::AbortBreak),
            2 => Ok(Self::StopBreak),
            3 => Ok(Self::GetUpdatedBlock),
            4 => Ok(Self::DropItem),
            5 => Ok(Self::StartSleeping),
            6 => Ok(Self::StopSleeping),
            7 => Ok(Self::Respawn),
            8 => Ok(Self::Jump),
            9 => Ok(Self::StartSprint),
            10 => Ok(Self::StopSprint),
            11 => Ok(Self::StartSneak),
            12 => Ok(Self::StopSneak),
            13 => Ok(Self::CreativePlayerDestroyBlock),
            14 => Ok(Self::DimensionChangeAck),
            15 => Ok(Self::StartGlide),
            16 => Ok(Self::StopGlide),
            17 => Ok(Self::BuildDenied),
            18 => Ok(Self::CrackBreak),
            19 => Ok(Self::ChangeSkin),
            20 => Ok(Self::SetEnchantmentSeed),
            21 => Ok(Self::Swimming),
            22 => Ok(Self::StopSwimming),
            23 => Ok(Self::StartSpinAttack),
            24 => Ok(Self::StopSpinAttack),
            25 => Ok(Self::InteractBlock),
            26 => Ok(Self::PredictDestroyBlock),
            27 => Ok(Self::ContinueDestroyBlock),
            28 => Ok(Self::StartItemUseOn),
            29 => Ok(Self::StopItemUseOn),
            30 => Ok(Self::HandledTeleport),
            31 => Ok(Self::MissedSwing),
            32 => Ok(Self::StartCrawling),
            33 => Ok(Self::StopCrawling),
            34 => Ok(Self::StartFlying),
            35 => Ok(Self::StopFlying),
            36 => Ok(Self::ClientAckServerData),
            _ => Err(Error::other(format!("Invalid action ID: {}", action.0))),
        }
    }
}
