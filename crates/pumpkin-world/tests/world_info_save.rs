#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::fs::{self, File};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use pumpkin_nbt::{
    NbtCompound,
    nbt_compress::{read_gzip_compound_tag, write_gzip_compound_tag},
    tag::NbtTag,
};
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::world_seed::Seed;
use pumpkin_world::world_info::{
    LevelData, WorldInfoError, WorldInfoReader, WorldInfoWriter,
    anvil::{AnvilLevelInfo, LEVEL_DAT_BACKUP_FILE_NAME},
    data_files::{read_forced_chunks, write_forced_chunks},
};
use tempfile::TempDir;

#[test]
fn forced_chunks_survive_save_and_remain_dimension_specific() {
    let temp_dir = TempDir::new().unwrap();
    let overworld = temp_dir.path().join("overworld");
    let nether = temp_dir.path().join("nether");
    let initial = [Vector2::new(-3, 7), Vector2::new(0, 0)];

    assert!(read_forced_chunks(&overworld).is_empty());
    write_forced_chunks(&overworld, initial).unwrap();
    write_forced_chunks(&nether, [Vector2::new(9, -4)]).unwrap();
    assert!(overworld.join("data/minecraft/chunk_tickets.dat").is_file());
    assert_eq!(read_forced_chunks(&overworld), initial);
    assert_eq!(read_forced_chunks(&nether), [Vector2::new(9, -4)]);

    write_forced_chunks(&overworld, [Vector2::new(1, 2)]).unwrap();
    assert_eq!(read_forced_chunks(&overworld), [Vector2::new(1, 2)]);
    assert_eq!(read_forced_chunks(&nether), [Vector2::new(9, -4)]);
}

#[test]
fn vanilla_ticket_storage_preserves_other_persistent_tickets() {
    let temp_dir = TempDir::new().unwrap();
    let dimension = temp_dir.path();
    let data_dir = dimension.join("data/minecraft");
    fs::create_dir_all(&data_dir).unwrap();
    let path = data_dir.join("chunk_tickets.dat");

    let mut forced = NbtCompound::new();
    forced.put("chunk_pos", NbtTag::IntArray(vec![-3, 7]));
    forced.put_string("type", "minecraft:forced".to_string());
    forced.put_int("level", 31);

    let mut portal = NbtCompound::new();
    portal.put("chunk_pos", NbtTag::IntArray(vec![9, -4]));
    portal.put_string("type", "minecraft:portal".to_string());
    portal.put_int("level", 31);
    portal.put_long("ticks_left", 120);

    let mut data = NbtCompound::new();
    data.put_list(
        "tickets",
        vec![NbtTag::Compound(forced), NbtTag::Compound(portal.clone())],
    );
    data.put_string("extra", "keep data".to_string());
    let mut root = NbtCompound::new();
    root.put_int("DataVersion", 5023);
    root.put_string("extra", "keep root".to_string());
    root.put_compound("data", data);
    write_gzip_compound_tag(root, File::create(&path).unwrap()).unwrap();

    assert_eq!(read_forced_chunks(dimension), [Vector2::new(-3, 7)]);
    write_forced_chunks(dimension, [Vector2::new(1, 2)]).unwrap();

    let saved = read_gzip_compound_tag(File::open(&path).unwrap()).unwrap();
    assert_eq!(saved.get_int("DataVersion"), Some(5023));
    assert_eq!(saved.get_string("extra"), Some("keep root"));
    let data = saved.get_compound("data").unwrap();
    assert_eq!(data.get_string("extra"), Some("keep data"));
    let tickets = data.get_list("tickets").unwrap();
    assert_eq!(tickets.len(), 2);
    assert_eq!(tickets[0], NbtTag::Compound(portal));
    let forced = tickets[1].extract_compound().unwrap();
    assert_eq!(forced.get_int_array("chunk_pos"), Some([1, 2].as_slice()));
    assert_eq!(forced.get_string("type"), Some("minecraft:forced"));
    assert_eq!(forced.get_int("level"), Some(31));
}

#[test]
fn legacy_forced_chunks_migrate_once_to_vanilla_storage() {
    let temp_dir = TempDir::new().unwrap();
    let dimension = temp_dir.path();
    let data_dir = dimension.join("data/minecraft");
    fs::create_dir_all(&data_dir).unwrap();
    let old_path = data_dir.join("forced_chunks.dat");
    let new_path = data_dir.join("chunk_tickets.dat");
    let mut legacy = NbtCompound::new();
    legacy.put_list("chunks", vec![NbtTag::IntArray(vec![-3, 7])]);
    write_gzip_compound_tag(legacy, File::create(&old_path).unwrap()).unwrap();

    assert_eq!(read_forced_chunks(dimension), [Vector2::new(-3, 7)]);
    assert!(!old_path.exists());
    assert!(new_path.is_file());
    assert_eq!(read_forced_chunks(dimension), [Vector2::new(-3, 7)]);
}

#[test]
fn unreadable_legacy_forced_chunks_are_left_for_recovery() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path().join("data/minecraft");
    fs::create_dir_all(&data_dir).unwrap();
    let old_path = data_dir.join("forced_chunks.dat");
    fs::write(&old_path, b"invalid NBT").unwrap();

    assert!(read_forced_chunks(temp_dir.path()).is_empty());
    assert!(old_path.is_file());
    assert!(!data_dir.join("chunk_tickets.dat").exists());
}

#[test]
fn unreadable_ticket_storage_is_not_overwritten() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path().join("data/minecraft");
    fs::create_dir_all(&data_dir).unwrap();
    let path = data_dir.join("chunk_tickets.dat");
    fs::write(&path, b"invalid NBT").unwrap();

    assert!(write_forced_chunks(temp_dir.path(), [Vector2::new(1, 2)]).is_err());
    assert_eq!(fs::read(path).unwrap(), b"invalid NBT");
}

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
