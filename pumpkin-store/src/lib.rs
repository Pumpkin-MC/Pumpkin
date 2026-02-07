//! # pumpkin-store — Pluggable Game Data Storage
//!
//! Abstracts game data access behind the [`GameDataStore`] trait with two backends:
//!
//! - **`toml-store`** (default): Wraps `pumpkin-data` static arrays. Zero runtime cost,
//!   zero new dependencies. Developers see no difference from using pumpkin-data directly.
//!
//! - **`lance-store`** (opt-in): Embedded `LanceDB` with zero-copy Arrow IPC and `DataFusion` SQL.
//!   Enable with `--features lance-store`. Adds ~10MB to binary but unlocks SQL queries,
//!   columnar scans, and Arrow-native data sharing between subsystems.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use pumpkin_store::{GameDataStore, open_default_store};
//!
//! let store = open_default_store();
//! let block = store.block_by_state_id(1);
//! let item = store.item_by_name("diamond_sword");
//! ```
//!
//! ## Architecture
//!
//! ```text
//! ┌────────────────────────────────────┐
//! │        GameDataStore trait          │
//! │  block_by_*  item_by_*  recipes()  │
//! ├──────────────┬─────────────────────┤
//! │  StaticStore │    LanceStore       │
//! │  (default)   │  (--features lance) │
//! │  pumpkin-data│  Arrow + DataFusion │
//! └──────────────┴─────────────────────┘
//! ```
//!
//! ## Future: GEL (Graph Execution Language)
//!
//! The Lance backend is the substrate for a future Graph Execution Language layer
//! that compiles Java imports (e.g. Bukkit API calls) to graph operations over
//! Arrow columnar storage. See ARCH-020 and holograph (AdaWorldAPI/holograph).

mod error;
mod traits;

#[cfg(feature = "toml-store")]
mod static_store;

#[cfg(feature = "lance-store")]
mod lance_store;

pub use error::{StoreError, StoreResult};
pub use traits::{BlockRecord, EntityRecord, GameDataStore, ItemRecord, RecipeRecord};

#[cfg(feature = "toml-store")]
pub use static_store::StaticStore;

#[cfg(feature = "lance-store")]
pub use lance_store::LanceStore;

/// Open the default store based on enabled features.
///
/// - With `toml-store` (default): returns `StaticStore` wrapping pumpkin-data.
/// - With `lance-store`: returns `LanceStore` (caller must provide path).
///
/// This is the zero-config entry point. For Lance, use `LanceStore::open()` directly.
#[cfg(feature = "toml-store")]
#[must_use]
pub const fn open_default_store() -> StaticStore {
    StaticStore::new()
}
