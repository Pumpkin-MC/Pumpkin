# Session: core-004
**Agent:** core
**Date:** 2026-02-07
**Task:** Command audit, config audit, ownership gap documentation, config additions

## Context Loaded
- Rebased onto origin/master (d143573) — 12 new sessions merged since core-003
- Read all new sessions: 002_protocol, 002_redstone, 002_storage, 002_world, 003_world, 005_architect, 007_entity, 007_plugin, 008_items, 009_architect
- Read .claude/sessions/decisions/core.md (CORE-001 through CORE-005)
- Read .claude/sessions/decisions/architect.md (ARCH-001 through ARCH-020)
- This is the third core session (after 002_decomposition, 003_lifecycle-events)

## Preamble

I am the Core agent. I acknowledge:
- Entity session 007 (ENT-007): EntitySpawnEvent needs firing from entity spawning code. Entity agent says this is Core scope but `pumpkin/src/world/` is **NOT in my write_paths**. This is an ownership gap.
- Plugin session 007: ServerListPingEvent firing point requested — also likely in net/ which is Core FORBIDDEN.
- Architect sessions 005+009: Multi-version DTO (ARCH-016-019) and GameDataStore (ARCH-020) are deferred Phase 2, no current Core impact.
- Protocol session 002: SCustomPayload handler needs wiring in net/ — FORBIDDEN for Core.
- All other agents' work is additive and non-blocking for Core.

## Blocking Issues Found

### 1. Ownership Gap: `pumpkin/src/world/`
**Status:** BLOCKED — needs Architect resolution
**Issue:** Entity agent (ENT-007) needs `EntitySpawnEvent` fired during entity spawning in `pumpkin/src/world/mod.rs`. Neither Core nor Entity owns `pumpkin/src/world/`. Core's write_paths include `pumpkin/src/server/` but not `pumpkin/src/world/`. Entity's write_paths include `pumpkin/src/entity/` but not world.
**Impact:** Entity spawning for structures (World sessions 002/003: 13 structures) cannot fire plugin events. Plugin system remains disconnected from world spawning.
**Proposed resolution:** Architect should either:
  - (a) Add `pumpkin/src/world/` to Core's write_paths (since Core owns the tick loop that drives world ticking), or
  - (b) Add `pumpkin/src/world/` to a new "World" agent's write_paths, or
  - (c) Grant Entity temporary write access to `pumpkin/src/world/mod.rs` for the specific spawn event integration

### 2. Ownership Gap: `pumpkin/src/net/`
**Status:** BLOCKED — needs Architect resolution
**Issue:** Protocol agent needs SCustomPayload handler wired in `pumpkin/src/net/java/mod.rs`. Plugin agent needs ServerListPingEvent fired from connection handling. Both require `pumpkin/src/net/` which is FORBIDDEN for Core and not in Protocol agent's write_paths either (Protocol owns `pumpkin-protocol/` the crate, not `pumpkin/src/net/` the binary module).
**Impact:** Custom payload handling and server list ping events remain unimplemented.
**Proposed resolution:** Architect should assign `pumpkin/src/net/` ownership.

### 3. Deferred: Multi-version DTO (Phase 2)
**Status:** DEFERRED — no current action needed
**Impact:** ARCH-016 through ARCH-019 scope the multi-version support architecture. ARCH-018 (Config state bypass for pre-1.20.2 clients) will eventually need Core/Protocol changes to the connection state machine. Currently no action required.

## What I Did

### 1. Command System Completeness Audit

Compared Pumpkin's 50 commands against vanilla's 84 commands:
- **45 vanilla commands implemented** (54% coverage)
- **39 vanilla commands missing**
- **5 Pumpkin-specific commands** (plugin, plugins, pumpkin, banip, pardonip)

**Missing commands by priority:**

**Tier 1 — Essential (Core scope):**
- `execute` — Very complex, requires dispatcher redesign
- `function` / `schedule` / `return` — Datapack/command function system
- `save-all` / `save-off` / `save-on` — Simple server admin commands

**Tier 2 — Cross-agent (needs coordination):**
- `scoreboard` / `tag` / `team` — Entity progression system
- `advancement` — Entity achievement tracking
- `locate` — WorldGen structure finding
- `loot` — Items/WorldGen loot table system
- `attribute` — Entity attribute system
- `datapack` / `reload` — Core/Plugin datapack loading

**Tier 3 — Trivial aliases:**
- `tell` → `msg`, `w` → `msg`, `tp` → `teleport` — Registration aliases only

### 2. Configuration Completeness Audit

Compared Pumpkin config against vanilla server.properties:
- **Most fields present** — Pumpkin has extensive configuration
- **4 high-priority fields missing** — added this session (see below)
- **8+ medium-priority fields missing** — documented for future work

### 3. Added 4 Missing Config Fields (pumpkin-config/src/lib.rs)

| Field | Type | Default | Vanilla Equivalent |
|---|---|---|---|
| `allow_flight` | bool | false | allow-flight |
| `spawn_protection` | u32 | 16 | spawn-protection |
| `generate_structures` | bool | true | generate-structures |
| `player_idle_timeout` | u32 | 0 (disabled) | player-idle-timeout |

All defaults match vanilla. Fields are purely additive — no existing code changed.

**Note:** These fields are declared in config but not yet consumed by runtime code. The actual enforcement (e.g., kicking idle players, checking flight) lives in `pumpkin/src/entity/player/` (Entity scope) and `pumpkin/src/world/` (unowned). These config fields make the data available for those agents to consume.

## What I Changed

### Modified Files
- `pumpkin-config/src/lib.rs` — added 4 config fields to BasicConfiguration struct and Default impl

## Perspectives Consulted
- **Protocol Consultant**: SCustomPayload deserialization is done (Protocol session 002). Handler wiring is in net/ which is outside Core scope.
- **PluginAPI Consultant**: Plugin session 007 harvested Bukkit API reference data. ServerListPingEvent and EntitySpawnEvent are the top two unfired events. Both require write access outside Core scope.
- **WorldGen Consultant**: World sessions 002/003 added 13 structure generators. Mob spawning in structures will need EntitySpawnEvent wiring.
- **Entity Consultant**: Entity session 007 wired damage/death events but is blocked on EntitySpawnEvent (world/ ownership gap).

## What I Need From Others
- **Architect**: Resolve `pumpkin/src/world/` and `pumpkin/src/net/` ownership gaps. See blocking issues above. Without this, EntitySpawnEvent and SCustomPayload handler cannot be wired.
- **Architect**: Consider adding `execute` command to the dispatcher design. This is the most impactful missing command — datapacks and command chains depend on it. Estimated: very high complexity, requires selector context and position transforms in the dispatcher.

## What Others Should Know
- **4 new config fields** are available in `BasicConfiguration`: `allow_flight`, `spawn_protection`, `generate_structures`, `player_idle_timeout`. Consume them from your runtime code as needed.
- **Command aliases** (tell→msg, w→msg, tp→teleport) are trivial to add — just needs `CommandTree::new(["tp", "teleport"], ...)` pattern. I'll add them in a future session.
- **save-all/save-off/save-on** commands are simple Core-scope additions. Deferred to next session to keep this session focused on audits.

## Decisions Made

### CORE-006: Config fields added before runtime enforcement
**Date:** 2026-02-07
**Decision:** Config fields (allow_flight, spawn_protection, generate_structures, player_idle_timeout) are added to BasicConfiguration before the runtime code that consumes them is implemented.
**Rationale:** Config fields with sane defaults are harmless. They appear in the TOML file, allowing server operators to set values before enforcement code exists. Other agents can consume the fields from config without waiting for Core.
**Affects:** Core, Entity (allow_flight, player_idle_timeout), WorldGen (generate_structures, spawn_protection)
**Status:** active

### CORE-007: Command completeness — 39 missing, 7 are Core scope
**Date:** 2026-02-07
**Decision:** Of 39 missing vanilla commands, 7 fall under Core scope: execute, function, schedule, return, save-all, save-off, save-on. The save-* commands are simple (next session). Execute/function/schedule/return require significant dispatcher work (Architect coordination needed).
**Rationale:** Execute command is the most impactful missing command. It's also the most complex — requires positional context, selector modifiers, conditional execution, and subcommand chaining. Best tackled with Architect guidance.
**Affects:** Core, Architect
**Status:** active

## Tests
- `cargo test -p pumpkin` — **121 tests pass**, 0 failures
- `cargo check -p pumpkin-config -p pumpkin` — compiles cleanly, 0 errors

## Open Questions
1. Who owns `pumpkin/src/world/`? This is the #1 cross-agent blocker right now.
2. Who owns `pumpkin/src/net/`? Protocol agent owns the crate but not the binary module.
3. Should `execute` command be designed by Architect before Core implements? Its complexity warrants an architecture doc.
4. Should save-all/save-off/save-on commands call into the existing world.level autosave infrastructure, or use a separate mechanism?
