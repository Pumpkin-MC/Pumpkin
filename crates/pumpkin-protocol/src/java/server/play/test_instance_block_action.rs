use pumpkin_data::packet::serverbound::play::TEST_INSTANCE_BLOCK_ACTION;
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_util::version::JavaMinecraftVersion;
use std::io::Read;

use crate::{
    ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};

const MAX_TEST_ID_LENGTH: usize = 32767;

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
            data: TestInstanceBlockData::read(bytebuf, *version)?,
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
        self.data.write(&mut write, *version)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestInstanceBlockData<'a> {
    pub test: Option<&'a str>,
    pub size: VarIntVector3,
    pub rotation: pumpkin_data::block_rotation::Rotation,
    pub ignore_entities: bool,
    pub status: TestInstanceBlockStatus,
    pub error_message: Option<TextComponent>,
}

impl<'a> TestInstanceBlockData<'a> {
    fn read(bytebuf: &mut &'a [u8], version: JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let test = bytebuf.get_option(|b| b.get_str_bounded_borrowed(MAX_TEST_ID_LENGTH))?;
        let size = VarIntVector3::read(bytebuf)?;
        let rotation = match bytebuf.get_var_int()?.0.rem_euclid(4) {
            0 => pumpkin_data::block_rotation::Rotation::None,
            1 => pumpkin_data::block_rotation::Rotation::Clockwise90,
            2 => pumpkin_data::block_rotation::Rotation::Rotate180,
            _ => pumpkin_data::block_rotation::Rotation::CounterClockwise90,
        };
        let ignore_entities = bytebuf.get_bool()?;
        let status = match bytebuf.get_var_int()?.0 {
            1 => TestInstanceBlockStatus::Running,
            2 => TestInstanceBlockStatus::Finished,
            _ => TestInstanceBlockStatus::Cleared,
        };
        let error_message = bytebuf.get_option(|b| b.get_component_borrowed(&version))?;

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
        version: JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_option(&self.test, |w, t| {
            w.write_string_bounded(t, MAX_TEST_ID_LENGTH)
        })?;
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
            TestInstanceBlockStatus::Finished => 2,
        };
        write.write_var_int(&VarInt(status_val))?;
        write.write_option(&self.error_message, |w, msg| {
            w.write_component(msg, &version)
        })?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TestInstanceBlockStatus {
    Cleared,
    Running,
    Finished,
}

#[cfg(test)]
mod tests {
    use crate::ClientPacket;

    use super::*;

    #[test]
    fn roundtrip_test_instance_block_action_with_component_error() {
        let packet = STestInstanceBlockAction {
            pos: BlockPos::new(10, 64, -20),
            action: TestInstanceBlockAction::Set,
            data: TestInstanceBlockData {
                test: Some("minecraft:always_pass"),
                size: VarIntVector3 {
                    x: VarInt(3),
                    y: VarInt(4),
                    z: VarInt(5),
                },
                rotation: pumpkin_data::block_rotation::Rotation::Clockwise90,
                ignore_entities: true,
                status: TestInstanceBlockStatus::Finished,
                error_message: Some(TextComponent::text("Assertion failed at (1, 2, 3)")),
            },
        };

        let mut buf = Vec::new();
        packet
            .write_packet_data(&mut buf, &JavaMinecraftVersion::V_26_3)
            .unwrap();

        let mut slice = buf.as_slice();
        let decoded =
            STestInstanceBlockAction::read(&mut slice, &JavaMinecraftVersion::V_26_3).unwrap();
        assert!(slice.is_empty());
        assert_eq!(decoded.pos, packet.pos);
        assert_eq!(decoded.action, packet.action);
        assert_eq!(decoded.data, packet.data);
    }
}
