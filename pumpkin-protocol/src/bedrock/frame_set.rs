use std::io::Write;

use bytes::Bytes;
use pumpkin_macros::packet;

use crate::bedrock::{RAKNET_SPLIT, RakReliability};
use crate::codec::u24::U24;
use crate::ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError};
use crate::{ClientPacket, ServerPacket};

#[packet[0x80]]
pub struct FrameSet {
    pub sequence: U24,
    pub frames: Vec<Frame>,
}

impl ServerPacket for FrameSet {
    fn read(mut read: impl std::io::Read) -> Result<Self, ReadingError> {
        Ok(Self {
            sequence: read.get_u24()?,
            frames: Frame::read(read)?,
        })
    }
}

impl ClientPacket for FrameSet {
    fn write_packet_data(&self, mut write: impl Write) -> Result<(), WritingError> {
        write.write_u24_be(self.sequence)?;
        for frame in &self.frames {
            frame.write(&mut write)?;
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct Frame {
    pub reliability: RakReliability,
    pub payload: Bytes,
    pub reliable_index: u32,
    pub sequence_index: u32,
    pub order_index: u32,
    pub order_channel: u8,
    pub split_size: u32,
    pub split_id: u16,
    pub split_index: u32,
}

impl Frame {
    pub fn read(mut read: impl std::io::Read) -> Result<Vec<Self>, crate::ser::ReadingError> {
        let mut frames = Vec::new();

        while let Ok(header) = read.get_u8_be() {
            let reliability_id = (header & 0xE0) >> 5;
            let reliability = match RakReliability::from_id(reliability_id) {
                Some(reliability) => reliability,
                None => {
                    return Err(ReadingError::Message(format!(
                        "Invalid RakReliability {reliability_id}"
                    )));
                }
            };
            let split = (header & RAKNET_SPLIT) != 0;
            let length = (read.get_u16_be()? as f32 / 8.0).ceil();

            let reliable_index = if reliability.is_reliable() {
                read.get_u24()?.0
            } else {
                0
            };

            let sequence_index = if reliability.is_sequenced() {
                read.get_u24()?.0
            } else {
                0
            };

            let (order_index, order_channel) = if reliability.is_ordered() {
                (read.get_u24()?.0, read.get_u8_be()?)
            } else {
                (0, 0)
            };
            let (split_size, split_id, split_index) = if split {
                (read.get_u32_be()?, read.get_u16_be()?, read.get_u32_be()?)
            } else {
                (0, 0, 0)
            };
            let payload = read.read_boxed_slice(length as usize)?;
            frames.push(Self {
                reliability,
                payload: payload.into(),
                reliable_index,
                sequence_index,
                order_index,
                order_channel,
                split_size,
                split_id,
                split_index,
            });
        }

        Ok(frames)
    }

    fn write(&self, mut write: impl Write) -> Result<(), WritingError> {
        let is_split = self.split_size > 0;
        write.write_u8_be(
            (self.reliability.to_id() >> 5) & if is_split { RAKNET_SPLIT } else { 0 },
        )?;
        write.write_u16_be((self.payload.len() >> 3) as u16)?;
        if self.reliability.is_reliable() {
            write.write_u24_be(U24(self.reliable_index))?;
        }
        if self.reliability.is_sequenced() {
            write.write_u24_be(U24(self.sequence_index))?;
        }
        if self.reliability.is_ordered() {
            write.write_u24_be(U24(self.order_index))?;
            write.write_u8_be(self.order_channel)?;
        }
        if is_split {
            write.write_u32_be(self.split_size)?;
            write.write_u16_be(self.split_id)?;
            write.write_u32_be(self.split_index)?;
        }

        write.write_slice(&self.payload).unwrap();

        Ok(())
    }
}
