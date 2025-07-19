use std::io::{Read, Write};

use pumpkin_macros::packet;

use crate::{
    codec::u24,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};

#[packet(0xC0)]
pub struct Ack {
    sequences: Vec<u32>,
}

impl Ack {
    pub fn new(sequences: Vec<u32>) -> Self {
        Self { sequences }
    }

    fn write_range(start: u32, end: u32, mut write: impl Write) -> Result<(), WritingError> {
        if start == end {
            write.write_u8(1)?;
            u24::encode(&u24(start), &mut write)?;
        } else {
            write.write_u8(0)?;
            u24::encode(&u24(start), &mut write)?;
            u24::encode(&u24(end), &mut write)?;
        }
        Ok(())
    }

    pub fn read(mut read: impl Read) -> Result<Self, ReadingError> {
        let size = read.get_u16_be()?;
        // TODO: check size
        let mut sequences = Vec::with_capacity(size as usize);
        for _ in 0..size {
            let single = read.get_bool()?;
            if single {
                sequences.push(u24::decode(&mut read)?.0);
            } else {
                let start = u24::decode(&mut read)?;
                let end = u24::decode(&mut read)?;
                for i in start.0..end.0 {
                    sequences.push(i);
                }
            }
        }
        Ok(Self { sequences })
    }

    pub fn write(&self, mut writer: impl Write) -> Result<(), WritingError> {
        writer.write_u8(0xC0).unwrap();
        let mut count = 0;

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

        writer.write_u16_be(count)?;
        writer.write_slice(&buf)
    }
}
