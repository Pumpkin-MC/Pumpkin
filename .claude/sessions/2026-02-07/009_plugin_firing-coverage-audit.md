# Session 009 — Plugin Agent: Event Firing Coverage Audit + ARCH-023 Wiring

**Agent:** Plugin
**Date:** 2026-02-07
**Branch:** claude/plugin-api-events-5Q5l2

## Preamble — Logs Read

- Read all sessions from 2026-02-07 (sessions 001-009)
- Read decisions/architect.md (ARCH-001 through ARCH-024)
- Read decisions/plugin.md (PLUGIN-001 through PLUGIN-008)
- Read updated plugin prompt (P0 priority: event firing coverage)
- Noted Entity wired EntitySpawnEvent (PR #57), Core added save commands (PR #56)
- Noted Architect added ARCH-023 cross-agent wiring, pumpkin-store crate

## What Was Done

### 1. Rebased on latest master (e2fa5e8)

Picked up PRs #56-#62:
- Entity: EntitySpawnEvent wired in spawn_entity/natural_spawner
- Core: save-all/save-off/save-on commands
- Architect: pumpkin-store crate, ARCH-023/024, P0-P2 priority matrix, prompt updates
- Redstone: Component verification + 30 tests

### 2. Comprehensive Event Firing Audit (PLUGIN-009)

Initial audit (using grep for `fire(`) found only 14/39 events fired. Discovered that `send_cancellable!` macro also fires events — corrected audit found **25/39 events fired (64% coverage)**.

**Events fired via send_cancellable! (not caught by initial grep):**
- PlayerMoveEvent (play.rs:289, 403)
- PlayerChatEvent (play.rs:878, bedrock/play.rs:143)
- PlayerCommandSendEvent (play.rs:559, bedrock/play.rs:191)
- PlayerInteractEvent (play.rs:1797)
- PlayerLoginEvent (server/mod.rs:417)
- PlayerChangeWorldEvent (entity/player.rs:1698)
- PlayerTeleportEvent (entity/player.rs:1780, 2930)
- PlayerGamemodeChangeEvent (entity/player.rs:1939)
- ServerCommandEvent (lib.rs:541, 608)
- ServerBroadcastEvent (server/mod.rs:534)
- BlockBreakEvent (world/mod.rs:2977)

**Coverage summary:**
| Category | Defined | Fired | % |
|----------|---------|-------|---|
| Server | 7 | 7 | 100% |
| Entity | 4 | 4 | 100% |
| Player | 14 | 10 | 71% |
| Block | 11 | 4 | 36% |
| World | 3 | 0 | 0% |
| **Total** | **39** | **25** | **64%** |

### 3. Wiring Roadmap for Remaining 14 Events

All 14 unfired events require wiring from code outside Plugin's write_paths:

**Entity agent (4 events):**
- PlayerDeathEvent → entity/player.rs death handling
- PlayerRespawnEvent → entity/player.rs respawn handling
- PlayerDropItemEvent → entity/player.rs `drop_held_item()`
- PlayerItemConsumeEvent → entity/living.rs food consumption completion

**Core/WorldGen agent (5 events):**
- BlockPlaceEvent → world/mod.rs `set_block()` / place handlers
- BlockCanBuildEvent → world/mod.rs place validation
- ChunkLoad → world/mod.rs chunk lifecycle
- ChunkSave → world/mod.rs chunk lifecycle
- ChunkSend → world/mod.rs chunk send to player

**Redstone/Block agent (5 events):**
- BlockBurnEvent → fire spread tick logic
- BlockPhysicsEvent → neighbor update propagation
- BlockFromToEvent → liquid flow / dragon egg
- BlockGrowEvent → crop/sapling tick
- BlockFadeEvent → ice/snow/coral decay tick

## Decisions Made

- **PLUGIN-009:** Event firing coverage audit — 25/39 (64%). Wiring roadmap documented.

## What Others Should Know

- **Entity agent**: You need to wire 4 player events: PlayerDeathEvent, PlayerRespawnEvent, PlayerDropItemEvent, PlayerItemConsumeEvent. All event structs are defined and ready.
- **Core/WorldGen**: You need to wire 5 events: BlockPlaceEvent, BlockCanBuildEvent, ChunkLoad, ChunkSave, ChunkSend.
- **Redstone/Block**: You need to wire 5 events: BlockBurnEvent, BlockPhysicsEvent, BlockFromToEvent, BlockGrowEvent, BlockFadeEvent.
- **All**: The `send_cancellable!` macro fires events — use it instead of raw `plugin_manager.fire()` when you need cancellation support with rollback.

## What I Need From Others

- **Entity**: Wire PlayerDeathEvent, PlayerRespawnEvent, PlayerDropItemEvent, PlayerItemConsumeEvent
- **Core/WorldGen**: Wire BlockPlaceEvent, BlockCanBuildEvent, ChunkLoad/Save/Send
- **Redstone**: Wire BlockBurnEvent, BlockPhysicsEvent, BlockFromToEvent, BlockGrowEvent, BlockFadeEvent

## Tests

All 32 plugin tests pass. No new code changes in this session (audit + documentation only).

## Event Count

**Total: 39 events** (7 server, 14 player, 11 block, 4 entity, 3 world)
**Fired: 25 (64%)**
**Unfired: 14 (36%) — all need cross-agent wiring**
