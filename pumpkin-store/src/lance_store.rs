//! Optional Lance backend — zero-copy Arrow columnar storage.
//!
//! Enable with `--features lance-store`. Hydrates Lance tables FROM the default
//! [`StaticStore`](crate::StaticStore) — no external JSON files needed.
//!
//! # Architecture
//!
//! ```text
//! StaticStore (pumpkin-data, compile-time)
//!       │ hydrate_from()
//!       ▼
//! LanceStore (Arrow columnar, zero-copy reads)
//!       │
//!       └── lance 2.0 native queries (no `DataFusion` sidecar needed)
//! ```
//!
//! ## Lance 2.0 Note
//!
//! Lance 2.0 (released 2026-02-06) includes its own query engine — `DataFusion`
//! is NOT required as a sidecar. Lance 2.0 handles filtering, projection, and
//! scanning natively. This simplifies deps significantly.
//!
//! ## Hydration Flow
//!
//! 1. `LanceStore::open(path)` — opens or creates a Lance database at `path`
//! 2. `store.hydrate_from(&static_store)` — reads all records from `StaticStore`
//!    and writes them into Lance tables (blocks, items, entities)
//! 3. Subsequent reads are zero-copy Arrow — no serialization overhead
//!
//! Hydration only happens once. If the Lance tables already exist, `open()` skips it.

use crate::error::{StoreError, StoreResult};
use crate::traits::{BlockRecord, EntityRecord, GameDataStore, ItemRecord, RecipeRecord};

/// Lance-backed game data store.
///
/// Hydrates from any [`GameDataStore`] (typically [`StaticStore`](crate::StaticStore))
/// then serves zero-copy Arrow reads. Lance 2.0 provides native queries —
/// no `DataFusion` sidecar needed.
pub struct LanceStore {
    // Will hold: lancedb::Connection (Lance 2.0 API)
    // Stubbed for now — real deps added when chrono conflict resolves
    #[allow(dead_code)] // Used in Phase 4 for Lance 2.0 connection path
    path: String,
    hydrated: bool,
}

impl LanceStore {
    /// Open or create a Lance store at the given path.
    ///
    /// On first run, the store is empty — call [`hydrate_from`](Self::hydrate_from)
    /// to populate from a [`StaticStore`](crate::StaticStore). Subsequent opens
    /// detect existing tables and skip hydration.
    ///
    /// # Errors
    ///
    /// Returns `StoreError::Lance` if the database cannot be opened.
    #[allow(clippy::unused_async)] // Will await Lance 2.0 connection in Phase 4
    pub async fn open(path: &str) -> StoreResult<Self> {
        // TODO: Phase 4 — real Lance 2.0 connection
        // let db = lancedb::connect(path).execute().await
        //     .map_err(|e| StoreError::Lance(e.to_string()))?;
        // let has_tables = db.table_names().await.map(|t| !t.is_empty()).unwrap_or(false);
        Ok(Self {
            path: path.to_string(),
            hydrated: false,
        })
    }

    /// Hydrate Lance tables from a source [`GameDataStore`].
    ///
    /// Reads all blocks, items, and entities from the source store and writes
    /// them into Lance columnar tables. This is a one-time operation — if tables
    /// already exist, this is a no-op.
    ///
    /// Typically called with `StaticStore` as the source:
    /// ```rust,ignore
    /// let static_store = StaticStore::new();
    /// let mut lance = LanceStore::open("./data/lance").await?;
    /// lance.hydrate_from(&static_store).await?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `StoreError::Lance` if hydration fails.
    #[allow(clippy::unused_async)] // Will await Lance 2.0 table creation in Phase 4
    pub async fn hydrate_from(&mut self, _source: &dyn GameDataStore) -> StoreResult<()> {
        if self.hydrated {
            return Ok(());
        }

        // TODO: Phase 4 — iterate source store, build Arrow RecordBatches,
        // write to Lance tables:
        //
        // let blocks: Vec<BlockRecord> = (0..source.block_count())
        //     .filter_map(|id| source.block_by_id(id as u16).ok())
        //     .collect();
        //
        // let batch = blocks_to_record_batch(&blocks)?;
        // db.create_table("blocks", batch).execute().await?;
        //
        // Similarly for items, entities, recipes.

        self.hydrated = true;
        Ok(())
    }

    /// Query Lance tables directly using Lance 2.0's native query API.
    ///
    /// Lance 2.0 supports filtering, projection, and full-text search natively
    /// without requiring a `DataFusion` sidecar.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Lance 2.0 native query (not SQL — lance's own API)
    /// let results = store.query("blocks")
    ///     .filter("hardness > 5.0")
    ///     .select(&["name", "hardness"])
    ///     .execute().await?;
    /// ```
    #[allow(clippy::unused_async)] // Will await Lance 2.0 queries in Phase 4
    pub async fn query_table(
        &self,
        _table: &str,
        _filter: Option<&str>,
    ) -> StoreResult<Vec<serde_json::Value>> {
        // TODO: Phase 4 — Lance 2.0 native query API
        // let table = self.db.open_table(table).execute().await?;
        // let mut query = table.query();
        // if let Some(f) = filter {
        //     query = query.filter(f);
        // }
        // let batches = query.execute().await?;
        // Convert Arrow RecordBatches to JSON
        Err(StoreError::Lance(
            "Lance backend not yet implemented — use toml-store for now".to_string(),
        ))
    }

    /// Check whether this store has been hydrated with data.
    #[must_use]
    pub const fn is_hydrated(&self) -> bool {
        self.hydrated
    }
}

impl GameDataStore for LanceStore {
    fn block_by_id(&self, _id: u16) -> StoreResult<BlockRecord> {
        Err(StoreError::Lance("not yet implemented".to_string()))
    }

    fn block_by_name(&self, _name: &str) -> StoreResult<BlockRecord> {
        Err(StoreError::Lance("not yet implemented".to_string()))
    }

    fn block_by_state_id(&self, _state_id: u16) -> StoreResult<BlockRecord> {
        Err(StoreError::Lance("not yet implemented".to_string()))
    }

    fn block_count(&self) -> usize {
        0
    }

    fn item_by_name(&self, _name: &str) -> StoreResult<ItemRecord> {
        Err(StoreError::Lance("not yet implemented".to_string()))
    }

    fn item_count(&self) -> usize {
        0
    }

    fn entity_by_name(&self, _name: &str) -> StoreResult<EntityRecord> {
        Err(StoreError::Lance("not yet implemented".to_string()))
    }

    fn entity_count(&self) -> usize {
        0
    }

    fn recipes_for_output(&self, _item_name: &str) -> StoreResult<Vec<RecipeRecord>> {
        Err(StoreError::Lance("not yet implemented".to_string()))
    }

    fn recipe_count(&self) -> usize {
        0
    }
}
