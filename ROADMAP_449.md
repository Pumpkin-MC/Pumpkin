# Pumpkin 1.0.0 Roadmap — Phase 1 & 2 Technical Documentation

This document describes the technical changes made as part of [Issue #449](https://github.com/Pumpkin-MC/Pumpkin/issues/449): Pumpkin 1.0.0 Roadmap, Phase 1 (Planning) and Phase 2 (Implementation).

---

## 1. Plugin API Stable ABI Boundary

### Problem
WIT types used anonymous tuples (`tuple<f64, f64, f64>`) which, while technically stable in the WASM Component Model, lack named fields for documentation and IDE support. Additionally, no version pinning existed for the `wit-bindgen` code generator.

### Changes

#### `pumpkin-plugin-wit/v0.1/common.wit`
```wit
// Before:
type position = tuple<f64, f64, f64>;

// After:
record position {
    x: f64,
    y: f64,
    z: f64,
}
```

The deprecated `raw-text-component` type alias (unused) was removed.

#### `Cargo.toml` (workspace)
`wit-bindgen` pinned to `0.59` with `default-features = false, features = ["macros"]`.

#### Host-Side Conversions
All position conversion functions updated in `pumpkin/src/plugin/loader/wasm/wasm_host/wit/v0_1/`:

| File | Change |
|------|--------|
| `events/mod.rs` | `to_wasm_position()` and `from_wasm_position()` now use record field access |
| `entity.rs` | `teleport()`, `set_velocity()` use `.x`, `.y`, `.z` |
| `world.rs` | `play_sound()`, `spawn_particle()`, `create_explosion()`, `spawn_entity()`, `to_wit_bounding_box()` |
| `commands/mod.rs` | `position()` returns `Position { x, y, z }` record |

**Note:** `generated_packets.rs` was NOT changed — packet WITs use inline tuples, not the `common::Position` type.

### Migration
Existing WASM plugins must recompile against the updated WIT definitions. Plugin source code using `Position` as a tuple will need to switch to field access.

---

## 2. Panic Trigger Audit

### Scope
Scanned ~1,400+ lines across `pumpkin-protocol`, `pumpkin/src/net/`, `pumpkin/src/server/`, `pumpkin/src/world/`, and `pumpkin-plugin-api` for `.unwrap()` and `.expect()` calls.

### Categorization

| Category | Count Fixed | Strategy |
|----------|-------------|----------|
| **Network encode/decode** | 12 | Return `Result` with error description; close connection |
| **Registry lookups** | 6 | `let Some(x) = ... else { continue/return; }` |
| **Weak ref upgrades** | 3 | `let Some(server) = ... else { return; }` / `expect()` |
| **Mutex locks (plugin API)** | 8 | `.unwrap_or_else(\|e\| e.into_inner())` — recover from poisoned state |
| **Mutex locks (world)** | 8 | `.unwrap_or_else(PoisonError::into_inner)` |
| **Mutex locks (network)** | 2 | `.unwrap_or_else(PoisonError::into_inner)` |
| **Type downcasts** | 2 | `let Some(x) = ... else { warn + return; }` |
| **Config/validation** | 3 | `let...else` with kick/error message |
| **Unix time** | 2 | `.unwrap_or_default()` |
| **Partial comparison** | 2 | `.unwrap_or(Ordering::Equal)` |
| **File I/O** | 2 | `if let Err(e) = ... { warn!(...) }` |
| **NBT serialization** | 2 | `.expect("writing to Vec is infallible")` |
| **WASM resource table** | 5 | `?` operator / descriptive `.expect()` messages |
| **Startup invariants** | 3 | Kept `.expect()` — acceptable at init (key gen, world load) |
| **Test code** | ~15 | Kept — test panics are acceptable |

### High-Risk Changes (Network-Facing)

#### `packet_encoder.rs` — Writer Safety
```rust
// Before:
self.writer.as_mut().unwrap().write_all(&data).await

// After:
self.writer.as_mut()
    .ok_or_else(|| PacketEncodeError::Message("writer already consumed".into()))?
    .write_all(&data).await
```

#### `net/java/play.rs` — Registry Lookups
```rust
// Before:
let stack = ItemStack::new(1, Item::from_id(block.item_id).unwrap());

// After:
let Some(item) = Item::from_id(block.item_id) else { return; };
let stack = ItemStack::new(1, item);
```

#### `net/bedrock/play.rs` — Client Validation
```rust
// Before:
let action = PlayerAction::try_from(packet.action.0).unwrap();

// After:
let Ok(action) = PlayerAction::try_from(packet.action.0) else {
    tracing::warn!("Invalid player action: {}", packet.action.0);
    return;
};
```

### Acceptable Panics (Not Changed)
- `key_store.rs` — RSA key generation at startup (server cannot function without keys)
- `server/mod.rs` — World loading at startup (`expect("World loading panicked")`)
- `DowncastResourceExt` trait impls — Type-system invariant panics with descriptive messages
- Fuzz targets (`pumpkin-protocol/fuzz/`) — Test-only code
- Const `NonZeroUsize::new(N).unwrap()` — Compile-time verified

### Medium-Risk Changes (World/Server/Plugin — Phase 2B)

Scanned ~35 `.unwrap()` locations across `pumpkin/src/world/`, `pumpkin/src/server/`, and `pumpkin-plugin-api` WASM loaders.

#### `world/portal/nether.rs` — Portal Position Search
```rust
// Before:
if ideal_pos.is_none() || dist < ideal_pos.as_ref().unwrap().2

// After:
if ideal_pos.as_ref().is_none_or(|p| dist < p.2)
```

#### `world/natural_spawner.rs` — Mutex Poison Recovery
```rust
// Before:
Self(std::sync::Mutex::new(self.0.lock().unwrap().clone()))

// After:
Self(std::sync::Mutex::new(
    self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone(),
))
```

#### `world/natural_spawner.rs` — Entity Type Lookup
```rust
// Before:
let entity_type = EntityType::from_name(
    spawner.r#type.strip_prefix("minecraft:").unwrap_or(spawner.r#type),
).unwrap();

// After:
let Some(entity_type) = EntityType::from_name(
    spawner.r#type.strip_prefix("minecraft:").unwrap_or(spawner.r#type),
) else { continue; };
```

#### `world/mod.rs` — Heightmap Mutex
```rust
// Before:
chunk.heightmap.lock().unwrap().get(...)

// After:
chunk.heightmap.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get(...)
```

#### `world/mod.rs` — Partial Comparison
```rust
// Before:
.partial_cmp(&b.get_entity().pos.load().squared_distance_to_vec(&pos))
.unwrap()

// After:
.partial_cmp(&b.get_entity().pos.load().squared_distance_to_vec(&pos))
.unwrap_or(std::cmp::Ordering::Equal)
```

#### `world/mod.rs` — Weak Server Reference
```rust
// Before:
self.server.upgrade().unwrap().plugin_manager.fire(event).await;

// After:
self.server.upgrade()
    .expect("world holds strong reference to server")
    .plugin_manager.fire(event).await;
```

#### `world/loot.rs` — Registry Lookup
```rust
// Before:
let key = &item_entry.name.strip_prefix("minecraft:").unwrap();
vec![ItemStack::new(1, Item::from_registry_key(key).unwrap())]

// After:
let Some(key) = item_entry.name.strip_prefix("minecraft:") else {
    return Vec::new();
};
let Some(item) = Item::from_registry_key(key) else {
    return Vec::new();
};
vec![ItemStack::new(1, item)]
```

#### `world/chunker.rs` — Weak Ref + Const
```rust
// Before:
let server = player.world().server.upgrade().unwrap();
NonZeroU8::new(2).unwrap()

// After:
let Some(server) = player.world().server.upgrade() else {
    return NonZeroU8::new(8).expect("constant is nonzero");
};
NonZeroU8::new(2).expect("constant is nonzero")
```

#### `server/mod.rs` — File Copy
```rust
// Before:
fs::copy(dat_path, backup_path).unwrap();

// After:
if let Err(e) = fs::copy(dat_path, backup_path) {
    tracing::warn!("Failed to back up level.dat: {e}");
}
```

#### Plugin WASM Loaders — Error Propagation
```rust
// Before (server.rs):
self.add_player(player).expect("failed to add player resource")

// After:
self.add_player(player) // returns wasmtime::Result, propagated with ?

// Before (player.rs):
let server = self.server.as_ref().expect("server not available");

// After:
let server = self.server.as_ref()
    .ok_or_else(|| wasmtime::Error::msg("server not available"))?;

// Before (text.rs):
.expect("invalid handle")

// After:
.expect("invalid text-component resource handle")
```

#### `plugin/api/context.rs` — Directory Creation
```rust
// Before:
fs::create_dir_all(&path).unwrap();

// After:
if let Err(e) = fs::create_dir_all(&path) {
    tracing::warn!("Failed to create plugin data folder {}: {e}", path.display());
}
```

---

## 3. Tick Loop Timing Diagnostics

### Location
`pumpkin/src/server/ticker.rs:18-77` — Main tick loop  
`pumpkin/src/server/tick_rate_manager.rs` — Tick rate management

### Architecture

```
┌─────────────────────────────────────┐
│           Ticker::run()             │
│  ┌──────────────────────────────┐   │
│  │   tick_start_time = now()    │   │
│  │   manager.tick()             │   │
│  │   server.tick().await        │   │
│  │   duration = elapsed()       │   │
│  │   manager.record_tick(dur) ◄─┼───┤ ← New
│  │   sleep_until(next_tick)     │   │
│  └──────────────────────────────┘   │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│     ServerTickRateManager           │
│  tick_count:        AtomicU64       │
│  total_tick_nanos:  AtomicU64       │
│  rolling_tps_x100:  AtomicU32       │  ← Fixed-point TPS * 100
│  rolling_mspt_x100: AtomicU32       │  ← Fixed-point MSPT * 100
│                                      │
│  record_tick(nanos):                │
│    fetch_add(tick_count)            │
│    fetch_add(total_tick_nanos)      │
│    every 20 ticks:                  │
│      avg = total / count            │
│      tps_x100 = 1e11 / avg         │
│      mspt_x100 = avg / 10_000      │
└─────────────────────────────────────┘
```

### Zero-Allocation Guarantees
- No `Vec`, `String`, `Box`, or heap allocation in the hot path
- Only atomic operations: `fetch_add`, `load`, `store`
- Fixed-point arithmetic (no floating-point in the update path)
- Rolling window avoids unbounded accumulation

### Usage
```rust
let manager = &server.tick_rate_manager;

// After server has been running:
let tps = manager.current_tps();      // e.g., 19.95
let mspt = manager.current_mspt();    // e.g., 48.3
let avg_ns = manager.avg_tick_duration_nanos(); // e.g., 48_300_000
let total = manager.total_ticks();    // e.g., 120_000
```

---

## Files Modified

### `pumpkin-plugin-wit/v0.1/`
- `common.wit` — Position type changed to record; deprecated type removed

### `pumpkin-plugin-api/src/`
- `lib.rs` — Mutex poison handling in WIT exports
- `commands.rs` — Mutex poison handling in command registration
- `scheduler.rs` — Mutex poison handling in task scheduling
- `forms.rs` — Safe pattern matching for form response parsing

### `pumpkin-protocol/src/`
- `java/packet_encoder.rs` — Writer safety (9 locations)
- `codec/recipe.rs` — Tag lookup safety

### `pumpkin/src/net/`
- `java/mod.rs` — Packet serialization error handling
- `java/play.rs` — Registry lookup, downcast, weak ref, mutex safety
- `java/config.rs` — Game profile and view distance safety
- `java/login.rs` — Offline UUID generation safety
- `bedrock/play.rs` — Player action validation
- `bedrock/mod.rs` — Packet receiver channel safety
- `bedrock/connection.rs` — System time safety

### `pumpkin/src/server/`
- `tick_rate_manager.rs` — TPS/MSPT diagnostics
- `ticker.rs` — Integration of `record_tick()`
- `connection_cache.rs` — VarInt encode and JSON serialization safety

### `pumpkin/src/plugin/loader/wasm/wasm_host/wit/v0_1/`
- `events/mod.rs` — Position record conversion
- `entity.rs` — Position record access in entity operations
- `world.rs` — Position record access in world operations
- `commands/mod.rs` — Position record construction

### `pumpkin/src/world/` (Phase 2B)
- `portal/nether.rs` — Portal position search (`is_none_or()` combinator)
- `natural_spawner.rs` — Mutex poison recovery + entity type `let...else`
- `mod.rs` — Heightmap locks, `partial_cmp`, `server.upgrade()`, NBT serialization
- `loot.rs` — Registry lookup + slot pop
- `chunker.rs` — Weak server ref + `NonZeroU8` const

### `pumpkin/src/server/` (Phase 2B)
- `mod.rs` — `fs::copy` error handling

### `pumpkin-plugin-api/` (Phase 2B)
- `api/context.rs` — `fs::create_dir_all` error handling

### `pumpkin-plugin WASM loaders` (Phase 2B)
- `text.rs` — Descriptive expect messages
- `server.rs` — Error propagation with `?`
- `player.rs` — `DowncastResourceExt` expect messages + `ok_or_else` for server access
- `item_stack.rs` — Enchantment lookup + serialization expect messages

---

## Compiler Checks

All changes verified with:
```bash
cargo check --all-targets              # Full workspace compilation
cargo clippy --all-targets --workspace -- -D warnings  # Zero warnings
cargo fmt --check                      # Formatting
cargo test --workspace                 # 408 tests pass, 0 failures
```
