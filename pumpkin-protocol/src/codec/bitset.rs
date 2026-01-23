use std::io::{Error, ErrorKind, Read};

use crate::serial::PacketRead;

#[derive(Debug)]
pub struct Bitset<const N: usize> {
    pub bits: u128,
}

impl<const N: usize> Bitset<N> {
    pub fn get<T: Into<usize>>(&self, index: T) -> bool {
        let index: usize = index.into();
        assert!(index <= N, "");
        (self.bits & (1 << index)) != 0
    }

    pub fn set<T: Into<usize>>(&mut self, index: T, value: bool) {
        let index: usize = index.into();
        assert!(index <= N, "");
        if value {
            self.bits |= 1 << index;
        } else {
            self.bits &= !(1 << index);
        }
    }
}

impl<const N: usize> Default for Bitset<N> {
    fn default() -> Self {
        assert!(N <= 80,);
        Self { bits: 0 }
    }
}

impl<const N: usize> PacketRead for Bitset<N> {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut bitset = Self::default();

        for i in 0..N.div_ceil(8) {
            let byte = u8::read(reader)?;
            bitset.bits |= (u128::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                return Ok(bitset);
            }
        }
        Err(Error::new(ErrorKind::InvalidData, ""))
    }
}
