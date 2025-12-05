use std::{
    io::{Error, Read, Write},
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use aes::cipher::{BlockDecryptMut, BlockEncryptMut, BlockSizeUser, generic_array::GenericArray};
use bytes::Bytes;
use codec::var_int::VarInt;
use pumpkin_util::{
    resource_location::ResourceLocation,
    text::{TextComponent, style::Style},
};
use ser::{ReadingError, WritingError};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{DeserializeSeed, SeqAccess, Visitor},
};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use crate::packet::Packet;

pub mod bedrock;
pub mod codec;
pub mod java;
pub mod packet;
#[cfg(feature = "query")]
pub mod query;
pub mod ser;
pub mod serial;

pub const MAX_PACKET_SIZE: u64 = 2097152;
pub const MAX_PACKET_DATA_SIZE: usize = 8388608;

pub type FixedBitSet = Box<[u8]>;

/// Represents a compression threshold.
///
/// The threshold determines the minimum size of data that should be compressed.
/// Data smaller than the threshold will not be compressed.
pub type CompressionThreshold = usize;

/// Represents a compression level.
///
/// The level controls the amount of compression applied to the data.
/// Higher levels generally result in higher compression ratios, but also
/// increase CPU usage.
pub type CompressionLevel = u32;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ConnectionState {
    HandShake,
    Status,
    Login,
    Transfer,
    Config,
    Play,
}
pub struct InvalidConnectionState;

impl TryFrom<VarInt> for ConnectionState {
    type Error = InvalidConnectionState;

    fn try_from(value: VarInt) -> Result<Self, Self::Error> {
        let value = value.0;
        match value {
            1 => Ok(Self::Status),
            2 => Ok(Self::Login),
            3 => Ok(Self::Transfer),
            _ => Err(InvalidConnectionState),
        }
    }
}

struct IdOrVisitor<T>(PhantomData<T>);
impl<'de, T: Deserialize<'de>> Visitor<'de> for IdOrVisitor<T> {
    type Value = IdOr<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A VarInt followed by a value if the VarInt is 0")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        enum IdOrStateDeserializer<T> {
            Init,
            Id(u16),
            Value(T),
        }

        impl<'de, T: Deserialize<'de>> DeserializeSeed<'de> for &mut IdOrStateDeserializer<T> {
            type Value = ();

            fn deserialize<D: Deserializer<'de>>(
                self,
                deserializer: D,
            ) -> Result<Self::Value, D::Error> {
                match self {
                    IdOrStateDeserializer::Init => {
                        // Get the VarInt
                        let id = VarInt::deserialize(deserializer)?;
                        *self = IdOrStateDeserializer::<T>::Id(id.0.try_into().map_err(|_| {
                            serde::de::Error::custom(format!(
                                "{} cannot be mapped to a registry id",
                                id.0
                            ))
                        })?);
                    }
                    IdOrStateDeserializer::Id(id) => {
                        debug_assert!(*id == 0);
                        // Get the data
                        let value = T::deserialize(deserializer)?;
                        *self = IdOrStateDeserializer::Value(value);
                    }
                    IdOrStateDeserializer::Value(_) => unreachable!(),
                }

                Ok(())
            }
        }

        let mut state = IdOrStateDeserializer::<T>::Init;

        let _ = seq.next_element_seed(&mut state)?;

        match state {
            IdOrStateDeserializer::Id(id) => {
                if id > 0 {
                    return Ok(IdOr::Id(id - 1));
                }
            }
            _ => unreachable!(),
        }

        let _ = seq.next_element_seed(&mut state)?;

        match state {
            IdOrStateDeserializer::Value(val) => Ok(IdOr::Value(val)),
            _ => unreachable!(),
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum IdOr<T> {
    Id(u16),
    Value(T),
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for IdOr<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_seq(IdOrVisitor(PhantomData))
    }
}

impl<T: Serialize> Serialize for IdOr<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            IdOr::Id(id) => VarInt::from(*id + 1).serialize(serializer),
            IdOr::Value(value) => {
                #[derive(Serialize)]
                struct NetworkRepr<T: Serialize> {
                    zero_id: VarInt,
                    value: T,
                }
                NetworkRepr {
                    zero_id: 0.into(),
                    value,
                }
                .serialize(serializer)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct SoundEvent {
    pub sound_name: ResourceLocation,
    pub range: Option<f32>,
}

type Aes128Cfb8Dec = cfb8::Decryptor<aes::Aes128>;

pub struct StreamDecryptor<R: AsyncRead + Unpin> {
    cipher: Aes128Cfb8Dec,
    read: R,
}

impl<R: AsyncRead + Unpin> StreamDecryptor<R> {
    pub fn new(cipher: Aes128Cfb8Dec, stream: R) -> Self {
        Self {
            cipher,
            read: stream,
        }
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for StreamDecryptor<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let ref_self = self.get_mut();
        let read = Pin::new(&mut ref_self.read);
        let cipher = &mut ref_self.cipher;

        // Get the starting position
        let original_fill = buf.filled().len();
        // Read the raw data
        let internal_poll = read.poll_read(cx, buf);

        if matches!(internal_poll, Poll::Ready(Ok(_))) {
            // Decrypt the raw data in-place, note that our block size is 1 byte, so this is always safe
            for block in buf.filled_mut()[original_fill..].chunks_mut(Aes128Cfb8Dec::block_size()) {
                cipher.decrypt_block_mut(block.into());
            }
        }

        internal_poll
    }
}

type Aes128Cfb8Enc = cfb8::Encryptor<aes::Aes128>;

///NOTE: This makes lots of small writes; make sure there is a buffer somewhere down the line
pub struct StreamEncryptor<W: AsyncWrite + Unpin> {
    cipher: Aes128Cfb8Enc,
    write: W,
    last_unwritten_encrypted_byte: Option<u8>,
}

impl<W: AsyncWrite + Unpin> StreamEncryptor<W> {
    pub fn new(cipher: Aes128Cfb8Enc, stream: W) -> Self {
        debug_assert_eq!(Aes128Cfb8Enc::block_size(), 1);
        Self {
            cipher,
            write: stream,
            last_unwritten_encrypted_byte: None,
        }
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for StreamEncryptor<W> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        let ref_self = self.get_mut();
        let cipher = &mut ref_self.cipher;

        let mut total_written = 0;
        // Decrypt the raw data, note that our block size is 1 byte, so this is always safe
        for block in buf.chunks(Aes128Cfb8Enc::block_size()) {
            let mut out = [0u8];

            if let Some(out_to_use) = ref_self.last_unwritten_encrypted_byte {
                // This assumes that this `poll_write` is called on the same stream of bytes which I
                // think is a fair assumption, since thats an invariant for the TCP stream anyway.

                // This should never panic
                out[0] = out_to_use;
            } else {
                let out_block = GenericArray::from_mut_slice(&mut out);
                cipher.encrypt_block_b2b_mut(block.into(), out_block);
            }

            let write = Pin::new(&mut ref_self.write);
            match write.poll_write(cx, &out) {
                Poll::Pending => {
                    ref_self.last_unwritten_encrypted_byte = Some(out[0]);
                    if total_written == 0 {
                        //If we didn't write anything, return pending
                        return Poll::Pending;
                    } else {
                        // Otherwise, we actually did write something
                        return Poll::Ready(Ok(total_written));
                    }
                }
                Poll::Ready(result) => {
                    ref_self.last_unwritten_encrypted_byte = None;
                    match result {
                        Ok(written) => total_written += written,
                        Err(err) => return Poll::Ready(Err(err)),
                    }
                }
            }
        }

        Poll::Ready(Ok(total_written))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let ref_self = self.get_mut();
        let write = Pin::new(&mut ref_self.write);
        write.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let ref_self = self.get_mut();
        let write = Pin::new(&mut ref_self.write);
        write.poll_shutdown(cx)
    }
}

pub struct RawPacket {
    pub id: i32,
    pub payload: Bytes,
}

pub trait ClientPacket: Packet {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError>;
}

pub trait ServerPacket: Packet + Sized {
    fn read(read: impl Read) -> Result<Self, ReadingError>;
}

pub trait BClientPacket: Packet {
    fn write_packet(&self, writer: impl Write) -> Result<(), Error>;
}

pub trait BServerPacket: Packet + Sized {
    fn read(read: impl Read) -> Result<Self, Error>;
}

/// Errors that can occur during packet encoding.
#[derive(Error, Debug)]
pub enum PacketEncodeError {
    #[error("Packet exceeds maximum length: {0}")]
    TooLong(usize),
    #[error("Compression failed {0}")]
    CompressionFailed(String),
    #[error("Writing packet failed: {0}")]
    Message(String),
}

#[derive(Error, Debug)]
pub enum PacketDecodeError {
    #[error("failed to decode packet ID")]
    DecodeID,
    #[error("packet exceeds maximum length")]
    TooLong,
    #[error("packet length is out of bounds")]
    OutOfBounds,
    #[error("malformed packet length VarInt: {0}")]
    MalformedLength(String),
    #[error("failed to decompress packet: {0}")]
    FailedDecompression(String), // Updated to include error details
    #[error("packet is uncompressed but greater than the threshold")]
    NotCompressed,
    #[error("the connection has closed")]
    ConnectionClosed,
}

impl From<ReadingError> for PacketDecodeError {
    fn from(value: ReadingError) -> Self {
        Self::FailedDecompression(value.to_string())
    }
}

#[derive(Serialize)]
pub struct StatusResponse {
    /// The version on which the server is running. (Optional)
    pub version: Option<Version>,
    /// Information about currently connected players. (Optional)
    pub players: Option<Players>,
    /// The description displayed, also called MOTD (Message of the Day). (Optional)
    pub description: String,
    /// The icon displayed. (Optional)
    pub favicon: Option<String>,
    /// Whether players are forced to use secure chat.
    pub enforce_secure_chat: bool,
}
#[derive(Serialize)]
pub struct Version {
    /// The name of the version (e.g. 1.21.4)
    pub name: String,
    /// The protocol version (e.g. 767)
    pub protocol: u32,
}

#[derive(Serialize)]
pub struct Players {
    /// The maximum player count that the server allows.
    pub max: u32,
    /// The current online player count.
    pub online: u32,
    /// Information about currently connected players.
    /// Note: players can disable listing here.
    pub sample: Vec<Sample>,
}

#[derive(Serialize)]
pub struct Sample {
    /// The player's name.
    pub name: String,
    /// The player's UUID.
    pub id: String,
}

// basically game profile
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Property {
    pub name: String,
    // base 64
    pub value: String,
    // base 64
    pub signature: Option<String>,
}

#[derive(Serialize)]
pub struct KnownPack<'a> {
    pub namespace: &'a str,
    pub id: &'a str,
    pub version: &'a str,
}

#[derive(Serialize)]
pub enum NumberFormat {
    /// Show nothing.
    Blank,
    /// The styling to be used when formatting the score number.
    Styled(Style),
    /// The text to be used as a placeholder.
    Fixed(TextComponent),
}

/// For the first 8 values set means relative value while unset means absolute
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum PositionFlag {
    X,
    Y,
    Z,
    YRot,
    XRot,
    DeltaX,
    DeltaY,
    DeltaZ,
    RotateDelta,
}

impl PositionFlag {
    fn get_mask(&self) -> i32 {
        match self {
            PositionFlag::X => 1 << 0,
            PositionFlag::Y => 1 << 1,
            PositionFlag::Z => 1 << 2,
            PositionFlag::YRot => 1 << 3,
            PositionFlag::XRot => 1 << 4,
            PositionFlag::DeltaX => 1 << 5,
            PositionFlag::DeltaY => 1 << 6,
            PositionFlag::DeltaZ => 1 << 7,
            PositionFlag::RotateDelta => 1 << 8,
        }
    }

    pub fn get_bitfield(flags: &[PositionFlag]) -> i32 {
        flags.iter().fold(0, |acc, flag| acc | flag.get_mask())
    }
}

pub enum Label {
    BuiltIn(LinkType),
    TextComponent(Box<TextComponent>),
}

impl Serialize for Label {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Label::BuiltIn(link_type) => link_type.serialize(serializer),
            Label::TextComponent(component) => component.serialize(serializer),
        }
    }
}

#[derive(Serialize)]
pub struct Link<'a> {
    pub is_built_in: bool,
    pub label: Label,
    pub url: &'a String,
}

impl<'a> Link<'a> {
    pub fn new(label: Label, url: &'a String) -> Self {
        Self {
            is_built_in: match label {
                Label::BuiltIn(_) => true,
                Label::TextComponent(_) => false,
            },
            label,
            url,
        }
    }
}

#[derive(Clone, Copy)]
#[repr(i32)]
pub enum LinkType {
    BugReport = 0,
    CommunityGuidelines = 1,
    Support = 2,
    Status = 3,
    Feedback = 4,
    Community = 5,
    Website = 6,
    Forums = 7,
    News = 8,
    Announcements = 9,
}

impl Serialize for LinkType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        VarInt(*self as i32).serialize(serializer)
    }
}

#[cfg(test)]
mod test {
    use pumpkin_util::resource_location::ResourceLocation;
    use serde::{Deserialize, Serialize};

    use crate::{
        IdOr, SoundEvent,
        ser::{deserializer::Deserializer, serializer::Serializer},
    };

    #[test]
    fn test_serde_id_or_id() {
        let mut buf = Vec::new();

        let id = IdOr::<SoundEvent>::Id(0);
        id.serialize(&mut Serializer::new(&mut buf)).unwrap();

        let deser_id =
            IdOr::<SoundEvent>::deserialize(&mut Deserializer::new(buf.as_slice())).unwrap();

        assert!(id == deser_id);
    }

    #[test]
    fn test_serde_id_or_value() {
        let mut buf = Vec::new();
        let event = SoundEvent {
            sound_name: ResourceLocation::vanilla("test"),
            range: Some(1.0),
        };

        let id = IdOr::<SoundEvent>::Value(event);
        id.serialize(&mut Serializer::new(&mut buf)).unwrap();

        let deser_id =
            IdOr::<SoundEvent>::deserialize(&mut Deserializer::new(buf.as_slice())).unwrap();

        assert!(id == deser_id);
    }
}
