use std::{
    collections::HashSet,
    fs::OpenOptions,
    hash::RandomState,
    io::{ErrorKind, Read, Write},
};

use bitvec::{bits, order, vec::BitVec, view::BitView};
use pumpkin_core::math::ceil_log2;

use crate::{chunk::ChunkWritingError, level::LevelFolder, WORLD_HEIGHT};

use super::{
    ChunkBlocks, ChunkData, ChunkReader, ChunkReadingError, ChunkWriter, CompressionError,
    CHUNK_VOLUME, SUBCHUNK_VOLUME,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Compression {
    LZ4 = 1,
    Zstd = 2,
    Lzma = 3,
}

impl Compression {
    fn from_byte(byte: u8) -> Option<Self> {
        if byte == 0 {
            None
        } else {
            Some(match byte {
                1 => Compression::LZ4,
                2 => Compression::Zstd,
                _ => todo!(),
            })
        }
    }

    fn compress_data(
        &self,
        uncompressed_data: &[u8],
        compression_level: u32,
    ) -> Result<Vec<u8>, CompressionError> {
        match self {
            Compression::LZ4 => {
                let mut compressed_data = Vec::new();
                let mut encoder = lz4::EncoderBuilder::new()
                    .level(compression_level)
                    .build(&mut compressed_data)
                    .map_err(CompressionError::LZ4Error)?;
                if let Err(err) = encoder.write_all(uncompressed_data) {
                    return Err(CompressionError::LZ4Error(err));
                }
                if let (_output, Err(err)) = encoder.finish() {
                    return Err(CompressionError::LZ4Error(err));
                }
                Ok(compressed_data)
            }
            _ => todo!(),
        }
    }

    fn decompress_data(&self, compressed_data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        match self {
            Compression::LZ4 => {
                let mut decoder =
                    lz4::Decoder::new(compressed_data).map_err(CompressionError::LZ4Error)?;
                let mut decompressed_data = Vec::new();
                decoder
                    .read_to_end(&mut decompressed_data)
                    .map_err(CompressionError::LZ4Error)?;
                Ok(decompressed_data)
            }
            _ => todo!(),
        }
    }
}

#[derive(Clone, Default)]
pub struct PumpkinChunkFormat;

impl ChunkReader for PumpkinChunkFormat {
    fn read_chunk(
        &self,
        save_file: &LevelFolder,
        at: &pumpkin_core::math::vector2::Vector2<i32>,
    ) -> Result<super::ChunkData, ChunkReadingError> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(
                save_file
                    .region_folder
                    .join(format!("c.{}.{}.mcp", at.x, at.z)),
            )
            .map_err(|err| match err.kind() {
                std::io::ErrorKind::NotFound => ChunkReadingError::ChunkNotExist,
                kind => ChunkReadingError::IoError(kind),
            })?;

        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        if let Some(compression) = Compression::from_byte(data.remove(0)) {
            data = compression
                .decompress_data(&data)
                .map_err(ChunkReadingError::Compression)?;
        }

        let mut data: BitVec<u8, order::Lsb0> = BitVec::from_vec(data);

        let mut blocks = Vec::with_capacity(CHUNK_VOLUME);

        for _ in 0..WORLD_HEIGHT / 16 {
            let palette = {
                let mut palette = Vec::new();

                loop {
                    let block =
                        data.drain(..16)
                            .rev()
                            .fold(0, |acc, bit| if bit { (acc << 1) + 1 } else { acc << 1 });

                    if block == u16::MAX {
                        break;
                    } else {
                        palette.push(block);
                    }
                }
                palette
            };

            let block_bit_size = block_bit_size(&palette);

            let subchunk_blocks: BitVec<u8, order::Lsb0> =
                data.drain(..SUBCHUNK_VOLUME * block_bit_size).collect();

            blocks.extend(subchunk_blocks.chunks(block_bit_size).map(|b| {
                palette
                    .get(
                        b.iter()
                            .rev()
                            .fold(0, |acc, bit| if *bit { (acc << 1) + 1 } else { acc << 1 }),
                    )
                    .unwrap_or(&0)
            }));
        }

        Ok(ChunkData {
            blocks: ChunkBlocks {
                blocks: blocks
                    .try_into()
                    .or(Err(ChunkReadingError::RegionIsInvalid))?,
                ..Default::default()
            },
            position: *at,
        })
    }
}

impl ChunkWriter for PumpkinChunkFormat {
    fn write_chunk(
        &self,
        chunk_data: &ChunkData,
        level_folder: &LevelFolder,
        at: &pumpkin_core::math::vector2::Vector2<i32>,
    ) -> Result<(), super::ChunkWritingError> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(
                level_folder
                    .region_folder
                    .join(format!("c.{}.{}.mcp", at.x, at.z)),
            )
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

        let mut bits: BitVec<u8, order::Lsb0> = BitVec::new();

        for blocks in chunk_data.blocks.blocks.chunks(SUBCHUNK_VOLUME) {
            let mut palette: Vec<&u16> = HashSet::<&u16, RandomState>::from_iter(blocks.iter())
                .into_iter()
                .collect();
            palette.sort();

            let block_bit_size = block_bit_size(&palette);

            bits.extend_from_bitslice(
                BitVec::<u8, order::Lsb0>::from_vec(
                    palette
                        .iter()
                        .flat_map(|b| b.to_le_bytes())
                        .collect::<Vec<u8>>(),
                )
                .as_bitslice(),
            );
            bits.extend_from_bitslice(bits![1; 16]);

            for block in blocks {
                bits.extend_from_bitslice(
                    &palette
                        .binary_search(&block)
                        .map_err(|_| ChunkWritingError::IoError(ErrorKind::NotFound))?
                        .view_bits::<order::Lsb0>()[..block_bit_size],
                );
            }
        }

        // TODO: Config
        let compression = Compression::LZ4;

        file.write_all(&[compression as u8])
            .map_err(|e| ChunkWritingError::IoError(e.kind()))?;
        file.write_all(
            &compression
                .compress_data(bits.as_raw_slice(), 10)
                .map_err(ChunkWritingError::Compression)?,
        )
        .map_err(|e| ChunkWritingError::IoError(e.kind()))?;

        Ok(())
    }
}

fn block_bit_size<T>(palette: &Vec<T>) -> usize {
    if palette.len() < 16 {
        4
    } else {
        ceil_log2(palette.len() as u32).max(4) as usize
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use pumpkin_core::math::vector2::Vector2;

    use crate::{
        chunk::{pumpkin::PumpkinChunkFormat, ChunkReader, ChunkReadingError},
        level::LevelFolder,
    };

    #[test]
    fn not_existing() {
        let region_path = PathBuf::from("not_existing");
        let result = PumpkinChunkFormat.read_chunk(
            &LevelFolder {
                root_folder: PathBuf::from(""),
                region_folder: region_path,
            },
            &Vector2::new(0, 0),
        );
        assert!(matches!(result, Err(ChunkReadingError::ChunkNotExist)));
    }
}
