use pumpkin_data::packet::serverbound::PLAY_TEST_INSTANCE_BLOCK_ACTION;
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::BlockPos;

use crate::VarInt;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;
use std::io::Read;

#[java_packet(PLAY_TEST_INSTANCE_BLOCK_ACTION)]
pub struct STestInstanceBlockAction<'a> {
    pub pos: BlockPos,
    pub action: TestInstanceBlockAction,
    pub data: TestInstanceBlockData<'a>,
}

impl<'a> ServerPacket<'a> for STestInstanceBlockAction<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            pos: BlockPos::from_i64(bytebuf.get_i64_be()?),
            action: TestInstanceBlockAction::read(bytebuf)?,
            data: TestInstanceBlockData::read(bytebuf)?,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TestInstanceBlockAction {
    Init,
    Query,
    Set,
    Reset,
    Save,
    Export,
    Run,
}

impl TestInstanceBlockAction {
    fn read(bytebuf: &mut impl Read) -> Result<Self, ReadingError> {
        match bytebuf.get_var_int()?.0 {
            0 => Ok(Self::Init),
            1 => Ok(Self::Query),
            2 => Ok(Self::Set),
            3 => Ok(Self::Reset),
            4 => Ok(Self::Save),
            5 => Ok(Self::Export),
            6 => Ok(Self::Run),
            _ => Err(ReadingError::Message(
                "Invalid TestInstanceBlockAction".to_string(),
            )),
        }
    }
}

pub struct VarIntVector3 {
    pub x: VarInt,
    pub y: VarInt,
    pub z: VarInt,
}

impl VarIntVector3 {
    fn read(bytebuf: &mut impl Read) -> Result<Self, ReadingError> {
        Ok(Self {
            x: bytebuf.get_var_int()?,
            y: bytebuf.get_var_int()?,
            z: bytebuf.get_var_int()?,
        })
    }
}

pub struct TestInstanceBlockData<'a> {
    pub test: Option<&'a str>,
    pub size: VarIntVector3,
    pub rotation: pumpkin_data::block_rotation::Rotation,
    pub ignore_entities: bool,
    pub status: TestInstanceBlockStatus,
    pub error_message: Option<&'a str>,
}

impl<'a> TestInstanceBlockData<'a> {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        let test = bytebuf.get_option(crate::ser::NetworkReadSliceExt::get_str_borrowed)?;
        let size = VarIntVector3::read(bytebuf)?;
        let rotation = match bytebuf.get_var_int()?.0 {
            0 => pumpkin_data::block_rotation::Rotation::None,
            1 => pumpkin_data::block_rotation::Rotation::Clockwise90,
            2 => pumpkin_data::block_rotation::Rotation::Rotate180,
            3 => pumpkin_data::block_rotation::Rotation::CounterClockwise90,
            _ => return Err(ReadingError::Message("Invalid Rotation".to_string())),
        };
        let ignore_entities = bytebuf.get_bool()?;
        let status = match bytebuf.get_var_int()?.0 {
            0 => TestInstanceBlockStatus::Cleared,
            1 => TestInstanceBlockStatus::Running,
            2 => TestInstanceBlockStatus::Success,
            3 => TestInstanceBlockStatus::Failed,
            _ => {
                return Err(ReadingError::Message(
                    "Invalid TestInstanceBlockStatus".to_string(),
                ));
            }
        };
        let error_message =
            bytebuf.get_option(crate::ser::NetworkReadSliceExt::get_str_borrowed)?;

        Ok(Self {
            test,
            size,
            rotation,
            ignore_entities,
            status,
            error_message,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TestInstanceBlockStatus {
    Cleared,
    Running,
    Success,
    Failed,
}
