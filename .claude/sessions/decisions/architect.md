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
