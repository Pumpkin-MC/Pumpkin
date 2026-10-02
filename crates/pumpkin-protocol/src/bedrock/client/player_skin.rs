// Last verified for v2169

use std::io::{Error, Write};

use pumpkin_macros::packet;
use uuid::Uuid;

use crate::serial::PacketWrite;

use super::Skin;

/// Tells a client that a player in its tab list changed their skin.
#[packet(93)]
pub struct CPlayerSkin<'a> {
    pub uuid: Uuid,
    pub skin: &'a Skin,
    pub new_skin_name: &'a str,
    pub old_skin_name: &'a str,
}

impl PacketWrite for CPlayerSkin<'_> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.uuid.write(writer)?;
        self.skin.write(writer)?;
        self.new_skin_name.write(writer)?;
        self.old_skin_name.write(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Packet;

    #[test]
    fn player_skin_ends_with_the_skin_names() {
        assert_eq!(<CPlayerSkin as Packet>::PACKET_ID, 93);

        let uuid = Uuid::from_u128(0x0123_4567_89ab_cdef_0123_4567_89ab_cdef);
        let skin = Skin::steve();
        let mut encoded = Vec::new();
        CPlayerSkin {
            uuid,
            skin: &skin,
            new_skin_name: "new",
            old_skin_name: "",
        }
        .write(&mut encoded)
        .unwrap();

        // Since v2168 there is no trailing trusted flag after the names; it lives in the skin.
        let mut expected = Vec::new();
        uuid.write(&mut expected).unwrap();
        skin.write(&mut expected).unwrap();
        expected.extend([3, b'n', b'e', b'w', 0]);
        assert_eq!(encoded, expected);
    }
}
