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
        write.write_u8(0x84)?;
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
    pub reliable_number: u32,
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
            let mut frame = Self::default();
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
            let length = read.get_u16_be()? >> 3;

            if reliability.is_reliable() {
                frame.reliable_number = read.get_u24()?.0
            }

            if reliability.is_sequenced() {
                frame.sequence_index = read.get_u24()?.0
            }

            if reliability.is_ordered() {
                frame.order_index = read.get_u24()?.0;
                frame.order_channel = read.get_u8_be()?;
            }

            if split {
                frame.split_size = read.get_u32_be()?;
                frame.split_id = read.get_u16_be()?;
                frame.split_index = read.get_u32_be()?;
            }

            frame.reliability = reliability;
            frame.payload = read.read_boxed_slice(length as usize)?.into();
            frames.push(frame);
        }

        Ok(frames)
    }

    pub fn write(&self, mut write: impl Write) -> Result<(), WritingError> {
        let is_split = self.split_size > 0;
        write
            .write_u8((self.reliability.to_id() << 5) & if is_split { RAKNET_SPLIT } else { 0 })?;
        write.write_u16_be((self.payload.len() << 3) as u16)?;
        if self.reliability.is_reliable() {
            write.write_u24_be(U24(self.reliable_number))?;
        }
        if self.reliability.is_sequenced() {
            write.write_u24_be(U24(self.sequence_index))?;
        }
        if self.reliability.is_ordered() {
            write.write_u24_be(U24(self.order_index))?;
            write.write_u8(self.order_channel)?;
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
