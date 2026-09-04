#![deny(clippy::unwrap_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::{
    io::{Error, Read, Write},
    pin::Pin,
    task::{Context, Poll},
};

use aes::cipher::BlockSizeUser;
use bytes::Bytes;
use codec::var_int::VarInt;
use pumpkin_util::{
    resource_location::ResourceLocation,
    text::{TextComponent, style::Style},
    version::JavaMinecraftVersion,
};
use ser::{ReadingError, WritingError};

use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub use crate::packet::{MultiVersionJavaPacket, Packet};

pub mod bedrock;
pub mod codec;
pub mod java;
pub mod packet;
#[cfg(feature = "query")]
pub mod query;
pub mod rcon;
pub mod ser;
pub mod serial;

pub const MAX_PACKET_SIZE: u64 = 2_097_152;
pub const MAX_PACKET_DATA_SIZE: usize = 8_388_608;

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

#[derive(PartialEq, Eq, Clone)]
pub enum IdOr<T> {
    Id(u16),
    Value(T),
}

impl<T> IdOr<T> {
    pub fn read<R: ser::NetworkReadExt>(
        read: &mut R,
        read_value: impl FnOnce(&mut R) -> Result<T, ser::ReadingError>,
    ) -> Result<Self, ser::ReadingError> {
        let id = read.get_var_int()?.0;
        if id == 0 {
            Ok(Self::Value(read_value(read)?))
        } else {
            Ok(Self::Id((id - 1) as u16))
        }
    }

    pub fn write<W: ser::NetworkWriteExt>(
        &self,
        write: &mut W,
        write_value: impl FnOnce(&mut W, &T) -> Result<(), ser::WritingError>,
    ) -> Result<(), ser::WritingError> {
        match self {
            Self::Id(id) => write.write_var_int(&((*id as i32) + 1).into()),
            Self::Value(value) => {
                write.write_var_int(&0.into())?;
                write_value(write, value)
            }
        }
    }
}

#[derive(Clone, PartialEq)]
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
    pub const fn new(cipher: Aes128Cfb8Dec, stream: R) -> Self {
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

        if matches!(internal_poll, Poll::Ready(Ok(()))) {
            // Decrypt the raw data in-place, note that our block size is 1 byte, so this is always safe
            for block in buf.filled_mut()[original_fill..].chunks_mut(Aes128Cfb8Dec::block_size()) {
                cipher.decrypt(block);
            }
        }

        internal_poll
    }
}

type Aes128Cfb8Enc = cfb8::Encryptor<aes::Aes128>;

const STREAM_ENCRYPTION_BUFFER_SIZE: usize = 16 * 1024;

/// Coalesces CFB8 ciphertext so encrypted bytes do not become one-byte
/// downstream writes.
///
/// Ciphertext stays owned by this adapter until the wrapped writer accepts it,
/// which also makes partial writes and `Poll::Pending` safe.
pub struct StreamEncryptor<W: AsyncWrite + Unpin> {
    cipher: Aes128Cfb8Enc,
    write: W,
    pending_ciphertext: Vec<u8>,
    pending_offset: usize,
}

impl<W: AsyncWrite + Unpin> StreamEncryptor<W> {
    pub fn new(cipher: Aes128Cfb8Enc, stream: W) -> Self {
        Self {
            cipher,
            write: stream,
            pending_ciphertext: Vec::with_capacity(STREAM_ENCRYPTION_BUFFER_SIZE),
            pending_offset: 0,
        }
    }

    fn poll_drain_pending(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        while self.pending_offset < self.pending_ciphertext.len() {
            match Pin::new(&mut self.write)
                .poll_write(cx, &self.pending_ciphertext[self.pending_offset..])
            {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Ok(0)) => {
                    return Poll::Ready(Err(Error::new(
                        std::io::ErrorKind::WriteZero,
                        "failed to write buffered encrypted data",
                    )));
                }
                Poll::Ready(Ok(written)) => self.pending_offset += written,
                Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
            }
        }

        self.pending_ciphertext.clear();
        self.pending_offset = 0;
        Poll::Ready(Ok(()))
    }

    fn buffer_plaintext(&mut self, plaintext: &[u8]) -> Result<(), Error> {
        let start = self.pending_ciphertext.len();
        self.pending_ciphertext.resize(start + plaintext.len(), 0);
        if self
            .cipher
            .encrypt_b2b(plaintext, &mut self.pending_ciphertext[start..])
            .is_err()
        {
            self.pending_ciphertext.truncate(start);
            return Err(Error::other("Encryption input/output lengths do not match"));
        }
        Ok(())
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for StreamEncryptor<W> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        let ref_self = self.get_mut();

        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }

        if ref_self.pending_offset > 0
            || ref_self.pending_ciphertext.len() == STREAM_ENCRYPTION_BUFFER_SIZE
        {
            match ref_self.poll_drain_pending(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Ok(())) => {}
                Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
            }
        }

        let available = STREAM_ENCRYPTION_BUFFER_SIZE - ref_self.pending_ciphertext.len();
        let plaintext_len = buf.len().min(available);
        ref_self.buffer_plaintext(&buf[..plaintext_len])?;

        // The corresponding ciphertext is now owned by this adapter. It may be
        // drained later without relying on the caller to repeat the same input.
        Poll::Ready(Ok(plaintext_len))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let ref_self = self.get_mut();
        match ref_self.poll_drain_pending(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(())) => Pin::new(&mut ref_self.write).poll_flush(cx),
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let ref_self = self.get_mut();
        match ref_self.poll_drain_pending(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(())) => Pin::new(&mut ref_self.write).poll_shutdown(cx),
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
        }
    }
}

pub struct RawPacket {
    pub id: i32,
    pub payload: Bytes,
}

pub trait ClientPacket: MultiVersionJavaPacket {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError>;

    fn write_packet(
        &self,
        version: &JavaMinecraftVersion,
        write: impl Write,
    ) -> Result<(), WritingError> {
        crate::java::packet_encoder::write_packet(self, version, write)
    }

    fn serialize_packet(&self, version: &JavaMinecraftVersion) -> Result<Bytes, WritingError> {
        crate::java::packet_encoder::serialize_packet(self, version)
    }
}

pub trait ServerPacket<'a>: MultiVersionJavaPacket + Sized {
    fn read(read: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError>;
}

pub trait BClientPacket: Packet {
    fn write_packet(&self, writer: impl Write) -> Result<(), Error>;

    fn serialize_packet(&self) -> Result<Bytes, Error> {
        crate::bedrock::packet_encoder::serialize_packet(self)
    }
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
    #[error("{0}")]
    Message(String),
}

impl From<ReadingError> for PacketDecodeError {
    fn from(value: ReadingError) -> Self {
        Self::FailedDecompression(value.to_string())
    }
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    /// The version on which the server is running. (Optional)
    pub version: Option<Version>,
    /// Information about currently connected players. (Optional)
    pub players: Option<Players>,
    /// The description displayed, also called MOTD (Message of the Day). (Optional)
    pub description: TextComponent,
    /// The icon displayed. (Optional)
    pub favicon: Option<String>,
    /// Whether players are forced to use secure chat.
    pub enforce_secure_chat: bool,
}
#[derive(Clone, serde::Serialize)]
pub struct Version {
    /// The name of the version (e.g. 1.21.4)
    pub name: String,
    /// The protocol version (e.g. 767)
    pub protocol: u32,
}

#[derive(Clone, serde::Serialize)]
pub struct Players {
    /// The maximum player count that the server allows.
    pub max: u32,
    /// The current online player count.
    pub online: u32,
    /// Information about currently connected players.
    /// Note: players can disable listing here.
    pub sample: Vec<Sample>,
}

#[derive(Clone, serde::Serialize)]
pub struct Sample {
    /// The player's name.
    pub name: String,
    /// The player's UUID.
    pub id: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Property {
    pub name: Box<str>,
    pub value: Box<str>,
    pub signature: Option<Box<str>>,
}

impl Property {
    pub fn read(read: &mut impl ser::NetworkReadExt) -> Result<Self, ser::ReadingError> {
        Ok(Self {
            name: read.get_str()?,
            value: read.get_str()?,
            signature: read.get_option(ser::NetworkReadExt::get_str)?,
        })
    }

    pub fn write(&self, write: &mut impl ser::NetworkWriteExt) -> Result<(), ser::WritingError> {
        write.write_string(&self.name)?;
        write.write_string(&self.value)?;
        write.write_option(&self.signature, |w, v| w.write_string(v))?;
        Ok(())
    }
}

pub struct KnownPack<'a> {
    pub namespace: &'a str,
    pub id: &'a str,
    pub version: &'a str,
}

impl KnownPack<'_> {
    pub fn write(&self, write: &mut impl ser::NetworkWriteExt) -> Result<(), ser::WritingError> {
        write.write_string(self.namespace)?;
        write.write_string(self.id)?;
        write.write_string(self.version)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NumberFormat {
    /// Show nothing.
    Blank,
    /// The styling to be used when formatting the score number.
    Styled(Style),
    /// The text to be used as a placeholder.
    Fixed(TextComponent),
}

impl NumberFormat {
    pub fn write(&self, write: &mut impl ser::NetworkWriteExt) -> Result<(), ser::WritingError> {
        match self {
            Self::Blank => write.write_var_int(&0.into()),
            Self::Styled(_style) => {
                write.write_var_int(&1.into())?;
                // TODO: Style write
                Ok(())
            }
            Self::Fixed(text) => {
                write.write_var_int(&2.into())?;
                write.write_slice(&text.encode())?;
                Ok(())
            }
        }
    }
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
    const fn get_mask(&self) -> i32 {
        match self {
            Self::X => 1 << 0,
            Self::Y => 1 << 1,
            Self::Z => 1 << 2,
            Self::YRot => 1 << 3,
            Self::XRot => 1 << 4,
            Self::DeltaX => 1 << 5,
            Self::DeltaY => 1 << 6,
            Self::DeltaZ => 1 << 7,
            Self::RotateDelta => 1 << 8,
        }
    }

    #[must_use]
    pub fn get_bitfield(flags: &[Self]) -> i32 {
        flags.iter().fold(0, |acc, flag| acc | flag.get_mask())
    }

    #[must_use]
    pub fn from_bitfield(bits: i32) -> Vec<Self> {
        let all = [
            Self::X,
            Self::Y,
            Self::Z,
            Self::YRot,
            Self::XRot,
            Self::DeltaX,
            Self::DeltaY,
            Self::DeltaZ,
            Self::RotateDelta,
        ];
        all.into_iter()
            .filter(|flag| (bits & flag.get_mask()) != 0)
            .collect()
    }
}

#[derive(Clone, Debug)]
pub enum Label {
    BuiltIn(LinkType),
    TextComponent(Box<TextComponent>),
}

pub struct Link<'a> {
    pub is_built_in: bool,
    pub label: Label,
    pub url: &'a String,
}

impl<'a> Link<'a> {
    #[must_use]
    pub const fn new(label: Label, url: &'a String) -> Self {
        Self {
            is_built_in: match label {
                Label::BuiltIn(_) => true,
                Label::TextComponent(_) => false,
            },
            label,
            url,
        }
    }

    pub fn write(&self, write: &mut impl ser::NetworkWriteExt) -> Result<(), ser::WritingError> {
        match &self.label {
            Label::BuiltIn(link_type) => {
                write.write_bool(true)?;
                write.write_var_int(&(*link_type as i32).into())?;
            }
            Label::TextComponent(text_component) => {
                write.write_bool(false)?;
                write.write_slice(&text_component.encode())?;
            }
        }
        write.write_string(self.url)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
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

#[cfg(test)]
mod stream_encryptor_tests {
    use super::*;
    use aes::cipher::KeyIvInit;
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };
    use tokio::io::AsyncWriteExt;

    struct BackpressureWriter {
        bytes: Arc<Mutex<Vec<u8>>>,
        writes: Arc<AtomicUsize>,
        max_write: usize,
        return_pending: bool,
    }

    impl AsyncWrite for BackpressureWriter {
        fn poll_write(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            if self.return_pending {
                self.return_pending = false;
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            self.return_pending = true;

            let len = buf.len().min(self.max_write);
            self.bytes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .extend_from_slice(&buf[..len]);
            self.writes.fetch_add(1, Ordering::Relaxed);
            Poll::Ready(Ok(len))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn stream_encryptor_preserves_ciphertext_through_partial_pending_writes() {
        let key = [0x2a; 16];
        let plaintext: Vec<u8> = (0..50_000).map(|index| (index % 251) as u8).collect();
        let mut expected = vec![0; plaintext.len()];
        cfb8::Encryptor::<aes::Aes128>::new_from_slices(&key, &key)
            .unwrap()
            .encrypt_b2b(&plaintext, &mut expected)
            .unwrap();

        let bytes = Arc::new(Mutex::new(Vec::new()));
        let writes = Arc::new(AtomicUsize::new(0));
        let sink = BackpressureWriter {
            bytes: bytes.clone(),
            writes: writes.clone(),
            max_write: 137,
            return_pending: true,
        };
        let cipher = cfb8::Encryptor::<aes::Aes128>::new_from_slices(&key, &key).unwrap();
        let mut writer = StreamEncryptor::new(cipher, sink);

        for chunk in plaintext.chunks(113) {
            writer.write_all(chunk).await.unwrap();
        }
        writer.shutdown().await.unwrap();

        assert_eq!(
            *bytes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            expected
        );
        assert!(writes.load(Ordering::Relaxed) < plaintext.len());
    }

    #[tokio::test]
    async fn stream_encryptor_coalesces_small_writes_until_flush() {
        let key = [0x4c; 16];
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let writes = Arc::new(AtomicUsize::new(0));
        let sink = BackpressureWriter {
            bytes: bytes.clone(),
            writes: writes.clone(),
            max_write: STREAM_ENCRYPTION_BUFFER_SIZE,
            return_pending: false,
        };
        let cipher = cfb8::Encryptor::<aes::Aes128>::new_from_slices(&key, &key).unwrap();
        let mut writer = StreamEncryptor::new(cipher, sink);

        for _ in 0..16 {
            writer.write_all(&[0x5a; 64]).await.unwrap();
        }
        assert_eq!(writes.load(Ordering::Relaxed), 0);

        writer.flush().await.unwrap();
        assert_eq!(writes.load(Ordering::Relaxed), 1);
        assert_eq!(
            bytes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .len(),
            1024
        );
    }
}
