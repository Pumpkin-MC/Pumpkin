use std::{
    io::{Error, ErrorKind, Read},
    usize,
};

use crate::serial::PacketRead;

#[derive(Debug)]
pub struct Bitset<const N: usize> {
    pub bits: u128,
}

impl<const N: usize> Bitset<N> {
    pub const fn new() -> Self {
        if N > 80 {
            panic!()
        }
        Self { bits: 0 }
    }

    pub fn get<T: Into<usize>>(&self, index: T) -> bool {
        let index: usize = index.into();
        if index > N {
            panic!("")
        }
        (self.bits & (1 << index)) != 0
    }

    pub fn set<T: Into<usize>>(&mut self, index: T, value: bool) {
        let index: usize = index.into();
        if index > N {
            panic!("")
        }
        if value {
            self.bits |= 1 << index;
        } else {
            self.bits &= !(1 << index);
        }
    }
}

impl<const N: usize> PacketRead for Bitset<N> {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut bitset = Bitset::<N>::new();

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
