use pumpkin_macros::packet;

use crate::{ClientPacket, codec::u24::U24, ser::NetworkWriteExt};

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
            write.write_u8_be(1)?;
            U24::encode(&U24(start), &mut write)?;
        } else {
            write.write_u8_be(0)?;
            U24::encode(&U24(start), &mut write)?;
            U24::encode(&U24(end), &mut write)?;
        }
        Ok(())
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
