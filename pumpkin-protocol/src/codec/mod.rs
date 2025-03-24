use std::{
    io::{Read, Write},
    num::NonZeroUsize,
};

use crate::ser::{ReadingError, WritingError};

pub mod bit_set;
pub mod identifier;
pub mod slot;
pub mod var_int;
pub mod var_long;

pub trait Codec<T> {
    const MAX_SIZE: NonZeroUsize;

    fn written_size(&self) -> usize;

    fn encode(&self, write: &mut impl Write) -> Result<(), WritingError>;

    fn decode(read: &mut impl Read) -> Result<T, ReadingError>;
}
