use std::io::{Error, Read, Write};
use uuid::Uuid;

use crate::{
    codec::{var_uint::VarUInt, var_ulong::VarULong},
    serial::{PacketRead, PacketWrite},
};
use pumpkin_macros::packet;

#[derive(Debug)]
#[packet(152)]
pub struct SEmoteList {
    pub runtime_entity_id: VarULong,
    pub emote_pieces: Vec<Uuid>,
}

impl PacketRead for SEmoteList {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let runtime_entity_id = VarULong::read(reader)?;
        let len = VarUInt::read(reader)?.0 as usize;
        let mut emote_pieces = Vec::with_capacity(len);
        for _ in 0..len {
            emote_pieces.push(Uuid::read(reader)?);
        }
        Ok(Self {
            runtime_entity_id,
            emote_pieces,
        })
    }
}

impl PacketWrite for SEmoteList {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.runtime_entity_id.write(writer)?;
        VarUInt(self.emote_pieces.len() as u32).write(writer)?;
        for piece in &self.emote_pieces {
            piece.write(writer)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn emote_list_serialization() {
        let packet = SEmoteList {
            runtime_entity_id: VarULong(123),
            emote_pieces: vec![Uuid::new_v4(), Uuid::new_v4()],
        };

        let mut buf = Vec::new();
        packet.write(&mut buf).unwrap();

        let mut reader = Cursor::new(buf);
        let decoded = SEmoteList::read(&mut reader).unwrap();

        assert_eq!(packet.runtime_entity_id.0, decoded.runtime_entity_id.0);
        assert_eq!(packet.emote_pieces, decoded.emote_pieces);
    }
}
