use std::io::{Error, Write};

use flate2::{Compression, write::DeflateEncoder};

use crate::{
    CompressionLevel, CompressionThreshold,
    bedrock::{BEDROCK_GAME_PACKET, SubClient},
    codec::var_uint::VarUInt,
    ser::NetworkWriteExt,
};

/// Encoder: Server -> Client
/// Supports Zlib compression.
pub struct BedrockBatchEncoder {
    // compression and compression threshold
    compression: Option<(CompressionThreshold, CompressionLevel)>,
}

impl Default for BedrockBatchEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl BedrockBatchEncoder {
    #[must_use]
    pub const fn new() -> Self {
        Self { compression: None }
    }

    pub const fn set_compression(
        &mut self,
        compression_info: (CompressionThreshold, CompressionLevel),
    ) {
        self.compression = Some(compression_info);
    }

    pub fn write_game_packet(
        &self,
        packet_id: u16,
        sub_client_sender: SubClient,
        sub_client_target: SubClient,
        packet_payload: &[u8],
        mut writer: impl Write,
    ) -> Result<(), Error> {
        let mut inner_buffer = Vec::new();

        // Gamepacket ID Header (14 bits)
        let header_value: u32 = u32::from(packet_id)
            | ((sub_client_sender as u32) << 10)
            | ((sub_client_target as u32) << 12);
        let fourteen_bit_header = header_value & 0x3FFF;

        let header_varint = VarUInt(fourteen_bit_header);
        let total_content_length = (header_varint.written_size() + packet_payload.len()) as u32;

        inner_buffer
            .write_var_uint(&VarUInt(total_content_length))
            .map_err(|_| Error::other("Failed to write total content length"))?;
        inner_buffer
            .write_var_uint(&header_varint)
            .map_err(|_| Error::other("Failed to write header varint"))?;
        inner_buffer.write_all(packet_payload)?;

        // Handle Outer Container
        writer
            .write_u8(BEDROCK_GAME_PACKET)
            .map_err(|e| Error::other(e.to_string()))?; // Bedrock Game Packet Header

        let mut data_to_write = Vec::new();

        if let Some((_threshold, level)) = self.compression {
            // Write Compression Method (0x00 for Zlib)
            data_to_write.push(0x00);

            let mut encoder = DeflateEncoder::new(Vec::new(), Compression::new(level));
            encoder.write_all(&inner_buffer)?;
            let compressed_data = encoder.finish()?;

            data_to_write.extend_from_slice(&compressed_data);
        } else {
            data_to_write.extend_from_slice(&inner_buffer);
        }

        writer.write_all(&data_to_write)?;

        Ok(())
    }

    pub fn write_packet<P: crate::BClientPacket + ?Sized>(
        &self,
        packet: &P,
        writer: impl Write,
    ) -> Result<(), Error> {
        let mut packet_payload = Vec::new();
        packet.write_packet(&mut packet_payload)?;
        self.write_game_packet(
            P::PACKET_ID as u16,
            SubClient::Main,
            SubClient::Main,
            &packet_payload,
            writer,
        )
    }

    pub fn serialize_packet<P: crate::BClientPacket + ?Sized>(
        &self,
        packet: &P,
    ) -> Result<bytes::Bytes, Error> {
        let mut buf = Vec::new();
        self.write_packet(packet, &mut buf)?;
        Ok(buf.into())
    }
}

pub fn write_packet<P: crate::BClientPacket + ?Sized>(
    packet: &P,
    writer: impl Write,
) -> Result<(), Error> {
    BedrockBatchEncoder::new().write_packet(packet, writer)
}

pub fn serialize_packet<P: crate::BClientPacket + ?Sized>(
    packet: &P,
) -> Result<bytes::Bytes, Error> {
    BedrockBatchEncoder::new().serialize_packet(packet)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bedrock::packet_decoder::BedrockBatchDecoder;
    use std::io::Cursor;

    #[tokio::test]
    async fn bedrock_compression_cycle() -> Result<(), Box<dyn std::error::Error>> {
        let mut encoder = BedrockBatchEncoder::new();
        encoder.set_compression((256, 6));

        let packet_id = 1;
        let payload = b"Hello Bedrock Compression!";
        let mut encoded_buf = Vec::new();

        encoder.write_game_packet(
            packet_id,
            SubClient::Main,
            SubClient::Main,
            payload,
            &mut encoded_buf,
        )?;

        let mut decoder = BedrockBatchDecoder::new();
        decoder.set_compression(256);

        let decompressed_payload = decoder.get_packet_payload(encoded_buf).await?;
        let mut cursor = Cursor::new(decompressed_payload);
        let raw_packet = decoder.get_game_packet(&mut cursor)?;

        assert_eq!(raw_packet.id, packet_id as i32);
        assert_eq!(raw_packet.payload.as_ref(), payload);
        Ok(())
    }
}
