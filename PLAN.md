# Pumpkin 1.0.0 Roadmap - Phase 1 Planning Document

> **Status: IMPLEMENTATION COMPLETE** — All tasks in this plan have been implemented and verified.
> See [ROADMAP_449.md](ROADMAP_449.md) for technical details and [CHANGELOG.md](CHANGELOG.md) for a user-facing summary.

## Executive Summary

This document outlines the audit findings and step-by-step execution plan for Phase 1 of the Pumpkin 1.0.0 Roadmap (Issue #449). The three focus areas are:

1. **Plugin API Stable ABI Boundary** - Standardize WIT types for cross-compiler compatibility
2. **Panic Trigger Audit** - Remove `.unwrap()`/`.expect()` from production code paths
3. **Tick Loop Timing Diagnostics** - Add zero-allocation TPS/tick duration metrics

---

## 1. Plugin API Stable ABI Boundary (`pumpkin-plugin-api` / `pumpkin-plugin-wit`)

### Audit Findings

#### WIT Type Analysis (`pumpkin-plugin-wit/v0.1/`)

**Stable ABI Compliant Types:**
- Primitive types: `s32`, `s64`, `f32`, `f64`, `u8`, `u32`, `bool` - fixed layout
- `record` with fixed-size fields (e.g., `block-pos { x: s32, y: s32, z: s32 }`)
- `enum` with explicit discriminants (e.g., `hand { left, right }`, `game-mode`, `entity-pose`)
- `variant` with explicit cases (e.g., `serverbound-packet`, `clientbound-packet`, `event`)

**ABI Risk Areas (Require Review/Changes):**

| File / Type | Issue | Risk Level | Required Action |
|-------------|-------|------------|-----------------|
| `common.wit:15` `position = tuple<f64, f64, f64>` | Tuple layout not guaranteed stable across WASM target versions | High | Replace with `record position { x: f64, y: f64, z: f64 }` |
| `common.wit:5` `raw-text-component = list<u8>` | Deprecated; uses variable-length list | Medium | Remove deprecated type; enforce `text-component` resource |
| `text.wit` `text-component` resource | Resource handles require stable handle representation | Medium | Document handle ABI; ensure `wit-bindgen` version pinned |
| `event.wit:327-341` `event` variant with 40+ cases | Large variant discriminant size may change | Medium | Consider splitting into smaller variant groups; document discriminant policy |
| `entity-types.wit`, `item-stack.wit` | Complex nested records with `option<>` and `list<>` | Low | Verify `wit-bindgen` generates stable layout; add `#[repr(C)]` hints if needed |

#### Rust API Layer (`pumpkin-plugin-api/src/`)

**Critical Issues:**

| File | Line | Issue | Fix Required |
|------|------|-------|--------------|
| `lib.rs:204-209` | `plugin()` function | `unsafe { PLUGIN.as_deref_mut().unwrap() }` - panics if called before init | Replace with `Option` return or `expect` with clear message |
| `lib.rs:127` | `EVENT_HANDLERS.lock().unwrap()` | Poisoned mutex panic in WIT export | Use `lock().unwrap_or_else(PoisonError::into_inner)` |
| `lib.rs:144` | `COMMAND_HANDLERS.lock().unwrap()` | Same as above | Same fix |
| `lib.rs:157` | `TASK_HANDLERS.lock().unwrap()` | Same as above | Same fix |
| `commands.rs:50, 70` | `COMMAND_HANDLERS.lock().unwrap()` | Mutex poison panic in command registration | Use `unwrap_or_else` |
| `forms.rs:191, 193, 195, 205, 206` | `.unwrap()` on JSON parsing | Network-facing parsing - should return `Result` | Return `FormResponse::Closed` on parse error |

**Recommended ABI Stabilization Steps:**
1. Pin `wit-bindgen` version in `Cargo.toml` (e.g., `wit-bindgen = "=0.35.0"`)
2. Add `#[repr(C)]` to all generated Rust types in `wit` module
3. Replace all `tuple<>` with `record {}` in WIT
4. Remove deprecated `raw-text-component` type
5. Add `wit-bindgen` config: `wasm: { import_memory: true, export_memory: true }`
6. Run `cargo check --target wasm32-wasip1` to verify WASM compatibility

---

## 2. Panic Trigger Audit (`unwrap` / `expect` Removal)

### Summary Statistics
| Crate / Path | `.unwrap()` Count | `.expect()` Count | High-Risk (Network/Protocol) |
|--------------|-------------------|-------------------|------------------------------|
| `pumpkin-protocol/src/` | 27 | 6 | **21** |
| `pumpkin/src/net/` | 15 | 3 | **12** |
| `pumpkin/src/world/` | 35 | 2 | **8** |
| `pumpkin/src/server/` | 18 | 4 | **3** |
| `pumpkin/src/plugin/` | 12 | 5 | **2** |
| **Total** | **107** | **20** | **~46** |

### Categorized Findings

#### HIGH-RISK: Network Decoder / Protocol Crashes (Production-Facing)

**pumpkin-protocol - Packet Encoding/Decoding:**
| File | Line | Context | Fix Strategy |
|------|------|---------|--------------|
| `java/packet_encoder.rs` | 248, 252, 257, 282, 286, 291, 309, 314, 326 | `.unwrap()` on `writer.as_mut()` + encode results | Return `Result`; close connection on error |
| `java/client/play/sound_effect.rs` | 87, 91, 109, 132, 154 | `.expect()`/`.unwrap()` on sound remap & VarInt decode | Validate sound IDs at load; return `DecodeError` |
| `java/client/play/entity_sound_effect.rs` | 95, 99, 117, 140, 162 | Same pattern as sound_effect | Same fix |
| `bedrock/packet_decoder.rs` | 238, 245 | `.expect()` on encode/decode | Return `Result<Packet, DecodeError>` |
| `bedrock/server/login.rs` | 74, 77, 80, 84, 87 | `.expect()` on packet construction | Validate lengths; return `LoginError` |
| `codec/recipe.rs` | 23 | `.expect("Crafting recipe used invalid tag")` | Return `RecipeError::InvalidTag` |

**pumpkin/src/net/ - Network Handlers:**
| File | Line | Context | Fix Strategy |
|------|------|---------|--------------|
| `java/play.rs` | 742, 806, 873, 940, 1371 | `.unwrap()` on Item/Entity lookups + `upgrade()` | Return `Result`; send disconnect packet on failure |
| `mod.rs` | 101, 318, 356, 626 | `.unwrap()` on config values + formatting | Use `NonZeroU8::new(8).expect("config")` at startup; `format!` can't fail |
| `rcon/mod.rs` | 21 | `.unwrap()` on TCP bind | Return `Result`; log error and retry |
| `query.rs` | 24, 46, 138 | `.expect()` on socket bind + CString | Return `Result`; handle gracefully |

#### MEDIUM-RISK: Internal State Invariants (Logic Bugs)

**pumpkin/src/world/:**
| File | Line | Context | Fix Strategy |
|------|------|---------|--------------|
| `portal/nether.rs` | 642, 648 | `.unwrap()` on `Option<&BlockPos>` | Use `if let Some(pos) = ...` or `unwrap_or_default()` |
| `natural_spawner.rs` | 197, 206, 213, 225, 503, 543, 666, 668 | Mutex locks + `.unwrap()` | Use `lock().unwrap_or_else(PoisonError::into_inner)` |
| `mod.rs` | 272, 647, 1009, 1638, 1737, 1770, 2420, 2555, 2866, 3014, 3886, 3933, 4099, 4496, 4996, 5015, 5041, 5072 | Various `.unwrap()` on locks, options, NBT | Convert to `Result` or handle `None` gracefully |
| `loot.rs` | 388, 389, 798 | `.unwrap()` on string strip + registry lookup | Return `Option<ItemStack>`; handle missing registry entry |
| `chunker.rs` | 16, 25 | `.unwrap()` on `upgrade()` + `NonZeroU8` | Handle weak ref upgrade failure; const `NonZeroU8` |

**pumpkin/src/server/:**
| File | Line | Context | Fix Strategy |
|------|------|---------|--------------|
| `mod.rs` | 187, 303, 348, 407, 453, 501, 511, 521, 1001 | `.expect()` on world load, thread pool, entity registry | Move to startup `Result`; panic only at init with clear msg |
| `key_store.rs` | 29, 44 | `.expect()` on key generation | Return `Result`; fail startup cleanly |
| `scheduler.rs` | 128 | `.unwrap()` on task pop | Handle empty queue gracefully |

**pumpkin/src/plugin/:**
| File | Line | Context | Fix Strategy |
|------|------|---------|--------------|
| `loader/wasm/wasm_host/wit/v0_1/world.rs` | 873, 895, 917 | `.unwrap()` on heightmap + lock | Use `lock().unwrap_or_else` |
| `loader/wasm/wasm_host/wit/v0_1/text.rs` | 55, 57, 64, 66, 73 | `.expect()` on resource table lookups | Return `Err(WitError::InvalidHandle)` |
| `loader/wasm/wasm_host/wit/v0_1/server.rs` | 118, 172, 194 | `.expect()` on resource insertion | Return `Result` |
| `loader/wasm/wasm_host/wit/v0_1/player.rs` | 465, 477, 512, 516, 524, 528, 536, 742, 758, 775, 792, 807, 1042, 1054 | `.expect()`/`.unwrap()` on resource access + server ref | Return `WitError`; check `Option` |
| `loader/wasm/wasm_host/wit/v0_1/item_stack.rs` | 40, 423 | `.unwrap()` on enchantment + NBT serialize | Return `Result` |
| `api/context.rs` | 85 | `.unwrap()` on `create_dir_all` | Return `Result` or log error |

#### LOW-RISK: Test/Fuzz/Build-Time Only (No Production Impact)

- `pumpkin-protocol/fuzz/` - fuzz targets only
- `pumpkin-protocol/src/codec/var_*.rs` - `NonZeroUsize::new(N).unwrap()` at const init (safe)
- `pumpkin/src/net/lan_broadcast.rs`, `proxy/velocity.rs` - startup-only binds

---

### Error Propagation Strategy by Category

| Category | Propagation Method | Connection Handling |
|----------|-------------------|---------------------|
| **Packet Decode** | `Result<Packet, DecodeError>` → `?` | Send `Disconnect` packet with reason; close TCP |
| **Packet Encode** | `Result<(), EncodeError>` → `?` | Log error; close connection |
| **Registry Lookup** | `Option<T>` / `Result<T, RegistryError>` | Treat as protocol violation; disconnect |
| **Mutex Lock** | `lock().unwrap_or_else(PoisonError::into_inner)` | Log poison; continue with recovered data |
| **Weak Ref Upgrade** | `upgrade().ok_or(ServerShutdown)` | Treat as client disconnect |
| **Config Validation** | `expect()` at startup only | Fail fast at server start (acceptable) |
| **File I/O (world save)** | `Result<(), IoError>` → `?` | Log error; retry or mark chunk dirty |
| **WASM Resource Table** | `Result<T, WitError>` | Return trap to plugin; log host-side |

---

## 3. Tick Loop Timing Diagnostics

### Current Tick Loop Location
**File:** `pumpkin/src/server/ticker.rs` (lines 18-77)

```rust
pub async fn run(server: &Arc<Server>) {
    let mut next_tick = Instant::now();
    'ticker: loop {
        let tick_start_time = std::time::Instant::now();
        // ... tick logic ...
        let tick_duration_nanos = tick_start_time.elapsed().as_nanos() as i64;
        
        server.update_tick_times(tick_duration_nanos).await;
        
        let tick_interval = if manager.is_sprinting() {
            Duration::ZERO
        } else {
            Duration::from_nanos(manager.nanoseconds_per_tick() as u64)
        };
        
        next_tick += tick_interval;
        tokio::select! {
            () = sleep_until(next_tick) => {},
            () = STOP_INTERRUPT.cancelled() => break 'ticker,
        }
    }
}
```

### Existing TPS Infrastructure
**File:** `pumpkin/src/server/tick_rate_manager.rs`

Already tracks:
- `nanoseconds_per_tick` (target)
- Sprint tick timing (`start_sprint_tick_work` / `end_sprint_tick_work`)
- `finish_tick_sprint()` calculates TPS/MSPT for sprint periods only

**Missing:** Continuous average TPS and tick duration for normal operation.

### Zero-Allocation Timer Design

Add to `ServerTickRateManager` (lock-free, atomic-only):

```rust
// Add to ServerTickRateManager struct:
tick_count: AtomicU64,           // Total ticks processed
total_tick_nanos: AtomicU64,     // Accumulated tick duration (ns)
last_tps_update: AtomicU64,      // Timestamp of last TPS calculation (Instant::now() as nanos)
rolling_tps: AtomicU32,          // TPS * 100 (fixed-point)
rolling_mspt: AtomicU32,         // MSPT * 100 (fixed-point)

// In tick() method (called every tick):
fn record_tick(&self, duration_nanos: u64) {
    let prev_count = self.tick_count.fetch_add(1, Ordering::Relaxed);
    self.total_tick_nanos.fetch_add(duration_nanos, Ordering::Relaxed);
    
    // Update rolling averages every 20 ticks (~1 second at 20 TPS)
    if prev_count % 20 == 0 {
        let total_nanos = self.total_tick_nanos.load(Ordering::Relaxed);
        let count = self.tick_count.load(Ordering::Relaxed);
        if count > 0 {
            let avg_nanos = total_nanos / count;
            self.rolling_mspt.store((avg_nanos / 10_000) as u32, Ordering::Relaxed); // ms * 100
            self.rolling_tps.store((1_000_000_000_00 / avg_nanos) as u32, Ordering::Relaxed); // tps * 100
        }
    }
}

// Public getters (lock-free):
pub fn current_tps(&self) -> f32 { self.rolling_tps.load(Ordering::Relaxed) as f32 / 100.0 }
pub fn current_mspt(&self) -> f32 { self.rolling_mspt.load(Ordering::Relaxed) as f32 / 100.0 }
pub fn avg_tick_duration_ns(&self) -> u64 { 
    let count = self.tick_count.load(Ordering::Relaxed);
    if count == 0 { return 0; }
    self.total_tick_nanos.load(Ordering::Relaxed) / count
}
```

**Integration in `ticker.rs`:**
```rust
// After server.tick().await:
let tick_duration_nanos = tick_start_time.elapsed().as_nanos() as u64;
server.tick_rate_manager.record_tick(tick_duration_nanos);
```

**Zero-Allocation Guarantees:**
- No `Vec`, `String`, `Box`, or heap allocation in hot path
- Only atomic operations (`fetch_add`, `load`, `store`)
- Fixed-point arithmetic (no `f64` in hot path)
- Rolling window avoids unbounded accumulation

---

## Phase 1 Execution Plan

### Task 1: Plugin API Stable ABI

| Step | Action | Files | Verification |
|------|--------|-------|--------------|
| 1.1 | Pin `wit-bindgen` version in `pumpkin-plugin-api/Cargo.toml` | `Cargo.toml` | `cargo check` |
| 1.2 | Replace `tuple<f64,f64,f64>` with `record position {x:f64,y:f64,z:f64}` | `pumpkin-plugin-wit/v0.1/common.wit:15` | `cargo check --target wasm32-wasip1` |
| 1.3 | Remove deprecated `raw-text-component` type | `common.wit:5` | `cargo check` |
| 1.4 | Fix mutex poison handling in WIT exports | `pumpkin-plugin-api/src/lib.rs:127,144,157` | `cargo clippy -- -D warnings` |
| 1.5 | Fix `plugin()` panic on uninitialized access | `pumpkin-plugin-api/src/lib.rs:204-209` | `cargo test` |
| 1.6 | Fix form response parsing to return `Result` | `pumpkin-plugin-api/src/forms.rs:191,193,195,205,206` | `cargo test` |
| 1.7 | Add `#[repr(C)]` to generated types (verify in `wit` module) | `pumpkin-plugin-api/src/lib.rs:67-77` | `cargo check --target wasm32-wasip1` |
| 1.8 | Run full workspace check | All | `cargo check --all-targets --workspace` |

### Task 2: Panic Trigger Removal

#### Phase 2A: High-Risk Network/Protocol (Week 1)

| Step | Target Files | Pattern | Replacement |
|------|--------------|---------|-------------|
| 2A.1 | `pumpkin-protocol/src/java/packet_encoder.rs` | `.unwrap()` on encode | Return `Result`; propagate to connection handler |
| 2A.2 | `pumpkin-protocol/src/java/client/play/sound_effect.rs` | `.expect()`/`.unwrap()` | Validate at load; return `DecodeError` |
| 2A.3 | `pumpkin-protocol/src/java/client/play/entity_sound_effect.rs` | Same | Same |
| 2A.4 | `pumpkin-protocol/src/bedrock/packet_decoder.rs` | `.expect()` | Return `Result<Packet, DecodeError>` |
| 2A.5 | `pumpkin-protocol/src/bedrock/server/login.rs` | `.expect()` on write/read | Return `LoginError`; disconnect cleanly |
| 2A.6 | `pumpkin-protocol/src/codec/recipe.rs:23` | `.expect()` | Return `RecipeError` |
| 2A.7 | `pumpkin/src/net/java/play.rs` | `.unwrap()` on registry/weak ref | Return `Result`; send disconnect packet |
| 2A.8 | `pumpkin/src/net/mod.rs` | `.unwrap()` on config | Move validation to startup |

**Verification after each step:**
```bash
cargo check -p pumpkin-protocol
cargo check -p pumpkin
cargo clippy -p pumpkin-protocol -- -D warnings
cargo clippy -p pumpkin -- -D warnings
cargo test -p pumpkin-protocol
cargo test -p pumpkin net::
```

#### Phase 2B: Medium-Risk World/Server (Week 2)

| Step | Target Files | Pattern | Replacement |
|------|--------------|---------|-------------|
| 2B.1 | `pumpkin/src/world/portal/nether.rs:642,648` | `.unwrap()` on `Option` | `if let Some` / `unwrap_or` |
| 2B.2 | `pumpkin/src/world/natural_spawner.rs` | Mutex `.unwrap()` | `lock().unwrap_or_else(PoisonError::into_inner)` |
| 2B.3 | `pumpkin/src/world/mod.rs` (18 locations) | Various `.unwrap()` | Context-specific `Result`/`Option` handling |
| 2B.4 | `pumpkin/src/world/loot.rs:388,389,798` | `.unwrap()` on lookup | Return `Option`/`Result` |
| 2B.5 | `pumpkin/src/server/mod.rs` (9 locations) | `.expect()` at startup | Keep at init with clear message; or `Result` |
| 2B.6 | `pumpkin/src/server/key_store.rs:29,44` | `.expect()` on crypto | Return `Result`; fail startup cleanly |
| 2B.7 | `pumpkin/src/plugin/loader/wasm/wasm_host/wit/v0_1/*.rs` | `.expect()`/`.unwrap()` on resources | Return `WitError` variants |

**Verification:**
```bash
cargo check -p pumpkin
cargo clippy -p pumpkin -- -D warnings
cargo test -p pumpkin world:: server:: plugin::
```

#### Phase 2C: Low-Risk / Test Code (Optional)

- Fuzz targets: keep `.unwrap()` (test-only)
- Const `NonZeroUsize::new().unwrap()`: keep (compile-time verified)
- Startup-only `.expect()` with clear messages: acceptable per AGENTS.md

### Task 3: Tick Loop Diagnostics

| Step | Action | Files | Verification |
|------|--------|-------|--------------|
| 3.1 | Add atomic fields to `ServerTickRateManager` | `pumpkin/src/server/tick_rate_manager.rs` | `cargo check -p pumpkin` |
| 3.2 | Implement `record_tick(duration_nanos)` method | `tick_rate_manager.rs` | `cargo test -p pumpkin tick_rate` |
| 3.3 | Call `record_tick` from `Ticker::run` | `pumpkin/src/server/ticker.rs:44` | `cargo test -p pumpkin ticker` |
| 3.4 | Add public getters: `current_tps()`, `current_mspt()`, `avg_tick_duration_ns()` | `tick_rate_manager.rs` | `cargo test` |
| 3.5 | Export via server status / commands | `pumpkin/src/server/mod.rs` or commands | Manual test |
| 3.6 | Benchmark: verify zero allocations in hot path | `cargo bench` (if available) | `perf record` / `heaptrack` |

---

## Compiler Checks at Each Step

### Required Commands (from AGENTS.md)

```bash
# After ANY code change:
cargo check --all-targets --workspace
cargo clippy --all-targets --workspace -- -D warnings
cargo fmt --check --all

# Targeted checks:
cargo check -p pumpkin-plugin-api --target wasm32-wasip1
cargo check -p pumpkin-protocol
cargo check -p pumpkin

# Test suites:
cargo test -p pumpkin-plugin-api
cargo test -p pumpkin-protocol
cargo test -p pumpkin

# Specific test filters:
cargo test -p pumpkin net::          # network tests
cargo test -p pumpkin world::         # world tests  
cargo test -p pumpkin server::        # server/tick tests
cargo test -p pumpkin plugin::        # plugin tests
```

### Clippy Lints to Enable (in addition to `-D warnings`)

```toml
# In Cargo.toml [workspace.lints.clippy]
unwrap_used = "deny"           # Catch remaining .unwrap()
expect_used = "deny"           # Catch remaining .expect()
panic = "deny"                 # Catch explicit panic!
todo = "warn"                  # Track incomplete work
```

---

## File Change Summary

### Plugin API ABI (`pumpkin-plugin-api`, `pumpkin-plugin-wit`)
- `pumpkin-plugin-wit/v0.1/common.wit` - Fix position tuple, remove deprecated type
- `pumpkin-plugin-api/Cargo.toml` - Pin wit-bindgen
- `pumpkin-plugin-api/src/lib.rs` - Fix mutex poison, plugin init panic
- `pumpkin-plugin-api/src/forms.rs` - Fix form response parsing
- `pumpkin-plugin-api/src/commands.rs` - Fix mutex poison

### Panic Audit - High Risk (`pumpkin-protocol`)
- `pumpkin-protocol/src/java/packet_encoder.rs` - 9 locations
- `pumpkin-protocol/src/java/client/play/sound_effect.rs` - 5 locations
- `pumpkin-protocol/src/java/client/play/entity_sound_effect.rs` - 5 locations
- `pumpkin-protocol/src/bedrock/packet_decoder.rs` - 2 locations
- `pumpkin-protocol/src/bedrock/server/login.rs` - 5 locations
- `pumpkin-protocol/src/codec/recipe.rs` - 1 location

### Panic Audit - High Risk (`pumpkin/src/net/`)
- `pumpkin/src/net/java/play.rs` - 5+ locations
- `pumpkin/src/net/mod.rs` - 4 locations
- `pumpkin/src/net/rcon/mod.rs` - 1 location
- `pumpkin/src/net/query.rs` - 3 locations

### Panic Audit - Medium Risk (`pumpkin/src/world/`, `server/`, `plugin/`)
- `pumpkin/src/world/portal/nether.rs` - 2 locations
- `pumpkin/src/world/natural_spawner.rs` - 8 locations
- `pumpkin/src/world/mod.rs` - 18+ locations
- `pumpkin/src/world/loot.rs` - 3 locations
- `pumpkin/src/world/chunker.rs` - 2 locations
- `pumpkin/src/server/mod.rs` - 9 locations
- `pumpkin/src/server/key_store.rs` - 2 locations
- `pumpkin/src/server/scheduler.rs` - 1 location
- `pumpkin/src/plugin/loader/wasm/wasm_host/wit/v0_1/*.rs` - 20+ locations

### Tick Diagnostics
- `pumpkin/src/server/tick_rate_manager.rs` - Add atomic fields + methods
- `pumpkin/src/server/ticker.rs` - Call `record_tick` in loop

---

## Next Steps

Once this `PLAN.md` is reviewed and confirmed, Phase 2 (Implementation) will proceed with:

1. **Plugin API ABI fixes** (Task 1) - ~1 day
2. **High-risk panic removal** (Task 2A) - ~2 days
3. **Medium-risk panic removal** (Task 2B) - ~3 days
4. **Tick diagnostics implementation** (Task 3) - ~1 day

**Total estimated: ~7 working days**

---

*Please review this plan and confirm before proceeding to implementation phase.*