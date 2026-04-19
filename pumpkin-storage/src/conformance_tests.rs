//! Behavioural tests shared by every storage-trait implementation. Each
//! backend calls into these helpers from its own `#[test]` function so
//! fixtures (temp dirs, fresh maps) stay local.

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::world_seed::Seed;
use temp_dir::TempDir;
use time::OffsetDateTime;
use uuid::Uuid;

use std::net::{IpAddr, Ipv4Addr};

use crate::banned_ip::BannedIpStorage;
use crate::banned_player::BannedPlayerStorage;
use crate::error::StorageError;
use crate::level_info::{LevelData, LevelInfoStorage};
use crate::player_data::PlayerDataStorage;
use crate::{MemoryStorage, NullStorage, VanillaStorage};

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
    let store = VanillaStorage::new(dir.path(), dir.path().join("data"));
    level_info_round_trip(&store).await;
}

#[tokio::test]
async fn vanilla_level_info_not_found_reports_not_found() {
    let dir = TempDir::new().unwrap();
    let store = VanillaStorage::new(dir.path(), dir.path().join("data"));
    let err = LevelInfoStorage::load(&store).await.unwrap_err();
    assert!(err.is_not_found(), "{err}");
    assert!(matches!(err, StorageError::NotFound { .. }));
}

#[tokio::test]
async fn level_info_null_always_empty() {
    let store = NullStorage::new();
    let err = LevelInfoStorage::load(&store).await.unwrap_err();
    assert!(err.is_not_found());
    // Save is a no-op that succeeds.
    LevelInfoStorage::save(&store, &LevelData::default(Seed(0)))
        .await
        .unwrap();
    // Still empty.
    assert!(LevelInfoStorage::load(&store).await.is_err());
}

async fn player_data_round_trip(store: &dyn PlayerDataStorage) {
    let uuid = Uuid::from_u128(0x1234_5678_90AB_CDEF_1122_3344_5566_7788);

    let err = store.load(uuid).await.expect_err("no data yet");
    assert!(err.is_not_found(), "{err}");
    assert!(store.list().await.unwrap().is_empty());

    let mut nbt = NbtCompound::new();
    nbt.put_string("name", "Alice".to_string());
    nbt.put_int("level", 7);
    store.save(uuid, &nbt).await.unwrap();

    let loaded = store.load(uuid).await.unwrap();
    assert_eq!(loaded.get_string("name").unwrap(), "Alice");
    assert_eq!(loaded.get_int("level").unwrap(), 7);

    let ids = store.list().await.unwrap();
    assert_eq!(ids, vec![uuid]);

    // Overwrite.
    let mut nbt = NbtCompound::new();
    nbt.put_int("level", 10);
    store.save(uuid, &nbt).await.unwrap();
    assert_eq!(
        store.load(uuid).await.unwrap().get_int("level").unwrap(),
        10
    );

    // Second uuid.
    let other = Uuid::from_u128(0xAA);
    store.save(other, &NbtCompound::new()).await.unwrap();
    let mut ids = store.list().await.unwrap();
    ids.sort();
    let mut expected = vec![uuid, other];
    expected.sort();
    assert_eq!(ids, expected);
}

#[tokio::test]
async fn player_data_round_trip_memory() {
    let store = MemoryStorage::new();
    player_data_round_trip(&store).await;
}

#[tokio::test]
async fn player_data_round_trip_vanilla() {
    let dir = TempDir::new().unwrap();
    let store = VanillaStorage::new(dir.path(), dir.path().join("data"));
    player_data_round_trip(&store).await;
}

#[tokio::test]
async fn player_data_null_always_empty() {
    let store = NullStorage::new();
    let uuid = Uuid::from_u128(1);
    assert!(PlayerDataStorage::load(&store, uuid).await.unwrap_err().is_not_found());
    PlayerDataStorage::save(&store, uuid, &NbtCompound::new())
        .await
        .unwrap();
    assert!(PlayerDataStorage::load(&store, uuid).await.unwrap_err().is_not_found());
    assert!(PlayerDataStorage::list(&store).await.unwrap().is_empty());
}

async fn banned_player_round_trip(store: &dyn BannedPlayerStorage) {
    let uuid = Uuid::from_u128(0xAB);
    let other = Uuid::from_u128(0xCD);

    assert!(store.list().await.unwrap().is_empty());
    assert!(!store.is_banned(uuid).await.unwrap());

    store
        .ban(
            uuid,
            "Alice",
            "Admin".to_string(),
            None,
            "spam".to_string(),
        )
        .await
        .unwrap();

    assert!(store.is_banned(uuid).await.unwrap());
    let entry = store.get(uuid).await.unwrap().unwrap();
    assert_eq!(entry.name, "Alice");
    assert_eq!(entry.reason, "spam");
    assert_eq!(entry.source, "Admin");

    // Re-banning replaces the existing entry.
    store
        .ban(
            uuid,
            "Alice",
            "Mod".to_string(),
            None,
            "grief".to_string(),
        )
        .await
        .unwrap();
    let entry = store.get(uuid).await.unwrap().unwrap();
    assert_eq!(entry.source, "Mod");
    assert_eq!(entry.reason, "grief");
    assert_eq!(store.list().await.unwrap().len(), 1);

    // Expired bans are filtered out.
    let past = OffsetDateTime::now_utc() - time::Duration::hours(1);
    store
        .ban(
            other,
            "Bob",
            "Admin".to_string(),
            Some(past),
            "old".to_string(),
        )
        .await
        .unwrap();
    assert!(!store.is_banned(other).await.unwrap());
    assert_eq!(store.list().await.unwrap().len(), 1);

    store.unban(uuid).await.unwrap();
    assert!(!store.is_banned(uuid).await.unwrap());
}

#[tokio::test]
async fn banned_player_round_trip_memory() {
    let store = MemoryStorage::new();
    banned_player_round_trip(&store).await;
}

#[tokio::test]
async fn banned_player_round_trip_vanilla() {
    let dir = TempDir::new().unwrap();
    let store = VanillaStorage::new(dir.path(), dir.path().join("data"));
    banned_player_round_trip(&store).await;
}

#[tokio::test]
async fn banned_player_null_always_empty() {
    let store = NullStorage::new();
    let uuid = Uuid::from_u128(1);
    BannedPlayerStorage::ban(
        &store,
        uuid,
        "Alice",
        "s".to_string(),
        None,
        "r".to_string(),
    )
    .await
    .unwrap();
    assert!(!BannedPlayerStorage::is_banned(&store, uuid).await.unwrap());
    assert!(BannedPlayerStorage::list(&store).await.unwrap().is_empty());
}

#[tokio::test]
async fn banned_player_vanilla_persists_across_instances() {
    let dir = TempDir::new().unwrap();
    {
        let store = VanillaStorage::new(dir.path(), dir.path().join("data"));
        BannedPlayerStorage::ban(
            &store,
            Uuid::from_u128(1),
            "Alice",
            "Admin".to_string(),
            None,
            "reason".to_string(),
        )
        .await
        .unwrap();
    }
    let store = VanillaStorage::new(dir.path(), dir.path().join("data"));
    assert_eq!(BannedPlayerStorage::list(&store).await.unwrap().len(), 1);
}

async fn banned_ip_round_trip(store: &dyn BannedIpStorage) {
    let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    assert!(store.list().await.unwrap().is_empty());
    assert!(!store.is_banned(ip).await.unwrap());

    store
        .ban(ip, "Admin".to_string(), None, "abuse".to_string())
        .await
        .unwrap();
    assert!(store.is_banned(ip).await.unwrap());
    let entry = store.get(ip).await.unwrap().unwrap();
    assert_eq!(entry.reason, "abuse");

    store.unban(ip).await.unwrap();
    assert!(!store.is_banned(ip).await.unwrap());
}

#[tokio::test]
async fn banned_ip_round_trip_memory() {
    banned_ip_round_trip(&MemoryStorage::new()).await;
}

#[tokio::test]
async fn banned_ip_round_trip_vanilla() {
    let dir = TempDir::new().unwrap();
    let store = VanillaStorage::new(dir.path(), dir.path().join("data"));
    banned_ip_round_trip(&store).await;
}

#[tokio::test]
async fn banned_ip_null_always_empty() {
    let store = NullStorage::new();
    let ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
    BannedIpStorage::ban(&store, ip, "s".to_string(), None, "r".to_string())
        .await
        .unwrap();
    assert!(!BannedIpStorage::is_banned(&store, ip).await.unwrap());
    assert!(BannedIpStorage::list(&store).await.unwrap().is_empty());
}

#[tokio::test]
async fn vanilla_level_info_writes_backup_on_load() {
    use crate::vanilla::{LEVEL_DAT_BACKUP_FILE_NAME, LEVEL_DAT_FILE_NAME};

    let dir = TempDir::new().unwrap();
    let store = VanillaStorage::new(dir.path(), dir.path().join("data"));
    LevelInfoStorage::save(&store, &LevelData::default(Seed(1)))
        .await
        .unwrap();

    assert!(dir.path().join(LEVEL_DAT_FILE_NAME).exists());
    assert!(!dir.path().join(LEVEL_DAT_BACKUP_FILE_NAME).exists());

    LevelInfoStorage::load(&store).await.unwrap();
    assert!(dir.path().join(LEVEL_DAT_BACKUP_FILE_NAME).exists());
}
