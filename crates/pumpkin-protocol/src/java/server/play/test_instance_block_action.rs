use pumpkin_data::packet::serverbound::play::TEST_INSTANCE_BLOCK_ACTION;
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::BlockPos;

use crate::VarInt;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;
use std::io::Read;

#[java_packet(TEST_INSTANCE_BLOCK_ACTION)]
pub struct STestInstanceBlockAction<'a> {
    pub pos: BlockPos,
    pub action: TestInstanceBlockAction,
    pub data: TestInstanceBlockData<'a>,
}

impl<'a> ServerPacket<'a> for STestInstanceBlockAction<'a> {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            pos: bytebuf.get_block_pos(version)?,
            action: TestInstanceBlockAction::read(bytebuf)?,
            data: TestInstanceBlockData::read(bytebuf)?,
        })
    }
}

impl crate::ClientPacket for STestInstanceBlockAction<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_block_pos(&self.pos, version)?;
        self.action.write(&mut write)?;
        self.data.write(&mut write)?;
        Ok(())
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

    fn write(
        self,
        write: &mut impl crate::ser::NetworkWriteExt,
    ) -> Result<(), crate::ser::WritingError> {
        let val = match self {
            Self::Init => 0,
            Self::Query => 1,
            Self::Set => 2,
            Self::Reset => 3,
            Self::Save => 4,
            Self::Export => 5,
            Self::Run => 6,
        };
        write.write_var_int(&VarInt(val))
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

    fn write(
        &self,
        write: &mut impl crate::ser::NetworkWriteExt,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.x)?;
        write.write_var_int(&self.y)?;
        write.write_var_int(&self.z)?;
        Ok(())
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

    fn write(
        &self,
        write: &mut impl crate::ser::NetworkWriteExt,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_option(&self.test, |w, t| w.write_string(t))?;
        self.size.write(write)?;
        let rot_val = match self.rotation {
            pumpkin_data::block_rotation::Rotation::None => 0,
            pumpkin_data::block_rotation::Rotation::Clockwise90 => 1,
            pumpkin_data::block_rotation::Rotation::Rotate180 => 2,
            pumpkin_data::block_rotation::Rotation::CounterClockwise90 => 3,
        };
        write.write_var_int(&VarInt(rot_val))?;
        write.write_bool(self.ignore_entities)?;
        let status_val = match self.status {
            TestInstanceBlockStatus::Cleared => 0,
            TestInstanceBlockStatus::Running => 1,
            TestInstanceBlockStatus::Success => 2,
            TestInstanceBlockStatus::Failed => 3,
        };
        write.write_var_int(&VarInt(status_val))?;
        write.write_option(&self.error_message, |w, msg| w.write_string(msg))?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TestInstanceBlockStatus {
    Cleared,
    Running,
    Success,
    Failed,
}
