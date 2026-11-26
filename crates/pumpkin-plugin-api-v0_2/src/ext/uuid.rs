use std::fmt;

use crate::wit::pumpkin::plugin::uuid::Uuid;

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            (self.high >> 32) as u32,
            ((self.high >> 16) & 0xffff) as u16,
            (self.high & 0xffff) as u16,
            (self.low >> 48) as u16,
            self.low & 0x0000_ffff_ffff_ffff,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_uuid_correctly() {
        let uuid = Uuid {
            high: 0x0011223344556677,
            low: 0x8899aabbccddeeff,
        };

        assert_eq!(uuid.to_string(), "00112233-4455-6677-8899-aabbccddeeff");
    }
}
