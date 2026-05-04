//! Behavioural tests shared by every storage-trait implementation. Each
//! backend calls into these helpers from its own `#[test]` function so
//! fixtures (temp dirs, fresh maps) stay local.

use pumpkin_util::world_seed::Seed;
use temp_dir::TempDir;

use crate::world_info::{LevelData, WorldInfoStorage};
use crate::{MemoryStorage, VanillaStorage};

async fn world_info_round_trip(store: &dyn WorldInfoStorage) {
    let err = store
        .load()
        .await
        .expect_err("empty store must report not found");
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
async fn world_info_round_trip_memory() {
    let store = MemoryStorage::new();
    world_info_round_trip(&store).await;
}

#[tokio::test]
async fn world_info_round_trip_vanilla() {
    let dir = TempDir::new().unwrap();
    let store = VanillaStorage::new(dir.path());
    world_info_round_trip(&store).await;
}

#[tokio::test]
async fn vanilla_world_info_not_found_reports_not_found() {
    let dir = TempDir::new().unwrap();
    let store = VanillaStorage::new(dir.path());
    let err = WorldInfoStorage::load(&store).await.unwrap_err();
    assert!(err.is_not_found(), "{err}");
}
