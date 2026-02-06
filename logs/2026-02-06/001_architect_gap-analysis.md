# Session: architect-001
**Agent:** architect
**Date:** 2026-02-06 21:15 UTC
**Task:** Map Pumpkin fork crate structure to agent boundaries, produce gap analysis

## Context Loaded
- No prior logs exist — this is the first session
- Read logs/decisions/architect.md (0 decisions)
- Read all 9 contracts
- Read ORCHESTRATOR.md

## What I Did

### 1. Mapped Pumpkin Crate Structure

The fork (AdaWorldAPI/Pumpkin) is a **working** Minecraft 1.21.11 server: 992 Rust source files, ~151K lines. Not a stub.

**Crate dependency graph:**
```
pumpkin-nbt (leaf)
  └─ pumpkin-util
      ├─ pumpkin-config
      └─ pumpkin-data
          ├─ pumpkin-macros
          ├─ pumpkin-world
          └─ pumpkin-protocol
              └─ pumpkin-inventory
                  └─ pumpkin (binary — depends on all)
```

### 2. Agent Boundary Rulings

See decisions ARCH-001 through ARCH-004 below.

### 3. Gap Analysis Per Agent

#### PROTOCOL — ~75% complete
**Exists:** Full Java handshake/login/play for 1.21.11. Bedrock framing started. VarInt, compression, encryption.
**Gaps:**
- Bedrock play-state packets (only framing exists)
- Per-packet audit needed against wiki.vg
- Multi-version negotiation
- Query protocol completeness

**First task:** Per-packet completeness audit against wiki.vg 1.21.4 play state

#### WORLDGEN — ~70% complete
**Exists:** Noise terrain, multi-noise biomes, 4 structures (buried treasure, nether fortress, stronghold, swamp hut), 60+ decoration features, cave carving, surface builders.
**Gaps:**
- ~16+ structure types missing (villages, ocean monuments, mansions, bastions, etc.)
- Biome blending may be incomplete
- Lighting engine unknown status
- Chunk load/unload optimization

**First task:** Structure generation audit — Pumpkin has 4 of ~20+ structures

#### ENTITY — ~30% complete
**Exists:** Player entity, 6 hostile mobs (creeper, drowned, silverfish, skeleton, zombie, zombie villager), 3 passive (iron/snow golem, wolf), 2 bosses, AI goal selector, basic pathfinding, combat, 5 projectiles.
**Gaps:**
- ~70 mob types not implemented (MC 1.21 has 79+)
- No villager trading/breeding
- No rideable entities (horse, boat, minecart AI)
- No fish/ocean mobs
- No warden, elder guardian (complex)
- Spawning rules per biome missing
- Entity metadata sync unknown completeness

**First task:** Spawning/despawning rules framework, then common overworld mobs

#### REDSTONE — ~60% complete
**Exists:** 6284 lines, 20+ files. Repeater, comparator, copper bulb, dropper, lever, observer, pressure plates, rails, redstone block/lamp/torch/wire, target block, tripwire, buttons, pistons. Turbo mode.
**Gaps:**
- Quasi-connectivity status unknown
- Signal propagation correctness vs vanilla update order
- Hopper-redstone interaction
- Dispenser/dropper full integration
- Sculk sensor
- Technical player relied-upon "bugs"

**First task:** Vanilla-parity signal propagation test suite

#### STORAGE — ~80% complete
**Exists:** Full NBT parser/writer (2611 lines), compound tags, ser/deser, compression.
**Gaps:**
- SNBT parsing
- Long array tag verification
- Large NBT performance
- Player data save/load completeness

**First task:** Verify all NBT tag types, add missing

#### ITEMS — ~50% complete
**Exists:** Inventory management (22 files), crafting system, 28 item behaviors, furnace processing, drag handling, equipment.
**Gaps:**
- Recipe coverage unknown completeness
- Enchantment system
- Loot tables
- Smithing/brewing
- Durability/repair
- Bundle, decorated pot, newer items

**First task:** Recipe coverage audit against MC 1.21

#### CORE — ~65% complete
**Exists:** Working 20 TPS tick loop (with sprint), 50+ commands, config system, RCON, LAN broadcast, query.
**Gaps:**
- lib.rs is 23K lines (god object, needs decomposition)
- Tick loop doesn't explicitly phase subsystems
- Some commands may be partial
- Performance profiling infrastructure
- Graceful shutdown completeness

**First task:** lib.rs decomposition plan

#### PLUGINAPI — ~25% complete
**Exists:** Basic loader, API context, event skeleton (block/player/server/world categories), API macros.
**Gaps:**
- Event priority/cancellation
- Plugin sandboxing
- API coverage
- Hot-reload
- Mindcraft compatibility (not started)

**First task:** Phase 3-4 — blocked on Core and Entity stability

## What I Learned

1. **Pumpkin is far more complete than expected.** Working server with serious terrain gen, protocol, and gameplay.
2. **The god object problem is real.** `pumpkin/src/lib.rs` at 23K lines is the biggest risk. Core agent must decompose it before other agents can work cleanly.
3. **Entity is the biggest gap.** Only ~10 of 79+ mobs. This is the most labor-intensive subsystem.
4. **Redstone is surprisingly advanced** with a turbo mode, but vanilla parity is unverified.
5. **The block/ module is a boundary hazard.** 138 files serving at least 3 agents (Redstone, WorldGen, Core). The block-ownership.toml is critical.
6. **Pumpkin already has Bedrock support started** — this is ahead of the ORCHESTRATOR's original scope.

## What I Changed
- Created `ORCHESTRATOR.md` (adapted from greenfield to fork-aware)
- Created all 9 agent contracts in `contracts/`
- Created `.claude/rules/session-protocol.md`
- Created decision log stubs for all agents
- Copied `start-session.sh` and `.githooks/pre-commit`

## What I Need From Others
- **All agents**: Acknowledge this gap analysis in your first session preamble
- **Core**: lib.rs decomposition is the highest priority blocker — plan the breakup before other agents run into conflicts

## What Others Should Know
- **Existing Pumpkin code is your foundation.** Do not rewrite from scratch. Extend.
- **The block/ module has per-file ownership.** Check contracts/block-ownership.toml before touching any block file.
- **pumpkin-data/ is generated code.** The build script (`build.rs`) generates Rust from JSON data. Don't hand-edit generated files.
- **Cargo workspace test commands** use `-p {crate-name}` not folder names. e.g. `cargo test -p pumpkin-protocol`

## Decisions Made

### ARCH-001: Block module ownership split
**Decision:** Redstone owns `block/blocks/redstone/` and `block/blocks/piston/`. WorldGen owns `block/registry.rs`. Architect resolves `block/mod.rs` conflicts. All other block files assigned per-domain in `contracts/block-ownership.toml`.
**Rationale:** The block/ module serves multiple agents. Clean ownership prevents merge conflicts.
**Affects:** Redstone, WorldGen, Architect

### ARCH-002: Storage vs WorldGen boundary for Anvil files
**Decision:** Storage owns NBT wire format (`pumpkin-nbt/`). WorldGen owns chunk IO that uses NBT (`pumpkin-world/src/level.rs` and related). If Storage changes NBT format, WorldGen must acknowledge.
**Rationale:** NBT is a serialization format (Storage domain). Chunk persistence is world management (WorldGen domain). They share an interface, not an implementation.
**Affects:** Storage, WorldGen

### ARCH-003: Data loading ownership
**Decision:** Items owns runtime data loading (`pumpkin/src/data/`). Architect owns compile-time generated data (`pumpkin-data/`). Items must not modify `pumpkin-data/build.rs` or generated output.
**Rationale:** Generated data is a build artifact. Runtime loading is gameplay logic.
**Affects:** Items, Architect

### ARCH-004: lib.rs decomposition authority
**Decision:** Core owns `lib.rs` but any refactor that moves code into another agent's module requires that agent to acknowledge before merge. Core must publish a decomposition plan as its first session.
**Rationale:** lib.rs touches every subsystem. Uncoordinated decomposition will break everyone.
**Affects:** All agents

## Tests
- No code changes — structural analysis only
- `cargo check --workspace` status: not verified (Rust 1.89 required, may not be available in this environment)

## Open Questions
1. **Do we track upstream Pumpkin?** Should we periodically merge from Pumpkin-MC/Pumpkin, or fully diverge? This affects how aggressively we restructure.
2. **Bedrock scope:** Pumpkin has started Bedrock support. Do we invest in it or deprioritize?
3. **Target MC version:** Pumpkin tracks 1.21.11. Do we pin to that or plan for 1.22 when it drops?
