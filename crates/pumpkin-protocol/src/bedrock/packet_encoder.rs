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
        if packet_id > 0x3ff {
            return Err(Error::other("Bedrock packet ID exceeds 10 bits"));
        }
        let header =
            packet_id | ((sub_client_sender as u16) << 10) | ((sub_client_target as u16) << 12);
        let mut inner_buffer = Vec::new();
        Self::write_game_packet_data(header, packet_payload, &mut inner_buffer)?;
        self.write_batch(&inner_buffer, &mut writer)
    }

    pub fn write_game_packet_data(
        header: u16,
        packet_payload: &[u8],
        mut writer: impl Write,
    ) -> Result<(), Error> {
        if header > 0x3fff || packet_payload.len() > crate::MAX_PACKET_DATA_SIZE {
            return Err(Error::other("Invalid Bedrock packet header or size"));
        }
        let header_varint = VarUInt(u32::from(header));
        let total_content_length = (header_varint.written_size() + packet_payload.len()) as u32;

        writer
            .write_var_uint(&VarUInt(total_content_length))
            .map_err(|_| Error::other("Failed to write total content length"))?;
        writer
            .write_var_uint(&header_varint)
            .map_err(|_| Error::other("Failed to write header varint"))?;
        writer.write_all(packet_payload)
    }

    pub fn write_batch(&self, batch: &[u8], mut writer: impl Write) -> Result<(), Error> {
        if batch.len() > crate::MAX_PACKET_DATA_SIZE {
            return Err(Error::other("Bedrock batch is too large"));
        }
        writer
            .write_u8(BEDROCK_GAME_PACKET)
            .map_err(|e| Error::other(e.to_string()))?; // Bedrock Game Packet Header

        let mut data_to_write = Vec::new();

        if let Some((threshold, level)) = self.compression {
            if batch.len() < threshold {
                data_to_write.push(0xff);
                data_to_write.extend_from_slice(batch);
            } else {
                data_to_write.push(0x00);
                let mut encoder = DeflateEncoder::new(Vec::new(), Compression::new(level));
                encoder.write_all(batch)?;
                data_to_write.extend_from_slice(&encoder.finish()?);
            }
        } else {
            data_to_write.extend_from_slice(batch);
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

    #[tokio::test]
    async fn raw_batch_reframing_preserves_headers_and_modified_packets()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut batch = Vec::new();
        BedrockBatchEncoder::write_game_packet_data(0x3401, &[1, 2], &mut batch)?;
        BedrockBatchEncoder::write_game_packet_data(0x1880, &[3, 4], &mut batch)?;
        let mut decoder = BedrockBatchDecoder::new();
        let mut original = Cursor::new(batch);
        let (header, mut packet) = decoder.get_game_packet_with_header(&mut original)?;
        packet.id = 255;
        packet.payload = bytes::Bytes::from_static(&[5, 6, 7]);
        let _cancelled = decoder.get_game_packet_with_header(&mut original)?;
        let mut edited = Vec::new();
        BedrockBatchEncoder::write_game_packet_data(
            (header & !0x3ff) | packet.id as u16,
            &packet.payload,
            &mut edited,
        )?;

        for (threshold, method) in [(0, 0x00), (256, 0xff)] {
            let mut encoder = BedrockBatchEncoder::new();
            encoder.set_compression((threshold, 4));
            let mut frame = Vec::new();
            encoder.write_batch(&edited, &mut frame)?;
            assert_eq!(frame[1], method);
            decoder.set_compression(threshold);
            let mut result = Cursor::new(decoder.get_packet_payload(frame).await?);
            let (header, packet) = decoder.get_game_packet_with_header(&mut result)?;
            assert_eq!(header, 0x34ff);
            assert_eq!(packet.id, 255);
            assert_eq!(packet.payload.as_ref(), &[5, 6, 7]);
            assert_eq!(result.position() as usize, result.get_ref().len());
        }
        let mut noncanonical = Cursor::new(vec![3, 0x80, 0, 42]);
        assert_eq!(
            decoder.get_game_packet(&mut noncanonical)?.payload.as_ref(),
            &[42]
        );
        assert!(BedrockBatchEncoder::write_game_packet_data(0x4000, &[], Vec::new()).is_err());
        Ok(())
    }
}
