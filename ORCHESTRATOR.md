# Minecraft Rust Server — Agent-to-Agent Orchestrator

## PROJECT IDENTITY

**Repository:** `AdaWorldAPI/Pumpkin`
**Origin:** Fork of [Pumpkin-MC/Pumpkin](https://github.com/Pumpkin-MC/Pumpkin)
**Goal:** Extend and harden Pumpkin into a fully featured Minecraft Java Edition server, built by coordinated AI agent sessions.
**Protocol Spec:** [wiki.vg](https://wiki.vg/Protocol) (source of truth for all packet/format work)
**Current Version:** Minecraft 1.21.11 (protocol already implemented)

---

## THE ONE LAW

> **You must read before you write. Always. No exceptions.**

Before appending your session log, you read every log written today and the last 5 from yesterday.
Before touching code, you read every DECISION that touches your scope.
Before modifying a shared crate, you read who depends on it.

Violation of read-before-write is a build-breaking event. Your session is invalid.

---

## PUMPKIN CRATE → AGENT MAPPING

Unlike a greenfield project, this is a living codebase. Agents own **crates and modules**, not empty folders.

### Crate Structure (as forked)

```
Pumpkin/
├── pumpkin/                  # Main server binary (the "god crate")
│   └── src/
│       ├── block/            # Block behaviors, redstone (138 files)
│       ├── command/          # Command system (89 files)
│       ├── data/             # Vanilla data loading (7 files)
│       ├── entity/           # Entities, AI, combat (54 files)
│       ├── item/             # Item behaviors (28 files)
│       ├── net/              # Network handling (21 files)
│       ├── plugin/           # Plugin system (29 files)
│       ├── server/           # Server core, ticker (6 files)
│       ├── world/            # World management (15 files)
│       ├── lib.rs            # Server bootstrap (23K lines)
│       └── main.rs           # Entry point
├── pumpkin-protocol/         # Packet definitions, serialization (244 files)
├── pumpkin-world/            # Terrain gen, biomes, chunks (213 files)
├── pumpkin-inventory/        # Inventory, crafting (22 files)
├── pumpkin-nbt/              # NBT format (6 files)
├── pumpkin-data/             # Generated block/item/entity data (6 files)
├── pumpkin-util/             # Shared utilities, math, text (38 files)
├── pumpkin-config/           # Server configuration (20 files)
├── pumpkin-macros/           # Proc macros (1 file)
└── pumpkin-api-macros/       # Plugin API macros (1 file)
```

### Agent Roster

| Agent | Owns (write scope) | Responsibility |
|---|---|---|
| **Architect** | `pumpkin-util/`, `pumpkin-data/`, `pumpkin-macros/`, `Cargo.toml`, `contracts/`, `specs/`, orchestration files | Shared types, trait design, spec ingestion, gap analysis, conflict resolution, cross-agent integration |
| **Protocol** | `pumpkin-protocol/`, `pumpkin/src/net/` | Packet definitions, serialization, connection state machine, compression, encryption |
| **WorldGen** | `pumpkin-world/`, `pumpkin/src/world/` | Terrain generation, biomes, structures, chunk management, lighting |
| **Entity** | `pumpkin/src/entity/` | Mobs, players, physics, pathfinding, AI, combat, spawning |
| **Redstone** | `pumpkin/src/block/blocks/redstone/`, `pumpkin/src/block/blocks/piston/` | Redstone signal propagation, components, quasi-connectivity |
| **Storage** | `pumpkin-nbt/` | NBT parsing/writing, Anvil region files, world save/load |
| **Items** | `pumpkin-inventory/`, `pumpkin/src/item/`, `pumpkin/src/data/` | Recipes, loot tables, inventory, crafting, enchantments, registries |
| **Core** | `pumpkin/src/server/`, `pumpkin/src/command/`, `pumpkin/src/main.rs`, `pumpkin/src/lib.rs`, `pumpkin-config/` | Tick loop, scheduler, config, bootstrap, command dispatch |
| **PluginAPI** | `pumpkin/src/plugin/`, `pumpkin-api-macros/` | Plugin interface, event system, mod API |

### Shared Zone: `pumpkin/src/block/`

The `pumpkin/src/block/` module is a **shared zone** with special rules:
- **Redstone agent** owns `block/blocks/redstone/` and `block/blocks/piston/`
- **WorldGen agent** owns `block/registry.rs` and block state management
- **Architect** resolves any conflict in the shared `block/mod.rs` and `block/blocks/mod.rs` files
- All other block files (doors, beds, crops, etc.) are claimed by the agent whose domain they serve, documented per-file in `contracts/block-ownership.toml`

### Agent Communication

Agents **do not** modify each other's crates. All cross-crate interfaces flow through:
1. **`pumpkin-util/`** — shared types, math, text components (Architect-owned)
2. **`pumpkin-data/`** — generated block/item/entity data (Architect-owned)
3. **Rust traits** — defined in the crate that implements them, consumed via Cargo dependencies

If an agent needs a new shared type in `pumpkin-util/` or `pumpkin-data/`, they request it in their session log under "What I Need From Others → Architect."

---

## FOLDER STRUCTURE (orchestration overlay)

These directories are **added** to the Pumpkin fork:

```
Pumpkin/
├── ORCHESTRATOR.md              # THIS FILE — the constitution
├── .claude/
│   └── rules/
│       └── session-protocol.md  # Auto-loaded into every Claude Code session
├── contracts/
│   ├── architect.toml
│   ├── protocol.toml
│   ├── world.toml
│   ├── entity.toml
│   ├── redstone.toml
│   ├── storage.toml
│   ├── items.toml
│   ├── core.toml
│   ├── plugin.toml
│   └── block-ownership.toml     # Per-file ownership of shared block/ module
├── specs/
│   ├── protocol/                # wiki.vg data, machine-readable
│   ├── data/                    # MC data dumps (blocks, items, recipes)
│   ├── world/                   # World gen specs, biome data
│   └── entity/                  # Entity specs, mob behaviors
├── logs/
│   ├── YYYY-MM-DD/
│   │   ├── 001_agent_description.md
│   │   └── ...
│   └── decisions/
│       ├── architect.md
│       ├── protocol.md
│       ├── world.md
│       ├── entity.md
│       ├── redstone.md
│       ├── storage.md
│       ├── items.md
│       ├── core.md
│       └── plugin.md
├── reference/                   # External reference material
├── .githooks/
│   └── pre-commit
└── start-session.sh
```

---

## CONTRACT FORMAT

Each agent has a `contracts/{agent}.toml`:

```toml
[identity]
name = "protocol"
responsibility = "All Minecraft protocol handling: packets, serialization, connection lifecycle"

[boundaries]
write_paths = ["pumpkin-protocol/", "pumpkin/src/net/", "tests/network/"]
read_paths = [
    "pumpkin-protocol/",
    "pumpkin/src/net/",
    "pumpkin-util/",
    "pumpkin-data/",
    "specs/protocol/",
    "logs/",
    "contracts/",
]
forbidden_paths = ["pumpkin/src/entity/", "pumpkin/src/world/", "pumpkin/src/block/blocks/redstone/"]

[interfaces]
# Traits I implement (in my crates)
implements = ["PacketHandler", "ConnectionManager"]

# Traits I consume from others (via Cargo deps)
consumes = ["ChunkProvider", "EntityTracker", "BlockStateRegistry"]

[tests]
must_pass = "cargo test -p pumpkin-protocol"
```

---

## SESSION LIFECYCLE

### Every session follows this exact sequence:

```
┌─────────────────────────────────────────────────┐
│  1. IDENTIFY                                     │
│     - Which agent am I?                          │
│     - What is my task?                           │
│     - What is my write scope?                    │
│                                                   │
│  2. READ (mandatory, no exceptions)              │
│     - logs/{today}/*.md (ALL of them)            │
│     - logs/{yesterday}/ (last 5 entries)         │
│     - logs/decisions/{my-agent}.md               │
│     - logs/decisions/architect.md                │
│     - Any log that mentions my agent by name     │
│     - contracts/{my-agent}.toml                  │
│                                                   │
│  3. WRITE PREAMBLE                               │
│     - Prove you read by summarizing what you saw │
│     - Acknowledge pending requests aimed at you  │
│     - State your plan for this session           │
│                                                   │
│  4. WORK                                         │
│     - Stay inside your write_paths               │
│     - Existing Pumpkin code is your foundation   │
│     - wiki.vg spec is truth, Pumpkin is guidance │
│     - Run your tests before finishing            │
│                                                   │
│  5. LOG (mandatory, no exceptions)               │
│     - Write logs/{today}/{seq}_{agent}_{desc}.md │
│     - Update logs/decisions/{agent}.md if needed │
│     - Follow the session log format exactly      │
│                                                   │
│  6. COMMIT                                       │
│     - Pre-commit hook validates write boundaries │
│     - Tests must pass                            │
│     - Log must exist                             │
└─────────────────────────────────────────────────┘
```

---

## SESSION LOG FORMAT

File: `logs/{YYYY-MM-DD}/{SEQ}_{agent}_{short-description}.md`

Example: `logs/2026-02-06/003_protocol_chunk-packet.md`

```markdown
# Session: {agent}-{SEQ}
**Agent:** {agent}
**Date:** {YYYY-MM-DD HH:MM UTC}
**Task:** {one-line description}

## Context Loaded
- Read logs/{today}/ entries {range}
- Read logs/{yesterday}/ entries {range}
- Read logs/decisions/{agent}.md ({N} decisions)
- Read logs/decisions/architect.md ({N} decisions)
- {Specific acknowledgments of requests/changes that affect this session}

## What I Did
- {Bullet points of concrete actions taken}

## What I Learned
- {Discoveries, gotchas, things that weren't obvious from the spec}
- {Behavior differences from vanilla Pumpkin or vanilla MC}

## What I Changed
- `path/to/file.rs` — {what changed}
- ⚠️ SHARED TYPE CHANGE: `pumpkin-util/src/X.rs` — {what changed, why}

## What I Need From Others
- **{agent}**: {specific request with file/line references}

## What Others Should Know
- {Conventions established that affect shared interfaces}

## Decisions Made
- {Decision}: {rationale} → logged in decisions/{agent}.md as {ID}

## Tests
- `tests/{agent}/{test_name}.rs` — {what it tests}
- Result: {PASS/FAIL with details}

## Open Questions
- {Questions requiring Architect ruling}
```

---

## DECISIONS LOG FORMAT

File: `logs/decisions/{agent}.md`

```markdown
# {Agent} — Decisions

## {AGENT}-001: {title}
**Date:** {YYYY-MM-DD}
**Session:** {log file reference}
**Decision:** {what was decided}
**Rationale:** {why}
**Affects:** {which agents/interfaces}
**Status:** active | superseded by {ID}
```

Decisions are **append-only**. Never delete. If overriding, add new with `supersedes: {ID}`.

---

## SHARED INTERFACE CHANGE PROTOCOL

When any agent needs a change to `pumpkin-util/` or `pumpkin-data/`:

1. Agent documents the need in session log under "What I Need From Others → Architect"
2. Agent proposes the change (show the type/trait signature they need)
3. **Only the Architect agent may modify shared crates**
4. Architect session that modifies shared crates must:
   - Flag with ⚠️ in the session log
   - List every agent that consumes the changed interface
   - Explain migration path if breaking
5. All affected agents must acknowledge the change in their next session preamble

---

## CONFLICT RESOLUTION

When agents disagree:

1. Both agents document their position in their session logs
2. Architect reads both, makes a ruling
3. Ruling is logged in `logs/decisions/architect.md`
4. Both agents acknowledge in their next preamble
5. No agent may relitigate without new evidence

---

## BOOTSTRAP SEQUENCE

### Phase 0: Scaffold ✓ (complete)
- Fork Pumpkin
- Write ORCHESTRATOR.md (this file)
- Write all contracts
- Set up pre-commit hooks and session scripts

### Phase 1: Audit & Map (current)
- **Architect 001**: Map Pumpkin crate structure → agent boundaries, produce gap analysis
- **Architect 002**: Identify incomplete/missing features per agent
- **Architect 003**: Design block-ownership.toml for the shared block/ module
- **Architect 004**: Establish initial shared trait boundaries in pumpkin-util/

### Phase 2: Hardening (agents work in parallel)
- **Protocol**: Complete missing packet types, add Bedrock parity
- **WorldGen**: Complete structure generation, cave carving, biome blending
- **Entity**: Complete mob AI, pathfinding improvements, combat mechanics
- **Storage**: Harden NBT, add missing Anvil features, player data
- **Items**: Complete crafting recipes, enchantments, loot tables
- **Core**: Optimize tick loop, improve command system, config hardening
- **Redstone**: Complete signal propagation, quasi-connectivity, all components

### Phase 3: Integration
- **Architect**: Cross-agent integration tests
- **Core**: Performance profiling, optimization
- **PluginAPI**: Event system, plugin loading, Mindcraft compatibility

### Phase 4: Differentiation
- **Custom features beyond vanilla Pumpkin**
- **Performance optimizations**
- **Plugin ecosystem**

---

## QUALITY GATES

Before any phase transition:

1. `cargo test --all` passes
2. `cargo clippy --all -- -D warnings` passes
3. Server boots and accepts connections (Phase 2+)
4. Vanilla client can connect and see the world (Phase 3+)
5. No unresolved items in any agent's "What I Need From Others" older than 3 sessions
6. All shared type changes acknowledged by consuming agents

---

## NOTES FOR HUMAN OPERATOR

- You assign sessions. Agents don't self-schedule.
- You can run multiple agents in parallel if their scopes don't overlap.
- If two agents need the same shared crate change, run Architect between them.
- The Architect agent is the most important — run it whenever there's cross-cutting work.
- Logs are your project management tool. Read them. They're written for you.
- When in doubt, run an Architect session to assess state and resolve blockers.
- **This is a fork, not greenfield.** Agents extend existing code, they don't rewrite from scratch unless specifically tasked.
