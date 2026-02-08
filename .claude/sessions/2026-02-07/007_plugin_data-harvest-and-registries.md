# Session 007 — Plugin Agent: Data Harvest & Registry Packages

**Agent:** plugin
**Date:** 2026-02-07
**Branch:** claude/plugin-api-events-5Q5l2

## Preamble

Read all prior session logs (001–006), decisions for plugin and architect, CLAUDE.md, and plugin.md prompt. Blackboard hydrated — inbox empty.

Key context from prior sessions:
- 003_plugin: Created 28 events, fixed priority sorting, added Monitor + ignore_cancelled
- 004_architect: Added Payload::is_cancelled() via derive macro (ARCH-015), then renamed to cancelled() (ARCH-011 violation), then reverted back to is_cancelled() (commit 5cabb27)
- PLUGIN-004 unblocked: Architect landed is_cancelled(), ignore_cancelled filtering ready to implement

## What I Did

### 1. Rebased onto latest master
- Clean rebase, 0 conflicts. All prior plugin commits already merged via PRs.
- Master advanced 15 commits including architect's blackboard, CLAUDE.md, agent prompts.

### 2. Updated stale references
- Removed "blocked on Architect" comments from fire() method and PLUGIN-004 decision
- Updated PLUGIN-004 status: "ready to implement filtering"

### 3. Built TOML registry system (.claude/registry/)

**bukkit_api.toml** (60KB):
- 283 Bukkit events as individual [[events]] entries (281 Bukkit + 2 Pumpkin-specific)
- Each entry: name, package, cancellable, rust_equivalent, rust_path, fields, status, owner
- 28 implemented, 1 stub, 252 missing
- 9 interfaces, 5 enums, 1 annotation (EventHandler with ignoreCancelled)
- Owner field verified against all agent contracts
- Sprint summary with counts by agent

**entities.toml** (41KB):
- 149 entities from PrismarineJS 1.21.4 with hitboxes, categories
- 39 effects, 40 foods, 49 damage types
- pumpkin_status field for each entity

**items.toml** (10KB):
- Item component type summary (1385 items, not individually listed)
- 42 enchantments, recipe/loot table counts
- Pumpkin coverage assessment

**blocks.toml** (12KB):
- 198 block types, mining speed table
- Worldgen inventory (135 biomes, 47 structures)
- Block ownership cross-ref

### 4. Built markdown reference packages (.claude/reference/)
- 7 agent-specific briefings with gap analysis
- SPRINT-INDEX.md linking registries + references with query examples

### 5. Harvested MC 1.16.5 PrismarineJS data
- Downloaded 16 files from PrismarineJS inheritance chain (1.16 → 1.16.1 → 1.16.2 → 1.16.4)
- Stored in .claude/specs/data/1.16.5/prismarine/
- Built delta registries: entities_1_16_5.toml (108 entities), items_1_16_5.toml (38 enchantments, 33 effects, 40 foods)
- Each entry annotated with in_1_21_4 flag for version delta tracking

### 6. Bukkit API documentation findings
- Bukkit's Cancellable interface: `isCancelled()` / `setCancelled(boolean)` — the `is` prefix is canonical
- Bukkit's @EventHandler: `ignoreCancelled = true` skips handler if event already cancelled
- Architect's commit 75c97c7 renamed `is_cancelled` → `cancelled` (ARCH-011 violation) but was reverted in 5cabb27

## Version Delta Summary (1.16.5 → 1.21.4)

| Category | 1.16.5 | 1.21.4 | Delta |
|----------|--------|--------|-------|
| Entities | 108 | 149 | +42, -1 rename |
| Blocks | 763 | 1095 | +334, -2 renames |
| Items | 975 | 1385 | +413 |
| Effects | 33 | 39 | +8 |
| Enchantments | 38 | 42 | +4 |
| Biomes | 79 | 135 | +56 |
| Recipes | 562 | 1370 | +808 |

## Decisions Made

### PLUGIN-005: Multi-version data harvest with delta annotations
**Decision:** Store versioned data in .claude/specs/data/{version}/ with TOML registries that include `in_1_21_4` cross-reference flags.
**Rationale:** Enables future multi-version support. Agents can see exactly what changed between versions without manual diffing.
**Status:** active — FLOW

## What Others Should Know

- **All agents:** Your sprint backlog is queryable: `grep 'owner = "YOUR_AGENT"' .claude/registry/bukkit_api.toml | grep name`
- **All agents:** Read `.claude/reference/SPRINT-INDEX.md` on session start
- **Entity:** You own 118 missing Bukkit events. Top priorities in your reference file.
- **Items:** You own 46 missing events + recipe/loot/enchantment systems.
- **Architect:** When implementing multi-version support, delta registries exist for 1.16.5 vs 1.21.4.

## What I Need From Others

- **All agents:** When you implement an event, update `status = "implemented"` in bukkit_api.toml
- **Core:** Need ServerListPingEvent firing point for protocol agent

## Files Created/Modified

### Created (20 files)
- .claude/registry/bukkit_api.toml
- .claude/registry/entities.toml
- .claude/registry/items.toml
- .claude/registry/blocks.toml
- .claude/registry/entities_1_16_5.toml
- .claude/registry/items_1_16_5.toml
- .claude/reference/SPRINT-INDEX.md
- .claude/reference/{entity,world,items,plugin,protocol,redstone,core}-data.md
- .claude/specs/data/1.16.5/README.md
- .claude/specs/data/1.16.5/prismarine/*.json (16 files)

### Modified (2 files)
- pumpkin/src/plugin/mod.rs — updated stale architect comment
- .claude/sessions/decisions/plugin.md — PLUGIN-004 status update
