use std::io::{Error, Write};

use uuid::Uuid;

use crate::serial::PacketWrite;

impl PacketWrite for Uuid {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let uuid = self.as_u128();
        ((uuid >> 64) as u64).write(writer)?;
        (uuid as u64).write(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serial::PacketRead;

    #[test]
    fn uuid_uses_bedrock_word_order() {
        let uuid = uuid::uuid!("40027bad-77e4-474b-9d90-27bf8e2deb74");
        let mut encoded = Vec::new();

        uuid.write(&mut encoded).unwrap();

        assert_eq!(
            encoded,
            [
                0x4b, 0x47, 0xe4, 0x77, 0xad, 0x7b, 0x02, 0x40, 0x74, 0xeb, 0x2d, 0x8e, 0xbf, 0x27,
                0x90, 0x9d,
            ]
        );
        assert_eq!(Uuid::read(&mut encoded.as_slice()).unwrap(), uuid);
    }
}
