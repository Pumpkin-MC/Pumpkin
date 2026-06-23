//! Binary parser for the `.slime` container format (AdvancedSlimePaper v13).
//!
//! Layout mirrors AdvancedSlimePaper `dev/26.2` (`SlimeSerializer` /
//! `v13SlimeWorldDeSerializer`): big-endian integers and two zstd-compressed
//! payloads (the chunks, then the world "extra"), each framed as
//! `[i32 compressed_len][i32 raw_len][compressed bytes]`. NBT segments are
//! framed as `[i32 length][length bytes]`, where a length of zero means absent.

use std::io::Read;

use bytes::{Buf, Bytes};
use ruzstd::decoding::StreamingDecoder;

use super::{
    FLAG_BLOCK_TICKS, FLAG_FLUID_TICKS, FLAG_POI, KNOWN_FLAGS, LIGHT_ARRAY_SIZE, SLIME_MAGIC,
    SLIME_VERSION, SlimeError,
    model::{SlimeChunk, SlimeSection, SlimeWorld},
};

/// Caps pre-allocation only; it never limits how much data is actually read
/// (reading past the end fails cleanly with [`SlimeError::Truncated`]).
const PREALLOC_CAP: usize = 4096;

fn read_u8(buf: &mut Bytes) -> Result<u8, SlimeError> {
    if buf.remaining() < 1 {
        return Err(SlimeError::Truncated);
    }
    Ok(buf.get_u8())
}

fn read_i32(buf: &mut Bytes) -> Result<i32, SlimeError> {
    if buf.remaining() < size_of::<i32>() {
        return Err(SlimeError::Truncated);
    }
    Ok(buf.get_i32())
}

fn take(buf: &mut Bytes, n: usize) -> Result<Bytes, SlimeError> {
    if buf.remaining() < n {
        return Err(SlimeError::Truncated);
    }
    Ok(buf.split_to(n))
}

/// Read an `[i32 length][length bytes]` NBT segment; a zero length yields empty bytes.
fn read_segment(buf: &mut Bytes) -> Result<Bytes, SlimeError> {
    let len = read_i32(buf)?;
    let len = usize::try_from(len).map_err(|_| SlimeError::Corrupt("negative segment length"))?;
    take(buf, len)
}

/// Decompress a `[i32 compressed_len][i32 raw_len][zstd]` block.
fn decompress_block(buf: &mut Bytes) -> Result<Bytes, SlimeError> {
    let compressed_len = read_i32(buf)?;
    let _raw_len = read_i32(buf)?; // legacy; the zstd frame is self-describing.
    let compressed_len = usize::try_from(compressed_len)
        .map_err(|_| SlimeError::Corrupt("negative compressed length"))?;
    let compressed = take(buf, compressed_len)?;

    let mut decoder =
        StreamingDecoder::new(compressed.as_ref()).map_err(|_| SlimeError::Decompress)?;
    let mut out = Vec::new();
    decoder.read_to_end(&mut out)?;
    Ok(Bytes::from(out))
}

/// Parse a `.slime` file (format version 13) into a [`SlimeWorld`].
///
/// # Errors
///
/// Returns [`SlimeError`] if the magic or version are wrong, the data is
/// truncated or corrupt, or zstd decompression fails.
pub fn read_slime_world(mut buf: Bytes) -> Result<SlimeWorld, SlimeError> {
    let magic = take(&mut buf, SLIME_MAGIC.len())?;
    if magic.as_ref() != SLIME_MAGIC {
        return Err(SlimeError::BadMagic);
    }
    let version = read_u8(&mut buf)?;
    if version != SLIME_VERSION {
        return Err(SlimeError::UnsupportedVersion(version));
    }

    let world_version = read_i32(&mut buf)?;
    let flags = read_u8(&mut buf)?;

    let mut chunk_stream = decompress_block(&mut buf)?;
    let chunks = read_chunks(&mut chunk_stream, flags)?;

    // The "extra" block is always written, but is non-essential for loading.
    let extra = decompress_block(&mut buf).unwrap_or_default();

    Ok(SlimeWorld {
        world_version,
        flags,
        chunks,
        extra,
    })
}

fn read_chunks(buf: &mut Bytes, flags: u8) -> Result<Vec<SlimeChunk>, SlimeError> {
    let count = read_i32(buf)?;
    let count = usize::try_from(count).map_err(|_| SlimeError::Corrupt("negative chunk count"))?;

    let mut chunks = Vec::with_capacity(count.min(PREALLOC_CAP));
    for _ in 0..count {
        chunks.push(read_chunk(buf, flags)?);
    }
    Ok(chunks)
}

fn read_chunk(buf: &mut Bytes, flags: u8) -> Result<SlimeChunk, SlimeError> {
    let x = read_i32(buf)?;
    let z = read_i32(buf)?;

    let section_count = read_i32(buf)?;
    let section_count = usize::try_from(section_count)
        .map_err(|_| SlimeError::Corrupt("negative section count"))?;
    let mut sections = Vec::with_capacity(section_count.min(PREALLOC_CAP));
    for _ in 0..section_count {
        sections.push(read_section(buf)?);
    }

    let height_maps = read_segment(buf)?;

    let poi = if flags & FLAG_POI != 0 {
        read_segment(buf)?
    } else {
        Bytes::new()
    };
    let block_ticks = if flags & FLAG_BLOCK_TICKS != 0 {
        read_segment(buf)?
    } else {
        Bytes::new()
    };
    let fluid_ticks = if flags & FLAG_FLUID_TICKS != 0 {
        read_segment(buf)?
    } else {
        Bytes::new()
    };

    // Skip any unknown (future) additional-data segments to stay in sync.
    for _ in 0..(flags & !KNOWN_FLAGS).count_ones() {
        let _unknown = read_segment(buf)?;
    }

    let block_entities = read_segment(buf)?;
    let entities = read_segment(buf)?;
    let _chunk_extra = read_segment(buf)?; // per-chunk persistent data, unused.

    Ok(SlimeChunk {
        x,
        z,
        sections,
        height_maps,
        block_entities,
        entities,
        block_ticks,
        fluid_ticks,
        poi,
    })
}

fn read_section(buf: &mut Bytes) -> Result<SlimeSection, SlimeError> {
    let section_flags = read_u8(buf)?;
    let block_light = if section_flags & 1 != 0 {
        Some(take(buf, LIGHT_ARRAY_SIZE)?)
    } else {
        None
    };
    let sky_light = if section_flags & (1 << 1) != 0 {
        Some(take(buf, LIGHT_ARRAY_SIZE)?)
    } else {
        None
    };
    let block_states = read_segment(buf)?;
    let biomes = read_segment(buf)?;
    Ok(SlimeSection {
        block_light,
        sky_light,
        block_states,
        biomes,
    })
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};
    use ruzstd::encoding::{CompressionLevel, compress_to_vec};

    use super::{SLIME_MAGIC, SLIME_VERSION, SlimeError, read_slime_world};

    fn be_i32(v: i32) -> [u8; 4] {
        v.to_be_bytes()
    }

    /// Frame a payload the way `SlimeSerializer.writeCompressed` does.
    fn compress_block(raw: &[u8]) -> Vec<u8> {
        let compressed = compress_to_vec(raw, CompressionLevel::Fastest);
        let mut framed = Vec::new();
        framed.extend_from_slice(&be_i32(i32::try_from(compressed.len()).unwrap()));
        framed.extend_from_slice(&be_i32(i32::try_from(raw.len()).unwrap()));
        framed.extend_from_slice(&compressed);
        framed
    }

    #[test]
    fn parses_minimal_world() {
        // One chunk at (2, 3) with a single empty section, no light, no flags.
        let mut chunk_stream = BytesMut::new();
        chunk_stream.put_i32(1); // chunk count
        chunk_stream.put_i32(2); // x
        chunk_stream.put_i32(3); // z
        chunk_stream.put_i32(1); // section count
        chunk_stream.put_u8(0); // section flags (no light)
        chunk_stream.put_i32(0); // block_states length
        chunk_stream.put_i32(0); // biomes length
        chunk_stream.put_i32(0); // heightmaps length
        chunk_stream.put_i32(0); // tileEntities length
        chunk_stream.put_i32(0); // entities length
        chunk_stream.put_i32(0); // chunk extra length

        let mut file = BytesMut::new();
        file.put_slice(&SLIME_MAGIC);
        file.put_u8(SLIME_VERSION);
        file.put_i32(3700); // world version (DataVersion)
        file.put_u8(0); // additional world data flags
        file.put_slice(&compress_block(&chunk_stream));
        file.put_slice(&compress_block(&[0x0a, 0x00, 0x00, 0x00])); // empty "extra" compound

        let world = read_slime_world(file.freeze()).expect("minimal world should parse");
        assert_eq!(world.world_version, 3700);
        assert_eq!(world.flags, 0);
        assert_eq!(world.chunks.len(), 1);
        let chunk = &world.chunks[0];
        assert_eq!((chunk.x, chunk.z), (2, 3));
        assert_eq!(chunk.sections.len(), 1);
        assert!(chunk.sections[0].block_light.is_none());
        assert!(chunk.sections[0].sky_light.is_none());
        assert!(chunk.sections[0].block_states.is_empty());
    }

    #[test]
    fn reads_section_light_arrays() {
        let mut chunk_stream = BytesMut::new();
        chunk_stream.put_i32(1); // chunk count
        chunk_stream.put_i32(0); // x
        chunk_stream.put_i32(0); // z
        chunk_stream.put_i32(1); // section count
        chunk_stream.put_u8(0b11); // section flags: block + sky light
        chunk_stream.put_slice(&[0x12u8; 2048]); // block light
        chunk_stream.put_slice(&[0x34u8; 2048]); // sky light
        chunk_stream.put_i32(0); // block_states length
        chunk_stream.put_i32(0); // biomes length
        chunk_stream.put_i32(0); // heightmaps length
        chunk_stream.put_i32(0); // tileEntities length
        chunk_stream.put_i32(0); // entities length
        chunk_stream.put_i32(0); // chunk extra length

        let mut file = BytesMut::new();
        file.put_slice(&SLIME_MAGIC);
        file.put_u8(SLIME_VERSION);
        file.put_i32(3700);
        file.put_u8(0);
        file.put_slice(&compress_block(&chunk_stream));
        file.put_slice(&compress_block(&[0x0a, 0x00, 0x00, 0x00]));

        let world = read_slime_world(file.freeze()).expect("world should parse");
        let section = &world.chunks[0].sections[0];
        assert_eq!(section.block_light.as_deref(), Some(&[0x12u8; 2048][..]));
        assert_eq!(section.sky_light.as_deref(), Some(&[0x34u8; 2048][..]));
    }

    #[test]
    fn rejects_bad_magic() {
        let mut file = BytesMut::new();
        file.put_slice(&[0x00, 0x00]);
        file.put_u8(SLIME_VERSION);
        let err = read_slime_world(file.freeze()).unwrap_err();
        assert!(matches!(err, SlimeError::BadMagic));
    }

    #[test]
    fn rejects_newer_version() {
        let mut file = BytesMut::new();
        file.put_slice(&SLIME_MAGIC);
        file.put_u8(99);
        let err = read_slime_world(file.freeze()).unwrap_err();
        assert!(matches!(err, SlimeError::UnsupportedVersion(99)));
    }

    #[test]
    fn rejects_truncated_data() {
        let mut file = BytesMut::new();
        file.put_slice(&SLIME_MAGIC);
        file.put_u8(SLIME_VERSION);
        file.put_i32(3700);
        // missing flags byte and everything after
        let err = read_slime_world(file.freeze()).unwrap_err();
        assert!(matches!(err, SlimeError::Truncated));
    }
}
