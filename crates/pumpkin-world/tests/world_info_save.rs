#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

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

    let mut updated = original;
    updated.level_name = "Updated World".to_string();

    let error = AnvilLevelInfo
        .write_world_info(&updated, temp_dir.path())
        .expect_err("a failed level.dat backup must make the save fail");

    assert!(matches!(error, WorldInfoError::IoError(_)));

    let reloaded = AnvilLevelInfo.read_world_info(temp_dir.path()).unwrap();
    assert_eq!(reloaded.level_name, "Original World");
}

#[cfg(unix)]
#[test]
fn write_world_info_reports_final_rename_failure() {
    let temp_dir = TempDir::new().unwrap();

    let mut original = LevelData::default(Seed(3510));
    original.level_name = "Original World".to_string();
    AnvilLevelInfo
        .write_world_info(&original, temp_dir.path())
        .unwrap();

    let path_new = temp_dir.path().join("level.dat_new");
    let path_old = temp_dir.path().join(LEVEL_DAT_BACKUP_FILE_NAME);

    fs::write(&path_new, []).unwrap();
    fs::write(&path_old, []).unwrap();

    fs::set_permissions(&path_new, fs::Permissions::from_mode(0o644)).unwrap();
    fs::set_permissions(&path_old, fs::Permissions::from_mode(0o644)).unwrap();
    fs::set_permissions(temp_dir.path(), fs::Permissions::from_mode(0o555)).unwrap();

    let mut updated = original;
    updated.level_name = "Updated World".to_string();

    let result = AnvilLevelInfo.write_world_info(&updated, temp_dir.path());

    fs::set_permissions(temp_dir.path(), fs::Permissions::from_mode(0o755)).unwrap();

    let error = result.expect_err("a failed final level.dat rename must make the save fail");
    assert!(matches!(error, WorldInfoError::IoError(_)));

    let reloaded = AnvilLevelInfo.read_world_info(temp_dir.path()).unwrap();
    assert_eq!(reloaded.level_name, "Original World");
}
