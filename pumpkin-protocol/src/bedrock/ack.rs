use std::io::{Error, Read, Write};

use pumpkin_macros::packet;

use crate::{
    codec::u24,
    serial::{PacketRead, PacketWrite},
};

#[packet(0xC0)]
pub struct Ack {
    sequences: Vec<u32>,
}

impl Ack {
    pub fn new(sequences: Vec<u32>) -> Self {
        Self { sequences }
    }

    fn write_range<W: Write>(start: u32, end: u32, writer: &mut W) -> Result<(), Error> {
        if start == end {
            1u8.write(writer)?;
            u24(start).write(writer)
        } else {
            0u8.write(writer)?;
            u24(start).write(writer)?;
            u24(end).write(writer)
        }
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let size = u16::read_be(reader)?;
        // TODO: check size
        let mut sequences = Vec::with_capacity(size as usize);
        for _ in 0..size {
            let single = bool::read(reader)?;
            if single {
                sequences.push(u24::read(reader)?.0);
            } else {
                let start = u24::read(reader)?.0;
                let end = u24::read(reader)?.0;
                for i in start..end {
                    sequences.push(i);
                }
            }
        }
        Ok(Self { sequences })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        0xC0u8.write(writer)?;
        let mut count: u16 = 0;

        let mut buf = Vec::new();

        let mut start = self.sequences[0];
        let mut end = start;
        for seq in self.sequences.clone() {
            if seq == end + 1 {
                end = seq
            } else {
                Self::write_range(start, end, &mut buf)?;
                count += 1;
                start = seq;
                end = seq;
            }
        }
        Self::write_range(start, end, &mut buf)?;
        count += 1;
        count.write_be(writer)?;
        writer.write_all(&buf)
    }
}
