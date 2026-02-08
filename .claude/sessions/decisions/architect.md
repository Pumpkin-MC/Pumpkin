# Architect — Decisions

## ARCH-001: Block module ownership split
**Date:** 2026-02-06
**Session:** .claude/sessions/2026-02-06/001_architect_gap-analysis.md
**Decision:** Redstone owns `block/blocks/redstone/` and `block/blocks/piston/`. WorldGen owns `block/registry.rs`. Architect resolves `block/mod.rs` and `block/blocks/mod.rs` conflicts. All other block files assigned per-domain in `contracts/block-ownership.toml`.
**Rationale:** The block/ module serves multiple agents. Clean ownership prevents merge conflicts.
**Affects:** Redstone, WorldGen, Architect
**Status:** active

## ARCH-002: Storage vs WorldGen boundary for Anvil files
**Date:** 2026-02-06
**Session:** .claude/sessions/2026-02-06/001_architect_gap-analysis.md
**Decision:** Storage owns NBT wire format (`pumpkin-nbt/`). WorldGen owns chunk IO that uses NBT (`pumpkin-world/src/level.rs` and related). Storage format changes require WorldGen acknowledgment.
**Rationale:** NBT is serialization (Storage). Chunk persistence is world management (WorldGen). Shared interface, not implementation.
**Affects:** Storage, WorldGen
**Status:** active

## ARCH-003: Data loading ownership
**Date:** 2026-02-06
**Session:** .claude/sessions/2026-02-06/001_architect_gap-analysis.md
**Decision:** Items owns runtime data loading (`pumpkin/src/data/`). Architect owns compile-time generated data (`pumpkin-data/`). Items must not modify `pumpkin-data/build.rs` or generated output.
**Rationale:** Generated data is build artifact. Runtime loading is gameplay logic.
**Affects:** Items, Architect
**Status:** active

## ARCH-004: lib.rs decomposition authority
**Date:** 2026-02-06
**Session:** .claude/sessions/2026-02-06/001_architect_gap-analysis.md
**Decision:** Core owns `lib.rs` but any refactor moving code into another agent's module requires that agent to acknowledge before merge. Core must publish decomposition plan as its first session.
**Rationale:** lib.rs touches every subsystem. Uncoordinated decomposition breaks everyone.
**Affects:** All agents
**Status:** active

## ARCH-005: Session logs live in .claude/sessions/ (TRACKED)
**Date:** 2026-02-06 (revised 2026-02-07)
**Session:** .claude/sessions/2026-02-06/002_architect_restructure-sessions.md
**Decision:** All session logs and decision logs live in `.claude/sessions/`. ~~This directory is gitignored.~~ REVISED: .claude/sessions/ is now tracked and committed. Agents must commit their session logs.
**Rationale:** Gitignoring sessions meant agent logs never reached the repo. Other agents in separate sessions could not read them. Logs must be committed to fulfill read-before-write.
**Affects:** All agents
**Status:** active (revised)

## ARCH-006: All orchestration lives under .claude/
**Date:** 2026-02-06
**Session:** .claude/sessions/2026-02-06/003_architect_consolidate-claude.md
**Decision:** Every orchestration file lives under .claude/. Pumpkin source tree root has zero orchestration artifacts.
**Rationale:** Fork source tree should be indistinguishable from upstream plus code changes.
**Affects:** All agents
**Status:** active

## ARCH-007: All .claude/ is tracked
**Date:** 2026-02-06 (revised 2026-02-07)
**Session:** .claude/sessions/2026-02-06/004_architect_setup-validation.md
**Decision:** ~~Only workspace-ephemeral directories under .claude/ are gitignored.~~ REVISED: Nothing under .claude/ is gitignored. Everything is tracked and committed: sessions, decisions, specs, prompts, contracts, reference.
**Rationale:** Gitignoring sessions broke the entire read-before-write protocol. Agents could not read each other's logs across separate sessions. The whole point of the log system is cross-session visibility.
**Affects:** All agents
**Status:** active (revised)

## ARCH-008: Navigator::is_idle() fix ownership
**Date:** 2026-02-07
**Decision:** This is a pre-existing bug in the Pumpkin codebase. Navigator lives in entity scope. Entity agent is authorized to fix Navigator::is_idle() to return correct state based on whether a path is active, but MUST NOT rename or restructure the Navigator struct. Fix the return value only.
**Rationale:** Entity's goal system (ENT-001 through ENT-003) depends on Navigator cycling correctly. WanderAround cannot hand off to other goals if is_idle() is stuck on false.
**Affects:** Entity
**Status:** active

## ARCH-009: Anvil deduplication — Storage provides, WorldGen consumes
**Date:** 2026-02-07
**Decision:** Storage's `pumpkin_nbt::anvil::RegionFile` is the canonical Anvil implementation. WorldGen should adopt it for region file I/O rather than maintaining a duplicate. WorldGen may wrap it with chunk-level convenience methods in pumpkin-world/ but must not fork or reimplement the Anvil logic.
**Rationale:** Storage session 001 produced a clean 420-line Anvil implementation with 17 tests. Duplicating this in pumpkin-world/ violates single-ownership. Shared interface through pumpkin-nbt/ crate dependency.
**Affects:** Storage, WorldGen
**Status:** active — WorldGen must acknowledge before adopting

## ARCH-010: Enderman teleportation is Entity scope
**Date:** 2026-02-07
**Decision:** Enderman teleportation is an entity behavior, not a world-level mechanic. Entity agent implements it using block query traits from pumpkin-util/. If Entity needs a "find valid teleport position" helper that queries world state, it requests the trait signature from Architect.
**Rationale:** Teleportation is mob AI (entity decides when/why to teleport). Block validity checking uses existing world query interfaces. No new world-level system needed.
**Affects:** Entity
**Status:** active

## ARCH-011: NEVER RENAME existing Pumpkin code
**Date:** 2026-02-07
**Decision:** No agent may rename, restructure, or modify existing Pumpkin variables, functions, structs, enums, modules, or file organization. Agents ADD and EXTEND only. This applies to all code that existed before our fork. The only exception is Architect resolving a documented blocker with explicit human operator approval.
**Rationale:** This is a public fork intended for upstream PRs. Renaming existing code breaks other contributors' work, creates unnecessary merge conflicts with upstream, and exceeds our mandate. We are here to complete missing features, not rewrite what works.
**Affects:** All agents
**Status:** active — NON-NEGOTIABLE

### ARCH-012: Vanilla Data Import
**Decision**: MC 1.21.4 vanilla data imported to `.claude/specs/data/` from misode/mcmeta.
Summary JSONs (blocks, registries, items, commands) accessible directly.
Full data (1370 recipes, 1237 loot tables, worldgen, tags, enchantments, damage types)
in `mcdata-1.21.4.zip` — agents extract locally as needed.
**Rationale**: Data-driven agents need canonical vanilla JSON to implement recipes,
loot tables, mob drops, worldgen, and tags without guessing.
**Affects**: Items, Entity, WorldGen, Storage, Redstone
**Status**: COMMITTED

## ARCH-013: PrismarineJS + Bukkit API Reference Data

**Decision**: Added two new data sources to `.claude/specs/data/`:
1. **PrismarineJS 1.21.4** — Entity hitboxes (width/height), food values (hunger/saturation), tool-material mining speeds, status effects. Full zip includes blocks, items, recipes, enchantments, biomes, particles. Source: github.com/PrismarineJS/minecraft-data
2. **Bukkit/Spigot API Reference** — Curated summary of 318 events across 11 packages (block, entity, player, inventory, world, server, etc.), key interfaces (Player, World, Block, ItemStack, JavaPlugin, BukkitScheduler), plugin lifecycle. Scraped from hub.spigotmc.org/javadocs/bukkit/

**Rationale**: Agents need behavioral data (hitboxes, food values, mining speeds) that misode/mcmeta doesn't provide. Plugin agent needs Bukkit API surface knowledge for compatibility layer design.

**Affects**: Entity (hitboxes, metadata), Items (foods, materials), Core (plugin API), all agents (Bukkit event mapping)

**Status**: COMMITTED (527fef50bcd3)

## ARCH-014: Stonecutting/smithing recipes generated in pumpkin-data build.rs
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/004_architect_recipe-codegen-and-event-macro.md
**Decision:** Recipe data for stonecutting (254), smithing_transform (12), and smithing_trim (18) is generated by pumpkin-data build.rs from assets/recipes.json. Three new static arrays: RECIPES_STONECUTTING, RECIPES_SMITHING_TRANSFORM, RECIPES_SMITHING_TRIM.
**Rationale:** Compile-time data from MC's JSON dumps. Follows ARCH-003 (Architect owns pumpkin-data). Unblocks Items agent (ITEMS-003).
**Affects:** Items, Architect
**Status:** active

## ARCH-015: Payload::is_cancelled() via Event derive field detection
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/004_architect_recipe-codegen-and-event-macro.md
**Decision:** Added `fn is_cancelled(&self) -> bool { false }` as default method on Payload. #[derive(Event)] detects `cancelled` field and generates override returning `self.cancelled`. Name follows Bukkit's `isCancelled()` convention (Rust snake_case: `is_cancelled`). Distinct from `Cancellable::cancelled()` to avoid trait method ambiguity on concrete types.
**Rationale:** Enables Bukkit-compatible ignore_cancelled filtering without trait object downcasting. Zero changes to existing event definitions.
**Affects:** Plugin, all agents that define events
**Status:** active

## ARCH-016: Multi-version support — tiered DTO rollback strategy
**Date:** 2026-02-07 (revised 2026-02-07)
**Decision:** Multi-version protocol support via DTO (Data Transfer Object) abstraction layer. One canonical internal model (1.21.11), version-specific serializers at network boundary. Implementation order works backwards from current version — each tier adds one more translation layer. Priority order: **1.18 > 1.16.5 > (1.14.x > 1.12)**. Tier 1 (1.18.2, protocol 758): chunk height only. Tier 2 (1.16.5, protocol 754): + Config state bypass + dimension codec + item NBT. Tier 3 (1.14.x, protocol 477): + chunk format v1 + villager overhaul. Tier 4 (1.12.2, protocol 340): + pre-Flattening block IDs (stretch goal). Tiers 3-4 are parenthesized — lower priority, attempt only after Tiers 1-2 are solid.
**Rationale:** Working backwards validates the DTO plumbing with the smallest delta first (1.18→1.21 is mostly chunk height). Each successive tier exercises harder translation layers incrementally. This avoids the risk of building all 5 layers at once and debugging a 20-variable system.
**Affects:** Protocol, Core, all agents
**Status:** active — DEFERRED (Phase 2, revised)

## ARCH-017: Tiered implementation order — 1.18 first, then 1.16.5
**Date:** 2026-02-07 (revised 2026-02-07)
**Session:** .claude/sessions/2026-02-07/005_architect_dto-1165-scoping.md
**Decision:** Tier 1 target is 1.18.2 (protocol 758), NOT 1.16.5. 1.18 is closest to current and only needs chunk height translation (-64..320 → 0..255 for old world format awareness, but 1.18 already has -64..320). The real translation needed is: packet ID remapping + minor field changes. This validates the DTO trait, the PacketId extension, and the version-conditional send path with minimal risk. Tier 2 (1.16.5) then adds the hard layers: Config state bypass, item component→NBT translation, dimension codec inline in Join Game. 5 total translation layers scoped in session 005.
**Rationale:** Smallest delta first. 1.18→1.21 exercises the DTO wiring without the Config state nightmare. If the plumbing works for 1.18, adding 1.16.5 layers is incremental. Estimated: Tier 1 ~500-800 lines, Tier 2 ~2000-3000 lines additional.
**Affects:** Protocol, Core, all agents
**Status:** active — DEFERRED (Phase 2, scoped)

## ARCH-018: Config state bypass for pre-1.20.2 clients
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/005_architect_dto-1165-scoping.md
**Decision:** Clients with protocol < 764 (pre-1.20.2) skip the Config connection state entirely. Registry data, tags, and feature flags are embedded in the Join Game packet or sent during early Play, matching vanilla 1.16.5 behavior. Pumpkin's connection state machine in `net/java/mod.rs` must branch on version after Login.
**Rationale:** The Config state was added in 1.20.2. 1.16.5 clients go Handshake→Login→Play. Trying to force old clients through Config would break the connection.
**Affects:** Protocol, Core
**Status:** active — DEFERRED (Phase 2)

## ARCH-019: DTO module lives in pumpkin-protocol/src/dto/
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/005_architect_dto-1165-scoping.md
**Decision:** All version translation code lives in `pumpkin-protocol/src/dto/`. Protocol agent owns it. The DTO module imports internal types but does not modify them. Existing packet definitions stay untouched (ARCH-011 compliant). Structure: `dto/mod.rs` (VersionAdapter trait), `dto/v1_21.rs` (passthrough), `dto/v1_16_5/` (translation modules for chunks, items, login, player_info).
**Rationale:** The DTO is a protocol concern — it translates wire formats. Keeping it in pumpkin-protocol/ maintains the single-crate boundary for network serialization. Purely additive module, zero changes to existing code.
**Affects:** Protocol
**Status:** active — DEFERRED (Phase 2)

## ARCH-020: PatchBukkit transcode + Storage DTO + LanceDB option
**Date:** 2026-02-07
**Decision:** Two-pronged architectural enhancement:

**Prong 1 — PatchBukkit Transcode:** Instead of maintaining a separate Java-bridge repo (JVM embedding + protobuf FFI), transcode PatchBukkit's Bukkit API knowledge into pure Rust. Harvest the event mapping catalog, API surface definitions, and plugin lifecycle from PatchBukkit's proto/Java/Rust layers and encode it as Rust trait impls in pumpkin's plugin system. This eliminates the JVM dependency for servers that only need Rust plugins while preserving Bukkit API compatibility at the type level.

**Prong 2 — Storage DTO with pluggable backends:** Abstract game data access behind a `GameDataStore` trait with two backend implementations:
1. **TOML/YAML backend** — Human-editable, file-based. For configs, small registries, dev/modding use.
2. **LanceDB backend** — Embedded columnar DB (no server process). Zero-copy via Apache Arrow IPC. For high-performance queries over large registries (26K+ block states, 1470 recipes, 149 entities, loot tables). DataFusion provides SQL query capability over Arrow RecordBatches.

**Key crates:** Lance 2.0+ / `lancedb` (embedded), `arrow` v57+ (zero-copy), `datafusion` v51+ (SQL engine)

**Storage DTO trait sketch:**
```rust
#[async_trait]
trait GameDataStore: Send + Sync {
    async fn blocks(&self) -> &dyn BlockRegistry;
    async fn items(&self) -> &dyn ItemRegistry;
    async fn recipes(&self) -> &dyn RecipeRegistry;
    async fn entities(&self) -> &dyn EntityRegistry;
    async fn query(&self, sql: &str) -> Result<RecordBatch>; // DataFusion SQL
}
```

**LanceDB benefits for Pumpkin:**
- Embedded (in-process, no daemon) — aligns with self-contained server model
- Zero-copy Arrow IPC between subsystems (world gen ↔ entity ↔ protocol)
- Columnar format ideal for batch operations (e.g., "all stone variants in chunk")
- Built-in versioning for snapshot/rollback of game state
- DataFusion SQL for admin/plugin queries ("what recipes use iron ingot?")

**Migration path:**
1. Define `GameDataStore` trait in `pumpkin-util/` or new `pumpkin-store/` crate
2. Implement TOML/YAML backend first (wraps current pumpkin-data generated arrays)
3. Implement LanceDB backend (imports same JSON source data into Lance format at build time)
4. Feature-flag selection: `--features lance-store` vs default TOML
5. Transcode PatchBukkit proto definitions → Rust DTO structs for the storage layer

**Rationale:** PatchBukkit's JVM bridge is the right design for running actual Java plugins, but for pure-Rust servers the JVM is dead weight. Transcoding the API knowledge into Rust DTOs with a pluggable storage backend gives us: (a) Bukkit-compatible API surface without JVM, (b) zero-copy data access via Arrow, (c) SQL query capability for plugins/admin, (d) human-editable fallback via TOML/YAML.

**Rust compatibility:** Lance 2.0 requires Rust 2024 edition (1.88+). Pumpkin MSRV is 1.89 — compatible. But lance-store is fully optional: `default = ["toml-store"]` compiles zero Lance/Arrow deps. The `lance-store` feature is an empty gate until chrono version conflict is resolved upstream. Verified: `--no-default-features`, default, and `--features lance-store` all build clean.

**Implementation status (Phase 1-2 DONE):** `pumpkin-store/` crate created as 10th workspace member. `GameDataStore` trait + `StaticStore` backend (9 tests pass). `LanceStore` stub behind feature gate. See session 009 for details.

**Affects:** All agents (new data access pattern), Plugin (API surface), Storage (backend impl), Items (recipe queries), Core (data loading)
**Status:** PHASE 1-2 DONE, PHASE 3-4 PENDING (lance deps blocked on chrono conflict)

## ARCH-021: Type corrections are NOT renames (ARCH-011 clarification)
**Date:** 2026-02-07
**Decision:** Fixing incorrect field types on existing enum variants or struct fields is authorized under ARCH-011 when ALL of the following conditions are met:
1. The **name** of the variant/field stays identical
2. The current type is **demonstrably wrong** per the Minecraft protocol specification (wiki.vg)
3. The variant/field is **not constructed anywhere** in the current codebase (no callers to break)
4. The fix is **required for correct serialization** — the current type makes the feature non-functional

**ARCH-011 says "never rename."** Changing `UpdateLatency(u8)` to `UpdateLatency(VarInt)` is not a rename — "UpdateLatency" stays "UpdateLatency." It's a type correction. This is analogous to ARCH-008 (Navigator::is_idle() return value fix).

**Specific authorizations for Protocol agent:**
- `PlayerAction::UpdateLatency(u8)` → `PlayerAction::UpdateLatency(VarInt)` — latency is VarInt per wiki.vg
- `PlayerAction::UpdateDisplayName(u8)` → `PlayerAction::UpdateDisplayName(Option<TextComponent<'a>>)` — spec says Optional Chat
- `PlayerAction::UpdateListOrder` → `PlayerAction::UpdateListOrder(VarInt)` — spec says VarInt sort priority

**What is NOT authorized:** Renaming `UpdateLatency` to `SetPing`, renaming `PlayerAction` to `PlayerInfoAction`, restructuring the enum into separate structs, moving the file, etc.

**Rationale:** ARCH-011 prevents unnecessary refactoring that breaks contributors. Type corrections that fix spec compliance on unused variants have zero blast radius — no callers exist, no one's code breaks. Blocking these fixes means leaving `todo!()` panics and incorrect serialization in production code, which is worse than the type change.
**Affects:** Protocol
**Status:** active

## ARCH-022: ARCH-019 (Protocol DTO) and ARCH-020 (Storage DTO) are complementary
**Date:** 2026-02-07
**Decision:** The two DTO layers serve different purposes and do not conflict:
- **ARCH-019** (Protocol DTO, `pumpkin-protocol/src/dto/`): Translates **wire formats** between Minecraft protocol versions. Lives at the network boundary. Owned by Protocol agent.
- **ARCH-020** (Storage DTO, `pumpkin-store/`): Abstracts **game data access** behind pluggable backends (static arrays, LanceDB). Lives at the data layer. Owned by Architect/Storage.

They are orthogonal: Protocol DTO translates packets at the network edge, Storage DTO serves game registries internally. A block lookup goes through `GameDataStore`, then the result may be serialized differently for 1.21 vs 1.16.5 clients via the Protocol DTO. No conflict.
**Affects:** Protocol, Storage, Architect
**Status:** active

## ARCH-023: Event-firing write access for integration points
**Date:** 2026-02-07
**Decision:** The ownership gap for `pumpkin/src/world/` and `pumpkin/src/net/` is resolved as follows:

**Primary ownership stays unchanged:**
- `pumpkin/src/world/` — WorldGen (structure generation, terrain, chunk management)
- `pumpkin/src/net/` — Protocol (packet handlers, connection state machine)

**Event-firing write access granted:**
- **Entity agent** gets write access to `pumpkin/src/world/natural_spawner.rs` and `pumpkin/src/world/mod.rs` — specifically for wiring `EntitySpawnEvent` at spawn call sites. Entity MUST NOT modify chunk generation, structure placement, or world management logic.
- **Plugin agent** gets write access to `pumpkin/src/net/java/play.rs` — specifically for wiring `ServerListPingEvent` in status handler and `CustomPayloadEvent` in play handler. Plugin MUST NOT modify packet parsing, connection state machine, or crypto/compression logic.
- **Core agent** gets write access to `pumpkin/src/world/mod.rs` — specifically for wiring Core lifecycle events (save, tick) that touch the world. Core MUST NOT modify terrain generation or chunk loading.

**How it works:** When an agent needs to fire an event from another agent's code, they add a single `plugin_manager.fire(Event { ... }).await` call at the appropriate point. The event type itself lives in `pumpkin/src/plugin/api/events/` (Plugin's scope). The fire call is a thin integration point, not a new feature.

**Rules for cross-agent event wiring:**
1. The change must be a single `fire()` call or equivalent — no logic changes
2. The event struct must already exist in Plugin's event module
3. The commit message must include `[{your-agent}+{owner-agent}]` prefix (e.g., `[entity+world]`)
4. The owning agent must be notified via broadcast before push

**Rationale:** Event wiring is the #1 cross-agent blocker. 37 event types exist but most are never fired because the fire call sites are in other agents' territory. Granting narrow write access for event wiring unblocks Entity (EntitySpawnEvent), Plugin (ServerListPingEvent), and Core (save events) without transferring ownership of the modules.
**Affects:** Entity, Plugin, Core, WorldGen, Protocol
**Status:** active

## ARCH-024: Items should NOT adopt GameDataStore yet
**Date:** 2026-02-07
**Decision:** Items agent continues using pumpkin-data statics directly for recipe matching. The `GameDataStore` trait (ARCH-020) is for future use when the Lance backend is ready. Adopting it now would add an indirection layer with zero benefit — `StaticStore` just wraps the same pumpkin-data arrays Items already uses.

When Lance backend is ready (Phase 4), Items will migrate recipe queries to `store.recipes_for_output()` and `store.sql("SELECT...")`. Until then, direct pumpkin-data access is correct.
**Affects:** Items
**Status:** active

## ARCH-025: Tiered Store Provider Architecture
**Date:** 2026-02-07
**Decision:** pumpkin-store implements three provider tiers via `StoreProvider` enum:
1. **Static** (default) — `StaticStore` wrapping pumpkin-data compile-time arrays. Zero runtime cost.
2. **Cached** — `CachedStore<S>` wrapping any `GameDataStore` + `HashMap` memoization. Each entry is a transparent `CacheEntry<T>` DTO with method/key metadata for inspection.
3. **Lance** — `LanceStore` hydrated FROM `StaticStore` via `hydrate_from()`. No external JSON files needed. Lance 2.0 native queries (no `DataFusion` sidecar).

Key design:
- `CacheEntry<T>` has `method: Cow<str>`, `key: Cow<str>`, `value: T` — transparent, serializable
- `CacheSnapshot` tracks cache sizes across all maps
- `LanceStore::hydrate_from(&dyn GameDataStore)` reads all records from source store, writes to Lance tables
- Static is always the source of truth — Cache and Lance bootstrap from it
- Lance 2.0 includes its own query engine; `DataFusion` is NOT needed as a sidecar

**Feature flags unchanged:** `default = ["toml-store"]`, `lance-store = []` (empty until Phase 4)
**Tests:** 23 pass (10 static + 13 cached), clippy clean
**Affects:** All agents (future consumers of `GameDataStore`)
**Status:** active

## ARCH-026: Calcite Arrow Java Provider for PatchBukkit
**Date:** 2026-02-07
**Decision:** Future Phase 5 adds Apache Calcite as the Java-side query engine for transcoded PatchBukkit plugins. The full zero-copy chain:

```
PatchBukkit Java plugins (Bukkit API calls)
    ↓
Apache Calcite (Java SQL engine + optimizer)
    ↓ Arrow IPC (zero-copy across JNI/FFI boundary)
pumpkin-store LanceStore (Rust, Lance 2.0)
    ↓ hydrate_from()
StaticStore (pumpkin-data, compile-time)
```

- Calcite provides SQL parsing, optimization, and execution on the Java side
- Arrow IPC provides zero-copy data sharing between Java and Rust (no serialization)
- Lance 2.0 on Rust side serves Arrow `RecordBatch` natively
- holograph XOR cache pattern guards against early zero-copy break at the boundary
- This is the GEL (Graph Execution Language) substrate: Java `@lance` annotations compile to Calcite queries over Arrow storage

**Prerequisites:**
- Phase 4 (real Lance deps) must be complete
- PatchBukkit proto definitions must be transcoded (Phase 3)
- Arrow Java and Calcite versions must align with Arrow Rust (currently arrow 57)

**Affects:** Plugin (PatchBukkit bridge), all agents (data consumers)
**Status:** planned (Phase 5)

## ARCH-027: Game Mapping Table + XOR for Mob Goal States
**Date:** 2026-02-07
**Decision:** Add a separate `game_mapping` table to Lance for cross-entity relationship data, and apply XOR write-through guard to mob goal state transitions.

**Game Mapping Table:**
A relationship table that cross-cuts entity tables (blocks, items, entities):
- `source_type` + `source_key` → `target_type` + `target_key`
- Examples: biome→entity_spawn, structure→loot_table, block→item_drop, mob→goal_state
- Not a game-object table — a *mapping* table. Lives alongside blocks/items/entities in Lance.
- Queryable via Lance 2.0 native API: `game_mapping.filter("source_type = 'biome' AND source_key = 'plains'")`

**XOR for Mob Goal States:**
The Entity agent's `GoalSelector` evaluates 16 goal types across 4 control slots (`MOVE`, `LOOK`, `JUMP`, `TARGET`) every tick, with priority-based switching. During tick evaluation:
1. Goals are stopped/started (ephemeral switching)
2. Goal state (target position, cooldown, entity refs) transfers between slots
3. XOR tag on goal state records detects if `Cow::Borrowed` fields (entity type names, block names in goals) were silently materialized to `Cow::Owned` during the switch

The `game_mapping` table stores the static goal→mob relationships (which goals each mob type has), while the XOR guard protects the runtime goal state during tick arbitration.

**Implementation path:**
1. Add `GameMappingRecord` to `pumpkin-store/src/traits.rs`
2. Add `game_mappings()` query to `GameDataStore` trait
3. In Phase 4, create `game_mapping` Lance table alongside blocks/items/entities
4. Entity agent integrates XOR guard into `PrioritizedGoal` state for tick-level verification

**Affects:** Entity (goal states), WorldGen (biome→spawn mappings), Items (block→drop mappings)
**Status:** planned

## ARCH-028: Three Store Scopes
**Date:** 2026-02-07
**Decision:** pumpkin-store data model has three distinct scopes with different storage strategies:

1. **Entity Data** (Item/Mob/Recipe) — keyed by name/ID. `GameDataStore` trait methods: `block_by_id`, `item_by_name`, `entity_by_name`, `recipes_for_output`. All tiers (Static/Cached/Lance) implement these.

2. **Worldmap** (Chunks/Regions/Biomes/Structures) — spatial data indexed by XYZ coordinates. Currently in pumpkin-world's Anvil format. Future: `WorldMapStore` trait for Lance columnar queries over spatial data (chunk sections, biome grids, structure bounding boxes).

3. **Activity Overlay** (State Transitions/XOR) — `MobGoalState` Hamming XOR overlay, `GameMappingRecord` cross-entity relationships. Tracks ephemeral state changes per tick. Bind/unbind semantics for entities entering/leaving spatial regions.

Each scope maps to different Lance tables and query patterns:
- Scope 1: dense lookup tables (blocks, items, entities, recipes)
- Scope 2: spatial index tables (world positions, chunk sections, biome grids)
- Scope 3: sparse overlay tables (goal state transitions, relationship mappings)

`StoreProvider::open()` is the meta-switch that NATs all commands to the right backend tier, transparent to callers.

**Affects:** All agents
**Status:** active

## ARCH-029: SIMD Content-Addressable Memory Vision (AVX-512)
**Date:** 2026-02-07
**Decision:** Long-term vision to partially replace per-entity tick iteration with SIMD batch processing over content-addressable spatial memory:

```text
Current:  for entity in entities { entity.tick() }  // sequential O(n)

Future:   CAM[x,y,z].bind(entity)     // entity enters spatial region
          CAM[x,y,z].unbind(entity)    // entity leaves spatial region
          AVX-512 batch tick per region // 16x f32 parallel per SIMD lane
```

Arrow columnar format is the substrate:
- Columns: `[x: f32, y: f32, z: f32, entity_id: u32, goal_state: u64]`
- AVX-512 `f32` lanes: 16 entities per instruction (512 bits / 32 bits)
- Bind/unbind: insert/delete from spatial Arrow `RecordBatch`
- Content-addressable: entities indexed by position, not linear entity ID
- XOR overlay (ARCH-027) per SIMD batch detects state transition anomalies

Prerequisites:
- ARCH-028 Scope 2 (Worldmap spatial store) must exist
- Lance Phase 4 (real Arrow `RecordBatch` in memory)
- Rust `std::arch::x86_64` for AVX-512 intrinsics (or `packed_simd2` / `std::simd`)
- Fallback to scalar for non-AVX-512 platforms (ARM NEON as Tier 2)

**Affects:** Entity (tick loop), Core (scheduler), WorldGen (chunk spatial index)
**Status:** vision (long-term)

## ARCH-030: Biome Height Reduction — 256-block Surface-Relative XOR
**Date:** 2026-02-07
**Decision:** The SpatialOverlay (ARCH-029) can optimize biome processing by using reduced 256-block height relative to surface XY with XOR overlay. Instead of storing the full 384-block world height (Y=-64 to Y=320), biomes compress to a 256-block height vector relative to the surface position at each (X,Z) column.

**Design:**
- Surface height map at (X,Z) → Y_surface (from heightmap data)
- Overlay height = Y - Y_surface, clamped to [0, 256) relative range
- Static table: Anvil heightmap snapshot (XY surface), zero-copy
- Ephemeral table: TNT/mining/mob height changes XOR'd against static
- Rollback: discard ephemeral, static remains intact

**Benefits:**
- 256-block range fits in 8 bits instead of 9 (384 range needs 9 bits)
- Surface-relative encoding means underground (negative relative Y) and sky (positive) are symmetric
- XOR between static surface and ephemeral changes isolates terrain modifications
- AVX-512: 64 height columns per SIMD operation (8-bit × 64 = 512 bits per lane)

**Affects:** WorldGen (heightmap data), Entity (mob height tracking), Redstone (signal propagation height)
**Status:** vision (extension of ARCH-029)

## ARCH-031: Redstone Computer Benchmark — 8 FPS Video Display
**Date:** 2026-02-07
**Decision:** Target benchmark for SIMD CAM optimization (ARCH-029): a redstone computer using 1-block-distance pyramid/reverse-pyramid signal propagation pattern that can drive a screen displaying video at 8 FPS.

**Why this benchmark matters:**
- Redstone computers are the hardest stress test for tick performance
- 1-block pyramid/reverse-pyramid is a common redstone CPU architecture (converging/diverging signal tree)
- 8 FPS at 20 TPS means 2.5 ticks per frame — every tick must evaluate thousands of redstone dust positions
- If the SIMD CAM + SpatialOverlay can handle this, it proves the architecture scales to real-world complexity
- Vanilla Minecraft redstone computers typically run at <1 FPS due to sequential tick evaluation

**Architecture fit:**
- Each redstone dust position is a spatial bind in the SpatialOverlay
- Signal propagation is XOR diff between tick N and tick N+1
- AVX-512 batch evaluates 16 redstone positions per instruction (f32 signal strength)
- Pyramid pattern maps naturally to spatial hash locality (adjacent blocks hash nearby)

**Affects:** Redstone (signal evaluation), Core (tick scheduler), Entity (SIMD CAM)
**Status:** vision (benchmark target for ARCH-029)

## ARCH-032: Redstone Agent Expanded to Block Event Wiring
**Date:** 2026-02-07
**Decision:** Redstone agent's core logic (comparator, observer, repeater, piston) is complete. Expanding its scope to cover all block state event wiring — fire spread, fluid flow, crop growth, snow decay, note blocks, sponge absorption.

**Ownership transfers (from block-ownership.toml):**
- `fire/` — from shared_structural → redstone (BlockBurnEvent)
- `plant/` — from worldgen → redstone (BlockGrowEvent)
- `snow.rs` — from worldgen → redstone (BlockFadeEvent)
- `fluid/` — from worldgen → redstone (BlockFromToEvent)
- `note.rs` — from shared_structural → redstone (NotePlayEvent)
- `sponge.rs` — from shared_structural → redstone (SpongeAbsorbEvent)

**Rationale:** Redstone is the domain expert for block state transitions, update ordering, and event firing. Their 4 high-priority events are done. The remaining 9 lower-priority events + 4 block events all involve block state changes that follow the same patterns Redstone already handles. No new agent needed — Redstone absorbs the work.

**Contract update:** Redstone write_paths now includes `fire/`, `plant/`, `snow.rs`, `fluid/`, `note.rs`, `sponge.rs` in addition to `redstone/` and `piston/`.

**Affects:** Redstone (expanded scope), WorldGen (loses plant/, snow.rs, fluid/ write access — READ still allowed)
**Status:** active

## ARCH-033: `/execute` Command Architecture
**Date:** 2026-02-08
**Decision:** `/execute` uses recursive dispatch with `ExecutionContext` struct, NOT a deep command tree. Each subcommand modifier (as, at, positioned, rotated, facing, align, anchored, in) transforms the context then re-dispatches. `run` dispatches the inner command through existing `CommandDispatcher::dispatch()` with the modified sender. Multi-target (`as @a`) fans out with cloned contexts. Implementation in 4 phases: (1) skeleton+run+as+at, (2) position/rotation modifiers, (3) conditionals, (4) store+advanced.
**Rationale:** The recursive nature of `/execute` subcommand chaining makes a flat tree impossible — any modifier can follow any other. Recursive dispatch matches vanilla behavior and keeps each modifier self-contained.
**Affects:** Core (implements), Plugin (event firing from execute)
**Status:** active

## ARCH-034: `/function`, `/schedule`, `/return` Command Design
**Date:** 2026-02-08
**Decision:** `/function` loads `.mcfunction` files from datapacks into `FxHashMap<ResourceLocation, Vec<String>>`, dispatches each line. Permission level from `function_permission_level` config (CORE-014). `/schedule` adds `ScheduledFunctions` to Server, checked each tick. `/return` uses `FunctionExecutionState` with early-exit. Implementation order: execute P1 → function → execute P2 → schedule → execute P3 → return → execute P4.
**Rationale:** Functions enable datapacks (critical for vanilla parity). Schedule depends on function. Return depends on function execution state. Execute phases interleave to provide incremental value.
**Affects:** Core (implements)
**Status:** active
