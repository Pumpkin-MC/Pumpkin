#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::fs;

use pumpkin_util::world_seed::Seed;
use pumpkin_world::world_info::{
    LevelData, WorldInfoError, WorldInfoReader, WorldInfoWriter,
    anvil::{AnvilLevelInfo, LEVEL_DAT_BACKUP_FILE_NAME},
};
use tempfile::TempDir;

#[test]
fn write_world_info_reports_backup_failure_and_keeps_previous_level_dat() {
    let temp_dir = TempDir::new().unwrap();

    let mut original = LevelData::default(Seed(3510));
    original.level_name = "Original World".to_string();
    AnvilLevelInfo
        .write_world_info(&original, temp_dir.path())
        .unwrap();

    fs::create_dir(temp_dir.path().join(LEVEL_DAT_BACKUP_FILE_NAME)).unwrap();

    let mut updated = original.clone();
    updated.level_name = "Updated World".to_string();

    let error = AnvilLevelInfo
        .write_world_info(&updated, temp_dir.path())
        .expect_err("a failed level.dat backup must make the save fail");

    assert!(matches!(error, WorldInfoError::IoError(_)));

    let reloaded = AnvilLevelInfo.read_world_info(temp_dir.path()).unwrap();
    assert_eq!(reloaded.level_name, "Original World");
}
