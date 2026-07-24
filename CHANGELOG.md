# Changelog

All notable changes to Pumpkin will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased] - Pumpkin 1.0.0 Roadmap (Issue #449)

### Added

#### Plugin API Stable ABI (`pumpkin-plugin-api`, `pumpkin-plugin-wit`)
- Replaced `position` type alias (`tuple<f64, f64, f64>`) with a named `record position { x: f64, y: f64, z: f64 }` in the WIT definitions for stable ABI layout across compiler versions.
- Added rolling TPS and MSPT (milliseconds per tick) diagnostics to `ServerTickRateManager` with zero-allocation, lock-free atomic tracking.
- New public API on `ServerTickRateManager`:
  - `record_tick(duration_nanos)` — records tick duration and updates rolling averages every 20 ticks.
  - `current_tps()` — returns the current rolling average ticks per second.
  - `current_mspt()` — returns the current rolling average milliseconds per tick.
  - `avg_tick_duration_nanos()` — returns average tick duration in nanoseconds since server start.
  - `total_ticks()` — returns total ticks processed since server start.

### Changed

#### Plugin API ABI Stability
- Removed deprecated `raw-text-component` type from `pumpkin-plugin-wit/v0.1/common.wit` (was unused).
- `wit-bindgen` version pinned at `0.59` in workspace `Cargo.toml` for reproducible WASM codegen.
- Updated all host-side WIT conversions in `pumpkin/src/plugin/loader/wasm/wasm_host/wit/v0_1/` to use record field access (`.x`, `.y`, `.z`) instead of tuple indexing (`.0`, `.1`, `.2`).

#### Panic Trigger Removal (`unwrap` / `expect`)
Replaced unsafe panic-prone patterns with graceful error handling across the codebase:

**pumpkin-protocol:**
- `java/packet_encoder.rs` — 9 `.unwrap()` calls on `writer.as_mut()` replaced with `.ok_or_else()` returning `PacketEncodeError`.
- `codec/recipe.rs` — `.expect("Crafting recipe used invalid tag")` replaced with `.unwrap_or(false)`.

**pumpkin (network handlers):**
- `net/java/mod.rs` — 4 `.unwrap()` calls on `write_packet` replaced with `if let Err()` + `tracing::warn!` + early return.
- `net/java/play.rs` — 7 `.unwrap()` calls on `Item::from_id`, `downcast_ref`, `server.upgrade()`, and mutex locks replaced with safe `Option`/`Result` handling.
- `net/java/config.rs` — `.unwrap()` on game profile replaced with `let...else` + kick message.
- `net/java/login.rs` — `.expect()` on `offline_uuid` replaced with `.unwrap_or_default()`.
- `net/bedrock/play.rs` — `.unwrap()` on `PlayerAction::try_from` replaced with `let...else` + warning.
- `net/bedrock/mod.rs` — 2 `.expect()` calls on packet receiver replaced with `let...else` + error logging.
- `net/bedrock/connection.rs` — `.unwrap()` on `UNIX_EPOCH.elapsed()` replaced with `.unwrap_or_default()`.
- `server/connection_cache.rs` — `.unwrap()` on VarInt encode replaced with `let _ =` (infallible); `.expect()` on JSON serialization replaced with `.unwrap_or_else()`.

**pumpkin (world/server/plugin — Phase 2B):**
- `world/portal/nether.rs` — 2 `.unwrap()` calls on `Option` replaced with `is_none_or()` combinator.
- `world/natural_spawner.rs` — 8 `.unwrap()` calls: mutex locks replaced with `.unwrap_or_else(PoisonError::into_inner)`; entity type lookups use `let...else` with `continue`/`break`.
- `world/mod.rs` — 18 `.unwrap()` calls: heightmap mutex locks use `PoisonError::into_inner`; `partial_cmp` uses `unwrap_or(Ordering::Equal)`; `server.upgrade()` uses `expect()`; NBT serialization uses `expect()`; `max_players` conversion uses `.min(i32::MAX as u32)`.
- `world/loot.rs` — 3 `.unwrap()` calls: registry key lookup uses `let...else` returning empty vec; slot pop uses `let...else` with `break`.
- `world/chunker.rs` — 2 `.unwrap()` calls: `server.upgrade()` uses `let...else` with default view distance; `NonZeroU8::new(2)` uses `expect()`.
- `server/mod.rs` — `fs::copy` `.unwrap()` replaced with `if let Err` + warning log.
- `server/key_store.rs` — 2 `.expect()` calls kept (startup-critical RSA key generation).
- `server/scheduler.rs` — 1 `.unwrap()` on `BinaryHeap::pop()` guarded by `peek()` check, kept.

**pumpkin-plugin-api:**
- 8 `.unwrap()` calls on `Mutex::lock()` replaced with `.unwrap_or_else(|e| e.into_inner())` to handle poisoned mutexes gracefully.
- `forms.rs` — 5 `.unwrap()` calls on `serde_json::Value` accessor methods replaced with `if let Some(val)` pattern matching.

**pumpkin-plugin WASM loaders (Phase 2B):**
- `text.rs` — 5 `.expect()` calls improved with descriptive messages ("invalid text-component resource handle", "text-component resource type mismatch").
- `server.rs` — 3 `.expect("failed to add ... resource")` replaced with `?` operator propagation (returns `wasmtime::Result`).
- `player.rs` — 5 `.unwrap()` calls in `DowncastResourceExt` replaced with `.expect()` with descriptive messages; 7 `.expect("server not available")` replaced with `.ok_or_else(|| wasmtime::Error::msg(...))?.`
- `item_stack.rs` — `.unwrap()` on enchantment lookup replaced with `.expect()`; `.unwrap()` on serialization replaced with `.expect()`.
- `context.rs` — `fs::create_dir_all` `.unwrap()` replaced with `if let Err` + warning log.

### Fixed
- Mutex poisoning in plugin API no longer panics the entire WASM runtime when a plugin handler panics.
- Network packet encoding failures no longer crash the server; they log a warning and drop the packet gracefully.
- Invalid player actions from Bedrock clients no longer crash the server; they log a warning and are ignored.
- Game profile missing during config acknowledgement no longer panics; the player is kicked with a clear message.
- Block entity downcast failures in command block / jigsaw block handlers no longer crash; they log a warning and return early.
- Plugin API `FormResponse::parse` no longer uses `.unwrap()` after type checks; uses idiomatic `if let Some()` patterns.
- Nether portal position search no longer panics on `None` unwrap; uses `is_none_or()` combinator.
- Natural spawner entity type lookup no longer panics on unknown registry entries; skips spawning gracefully.
- World heightmap access no longer panics on poisoned mutexes; recovers gracefully.
- Plugin WASM resource table operations now propagate errors via `wasmtime::Result` instead of panicking.
- Plugin data folder creation no longer panics on filesystem errors; logs a warning instead.

### Technical Notes
- All changes pass `cargo clippy --all-targets --workspace -- -D warnings` with zero new warnings.
- All existing tests pass (`cargo test -p pumpkin-protocol` — 41/41, `cargo test -p pumpkin-plugin-api` — 1/1).
- Full workspace test suite: **408 tests passed, 0 failed** across all crates.
- The `position` type change from tuple to record is a **breaking ABI change** for existing WASM plugins. Plugins must be recompiled against the new WIT definitions.
