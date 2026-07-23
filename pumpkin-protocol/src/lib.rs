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

pub struct StreamEncryptor<W: AsyncWrite + Unpin> {
    cipher: Aes128Cfb8Enc,
    write: W,
    pending_ciphertext: Vec<u8>,
    pending_offset: usize,
}

impl<W: AsyncWrite + Unpin> StreamEncryptor<W> {
    pub fn new(cipher: Aes128Cfb8Enc, stream: W) -> Self {
        debug_assert_eq!(Aes128Cfb8Enc::block_size(), 1);
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
}

impl<W: AsyncWrite + Unpin> AsyncWrite for StreamEncryptor<W> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        let ref_self = self.get_mut();

        match ref_self.poll_drain_pending(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Ok(())) => {}
            Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
        }

        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }

        let plaintext_len = buf.len().min(STREAM_ENCRYPTION_BUFFER_SIZE);
        ref_self.pending_ciphertext.resize(plaintext_len, 0);
        ref_self
            .cipher
            .encrypt_b2b(&buf[..plaintext_len], &mut ref_self.pending_ciphertext)
            .map_err(|_| Error::other("Encryption failed"))?;

        // The ciphertext is now owned by this adapter, so the caller can reuse the plaintext
        // buffer even if the underlying writer accepts only part of the encrypted output.
        match ref_self.poll_drain_pending(cx) {
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Pending | Poll::Ready(Ok(())) => Poll::Ready(Ok(plaintext_len)),
        }
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
}

pub trait ServerPacket: MultiVersionJavaPacket + Sized {
    fn read(read: impl Read, version: &JavaMinecraftVersion) -> Result<Self, ReadingError>;
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
    pub description: String,
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

#[derive(Clone)]
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
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use aes::cipher::KeyIvInit;
    use std::sync::{Arc, Mutex};
    use tokio::io::AsyncWriteExt;

    #[derive(Default)]
    struct WriteRecord {
        bytes: Vec<u8>,
        poll_write_calls: usize,
        flush_calls: usize,
        shutdown_calls: usize,
    }

    struct TestWriter {
        record: Arc<Mutex<WriteRecord>>,
        max_write: usize,
        return_pending: bool,
        pending_next: bool,
        return_zero: bool,
    }

    impl TestWriter {
        fn new(
            record: Arc<Mutex<WriteRecord>>,
            max_write: usize,
            return_pending: bool,
            pending_first: bool,
        ) -> Self {
            Self {
                record,
                max_write,
                return_pending,
                pending_next: pending_first,
                return_zero: false,
            }
        }
    }

    impl AsyncWrite for TestWriter {
        fn poll_write(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize, Error>> {
            self.record.lock().unwrap().poll_write_calls += 1;

            if self.pending_next {
                self.pending_next = false;
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }

            if self.return_zero {
                return Poll::Ready(Ok(0));
            }

            let written = buf.len().min(self.max_write);
            self.record
                .lock()
                .unwrap()
                .bytes
                .extend_from_slice(&buf[..written]);
            self.pending_next = self.return_pending;
            Poll::Ready(Ok(written))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
            self.record.lock().unwrap().flush_calls += 1;
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
            self.record.lock().unwrap().shutdown_calls += 1;
            Poll::Ready(Ok(()))
        }
    }

    fn encrypt_reference(mut plaintext: Vec<u8>, key: &[u8; 16]) -> Vec<u8> {
        let mut cipher = Aes128Cfb8Enc::new_from_slices(key, key).unwrap();
        cipher.encrypt(&mut plaintext);
        plaintext
    }

    #[tokio::test]
    async fn stream_encryptor_matches_reference_across_fragmented_writes() {
        let key = [0x7Bu8; 16];
        let plaintext: Vec<u8> = (0..(STREAM_ENCRYPTION_BUFFER_SIZE * 2 + 137))
            .map(|index| (index % 251) as u8)
            .collect();
        let expected = encrypt_reference(plaintext.clone(), &key);
        let record = Arc::new(Mutex::new(WriteRecord::default()));
        let writer = TestWriter::new(record.clone(), 7, true, true);
        let cipher = Aes128Cfb8Enc::new_from_slices(&key, &key).unwrap();
        let mut encryptor = StreamEncryptor::new(cipher, writer);

        let mut offset = 0;
        for chunk_len in [1, 31, 4096, 17, 8192, 257, STREAM_ENCRYPTION_BUFFER_SIZE] {
            if offset == plaintext.len() {
                break;
            }
            let end = (offset + chunk_len).min(plaintext.len());
            encryptor.write_all(&plaintext[offset..end]).await.unwrap();
            offset = end;
        }
        encryptor.write_all(&plaintext[offset..]).await.unwrap();
        encryptor.shutdown().await.unwrap();

        let record = record.lock().unwrap();
        assert_eq!(record.bytes, expected);
        assert!(record.poll_write_calls > plaintext.len() / 7);
        assert_eq!(record.shutdown_calls, 1);
    }

    #[tokio::test]
    async fn stream_encryptor_flushes_ciphertext_buffered_after_pending() {
        let key = [0x11u8; 16];
        let plaintext = b"buffered ciphertext".to_vec();
        let expected = encrypt_reference(plaintext.clone(), &key);
        let record = Arc::new(Mutex::new(WriteRecord::default()));
        let writer = TestWriter::new(record.clone(), plaintext.len(), false, true);
        let cipher = Aes128Cfb8Enc::new_from_slices(&key, &key).unwrap();
        let mut encryptor = StreamEncryptor::new(cipher, writer);

        assert_eq!(encryptor.write(&plaintext).await.unwrap(), plaintext.len());
        assert!(record.lock().unwrap().bytes.is_empty());

        encryptor.flush().await.unwrap();

        let record = record.lock().unwrap();
        assert_eq!(record.bytes, expected);
        assert_eq!(record.flush_calls, 1);
    }

    #[tokio::test]
    async fn stream_encryptor_shutdown_drains_ciphertext_buffered_after_pending() {
        let key = [0x19u8; 16];
        let plaintext = b"shutdown drains ciphertext".to_vec();
        let expected = encrypt_reference(plaintext.clone(), &key);
        let record = Arc::new(Mutex::new(WriteRecord::default()));
        let writer = TestWriter::new(record.clone(), plaintext.len(), false, true);
        let cipher = Aes128Cfb8Enc::new_from_slices(&key, &key).unwrap();
        let mut encryptor = StreamEncryptor::new(cipher, writer);

        assert_eq!(encryptor.write(&plaintext).await.unwrap(), plaintext.len());
        assert!(record.lock().unwrap().bytes.is_empty());

        encryptor.shutdown().await.unwrap();

        let record = record.lock().unwrap();
        assert_eq!(record.bytes, expected);
        assert_eq!(record.shutdown_calls, 1);
    }

    #[tokio::test]
    async fn stream_encryptor_batches_downstream_writes() {
        let key = [0x23u8; 16];
        let plaintext = vec![0xAB; STREAM_ENCRYPTION_BUFFER_SIZE * 2 + 1];
        let record = Arc::new(Mutex::new(WriteRecord::default()));
        let writer = TestWriter::new(record.clone(), usize::MAX, false, false);
        let cipher = Aes128Cfb8Enc::new_from_slices(&key, &key).unwrap();
        let mut encryptor = StreamEncryptor::new(cipher, writer);

        encryptor.write_all(&plaintext).await.unwrap();
        encryptor.shutdown().await.unwrap();

        let record = record.lock().unwrap();
        assert_eq!(record.poll_write_calls, 3);
        assert_eq!(record.bytes.len(), plaintext.len());
    }

    #[tokio::test]
    async fn stream_encryptor_reports_write_zero() {
        let key = [0x42u8; 16];
        let record = Arc::new(Mutex::new(WriteRecord::default()));
        let mut writer = TestWriter::new(record, 1, false, false);
        writer.return_zero = true;
        let cipher = Aes128Cfb8Enc::new_from_slices(&key, &key).unwrap();
        let mut encryptor = StreamEncryptor::new(cipher, writer);

        let error = encryptor.write(b"cannot write").await.unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::WriteZero);
    }
}
