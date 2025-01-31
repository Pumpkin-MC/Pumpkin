use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{chunk::ChunkWritingError, level::LevelFolder};
use log::error;
use pumpkin_config::ADVANCED_CONFIG;

use super::anvil::AnvilChunkFormat;
use super::{
    ChunkData, ChunkReader, ChunkReadingError, ChunkSerializingError, ChunkWriter,
    CompressionError, FileLocksManager,
};

///The side size of a region in chunks (one region is 32x32 chunks)
const REGION_SIZE: usize = 32;

///The number of bits that identify two chunks in the same region
const SUBREGION_BITS: u8 = pumpkin_util::math::ceil_log2(REGION_SIZE as u32);

///The number of chunks in a region
const CHUNK_COUNT: usize = REGION_SIZE * REGION_SIZE;

/// The signature of the linear file format
/// used as a header and footer described in https://gist.github.com/Aaron2550/5701519671253d4c6190bde6706f9f98
const SIGNATURE: [u8; 8] = (0xc3ff13183cca9d9a_u64).to_be_bytes();

/// The size of the ChunkHeaders table in bytes
const CHUNK_HEADER_BYTES_SIZE: usize = CHUNK_COUNT * size_of::<LinearChunkHeader>();

#[derive(Default, Clone, Copy)]
struct LinearChunkHeader {
    size: u32,
    timestamp: u32,
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum LinearVersion {
    #[default]
    ///Used for defaults and invalid values
    None = 0x00,
    ///used by linear.py in xymb-endcrystalme/LinearRegionFileFormatTools
    V1 = 0x01,
    ///Requires investigation about this value/version
    V2 = 0x02,
}
struct LinearFileHeader {
    /// ( 0.. 1 Bytes) The Version of the file format
    version: LinearVersion,
    /// ( 1.. 9 Bytes) The newest chunk timestamp
    newest_timestamp: u64,
    /// ( 9..10 Bytes) The zstd compression level used
    compression_level: u8,
    /// (10..12 Bytes) The count of non 0 size chunks
    chunks_count: u16,
    /// (12..16 Bytes) size of the Compressed Chunk Heades Bytes (fixed size) + Chunk Data Bytes (dynamic size)
    chunks_bytes: u32,
    /// (16..24 Bytes) hash of the region file (apparently not used)
    region_hash: u64,
}
struct LinearFile {
    chunks_headers: Box<[LinearChunkHeader; CHUNK_COUNT]>,
    chunks_data: Vec<u8>,
}

#[derive(Clone, Default)]
pub struct LinearChunkFormat;

impl LinearChunkHeader {
    fn from_bytes(bytes: &[u8]) -> Self {
        let (size_bytes, timestamp_bytes) = bytes.split_at(4);
        LinearChunkHeader {
            size: u32::from_be_bytes(size_bytes.try_into().unwrap()),
            timestamp: u32::from_be_bytes(timestamp_bytes.try_into().unwrap()),
        }
    }

    fn to_bytes(self) -> [u8; 8] {
        let mut bytes = [0u8; 8];
        bytes[0..4].copy_from_slice(&self.size.to_be_bytes());
        bytes[4..8].copy_from_slice(&self.timestamp.to_be_bytes());

        bytes
    }
}

impl From<u8> for LinearVersion {
    fn from(value: u8) -> Self {
        match value {
            0x01 => LinearVersion::V1,
            0x02 => LinearVersion::V2,
            _ => LinearVersion::None,
        }
    }
}

impl LinearFileHeader {
    const FILE_HEADER_SIZE: usize = 24;

    fn check_version(&self) -> Result<(), ChunkReadingError> {
        match self.version {
            LinearVersion::None => {
                error!("Invalid version in the file header");
                Err(ChunkReadingError::InvalidHeader)
            }
            LinearVersion::V2 => {
                error!("LinearFormat Version 2 for Chunks is not supported yet");
                Err(ChunkReadingError::InvalidHeader)
            }
            _ => Ok(()),
        }
    }
    fn from_bytes(bytes: &[u8]) -> Self {
        LinearFileHeader {
            version: bytes[0].into(),
            newest_timestamp: u64::from_be_bytes(bytes[1..9].try_into().unwrap()),
            compression_level: bytes[9],
            chunks_count: u16::from_be_bytes(bytes[10..12].try_into().unwrap()),
            chunks_bytes: u32::from_be_bytes(bytes[12..16].try_into().unwrap()),
            region_hash: u64::from_be_bytes(bytes[16..24].try_into().unwrap()),
        }
    }

    fn to_bytes(&self) -> [u8; Self::FILE_HEADER_SIZE] {
        let mut bytes = [0u8; LinearFileHeader::FILE_HEADER_SIZE];
        bytes[0] = self.version as u8;
        bytes[1..9].copy_from_slice(&self.newest_timestamp.to_be_bytes());
        bytes[9] = self.compression_level;
        bytes[10..12].copy_from_slice(&self.chunks_count.to_be_bytes());
        bytes[12..16].copy_from_slice(&self.chunks_bytes.to_be_bytes());
        bytes[16..24].copy_from_slice(&self.region_hash.to_be_bytes());

        bytes
    }
}

impl LinearFile {
    fn new() -> Self {
        LinearFile {
            chunks_headers: Box::new([LinearChunkHeader::default(); CHUNK_COUNT]),
            chunks_data: vec![],
        }
    }
    fn check_signature(file: &mut File) -> Result<(), ChunkReadingError> {
        let mut signature = [0; 8];

        file.seek(SeekFrom::Start(0))
            .map_err(|err| ChunkReadingError::IoError(err.kind()))?; //seek to the start of the file
        file.read_exact(&mut signature)
            .map_err(|err| ChunkReadingError::IoError(err.kind()))?;
        if signature != SIGNATURE {
            error!("Signature at the start of the file is invalid");
            return Err(ChunkReadingError::InvalidHeader);
        }

        file.seek(SeekFrom::End(-8))
            .map_err(|err| ChunkReadingError::IoError(err.kind()))?; //seek to the end of the file
        file.read_exact(&mut signature)
            .map_err(|err| ChunkReadingError::IoError(err.kind()))?;
        if signature != SIGNATURE {
            error!("Signature at the end of the file is invalid");
            return Err(ChunkReadingError::InvalidHeader);
        }

        file.rewind()
            .map_err(|err| ChunkReadingError::IoError(err.kind()))?; //rewind the file

        Ok(())
    }

    fn load(path: &Path) -> Result<Self, ChunkReadingError> {
        let mut file = OpenOptions::new()
            .read(true)
            .truncate(false)
            .open(path)
            .map_err(|err| match err.kind() {
                std::io::ErrorKind::NotFound => ChunkReadingError::ChunkNotExist,
                kind => ChunkReadingError::IoError(kind),
            })?;

        Self::check_signature(&mut file)?;

        // Skip the signature and read the header
        let mut header_bytes = [0; LinearFileHeader::FILE_HEADER_SIZE];
        file.seek(SeekFrom::Start(8))
            .map_err(|err| ChunkReadingError::IoError(err.kind()))?;
        file.read_exact(&mut header_bytes)
            .map_err(|err| ChunkReadingError::IoError(err.kind()))?;

        // Parse the header
        let file_header = LinearFileHeader::from_bytes(&header_bytes);
        file_header.check_version()?;

        // Read the compressed data
        let mut compressed_data = vec![0; file_header.chunks_bytes as usize];
        file.read_exact(compressed_data.as_mut_slice())
            .map_err(|err| ChunkReadingError::IoError(err.kind()))?;

        if compressed_data.len() != file_header.chunks_bytes as usize {
            error!(
                "Invalid compressed data size {} != {}",
                compressed_data.len(),
                file_header.chunks_bytes
            );
            return Err(ChunkReadingError::InvalidHeader);
        }

        // Uncompress the data (header + chunks)
        let buffer = zstd::decode_all(compressed_data.as_slice())
            .map_err(|err| ChunkReadingError::IoError(err.kind()))?;

        let (headers_buffer, chunks_buffer) = buffer.split_at(CHUNK_HEADER_BYTES_SIZE);

        // Parse the chunk headers
        let chunk_headers: [LinearChunkHeader; CHUNK_COUNT] = headers_buffer
            .chunks_exact(8)
            .map(LinearChunkHeader::from_bytes)
            .collect::<Vec<LinearChunkHeader>>()
            .try_into()
            .map_err(|_| ChunkReadingError::InvalidHeader)?;

        // Check if the total bytes of the chunks match the header
        let total_bytes = chunk_headers.iter().map(|header| header.size).sum::<u32>() as usize;
        if chunks_buffer.len() != total_bytes {
            error!(
                "Invalid total bytes of the chunks {} != {}",
                total_bytes,
                chunks_buffer.len(),
            );
            return Err(ChunkReadingError::InvalidHeader);
        }

        Ok(LinearFile {
            chunks_headers: Box::new(chunk_headers),
            chunks_data: chunks_buffer.to_vec(),
        })
    }

    fn save(&self, path: &Path) -> Result<(), ChunkWritingError> {
        // Parse the headers to a buffer
        let headers_buffer: Vec<u8> = self
            .chunks_headers
            .as_ref()
            .iter()
            .flat_map(|header| header.to_bytes())
            .collect();

        // Compress the data buffer
        let compressed_buffer = zstd::encode_all(
            [headers_buffer.as_slice(), self.chunks_data.as_slice()]
                .concat()
                .as_slice(),
            ADVANCED_CONFIG.chunk.compression.compression_level as i32,
        )
        .map_err(|err| ChunkWritingError::Compression(CompressionError::ZstdError(err)))?;

        // Update the header
        let file_header = LinearFileHeader {
            chunks_bytes: compressed_buffer.len() as u32,
            compression_level: ADVANCED_CONFIG.chunk.compression.compression_level as u8,
            chunks_count: self
                .chunks_headers
                .iter()
                .filter(|&header| header.size != 0)
                .count() as u16,
            newest_timestamp: self
                .chunks_headers
                .iter()
                .map(|header| header.timestamp)
                .max()
                .unwrap_or(0) as u64,
            version: LinearVersion::V1,
            region_hash: 0,
        }
        .to_bytes();

        // Write/OverWrite the data to the file
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

        file.write_all(
            [
                SIGNATURE.as_slice(),
                file_header.as_slice(),
                compressed_buffer.as_slice(),
                SIGNATURE.as_slice(),
            ]
            .concat()
            .as_slice(),
        )
        .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

        Ok(())
    }

    fn get_chunk(
        &self,
        at: &pumpkin_util::math::vector2::Vector2<i32>,
    ) -> Result<ChunkData, ChunkReadingError> {
        // We check if the chunk exists
        let chunk_index: usize = LinearChunkFormat::get_chunk_index(at);

        let chunk_size = self.chunks_headers[chunk_index].size as usize;
        if chunk_size == 0 {
            return Err(ChunkReadingError::ChunkNotExist);
        }

        // We iterate over the headers to sum the size of the chunks until the desired one
        let mut offset: usize = 0;
        for i in 0..chunk_index {
            offset += self.chunks_headers[i].size as usize;
        }

        ChunkData::from_bytes(&self.chunks_data[offset..offset + chunk_size], *at)
            .map_err(ChunkReadingError::ParsingError)
    }

    fn put_chunk(
        &mut self,
        chunk: &ChunkData,
        at: &pumpkin_util::math::vector2::Vector2<i32>,
    ) -> Result<(), ChunkSerializingError> {
        let chunk_index: usize = LinearChunkFormat::get_chunk_index(at);
        let chunk_raw = AnvilChunkFormat {} //We use Anvil format to serialize the chunk
            .to_bytes(chunk)?;

        let new_chunk_size = chunk_raw.len();
        let old_chunk_size = self.chunks_headers[chunk_index].size as usize;

        self.chunks_headers[chunk_index] = LinearChunkHeader {
            size: new_chunk_size as u32,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32,
        };

        // We calculate the start point of the chunk in the data buffer
        let mut offset: usize = 0;
        for i in 0..chunk_index {
            offset += self.chunks_headers[i].size as usize;
        }

        let old_total_size = self.chunks_data.len();
        let new_total_size = (old_total_size + new_chunk_size) - old_chunk_size;

        // We update the data buffer (avoiding reallocations)
        if new_chunk_size > old_chunk_size {
            self.chunks_data.resize(new_total_size, 0);
        }

        self.chunks_data.copy_within(
            offset + old_chunk_size..old_total_size,
            offset + new_chunk_size,
        );

        self.chunks_data[offset..offset + new_chunk_size].copy_from_slice(&chunk_raw);

        if new_chunk_size < old_chunk_size {
            self.chunks_data.truncate(new_total_size);
        }

        Ok(())
    }
}

impl LinearChunkFormat {
    const fn get_region_coords(at: &pumpkin_util::math::vector2::Vector2<i32>) -> (i32, i32) {
        (at.x >> SUBREGION_BITS, at.z >> SUBREGION_BITS) // Divide by 32 for the region coordinates
    }

    const fn get_chunk_index(at: &pumpkin_util::math::vector2::Vector2<i32>) -> usize {
        // we need only the 5 last bits of the x and z coordinates
        let decode_x = at.x - ((at.x >> SUBREGION_BITS) << SUBREGION_BITS);
        let decode_z = at.z - ((at.z >> SUBREGION_BITS) << SUBREGION_BITS);

        // we calculate the index of the chunk in the region file
        ((decode_z << SUBREGION_BITS) + decode_x) as usize
    }
}

impl ChunkReader for LinearChunkFormat {
    fn read_chunk(
        &self,
        save_file: &LevelFolder,
        at: &pumpkin_util::math::vector2::Vector2<i32>,
    ) -> Result<ChunkData, ChunkReadingError> {
        let (region_x, region_z) = LinearChunkFormat::get_region_coords(at);

        let path = save_file
            .region_folder
            .join(format!("./r.{}.{}.linear", region_x, region_z));

        tokio::task::block_in_place(|| {
            let file_lock = FileLocksManager::get_file_lock(&path);
            let _reader_lock = file_lock.blocking_read();
            //dbg!("Reading chunk at {:?}", at);
            LinearFile::load(&path)?.get_chunk(at)
        })
    }
}

impl ChunkWriter for LinearChunkFormat {
    fn write_chunk(
        &self,
        chunk: &ChunkData,
        level_folder: &LevelFolder,
        at: &pumpkin_util::math::vector2::Vector2<i32>,
    ) -> Result<(), ChunkWritingError> {
        let (region_x, region_z) = LinearChunkFormat::get_region_coords(at);

        let path = level_folder
            .region_folder
            .join(format!("./r.{}.{}.linear", region_x, region_z));

        tokio::task::block_in_place(|| {
            let file_lock = FileLocksManager::get_file_lock(&path);
            let _writer_lock = file_lock.blocking_write();
            //dbg!("Writing chunk at {:?}", at);

            let mut file_data = match LinearFile::load(&path) {
                Ok(file_data) => file_data,
                Err(ChunkReadingError::ChunkNotExist) => LinearFile::new(),
                Err(ChunkReadingError::IoError(err)) => {
                    error!("Error reading the data before write: {}", err);
                    return Err(ChunkWritingError::IoError(err));
                }
                Err(_) => return Err(ChunkWritingError::IoError(std::io::ErrorKind::Other)),
            };

            file_data
                .put_chunk(chunk, at)
                .map_err(|err| ChunkWritingError::ChunkSerializingError(err.to_string()))?;

            file_data.save(&path)
        })
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_util::math::vector2::Vector2;
    use std::fs;
    use std::path::PathBuf;

    use crate::chunk::ChunkWriter;
    use crate::generation::{get_world_gen, Seed};
    use crate::{
        chunk::{linear::LinearChunkFormat, ChunkReader, ChunkReadingError},
        level::LevelFolder,
    };

    #[test]
    fn not_existing() {
        let region_path = PathBuf::from("not_existing");
        let result = LinearChunkFormat.read_chunk(
            &LevelFolder {
                root_folder: PathBuf::from(""),
                region_folder: region_path,
            },
            &Vector2::new(0, 0),
        );
        assert!(matches!(result, Err(ChunkReadingError::ChunkNotExist)));
    }

    #[test]
    fn test_writing() {
        let generator = get_world_gen(Seed(0));
        let level_folder = LevelFolder {
            root_folder: PathBuf::from("./tmp_Linear"),
            region_folder: PathBuf::from("./tmp_Linear/region"),
        };
        if fs::exists(&level_folder.root_folder).unwrap() {
            fs::remove_dir_all(&level_folder.root_folder).expect("Could not delete directory");
        }

        fs::create_dir_all(&level_folder.region_folder).expect("Could not create directory");

        // Generate chunks
        let mut chunks = vec![];
        for x in -5..5 {
            for y in -5..5 {
                let position = Vector2::new(x, y);
                chunks.push((position, generator.generate_chunk(position)));
            }
        }

        for i in 0..5 {
            println!("Iteration {}", i + 1);
            for (at, chunk) in &chunks {
                LinearChunkFormat
                    .write_chunk(chunk, &level_folder, at)
                    .expect("Failed to write chunk");
            }

            let mut read_chunks = vec![];
            for (at, _chunk) in &chunks {
                read_chunks.push(
                    LinearChunkFormat
                        .read_chunk(&level_folder, at)
                        .expect("Could not read chunk"),
                );
            }

            for (at, chunk) in &chunks {
                let read_chunk = read_chunks
                    .iter()
                    .find(|chunk| chunk.position == *at)
                    .expect("Missing chunk");
                assert_eq!(chunk.subchunks, read_chunk.subchunks, "Chunks don't match");
            }
        }

        fs::remove_dir_all(&level_folder.root_folder).expect("Could not delete directory");

        println!("Checked chunks successfully");
    }
}
