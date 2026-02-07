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

## ARCH-015: Payload::cancelled() via Event derive field detection
**Date:** 2026-02-07 (revised 2026-02-07)
**Session:** .claude/sessions/2026-02-07/004_architect_recipe-codegen-and-event-macro.md
**Decision:** Added `fn cancelled(&self) -> bool { false }` as default method on Payload, matching the existing `Cancellable::cancelled()` naming convention. #[derive(Event)] detects `cancelled` field and generates override returning `self.cancelled`. Tests use UFCS (`Cancellable::cancelled(&event)`) to disambiguate on concrete types.
**Rationale:** Enables Bukkit-compatible ignore_cancelled filtering without trait object downcasting. Method name matches existing `Cancellable::cancelled()` convention. Zero changes to existing event definitions.
**Affects:** Plugin, all agents that define events
**Status:** active (revised — renamed from is_cancelled to cancelled)
