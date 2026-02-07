//! Optional Lance backend — zero-copy Arrow columnar storage with DataFusion SQL.
//!
//! Enable with `--features lance-store`. Adds LanceDB (embedded, no server process),
//! Apache Arrow for zero-copy IPC, and DataFusion for SQL queries.
//!
//! # Architecture (from holograph patterns)
//!
//! ```text
//! JSON registries ──build──▶ Lance tables (columnar, versioned)
//!                                │
//!                      ┌─────────┴──────────┐
//!                      ▼                    ▼
//!              Arrow RecordBatch      DataFusion SQL
//!              (zero-copy access)     ("SELECT * FROM blocks WHERE ...")
//! ```
//!
//! # Future: GEL (Graph Execution Language)
//!
//! The Lance backend is the substrate for a future Graph Execution Language layer
//! that compiles Java imports to graph operations over Arrow storage.
//! See AdaWorldAPI/holograph for the reference implementation.

use crate::error::{StoreError, StoreResult};
use crate::traits::{BlockRecord, EntityRecord, GameDataStore, ItemRecord, RecipeRecord};

/// Lance-backed game data store.
///
/// Embedded LanceDB instance with Arrow zero-copy reads and DataFusion SQL.
/// Data is imported from JSON registries at first run, then served from
/// Lance columnar format with no serialization overhead.
pub struct LanceStore {
    // Will hold: lancedb::Database, datafusion::prelude::SessionContext
    // Stubbed for now — implementation requires downloading game data JSONs
    _path: String,
}

impl LanceStore {
    /// Open or create a Lance store at the given path.
    ///
    /// On first run, imports game data from pumpkin-data JSON sources into
    /// Lance tables. Subsequent opens are instant (memory-mapped reads).
    ///
    /// # Errors
    ///
    /// Returns `StoreError::Lance` if the database cannot be opened.
    pub async fn open(path: &str) -> StoreResult<Self> {
        // TODO: Phase 4 implementation
        // let db = lancedb::connect(path).execute().await
        //     .map_err(|e| StoreError::Lance(e.to_string()))?;
        // let ctx = datafusion::prelude::SessionContext::new();
        // Register tables: blocks, items, recipes, entities
        Ok(Self {
            _path: path.to_string(),
        })
    }

    /// Execute a SQL query over game data tables.
    ///
    /// Available tables: `blocks`, `items`, `recipes`, `entities`, `biomes`
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let results = store.sql("SELECT name, hardness FROM blocks WHERE hardness > 5.0").await?;
    /// let results = store.sql("SELECT * FROM recipes WHERE result_item = 'diamond_sword'").await?;
    /// ```
    pub async fn sql(&self, _query: &str) -> StoreResult<Vec<serde_json::Value>> {
        // TODO: Phase 4 implementation
        // let df = self.ctx.sql(query).await?;
        // let batches = df.collect().await?;
        // Convert RecordBatches to JSON rows
        Err(StoreError::Lance(
            "Lance backend not yet implemented — use toml-store for now".to_string(),
        ))
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
