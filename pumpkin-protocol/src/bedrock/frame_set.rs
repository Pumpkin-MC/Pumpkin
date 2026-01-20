use std::io::{Error, Read, Write};

use crate::bedrock::{RAKNET_SPLIT, RakReliability};
use crate::codec::u24;
use crate::serial::{PacketRead, PacketWrite};

pub struct FrameSet {
    pub sequence: u24,
    pub frames: Vec<Frame>,
}

impl FrameSet {
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self {
            sequence: u24::read(reader)?,
            frames: Frame::read(reader)?,
        })
    }

    pub fn write_packet_data<W: Write>(&self, writer: &mut W, id: u8) -> Result<(), Error> {
        id.write(writer)?;
        self.sequence.write(writer)?;
        for frame in &self.frames {
            frame.write(writer)?;
        }
        Ok(())
    }
}

impl Default for FrameSet {
    fn default() -> Self {
        Self {
            sequence: u24(0),
            frames: Vec::default(),
        }
    }
}

#[derive(Default)]
pub struct Frame {
    pub reliability: RakReliability,
    // If we write a packet we dont want to own the payload to avoid cloning
    pub payload: Vec<u8>,
    pub reliable_number: u32,
    pub sequence_index: u32,
    pub order_index: u32,
    pub order_channel: u8,
    pub split_size: u32,
    pub split_id: u16,
    pub split_index: u32,
}

impl Frame {
    #[must_use] 
    pub const fn new_unreliable(payload: Vec<u8>) -> Self {
        Self {
            reliability: RakReliability::Unreliable,
            payload,
            reliable_number: 0,
            sequence_index: 0,
            order_index: 0,
            order_channel: 0,
            split_size: 0,
            split_id: 0,
            split_index: 0,
        }
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Vec<Self>, Error> {
        let mut frames = Vec::new();

        while let Ok(header) = u8::read(reader) {
            let mut frame = Self::default();
            let reliability_id = (header & 0xE0) >> 5;
            let reliability = match RakReliability::from_id(reliability_id) {
                Some(reliability) => reliability,
                None => return Err(Error::other("Invalid reliability")),
            };
            let split = (header & RAKNET_SPLIT) != 0;
            let length = u16::read_be(reader)? >> 3;

            if reliability.is_reliable() {
                frame.reliable_number = u24::read(reader)?.0;
            }

            if reliability.is_sequenced() {
                frame.sequence_index = u24::read(reader)?.0;
            }

            if reliability.is_ordered() {
                frame.order_index = u24::read(reader)?.0;
                frame.order_channel = u8::read(reader)?;
            }

            if split {
                frame.split_size = u32::read_be(reader)?;
                frame.split_id = u16::read_be(reader)?;
                frame.split_index = u32::read_be(reader)?;
            }

            frame.reliability = reliability;
            frame.payload = vec![0; length as usize];
            reader.read_exact(&mut frame.payload)?;
            frames.push(frame);
        }

        Ok(frames)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let is_split = self.split_size > 0;
        let mut flags = self.reliability.to_id() << 5;
        if is_split {
            flags |= RAKNET_SPLIT;
        }
        flags.write(writer)?;
        // Size
        ((self.payload.len() as u16) << 3).write_be(writer)?;
        if self.reliability.is_reliable() {
            u24(self.reliable_number).write(writer)?;
        }
        if self.reliability.is_sequenced() {
            u24(self.sequence_index).write(writer)?;
        }
        if self.reliability.is_ordered() {
            u24(self.order_index).write(writer)?;
            self.order_channel.write(writer)?;
        }
        if is_split {
            self.split_size.write_be(writer)?;
            self.split_id.write_be(writer)?;
            self.split_index.write_be(writer)?;
        }

        writer.write_all(&self.payload)
    }
}
