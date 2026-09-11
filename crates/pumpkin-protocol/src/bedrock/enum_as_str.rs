use crate::serial::{PacketRead, PacketWrite};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnumAsStr<T>(T);

impl<T: FromStr> FromStr for EnumAsStr<T>
where
    std::io::Error: From<T::Err>,
{
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(T::from_str(s)?))
    }
}

#[allow(clippy::to_string_trait_impl)]
impl<T: ToString> ToString for EnumAsStr<T> {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl<T: FromStr> PacketRead for EnumAsStr<T>
where
    std::io::Error: From<T::Err>,
{
    fn read<R: std::io::Read>(reader: &mut R) -> Result<Self, std::io::Error> {
        Self::from_str(&String::read(reader)?)
    }
}

impl<T: ToString> PacketWrite for EnumAsStr<T> {
    fn write<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        self.to_string().write(writer)
    }
}

impl<T> From<T> for EnumAsStr<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}
