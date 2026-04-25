//! Behavioural tests shared by every storage-trait implementation. Each
//! backend calls into these helpers from its own `#[test]` function so
//! fixtures (temp dirs, fresh maps) stay local.

use pumpkin_nbt::pnbt::PNbtCompound;
use pumpkin_util::world_seed::Seed;
use temp_dir::TempDir;
use time::OffsetDateTime;
use uuid::Uuid;

use std::net::{IpAddr, Ipv4Addr};

use pumpkin_util::permission::PermissionLvl;

use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;

use crate::banned_ip::BannedIpStorage;
use crate::banned_player::BannedPlayerStorage;
use crate::chunk::{ChunkStorage, LoadedData};
use crate::error::StorageError;
use crate::level_info::{LevelData, LevelInfoStorage};
use crate::memory::MemoryChunkStorage;
use crate::op::OpStorage;
use crate::player_data::PlayerDataStorage;
use crate::poi::{POI_TYPE_NETHER_PORTAL, PoiStorage};
use crate::user_cache::UserCacheStorage;
use crate::whitelist::WhitelistStorage;
use crate::{MemoryStorage, NullStorage, VanillaStorage};

async fn level_info_round_trip(store: &dyn LevelInfoStorage) {
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

    let mut nbt = PNbtCompound::new();
    nbt.put_string("Alice");
    nbt.put_i32(7);
    store.save(uuid, &nbt).await.unwrap();

    let mut loaded = store.load(uuid).await.unwrap();
    assert_eq!(loaded.get_string().unwrap(), "Alice");
    assert_eq!(loaded.get_i32().unwrap(), 7);

    let ids = store.list().await.unwrap();
    assert_eq!(ids, vec![uuid]);

    // Overwrite.
    let mut nbt = PNbtCompound::new();
    nbt.put_i32(10);
    store.save(uuid, &nbt).await.unwrap();
    assert_eq!(store.load(uuid).await.unwrap().get_i32().unwrap(), 10);

    // Second uuid.
    let other = Uuid::from_u128(0xAA);
    store.save(other, &PNbtCompound::new()).await.unwrap();
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
    assert!(
        PlayerDataStorage::load(&store, uuid)
            .await
            .unwrap_err()
            .is_not_found()
    );
    PlayerDataStorage::save(&store, uuid, &PNbtCompound::new())
        .await
        .unwrap();
    assert!(
        PlayerDataStorage::load(&store, uuid)
            .await
            .unwrap_err()
            .is_not_found()
    );
    assert!(PlayerDataStorage::list(&store).await.unwrap().is_empty());
}

async fn banned_player_round_trip(store: &dyn BannedPlayerStorage) {
    let uuid = Uuid::from_u128(0xAB);
    let other = Uuid::from_u128(0xCD);

    assert!(store.list().await.unwrap().is_empty());
    assert!(!store.is_banned(uuid).await.unwrap());

    store
        .ban(uuid, "Alice", "Admin".to_string(), None, "spam".to_string())
        .await
        .unwrap();

    assert!(store.is_banned(uuid).await.unwrap());
    let entry = store.get(uuid).await.unwrap().unwrap();
    assert_eq!(entry.name, "Alice");
    assert_eq!(entry.reason, "spam");
    assert_eq!(entry.source, "Admin");

    // Re-banning replaces the existing entry.
    store
        .ban(uuid, "Alice", "Mod".to_string(), None, "grief".to_string())
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
#[allow(clippy::semicolon_outside_block)]
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

async fn op_round_trip(store: &dyn OpStorage) {
    let uuid = Uuid::from_u128(0xEE);
    assert!(store.list().await.unwrap().is_empty());
    assert!(!store.is_op(uuid).await.unwrap());

    store
        .op(uuid, "Alice", PermissionLvl::Four, false)
        .await
        .unwrap();
    assert!(store.is_op(uuid).await.unwrap());
    let op = store.get(uuid).await.unwrap().unwrap();
    assert_eq!(op.name, "Alice");
    assert_eq!(op.level, PermissionLvl::Four);

    store.deop(uuid).await.unwrap();
    assert!(!store.is_op(uuid).await.unwrap());
}

#[tokio::test]
async fn op_round_trip_memory() {
    op_round_trip(&MemoryStorage::new()).await;
}

#[tokio::test]
async fn op_round_trip_vanilla() {
    let dir = TempDir::new().unwrap();
    let store = VanillaStorage::new(dir.path(), dir.path().join("data"));
    op_round_trip(&store).await;
}

#[tokio::test]
async fn op_null_always_empty() {
    let store = NullStorage::new();
    OpStorage::op(
        &store,
        Uuid::from_u128(1),
        "Alice",
        PermissionLvl::Four,
        false,
    )
    .await
    .unwrap();
    assert!(!OpStorage::is_op(&store, Uuid::from_u128(1)).await.unwrap());
    assert!(OpStorage::list(&store).await.unwrap().is_empty());
}

async fn whitelist_round_trip(store: &dyn WhitelistStorage) {
    let uuid = Uuid::from_u128(0x99);
    assert!(store.list().await.unwrap().is_empty());
    assert!(!store.is_whitelisted(uuid).await.unwrap());
    assert!(store.get(uuid).await.unwrap().is_none());

    store.add(uuid, "Alice").await.unwrap();
    assert!(store.is_whitelisted(uuid).await.unwrap());
    assert_eq!(store.list().await.unwrap().len(), 1);
    let entry = store.get(uuid).await.unwrap().unwrap();
    assert_eq!(entry.name, "Alice");

    store.remove(uuid).await.unwrap();
    assert!(!store.is_whitelisted(uuid).await.unwrap());
    assert!(store.get(uuid).await.unwrap().is_none());
}

#[tokio::test]
async fn whitelist_round_trip_memory() {
    whitelist_round_trip(&MemoryStorage::new()).await;
}

#[tokio::test]
async fn whitelist_round_trip_vanilla() {
    let dir = TempDir::new().unwrap();
    let store = VanillaStorage::new(dir.path(), dir.path().join("data"));
    whitelist_round_trip(&store).await;
}

#[tokio::test]
async fn whitelist_null_always_empty() {
    let store = NullStorage::new();
    WhitelistStorage::add(&store, Uuid::from_u128(1), "Alice")
        .await
        .unwrap();
    assert!(
        !WhitelistStorage::is_whitelisted(&store, Uuid::from_u128(1))
            .await
            .unwrap()
    );
    assert!(WhitelistStorage::list(&store).await.unwrap().is_empty());
}

async fn user_cache_round_trip(store: &dyn UserCacheStorage) {
    let uuid = Uuid::from_u128(0xBEEF);
    assert!(store.get_by_uuid(uuid).await.unwrap().is_none());
    assert!(store.get_by_name("alice").await.unwrap().is_none());

    store.upsert(uuid, "Alice").await.unwrap();

    let by_u = store.get_by_uuid(uuid).await.unwrap().unwrap();
    assert_eq!(by_u.name, "Alice");
    assert_eq!(by_u.uuid, uuid);

    // Case-insensitive name lookup.
    let by_n = store.get_by_name("ALICE").await.unwrap().unwrap();
    assert_eq!(by_n.uuid, uuid);

    // Re-upsert overwrites name.
    store.upsert(uuid, "Alice2").await.unwrap();
    assert_eq!(
        store.get_by_uuid(uuid).await.unwrap().unwrap().name,
        "Alice2"
    );
}

#[tokio::test]
async fn user_cache_round_trip_memory() {
    user_cache_round_trip(&MemoryStorage::new()).await;
}

#[tokio::test]
async fn user_cache_round_trip_vanilla() {
    let dir = TempDir::new().unwrap();
    let store = VanillaStorage::new(dir.path(), dir.path().join("data"));
    user_cache_round_trip(&store).await;
}

#[tokio::test]
async fn user_cache_null_returns_none() {
    let store = NullStorage::new();
    UserCacheStorage::upsert(&store, Uuid::from_u128(1), "Alice")
        .await
        .unwrap();
    assert!(
        UserCacheStorage::get_by_uuid(&store, Uuid::from_u128(1))
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        UserCacheStorage::get_by_name(&store, "Alice")
            .await
            .unwrap()
            .is_none()
    );
}

async fn poi_round_trip(store: &dyn PoiStorage) {
    let pos_a = BlockPos(Vector3::new(100, 64, 100));
    let pos_b = BlockPos(Vector3::new(110, 64, 100));
    let pos_far = BlockPos(Vector3::new(1000, 64, 1000));

    store.add(pos_a, POI_TYPE_NETHER_PORTAL).await.unwrap();
    store.add(pos_b, POI_TYPE_NETHER_PORTAL).await.unwrap();
    store.add(pos_far, POI_TYPE_NETHER_PORTAL).await.unwrap();

    let near = store
        .get_in_square(
            BlockPos(Vector3::new(105, 64, 100)),
            16,
            Some(POI_TYPE_NETHER_PORTAL),
        )
        .await
        .unwrap();
    assert_eq!(near.len(), 2);

    assert!(store.remove(pos_a).await.unwrap());
    let near = store
        .get_in_square(
            BlockPos(Vector3::new(105, 64, 100)),
            16,
            Some(POI_TYPE_NETHER_PORTAL),
        )
        .await
        .unwrap();
    assert_eq!(near.len(), 1);
}

#[tokio::test]
async fn poi_round_trip_memory() {
    poi_round_trip(&MemoryStorage::new()).await;
}

#[tokio::test]
async fn poi_round_trip_vanilla() {
    let dir = TempDir::new().unwrap();
    let store = VanillaStorage::new(dir.path(), dir.path().join("data"));
    poi_round_trip(&store).await;
}

#[tokio::test]
#[allow(clippy::semicolon_outside_block)]
async fn poi_vanilla_persists_across_instances() {
    let dir = TempDir::new().unwrap();
    {
        let store = VanillaStorage::new(dir.path(), dir.path().join("data"));
        PoiStorage::add(
            &store,
            BlockPos(Vector3::new(5, 64, 5)),
            POI_TYPE_NETHER_PORTAL,
        )
        .await
        .unwrap();
        store.save_all().await.unwrap();
    }
    let store = VanillaStorage::new(dir.path(), dir.path().join("data"));
    let results = store
        .get_in_square(
            BlockPos(Vector3::new(0, 64, 0)),
            32,
            Some(POI_TYPE_NETHER_PORTAL),
        )
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn poi_null_always_empty() {
    let store = NullStorage::new();
    PoiStorage::add(
        &store,
        BlockPos(Vector3::new(0, 0, 0)),
        POI_TYPE_NETHER_PORTAL,
    )
    .await
    .unwrap();
    assert!(
        PoiStorage::get_in_square(&store, BlockPos(Vector3::new(0, 0, 0)), 100, None)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn chunk_round_trip_memory() {
    let store: MemoryChunkStorage<i32> = MemoryChunkStorage::new();
    let a = Vector2::new(0, 0);
    let b = Vector2::new(1, 2);

    store.save_chunks(vec![(a, 42), (b, 7)]).await.unwrap();

    let (tx, mut rx) = tokio::sync::mpsc::channel(4);
    store.fetch_chunks(&[a, b, Vector2::new(99, 99)], tx).await;

    let mut hits = std::collections::HashSet::new();
    let mut missing = Vec::new();
    while let Some(msg) = rx.recv().await {
        match msg {
            LoadedData::Loaded(v) => {
                hits.insert(v);
            }
            LoadedData::Missing(p) => missing.push(p),
            LoadedData::Error { error, .. } => panic!("unexpected error: {error}"),
        }
    }
    assert!(hits.contains(&42));
    assert!(hits.contains(&7));
    assert_eq!(missing, vec![Vector2::new(99, 99)]);
}

#[tokio::test]
async fn chunk_null_always_missing() {
    use crate::NullStorage;
    let store = NullStorage::new();
    let pos = Vector2::new(3, 4);
    ChunkStorage::<i32>::save_chunks(&store, vec![(pos, 1)])
        .await
        .unwrap();
    let (tx, mut rx) = tokio::sync::mpsc::channel(4);
    ChunkStorage::<i32>::fetch_chunks(&store, &[pos], tx).await;
    match rx.recv().await.unwrap() {
        LoadedData::Missing(p) => assert_eq!(p, pos),
        _ => panic!("expected missing"),
    }
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
