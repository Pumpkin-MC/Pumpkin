use std::io::{Error, ErrorKind, Read, Write};

const MAX_ACK_RECORDS: u16 = 4096;
/// Upper bound on the total number of sequence numbers a single ACK/NACK
/// packet may expand to. Range records (`start..=end`) cover 24-bit values, so
/// without this cap a handful of records could expand to hundreds of millions
/// of entries and exhaust memory.
const MAX_ACK_SEQUENCES: usize = 4096;

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
            let (start, end) = if single {
                let seq = u24::read(reader)?.0;
                (seq, seq)
            } else {
                (u24::read(reader)?.0, u24::read(reader)?.0)
            };

            // Bound the total expanded sequences before growing the vector so a
            // record (or a few range records) cannot blow up into hundreds of
            // millions of entries. `end < start` yields an empty range below.
            let span = (end.saturating_sub(start) as usize).saturating_add(1);
            if sequences.len().saturating_add(span) > MAX_ACK_SEQUENCES {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Acknowledge packet expands to too many sequences.",
                ));
            }

            for i in start..=end {
                sequences.push(i);
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
