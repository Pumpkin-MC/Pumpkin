use std::{
    fs::OpenOptions,
    time::{SystemTime, UNIX_EPOCH},
};

use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use pumpkin_nbt::{deserializer::from_bytes, serializer::to_bytes};
use serde::{Deserialize, Serialize};

use crate::level::LevelFolder;

use super::{LevelData, WorldInfoError, WorldInfoReader, WorldInfoWriter};

const LEVEL_DAT_FILE_NAME: &str = "level.dat";

pub struct AnvilLevelInfo;

impl WorldInfoReader for AnvilLevelInfo {
    fn read_world_info(&self, level_folder: &LevelFolder) -> Result<LevelData, WorldInfoError> {
        let path = level_folder.root_folder.join(LEVEL_DAT_FILE_NAME);

        let world_info_file = OpenOptions::new().read(true).open(path)?;
        let compression_reader = GzDecoder::new(world_info_file);
        let info = from_bytes::<LevelDat>(compression_reader)
            .map_err(|e| WorldInfoError::DeserializationError(e.to_string()))?;

        // todo check version

        Ok(info.data)
    }
}

impl WorldInfoWriter for AnvilLevelInfo {
    fn write_world_info(
        &self,
        info: LevelData,
        level_folder: &LevelFolder,
    ) -> Result<(), WorldInfoError> {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let mut level_data = info.clone();
        level_data.last_played = since_the_epoch.as_millis() as i64;
        let level = LevelDat { data: level_data };

        // open file
        let path = level_folder.root_folder.join(LEVEL_DAT_FILE_NAME);
        let world_info_file = OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(path)?;

        // write compressed data into file
        let compression_writer = GzEncoder::new(world_info_file, Compression::best());
        // TODO: Proper error handling
        to_bytes(&level, compression_writer).unwrap();

        /*
        let mut raw_nbt = Vec::new();
        to_bytes(&level, &mut raw_nbt).unwrap();
        println!("{:02X?}", &raw_nbt);

        let mut encoder = GzEncoder::new(world_info_file, Compression::best());
        encoder.write_all(&raw_nbt).unwrap();
        */

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct LevelDat {
    // This tag contains all the level data.
    #[serde(rename = "Data")]
    pub data: LevelData,
}

#[cfg(test)]
mod test {

    use temp_dir::TempDir;

    use crate::{level::LevelFolder, world_info::LevelData};

    use super::{AnvilLevelInfo, WorldInfoReader, WorldInfoWriter};

    #[test]
    fn test_perserve_level_dat_seed() {
        let seed = 1337;

        let mut data = LevelData::default();
        data.world_gen_settings.seed = seed;

        let temp_dir = TempDir::new().unwrap();
        let level_folder = LevelFolder {
            root_folder: temp_dir.path().to_path_buf(),
            region_folder: temp_dir.path().join("region"),
        };

        AnvilLevelInfo
            .write_world_info(data, &level_folder)
            .unwrap();

        let data = AnvilLevelInfo.read_world_info(&level_folder).unwrap();

        assert_eq!(data.world_gen_settings.seed, seed);
    }
}
