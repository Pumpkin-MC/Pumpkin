use std::io::{Error, ErrorKind, Read, Write};
use uuid::Uuid;

use crate::{
    codec::{var_uint::VarUInt, var_ulong::VarULong},
    serial::{PacketRead, PacketWrite},
};
use pumpkin_macros::packet;

/// Bound for the emote piece list. Currently unreachable (dispatch is commented out),
/// but the unbounded `VarUInt` length would pre-allocate attacker-sized memory if re-enabled.
const MAX_EMOTE_PIECES: usize = 64;

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
        if len > MAX_EMOTE_PIECES {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("emote piece count {len} exceeds limit {MAX_EMOTE_PIECES}"),
            ));
        }
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

    #[test]
    fn accepts_emote_count_at_cap() {
        let packet = SEmoteList {
            runtime_entity_id: VarULong(123),
            emote_pieces: vec![Uuid::nil(); MAX_EMOTE_PIECES],
        };

        let mut buf = Vec::new();
        packet.write(&mut buf).unwrap();

        let decoded = SEmoteList::read(&mut Cursor::new(buf)).unwrap();
        assert_eq!(decoded.emote_pieces.len(), MAX_EMOTE_PIECES);
    }

    #[test]
    fn rejects_emote_count_over_cap() {
        let mut buf = Vec::new();
        VarULong(123).write(&mut buf).unwrap();
        VarUInt((MAX_EMOTE_PIECES + 1) as u32)
            .write(&mut buf)
            .unwrap();

        let err = SEmoteList::read(&mut Cursor::new(buf)).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidData);
    }
}
