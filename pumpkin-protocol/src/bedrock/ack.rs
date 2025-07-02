use pumpkin_macros::packet;

use crate::{
    ClientPacket, ServerPacket,
    codec::u24::U24,
    ser::{NetworkReadExt, NetworkWriteExt},
};

#[packet(0xC0)]
pub struct Ack {
    sequences: Vec<u32>,
}

impl Ack {
    pub fn new(sequences: Vec<u32>) -> Self {
        Self { sequences }
    }
}

impl Ack {
    fn write_range(
        start: u32,
        end: u32,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        if start == end {
            write.write_u8(1)?;
            U24::encode(&U24(start), &mut write)?;
        } else {
            write.write_u8(0)?;
            U24::encode(&U24(start), &mut write)?;
            U24::encode(&U24(end), &mut write)?;
        }
        Ok(())
    }
}

impl ServerPacket for Ack {
    fn read(mut read: impl std::io::Read) -> Result<Self, crate::ser::ReadingError> {
        let size = read.get_u16_be()?;
        // TODO: check size
        let mut sequences = Vec::with_capacity(size as usize);
        for _ in 0..size {
            let single = read.get_bool()?;
            if single {
                sequences.push(U24::decode(&mut read)?.0);
            } else {
                let start = U24::decode(&mut read)?;
                let end = U24::decode(&mut read)?;
                for i in start.0..end.0 {
                    sequences.push(i);
                }
            }
        }
        Ok(Self { sequences })
    }
}

impl ClientPacket for Ack {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        let mut buffer = Vec::new();
        let mut count = 0;

        let mut start = self.sequences[0];
        let mut end = start;
        for seq in self.sequences.clone() {
            if seq == end + 1 {
                end = seq
            } else {
                Self::write_range(start, end, &mut buffer)?;
                count += 1;
                start = seq;
                end = seq;
            }
        }
        Self::write_range(start, end, &mut buffer)?;
        count += 1;

        write.write_u16_be(count)?;
        write.write_slice(&buffer)?;

        Ok(())
    }
}
