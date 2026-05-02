use std::io::{Error, ErrorKind, Read, Write};

const MAX_ACK_RECORDS: u16 = 4096;

use crate::{
    codec::u24,
    serial::{PacketRead, PacketWrite},
};

pub struct Acknowledge {
    pub sequences: Vec<u32>,
}

impl Acknowledge {
    #[must_use]
    pub const fn new(sequences: Vec<u32>) -> Self {
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

        if size > MAX_ACK_RECORDS {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Acknowledge packet range is too large.",
            ));
        }

        let mut sequences = Vec::with_capacity(size as usize);
        for _ in 0..size {
            let single = bool::read(reader)?;
            if single {
                sequences.push(u24::read(reader)?.0);
            } else {
                let start = u24::read(reader)?.0;
                let end = u24::read(reader)?.0;
                for i in start..=end {
                    sequences.push(i);
                }
            }
        }
        Ok(Self { sequences })
    }

    pub fn write<W: Write>(&self, writer: &mut W, id: u8) -> Result<(), Error> {
        id.write(writer)?;
        if self.sequences.is_empty() {
            0u16.write_be(writer)?;
            return Ok(());
        }
        let mut count: u16 = 0;

        let mut buf = Vec::new();

        let mut sequences = self.sequences.clone();
        sequences.sort_unstable();

        let mut start = sequences[0];
        let mut end = start;
        for seq in sequences.iter().copied().skip(1) {
            if seq != end + 1 {
                Self::write_range(start, end, &mut buf)?;
                count += 1;
                start = seq;
            }
            end = seq;
        }
        Self::write_range(start, end, &mut buf)?;
        count += 1;
        count.write_be(writer)?;
        writer.write_all(&buf)
    }
}
