// Last verified for v2169

use uuid::Uuid;

use crate::{
    codec::var_ulong::VarULong,
    serial::{PacketRead, PacketWrite},
};
use pumpkin_macros::packet;

#[derive(Debug, PacketRead, PacketWrite)]
#[packet(152)]
pub struct SEmoteList {
    pub runtime_id: VarULong,
    pub emote_piece_ids: Vec<Uuid>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn emote_list_serialization() {
        let packet = SEmoteList {
            runtime_id: VarULong(123),
            emote_piece_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
        };

        let mut buf = Vec::new();
        packet.write(&mut buf).unwrap();

        let mut reader = Cursor::new(buf);
        let decoded = SEmoteList::read(&mut reader).unwrap();

        assert_eq!(packet.runtime_id.0, decoded.runtime_id.0);
        assert_eq!(packet.emote_piece_ids, decoded.emote_piece_ids);
    }
}
