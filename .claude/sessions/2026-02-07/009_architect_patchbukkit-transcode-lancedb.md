# Session 009: PatchBukkit Transcode + Storage DTO + LanceDB

**Agent:** Architect
**Date:** 2026-02-07
**Branch:** claude/architect-setup-LkWIY

## Preamble

Continuing from session where orchestration infrastructure (broadcast, task dispatch, cron.py) was built. Human operator proposed a two-pronged architecture:

1. **Transcode PatchBukkit to Rust** — harvest Bukkit API knowledge from the Java bridge repo, encode as pure Rust DTOs
2. **Storage DTO with LanceDB** — pluggable backend: TOML/YAML (human-editable) or LanceDB (zero-copy Arrow, embedded columnar DB)

## Context: PatchBukkit Investigation

PatchBukkit (https://github.com/AdaWorldAPI/PatchBukkit) is a Rust cdylib Pumpkin plugin that:
- Embeds a JVM via j4rs on a dedicated thread
- Bridges events via Protocol Buffers (protobuf FFI)
- Currently only `PlayerJoinEvent` is wired
- Has `PatchBukkitEvent` trait: `to_payload()` + `apply_modifications()`
- Java side has Bukkit API shim classes (org.bukkit.*)
- 9 gRPC RPCs in NativeBridge service

## Architecture: Storage DTO + LanceDB

### The Problem

Pumpkin currently generates 37+ Rust source files at build time from JSON registries via `pumpkin-data/build.rs`. This works but:
- All data is baked into the binary at compile time (no runtime flexibility)
- No query capability beyond array iteration
- No zero-copy sharing between subsystems
- Adding new data sources requires build.rs changes
- Plugins can't query game data with SQL-like syntax

### The Proposal

```
┌─────────────────────────────────────────────────────────────┐
│                    GameDataStore trait                       │
│  blocks() items() recipes() entities() biomes() query()     │
├──────────────────────┬──────────────────────────────────────┤
│   TomlYamlBackend    │         LanceBackend                 │
│   (human-editable)   │   (zero-copy, columnar, SQL)         │
│                      │                                      │
│  ┌──────────────┐    │   ┌──────────────┐                   │
│  │ config.toml  │    │   │  LanceDB     │                   │
│  │ recipes.yaml │    │   │  (embedded)  │                   │
│  │ blocks.toml  │    │   │              │                   │
│  └──────────────┘    │   │  Arrow IPC   │──→ zero-copy      │
│                      │   │  DataFusion  │──→ SQL queries    │
│                      │   └──────────────┘                   │
└──────────────────────┴──────────────────────────────────────┘
```

### Trait Sketch

```rust
// pumpkin-store/src/lib.rs (or pumpkin-util/)

#[async_trait]
pub trait GameDataStore: Send + Sync + 'static {
    /// Block registry access
    async fn get_block(&self, state_id: u16) -> Option<BlockData>;
    async fn get_block_by_name(&self, name: &str) -> Option<BlockData>;

    /// Item registry access
    async fn get_item(&self, id: u16) -> Option<ItemData>;

    /// Recipe queries
    async fn get_recipes_for_input(&self, item_id: u16) -> Vec<RecipeData>;
    async fn get_recipes_by_type(&self, recipe_type: &str) -> Vec<RecipeData>;

    /// Entity registry
    async fn get_entity(&self, type_id: u16) -> Option<EntityData>;

    /// Raw SQL query (DataFusion-backed for LanceDB, in-memory for TOML)
    async fn query(&self, sql: &str) -> Result<Vec<Row>>;

    /// Bulk export as Arrow RecordBatch (zero-copy for Lance, materialized for TOML)
    async fn export_arrow(&self, table: &str) -> Result<RecordBatch>;
}
```

### DTO Structs (transcoded from PatchBukkit protos)

```rust
// Derived from PatchBukkit's proto/patchbukkit/bridge.proto

/// Player abilities — maps to proto NativeBridge.GetAbilities/SetAbilities
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerAbilitiesDto {
    pub invulnerable: bool,
    pub flying: bool,
    pub allow_flying: bool,
    pub instant_break: bool,
    pub fly_speed: f32,
    pub walk_speed: f32,
}

/// Location — maps to proto NativeBridge.GetLocation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LocationDto {
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

/// Event call — maps to proto NativeBridge.CallEvent
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventCallDto {
    pub event_type: String,
    pub payload: Vec<u8>,     // serialized event data
    pub cancelled: bool,
}
```

### LanceDB Integration

```rust
// Feature-gated: --features lance-store

use lancedb::connect;
use arrow::record_batch::RecordBatch;
use datafusion::prelude::*;

pub struct LanceGameDataStore {
    db: lancedb::Database,
    ctx: SessionContext,  // DataFusion query engine
}

impl LanceGameDataStore {
    pub async fn open(path: &str) -> Result<Self> {
        let db = connect(path).execute().await?;
        let ctx = SessionContext::new();
        // Register tables: blocks, items, recipes, entities, biomes
        // Each table imported from JSON at first run, then zero-copy reads
        Ok(Self { db, ctx })
    }
}
```

### Migration Path

| Phase | Action | Effort |
|-------|--------|--------|
| 1 | Define `GameDataStore` trait in new `pumpkin-store/` crate | ~200 lines |
| 2 | Implement `TomlStore` wrapping current pumpkin-data arrays | ~400 lines |
| 3 | Import PatchBukkit proto → Rust DTO structs | ~300 lines |
| 4 | Feature-gated `LanceStore` backend | ~600 lines |
| 5 | Wire `GameDataStore` into PumpkinServer startup | ~100 lines |
| 6 | Plugin API: expose `query()` to Rust plugins | ~200 lines |

### Crate Dependencies

```toml
# pumpkin-store/Cargo.toml (new crate)
[dependencies]
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

[dependencies.toml]
version = "0.8"

[dependencies.lancedb]
version = "0.21"
optional = true

[dependencies.arrow]
version = "55"
optional = true

[dependencies.datafusion]
version = "51"
optional = true

[features]
default = ["toml-store"]
toml-store = ["toml"]
lance-store = ["lancedb", "arrow", "datafusion"]
```

## PatchBukkit Transcode Strategy

### What to harvest from PatchBukkit

1. **Proto definitions** → Rust DTO structs (serde + optional Arrow schema)
2. **Event mapping catalog** → Which Bukkit events map to which Pumpkin events
3. **API surface knowledge** → What methods Java plugins expect (Player, World, Block, ItemStack)
4. **Plugin lifecycle** → onEnable/onDisable/onLoad → Rust Plugin trait methods

### What stays Java (PatchBukkit keeps its purpose)

PatchBukkit remains the bridge for **actual Java Bukkit plugins** (.jar files). The transcode doesn't replace PatchBukkit — it harvests its knowledge to make Pumpkin's native Rust API Bukkit-compatible at the type level. Both can coexist:

- **Pure Rust server:** Uses `GameDataStore` + native plugin API (no JVM)
- **Java compat server:** Also loads PatchBukkit plugin which embeds JVM for .jar plugins

## Decisions Made

- **ARCH-020:** PatchBukkit transcode + Storage DTO + LanceDB option (PROPOSED)

## What Others Should Know

- **Storage agent:** You may own the `pumpkin-store/` crate if approved. Your Anvil expertise transfers directly to the TOML backend.
- **Plugin agent:** PatchBukkit coordination section added to your prompt. Events need clean, serializable field types.
- **Items agent:** Recipe queries via DataFusion SQL would replace manual array iteration.
- **All agents:** `GameDataStore` trait would become the canonical data access path, replacing direct use of pumpkin-data statics.

## What I Need From Others

- **Human operator:** Approval to proceed with ARCH-020 implementation
- **Storage agent:** Assessment of `pumpkin-store/` crate ownership
- **Protocol agent:** Compatibility check — does DTO layer (ARCH-019) conflict or complement?

## Open Questions

1. Should `pumpkin-store/` be a new workspace crate, or should this live in `pumpkin-util/`?
2. Should LanceDB data files ship in the binary or be generated at first run from JSON?
3. Does Arrow's MSRV conflict with Pumpkin's Rust 1.89 requirement? (Arrow targets latest stable, should be fine)
4. Should the TOML backend be the default for development and Lance for production?
