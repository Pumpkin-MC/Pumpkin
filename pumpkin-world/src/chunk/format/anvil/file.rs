use bytes::{Buf, BufMut, Bytes};
use itertools::Itertools;
use pumpkin_config::chunk::AnvilChunkConfig;
use pumpkin_util::math::vector2::Vector2;
use std::{
    io::SeekFrom,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::io::{AsyncSeekExt, AsyncWriteExt, BufWriter};
use tracing::{debug, trace};

use crate::chunk::{
    ChunkParsingError, ChunkReadingError, ChunkWritingError,
    io::{ChunkSerializer, LoadedData},
};

use super::{
    AnvilChunkData, AnvilChunkFile, AnvilChunkMetadata, CHUNK_COUNT, SECTOR_BYTES, SUBREGION_AND,
    SUBREGION_BITS, SingleChunkDataSerializer, WriteAction,
};

impl<S: SingleChunkDataSerializer> AnvilChunkFile<S> {
    #[must_use]
    pub const fn get_region_coords(at: &Vector2<i32>) -> (i32, i32) {
        // Divide by 32 for the region coordinates
        (at.x >> SUBREGION_BITS, at.y >> SUBREGION_BITS)
    }

    #[must_use]
    pub const fn get_chunk_index(x: i32, z: i32) -> usize {
        let local_x = x & SUBREGION_AND;
        let local_z = z & SUBREGION_AND;
        let index = (local_z << SUBREGION_BITS) + local_x;
        index as usize
    }

    async fn write_indices<I>(&self, path: &Path, indices: I) -> Result<(), std::io::Error>
    where
        I: IntoIterator<Item = usize>,
    {
        trace!("Writing in place: {}", path.display());

        let file = tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .append(false)
            .open(path)
            .await?;

        let mut write = BufWriter::new(file);
        // The first two sectors are reserved for the location table
        let mut header = Vec::with_capacity(SECTOR_BYTES * 2);

        // Location Table
        for metadata in &self.chunks_data {
            if let Some(chunk) = metadata {
                let sector_count = chunk.serialized_data.sector_count();
                header.put_u32((chunk.file_sector_offset << 8) | sector_count);
            } else {
                header.put_u32(0);
            }
        }

        // Timestamp Table
        for metadata in &self.chunks_data {
            if let Some(chunk) = metadata {
                header.put_u32(chunk.timestamp);
            } else {
                header.put_u32(0);
            }
        }

        // Write all 8 KiB in a single async call
        write.write_all(&header).await?;

        let mut chunks = indices
            .into_iter()
            .map(|index| {
                (
                    index,
                    self.chunks_data[index]
                        .as_ref()
                        .expect("We are trying to write a chunk, but it does not exist!"),
                )
            })
            .collect::<Vec<_>>();

        // Sort such that writes are in order
        chunks.sort_by_key(|chunk| chunk.1.file_sector_offset);

        #[cfg(debug_assertions)]
        {
            // Verify we are actually two sectors into the file
            let current_pos = write.stream_position().await?;
            assert_eq!(current_pos as usize, 2 * SECTOR_BYTES);
        };

        let mut current_sector = 2;
        for (index, chunk) in chunks {
            debug_assert!(
                current_sector <= chunk.file_sector_offset,
                "Current sector is {} but we want to write to {}!",
                current_sector,
                chunk.file_sector_offset
            );

            // Seek only if we need to
            if chunk.file_sector_offset != current_sector {
                trace!("Seeking to sector {}", chunk.file_sector_offset);
                let _ = write
                    .seek(SeekFrom::Start(
                        chunk.file_sector_offset as u64 * SECTOR_BYTES as u64,
                    ))
                    .await?;
                current_sector = chunk.file_sector_offset;
            }
            trace!(
                "Writing chunk {} - {}:{}",
                index,
                current_sector,
                chunk.serialized_data.sector_count()
            );

            current_sector += chunk.serialized_data.sector_count();

            chunk.serialized_data.write(&mut write).await?;
        }

        write.flush().await
    }

    /// Write entire file, disregarding saved offsets
    async fn write_all(&self, path: &Path) -> Result<(), std::io::Error> {
        let temp_path = path.with_extension("tmp");
        trace!("Writing tmp file to disk: {temp_path:?}");

        let file = tokio::fs::File::create(&temp_path).await?;
        let mut write = BufWriter::new(file);

        // Build the 8 KiB header in memory
        let mut header = Vec::with_capacity(SECTOR_BYTES * 2);
        let mut current_sector: u32 = 2;

        // Location Table
        for metadata in &self.chunks_data {
            if let Some(chunk) = metadata {
                let sector_count = chunk.serialized_data.sector_count();
                header.put_u32((current_sector << 8) | sector_count);
                current_sector += sector_count;
            } else {
                header.put_u32(0);
            }
        }

        // Timestamp Table
        for metadata in &self.chunks_data {
            if let Some(chunk) = metadata {
                header.put_u32(chunk.timestamp);
            } else {
                header.put_u32(0);
            }
        }

        // Write all 8 KiB in a single async call
        write.write_all(&header).await?;

        // Write chunk data
        for chunk in self.chunks_data.iter().flatten() {
            chunk.serialized_data.write(&mut write).await?;
        }

        write.flush().await?;
        tokio::fs::rename(temp_path, path).await?;
        Ok(())
    }
}

impl<S: SingleChunkDataSerializer> ChunkSerializer for AnvilChunkFile<S> {
    type Data = S;
    type WriteBackend = PathBuf;

    type ChunkConfig = AnvilChunkConfig;

    fn should_write(&self, is_watched: bool) -> bool {
        !is_watched
    }

    fn get_chunk_key(chunk: &Vector2<i32>) -> String {
        let (region_x, region_z) = Self::get_region_coords(chunk);
        format!("./r.{region_x}.{region_z}.mca")
    }

    async fn write(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        let mut write_action = self.write_action.lock().await;
        match &*write_action {
            WriteAction::Pass => {
                debug!(
                    "Skipping write for {}, as there were no dirty chunks",
                    path.display()
                );
                Ok(())
            }
            WriteAction::All => self.write_all(path).await,
            WriteAction::Parts(parts) => self.write_indices(path, parts.iter().copied()).await,
        }?;

        // If we still are in memory after this, we don't need to write again!
        *write_action = WriteAction::Pass;
        Ok(())
    }

    fn read(r: Bytes) -> Result<Self, ChunkReadingError> {
        let mut raw_file_bytes = r;

        if raw_file_bytes.len() < SECTOR_BYTES * 2 {
            return Err(ChunkReadingError::InvalidHeader);
        }

        let headers = raw_file_bytes.split_to(SECTOR_BYTES * 2);
        let (mut location_bytes, mut timestamp_bytes) = headers.split_at(SECTOR_BYTES);

        let mut chunk_file = Self::default();

        let mut last_offset = 2;
        for i in 0..CHUNK_COUNT {
            let timestamp = timestamp_bytes.get_u32();
            let location = location_bytes.get_u32();

            let sector_count = (location & 0xFF) as usize;
            let sector_offset = (location >> 8) as usize;
            let end_offset = sector_offset + sector_count;

            // If the sector offset or count is 0, the chunk is not present (we should not parse empty chunks)
            if sector_offset == 0 || sector_count == 0 {
                continue;
            }

            if sector_offset < 2 {
                return Err(ChunkReadingError::ParsingError(
                    ChunkParsingError::ErrorDeserializingChunk(format!(
                        "Chunk {i} starts in the region header at sector {sector_offset}"
                    )),
                ));
            }

            if end_offset > last_offset {
                last_offset = end_offset;
            }

            // We always subtract 2 for the first two sectors for the timestamp and location tables
            // that we walked earlier
            let bytes_offset = (sector_offset - 2)
                .checked_mul(SECTOR_BYTES)
                .ok_or_else(|| {
                    ChunkReadingError::ParsingError(ChunkParsingError::ErrorDeserializingChunk(
                        format!("Chunk {i} has an invalid sector offset"),
                    ))
                })?;
            let bytes_count = sector_count * SECTOR_BYTES;
            let bytes_end = bytes_offset.checked_add(bytes_count).ok_or_else(|| {
                ChunkReadingError::ParsingError(ChunkParsingError::ErrorDeserializingChunk(
                    format!("Chunk {i} has an invalid sector range"),
                ))
            })?;

            if bytes_end > raw_file_bytes.len() {
                return Err(ChunkReadingError::ParsingError(
                    ChunkParsingError::ErrorDeserializingChunk(format!(
                        "Not enough bytes available for the chunk {} ({} vs {})",
                        i,
                        bytes_count,
                        raw_file_bytes.len().saturating_sub(bytes_offset)
                    )),
                ));
            }

            let serialized_data =
                AnvilChunkData::from_bytes(raw_file_bytes.slice(bytes_offset..bytes_end))?;

            chunk_file.chunks_data[i] = Some(AnvilChunkMetadata {
                serialized_data,
                timestamp,
                file_sector_offset: sector_offset as u32,
            });
        }

        chunk_file.end_sector = last_offset as u32;
        Ok(chunk_file)
    }

    #[expect(clippy::too_many_lines)]
    async fn update_chunk(
        &mut self,
        chunk: &Self::Data,
        chunk_config: &Self::ChunkConfig,
    ) -> Result<(), ChunkWritingError> {
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        let index = Self::get_chunk_index(chunk.position().0, chunk.position().1);
        // Default to the compression type read from the file
        let compression_type = self.chunks_data[index]
            .as_ref()
            .and_then(|chunk_data| chunk_data.serialized_data.compression);
        let new_chunk_data =
            AnvilChunkData::from_chunk(chunk, compression_type, chunk_config).await?;

        let mut write_action = self.write_action.lock().await;
        if !chunk_config.write_in_place {
            *write_action = WriteAction::All;
        }

        match &*write_action {
            WriteAction::All => {
                trace!("Write action is all: setting chunk in place");
                // Doesn't matter, just add the data
                self.chunks_data[index] = Some(AnvilChunkMetadata {
                    serialized_data: new_chunk_data,
                    timestamp: epoch,
                    file_sector_offset: 0,
                });
            }
            _ => {
                match self.chunks_data[index].as_ref() {
                    None => {
                        trace!(
                            "Chunk {} does not exist, appending to EOF: {}:{}",
                            index,
                            self.end_sector,
                            new_chunk_data.sector_count()
                        );
                        // This chunk didn't exist before; append to EOF
                        let new_eof = self.end_sector + new_chunk_data.sector_count();
                        self.chunks_data[index] = Some(AnvilChunkMetadata {
                            serialized_data: new_chunk_data,
                            timestamp: epoch,
                            file_sector_offset: self.end_sector,
                        });
                        self.end_sector = new_eof;
                        write_action.maybe_update_chunk_index(index);
                    }
                    Some(old_chunk) => {
                        if old_chunk.serialized_data.sector_count() == new_chunk_data.sector_count()
                        {
                            trace!(
                                "Chunk {} exists, writing in place: {}:{}",
                                index,
                                old_chunk.file_sector_offset,
                                new_chunk_data.sector_count()
                            );
                            // We can just add it
                            self.chunks_data[index] = Some(AnvilChunkMetadata {
                                serialized_data: new_chunk_data,
                                timestamp: epoch,
                                file_sector_offset: old_chunk.file_sector_offset,
                            });
                            write_action.maybe_update_chunk_index(index);
                        } else {
                            // Walk back the end of the list; seeing if there's something that can fit
                            // in our spot. Here we play a game between is it worth it to do all
                            // this swapping. I figure if we don't find it after 64 chunks, just
                            // re-write the whole file instead
                            // The number is a guestimation and no rigorious thought when into it.
                            // The more we leapfrog like this, there is a higher
                            // (abiet still small) of these chunks being corrupted if we are doing a
                            // write operation when there is an un-clean shutdown
                            //
                            // Writing all is "safer" in the sense that no chunks will corrupt,
                            // but will still roll back the entire region if
                            // there is an unclean shutdown

                            let mut chunks = self
                                .chunks_data
                                .iter()
                                .enumerate()
                                .filter_map(|(index, chunk)| {
                                    chunk.as_ref().map(|chunk| (index, chunk))
                                })
                                .collect::<Vec<_>>();
                            chunks.sort_by_key(|chunk| chunk.1.file_sector_offset);

                            let mut chunks_to_shift = chunks
                                .into_iter()
                                .rev()
                                .take(64)
                                .take_while_inclusive(|chunk| {
                                    chunk.1.serialized_data.sector_count()
                                        != old_chunk.serialized_data.sector_count()
                                })
                                .collect::<Vec<_>>();

                            if chunks_to_shift.last().is_none_or(|chunk| chunk.0 == index) {
                                trace!(
                                    "Unable to find a chunk to swap with; falling back to serialize all",
                                );

                                // give up...
                                *write_action = WriteAction::All;
                                self.chunks_data[index] = Some(AnvilChunkMetadata {
                                    serialized_data: new_chunk_data,
                                    timestamp: epoch,
                                    file_sector_offset: 0,
                                });
                            } else {
                                // swap last element of the chunks to shift (the first because we
                                // reversed it) and shift the rest down
                                let swap = chunks_to_shift
                                    .pop()
                                    .expect("We just checked that this exists");

                                let indices_to_shift = chunks_to_shift
                                    .iter()
                                    .map(|(index, _)| index)
                                    .copied()
                                    .collect::<Vec<_>>();
                                let swapped_sectors = swap.1.serialized_data.sector_count();
                                let new_sectors = new_chunk_data.sector_count();
                                let swapped_index = swap.0;
                                let old_offset = old_chunk.file_sector_offset;
                                self.chunks_data[index] = Some(AnvilChunkMetadata {
                                    serialized_data: new_chunk_data,
                                    timestamp: epoch,
                                    file_sector_offset: swap.1.file_sector_offset,
                                });
                                write_action.maybe_update_chunk_index(index);

                                self.chunks_data[swapped_index]
                                    .as_mut()
                                    .expect("We checked if this was none")
                                    .file_sector_offset = old_offset;
                                write_action.maybe_update_chunk_index(swapped_index);

                                // Then offset everything else

                                // If positive, now larger -> shift right, else shift left
                                let offset = new_sectors as i64 - swapped_sectors as i64;

                                trace!(
                                    "Swapping {index} with {swapped_index}, shifting all chunks {swapped_index} and after by {offset}"
                                );

                                for shift_index in indices_to_shift {
                                    let chunk_data = self.chunks_data[shift_index]
                                        .as_mut()
                                        .expect("We checked if this was none");
                                    let new_offset = chunk_data.file_sector_offset as i64 + offset;
                                    chunk_data.file_sector_offset = new_offset as u32;
                                    write_action.maybe_update_chunk_index(shift_index);
                                }

                                // If the shift is negative then there will be trailing data, but i
                                // think that's fine

                                let new_end = self.end_sector as i64 + offset;
                                self.end_sector = new_end as u32;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn get_chunks(
        &self,
        chunks: Vec<Vector2<i32>>,
        stream: tokio::sync::mpsc::Sender<LoadedData<Self::Data, ChunkReadingError>>,
    ) {
        // Don't par iter here so we can prevent backpressure with the await in the async
        // runtime
        for chunk in chunks {
            let index = Self::get_chunk_index(chunk.x, chunk.y);
            let is_ok = match &self.chunks_data[index] {
                None => stream.send(LoadedData::Missing(chunk)).await.is_ok(),
                Some(chunk_metadata) => {
                    let chunk_data = &chunk_metadata.serialized_data;
                    let result = match chunk_data.to_chunk(chunk) {
                        Ok(chunk) => LoadedData::Loaded(chunk),
                        Err(err) => LoadedData::Error((chunk, err)),
                    };

                    stream.send(result).await.is_ok()
                }
            };

            if !is_ok {
                // Stream is closed. Stop unneeded work and IO
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::chunk::{ChunkData, ChunkReadingError, io::ChunkSerializer};

    use super::{AnvilChunkFile, Bytes, CHUNK_COUNT, SECTOR_BYTES, Vector2, WriteAction};

    type ChunkFile = AnvilChunkFile<ChunkData>;

    #[test]
    fn chunk_index_wraps_local_coordinates() {
        assert_eq!(ChunkFile::get_chunk_index(0, 0), 0);
        assert_eq!(ChunkFile::get_chunk_index(31, 31), CHUNK_COUNT - 1);
        assert_eq!(ChunkFile::get_chunk_index(32, 32), 0);
        assert_eq!(ChunkFile::get_chunk_index(-1, -1), CHUNK_COUNT - 1);
        assert_eq!(
            ChunkFile::get_chunk_index(33, -31),
            ChunkFile::get_chunk_index(1, 1)
        );
    }

    #[test]
    fn region_coords_floor_divide_chunk_coords() {
        assert_eq!(ChunkFile::get_region_coords(&Vector2::new(0, 0)), (0, 0));
        assert_eq!(ChunkFile::get_region_coords(&Vector2::new(31, 31)), (0, 0));
        assert_eq!(ChunkFile::get_region_coords(&Vector2::new(32, -1)), (1, -1));
        assert_eq!(
            ChunkFile::get_region_coords(&Vector2::new(-33, 64)),
            (-2, 2)
        );
    }

    #[test]
    fn chunk_key_uses_region_coordinates() {
        assert_eq!(
            ChunkFile::get_chunk_key(&Vector2::new(33, -1)),
            "./r.1.-1.mca"
        );
        assert_eq!(ChunkFile::get_chunk_key(&Vector2::new(0, 0)), "./r.0.0.mca");
    }

    #[test]
    fn read_rejects_short_header() {
        assert!(matches!(
            ChunkFile::read(Bytes::from_static(&[0u8; 16])),
            Err(ChunkReadingError::InvalidHeader)
        ));
    }

    #[test]
    fn read_accepts_empty_region_file() {
        let file = ChunkFile::read(Bytes::from(vec![0u8; SECTOR_BYTES * 2])).unwrap();
        assert_eq!(file.end_sector, 2);
        assert!(file.chunks_data.iter().all(Option::is_none));
    }

    #[test]
    fn write_action_tracks_dirty_indices() {
        let mut action = WriteAction::Pass;
        action.maybe_update_chunk_index(3);
        action.maybe_update_chunk_index(7);
        action.maybe_update_chunk_index(3);
        let WriteAction::Parts(parts) = &action else {
            panic!("expected WriteAction::Parts");
        };
        assert_eq!(*parts, vec![3, 7]);

        let mut action = WriteAction::All;
        action.maybe_update_chunk_index(1);
        assert!(matches!(action, WriteAction::All));
    }

    #[test]
    fn default_region_reserves_header_sectors() {
        let file = ChunkFile::default();
        assert_eq!(file.end_sector, 2);
        assert!(!file.should_write(true));
        assert!(file.should_write(false));
    }
}
