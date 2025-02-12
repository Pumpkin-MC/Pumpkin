use std::{
    fs::OpenOptions,
    io::{Read, Write},
    time::{SystemTime, UNIX_EPOCH},
};

use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use serde::{Deserialize, Serialize};

use crate::{
    level::LevelFolder,
    world_info::{MAXIMUM_SUPPORTED_WORLD_DATA_VERSION, MINIMUM_SUPPORTED_WORLD_DATA_VERSION},
};

use super::{LevelData, WorldInfoError, WorldInfoReader, WorldInfoWriter};

pub const LEVEL_DAT_FILE_NAME: &str = "level.dat";
pub const LEVEL_DAT_BACKUP_FILE_NAME: &str = "level.dat_old";

pub struct AnvilLevelInfo;

fn check_file_data_version(raw_nbt: &[u8]) -> Result<(), WorldInfoError> {
    // Define a struct that only has the data version
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct LevelData {
        data_version: u32,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct LevelDat {
        data: LevelData,
    }

    let info: LevelDat = fastnbt::from_bytes(raw_nbt)
        .map_err(|e|{
            log::error!("The world.dat file does not have a data version! This means it is either corrupt or very old (read unsupported)");
            WorldInfoError::DeserializationError(e.to_string())})?;

    let data_version = info.data.data_version;

    if !(MINIMUM_SUPPORTED_WORLD_DATA_VERSION..=MAXIMUM_SUPPORTED_WORLD_DATA_VERSION)
        .contains(&data_version)
    {
        Err(WorldInfoError::UnsupportedVersion(data_version))
    } else {
        Ok(())
    }
}

impl WorldInfoReader for AnvilLevelInfo {
    fn read_world_info(&self, level_folder: &LevelFolder) -> Result<LevelData, WorldInfoError> {
        let path = level_folder.root_folder.join(LEVEL_DAT_FILE_NAME);

        let world_info_file = OpenOptions::new().read(true).open(path)?;
        // try to decompress using GZip
        let mut decoder = GzDecoder::new(world_info_file);
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;

        check_file_data_version(&decompressed_data)?;

        let info = fastnbt::from_bytes::<LevelDat>(&decompressed_data)
            .map_err(|e| WorldInfoError::DeserializationError(e.to_string()))?;

        // todo check version

        Ok(info.data)
    }
}

impl WorldInfoWriter for AnvilLevelInfo {
    fn write_world_info(
        &self,
        data: LevelData,
        level_folder: &LevelFolder,
    ) -> Result<(), WorldInfoError> {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let mut data = data.clone();
        data.last_played = since_the_epoch.as_millis() as i64;
        let level_dat = LevelDat { data };

        // convert it into nbt
        // TODO: Doesn't seem like pumpkin_nbt is working
        // TODO: fastnbt doesnt serialize bools
        let nbt = fastnbt::to_bytes(&level_dat).unwrap();

        // open file
        let path = level_folder.root_folder.join(LEVEL_DAT_FILE_NAME);
        let world_info_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?;
        // now compress using GZip, TODO: im not sure about the to_vec, but writer is not implemented for BytesMut, see https://github.com/tokio-rs/bytes/pull/478
        let mut encoder = GzEncoder::new(world_info_file, Compression::best());

        // write compressed data into file
        encoder.write_all(&nbt)?;

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
