//! Behavioural tests shared by every [`LevelInfoStorage`] (and future domain)
//! implementation. Each backend calls into these helpers from its own
//! `#[test]` function so fixtures (temp dirs, fresh maps) stay local.

use pumpkin_util::world_seed::Seed;
use temp_dir::TempDir;

use crate::error::StorageError;
use crate::level_info::{LevelData, LevelInfoStorage};
use crate::{MemoryStorage, VanillaStorage};

async fn level_info_round_trip(store: &dyn LevelInfoStorage) {
    let err = store.load().await.expect_err("empty store must report not found");
    assert!(err.is_not_found(), "unexpected error: {err}");

    let mut data = LevelData::default(Seed(42));
    data.spawn_x = 123;
    data.spawn_z = -45;

    store.save(&data).await.expect("save");
    let loaded = store.load().await.expect("load");
    assert_eq!(loaded.spawn_x, 123);
    assert_eq!(loaded.spawn_z, -45);
    assert_eq!(loaded.world_gen_settings.seed, data.world_gen_settings.seed);

    // Overwrite.
    data.spawn_x = 7;
    store.save(&data).await.expect("overwrite");
    let loaded = store.load().await.expect("reload");
    assert_eq!(loaded.spawn_x, 7);
}

#[tokio::test]
async fn level_info_round_trip_memory() {
    let store = MemoryStorage::new();
    level_info_round_trip(&store).await;
}

#[tokio::test]
async fn level_info_round_trip_vanilla() {
    let dir = TempDir::new().unwrap();
    let store = VanillaStorage::new(dir.path());
    level_info_round_trip(&store).await;
}

#[tokio::test]
async fn vanilla_level_info_not_found_reports_not_found() {
    let dir = TempDir::new().unwrap();
    let store = VanillaStorage::new(dir.path());
    let err = store.load().await.unwrap_err();
    assert!(err.is_not_found(), "{err}");
    assert!(matches!(err, StorageError::NotFound { .. }));
}

#[tokio::test]
async fn vanilla_level_info_writes_backup_on_load() {
    use crate::vanilla::{LEVEL_DAT_BACKUP_FILE_NAME, LEVEL_DAT_FILE_NAME};

    let dir = TempDir::new().unwrap();
    let store = VanillaStorage::new(dir.path());
    store.save(&LevelData::default(Seed(1))).await.unwrap();

    assert!(dir.path().join(LEVEL_DAT_FILE_NAME).exists());
    assert!(!dir.path().join(LEVEL_DAT_BACKUP_FILE_NAME).exists());

    store.load().await.unwrap();
    assert!(dir.path().join(LEVEL_DAT_BACKUP_FILE_NAME).exists());
}
