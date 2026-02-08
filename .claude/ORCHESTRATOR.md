# Minecraft Rust Server — Agent-to-Agent Orchestrator

## PROJECT IDENTITY

**Repository:** `AdaWorldAPI/Pumpkin`
**Goal:** Extend the Pumpkin Minecraft server (Rust, 1.21.11) to full vanilla parity using coordinated AI agent sessions.
**Upstream Reference:** [Pumpkin-MC/Pumpkin](https://github.com/Pumpkin-MC/Pumpkin) (fork as starting point)
**Protocol Spec:** [wiki.vg](https://wiki.vg/Protocol) (source of truth for all packet/format work)
**Target Version:** Minecraft 1.21.4

---

## THE ONE LAW

> **You must read before you write. Always. No exceptions.**

Before appending your session log, you read every log written today and the last 5 from yesterday.
Before touching code, you read every DECISION that touches your scope.
Before modifying a shared trait, you read who depends on it.

Violation of read-before-write is a build-breaking event. Your session is invalid.

---

## AGENTS

Each agent owns exactly one folder in their assigned crates. An agent may only write to its own folder and to `.claude/sessions/`. The sole exception is the **Architect**, who owns `pumpkin-util/` and can modify any CONTRACT.

### Agent Roster

| Agent | Owns | Responsibility |
|---|---|---|
| **Architect** | `pumpkin-util/`, `pumpkin-data/`, `pumpkin-macros/`, `.claude/contracts/`, `.claude/specs/` | Shared types, trait design, spec ingestion, gap analysis, conflict resolution |
| **Protocol** | `pumpkin-protocol/`, `pumpkin/src/net/` | All Minecraft protocol: packets, serialization, connection state |
| **WorldGen** | `pumpkin-world/`, `pumpkin/src/world/` | Terrain generation, biomes, structures, chunk management |
| **Entity** | `pumpkin/src/entity/` | Mobs, players, physics, pathfinding, collision, combat |
| **Redstone** | `pumpkin/src/block/blocks/redstone/`, `pumpkin/src/block/blocks/piston/` | Redstone logic, signal propagation, components |
| **Storage** | `pumpkin-nbt/` | NBT format, Anvil files, region I/O |
| **Items** | `pumpkin-inventory/`, `pumpkin/src/item/`, `pumpkin/src/data/` | Recipes, loot tables, inventory, crafting, enchantments |
| **Core** | `pumpkin/src/server/`, `pumpkin/src/command/`, `pumpkin/src/main.rs`, `pumpkin/src/lib.rs`, `pumpkin-config/` | Tick loop, scheduler, configuration, server bootstrap |
| **PluginAPI** | `pumpkin/src/plugin/`, `pumpkin-api-macros/` | Plugin interface, event system, mod API |

### Agent Communication

Agents **never** import from each other's folders directly. All inter-agent communication flows through `pumpkin-util/`:

```
pumpkin-util/
├── types/         # Shared data types (BlockPos, ChunkPos, ItemStack, etc.)
├── traits/        # Interface contracts between agents
│   ├── chunk_provider.rs      # WorldGen exposes, Protocol consumes
│   ├── entity_tracker.rs      # Entity exposes, Protocol consumes
│   ├── block_state.rs         # WorldGen exposes, Redstone consumes
│   ├── storage_backend.rs     # Storage exposes, WorldGen/Core consume
│   ├── recipe_registry.rs     # Items exposes, Core consumes
│   └── event_bus.rs           # Core exposes, PluginAPI consumes
├── constants/     # Magic numbers, protocol IDs, block IDs
└── errors/        # Shared error types
```

Only the **Architect** may add, modify, or remove files in `pumpkin-util/`. If an agent needs a new shared type or trait, it requests one in its session log under "What I Need From Others."

---

## FOLDER STRUCTURE

```
Pumpkin/                          # Fork of Pumpkin-MC/Pumpkin
├── ORCHESTRATOR.md               # THIS FILE — the constitution
├── .claude/
│   ├── rules/
│   │   └── session-protocol.md   # Loaded into every Claude Code session
│   ├── contracts/
│   │   ├── architect.toml
│   │   ├── block-ownership.toml  # Per-file ownership for shared block/ module
│   │   └── {agent}.toml
│   ├── specs/
│   │   ├── protocol/             # wiki.vg data, machine-readable
│   │   ├── data/                 # MC data dumps (blocks, items, recipes)
│   │   ├── world/                # World gen specs, biome data
│   │   └── entity/               # Entity specs, mob behaviors
│   └── sessions/
│       ├── YYYY-MM-DD/
│       │   ├── 001_agent_description.md
│       │   └── ...
│       └── decisions/
│           ├── architect.md
│           └── {agent}.md
├── pumpkin/                      # Main binary crate
│   └── src/
│       ├── lib.rs                # Core agent (23K lines — decomposition needed)
│       ├── main.rs               # Core agent: bootstrap
│       ├── server/               # Core agent: tick loop, scheduler
│       ├── command/              # Core agent: commands
│       ├── net/                  # Protocol agent: connection handling
│       ├── world/                # WorldGen agent: runtime world management
│       ├── entity/               # Entity agent: mobs, players, AI
│       ├── block/blocks/
│       │   ├── redstone/         # Redstone agent
│       │   ├── piston/           # Redstone agent
│       │   └── ...               # Per-file ownership, see block-ownership.toml
│       ├── item/                 # Items agent: item behaviors
│       ├── data/                 # Items agent: runtime data loading
│       └── plugin/               # PluginAPI agent
├── pumpkin-protocol/             # Protocol agent: packets, serialization
├── pumpkin-world/                # WorldGen agent: terrain, biomes, chunks
├── pumpkin-inventory/            # Items agent: inventory management
├── pumpkin-nbt/                  # Storage agent: NBT format
├── pumpkin-util/                 # Architect-owned: shared types, traits
├── pumpkin-data/                 # Architect-owned: generated data (build.rs)
├── pumpkin-macros/               # Architect-owned: proc macros
├── pumpkin-config/               # Core agent: configuration
├── pumpkin-api-macros/           # PluginAPI agent: API macros
├── start-session.sh
└── .githooks/pre-commit
```

---

## CONTRACT FORMAT

Each agent has a `.claude/contracts/{agent}.toml`:

```toml
[identity]
name = "protocol"
responsibility = "All Minecraft protocol handling: packets, serialization, connection lifecycle"

[boundaries]
write_paths = ["pumpkin-protocol/", "pumpkin/src/net/"]
read_paths = [
    "pumpkin-protocol/",
    "pumpkin/src/net/",
    "pumpkin-util/",
    ".claude/specs/protocol/",
    ".claude/sessions/",
    ".claude/contracts/",
]
forbidden_paths = ["pumpkin-world/", "pumpkin/src/entity/", "pumpkin/src/block/blocks/redstone/", "pumpkin-nbt/", "pumpkin-inventory/", "pumpkin/src/server/", "pumpkin/src/plugin/"]

[interfaces]
# Traits I implement (defined in pumpkin-util/)
implements = ["PacketHandler", "ConnectionManager"]

# Traits I consume from others
consumes = ["ChunkProvider", "EntityTracker", "StorageBackend"]

[specs]
primary = [".claude/specs/protocol/"]

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
│     - .claude/sessions/{today}/*.md (ALL of them)            │
│     - .claude/sessions/{yesterday}/ (last 5 entries)         │
│     - .claude/sessions/decisions/{my-agent}.md               │
│     - .claude/sessions/decisions/architect.md                │
│     - Any log that mentions my agent by name     │
│     - .claude/contracts/{my-agent}.toml                  │
│                                                   │
│  3. WRITE PREAMBLE                               │
│     - Prove you read by summarizing what you saw │
│     - Acknowledge pending requests aimed at you  │
│     - State your plan for this session           │
│                                                   │
│  4. WORK                                         │
│     - Stay inside your write_paths               │
│     - Use .claude/reference/pumpkin/ for inspiration     │
│     - Run your tests before finishing            │
│                                                   │
│  5. LOG (mandatory, no exceptions)               │
│     - Write .claude/sessions/{today}/{seq}_{agent}_{desc}.md │
│     - Update .claude/sessions/decisions/{agent}.md if needed │
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

File: `.claude/sessions/{YYYY-MM-DD}/{SEQ}_{agent}_{short-description}.md`

Example: `.claude/sessions/2026-02-06/003_protocol_chunk-packet.md`

```markdown
# Session: {agent}-{SEQ}
**Agent:** {agent}
**Date:** {YYYY-MM-DD HH:MM UTC}
**Task:** {one-line description}

## Context Loaded
- Read .claude/sessions/{today}/ entries {range}
- Read .claude/sessions/{yesterday}/ entries {range}
- Read .claude/sessions/decisions/{agent}.md ({N} decisions)
- Read .claude/sessions/decisions/architect.md ({N} decisions)
- {Specific acknowledgments of requests/changes that affect this session}

## What I Did
- {Bullet points of concrete actions taken}

## What I Learned
- {Discoveries, gotchas, things that weren't obvious from the spec}
- {Behavior differences from Pumpkin or vanilla}

## What I Changed
- `path/to/file.rs` — {what changed}
- `path/to/file.rs` — {what changed}
- ⚠️ SHARED TYPE CHANGE: `pumpkin-util/types/X.rs` — {what changed, why}

## What I Need From Others
- **{agent}**: {specific request with file/line references}
- **{agent}**: {specific request}

## What Others Should Know
- {Conventions established that affect shared interfaces}
- {Gotchas that other agents will hit if they don't know}

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

File: `.claude/sessions/decisions/{agent}.md`

```markdown
# {Agent} — Decisions

## {AGENT}-001: {title}
**Date:** {YYYY-MM-DD}
**Session:** {log file reference}
**Decision:** {what was decided}
**Rationale:** {why}
**Affects:** {which agents/interfaces}
**Status:** active | superseded by {ID}

## {AGENT}-002: {title}
...
```

Decisions are **append-only**. Never delete a decision. If overriding, add a new one with `supersedes: {ID}` and mark the old one `superseded by {ID}`.

---

## SHARED INTERFACE CHANGE PROTOCOL

When any agent needs a change to `pumpkin-util/`:

1. Agent documents the need in session log under "What I Need From Others → Architect"
2. Agent may propose the change (show the trait/type signature they need)
3. **Only the Architect agent may modify `pumpkin-util/`**
4. Architect session that modifies `pumpkin-util/` must:
   - Flag with ⚠️ in the session log
   - List every agent that consumes the changed interface
   - Explain migration path if breaking
5. All affected agents must acknowledge the change in their next session preamble

---

## CONFLICT RESOLUTION

When agents disagree (e.g., Protocol wants data shaped one way, WorldGen another):

1. Both agents document their position in their session logs
2. Architect reads both, makes a ruling
3. Ruling is logged in `.claude/sessions/decisions/architect.md`
4. Both agents acknowledge in their next preamble
5. No agent may relitigate a decided issue without new evidence

---

## SPEC ORGANIZATION

```
.claude/specs/
├── protocol/
│   ├── handshake.json     # From wiki.vg, machine-readable
│   ├── status.json
│   ├── login.json
│   ├── play.json           # The big one
│   └── README.md           # How these were generated
├── data/
│   ├── blocks.json         # All block states, from MC data generator
│   ├── items.json
│   ├── recipes/            # All recipe JSONs from MC data
│   ├── loot_tables/
│   ├── tags/               # Block/item/entity tags
│   └── README.md
├── world/
│   ├── biomes.json
│   ├── noise.md            # World gen algorithm docs
│   ├── structures.md
│   └── README.md
└── entity/
    ├── entities.json       # All entity types + metadata
    ├── mob_ai.md
    ├── pathfinding.md
    └── README.md
```

---

## PRE-COMMIT ENFORCEMENT

```bash
#!/bin/bash
# .githooks/pre-commit

set -e

# 1. Determine which agent is committing
AGENT=$(cat .current-agent 2>/dev/null)
if [ -z "$AGENT" ]; then
    echo "ERROR: No .current-agent file. Set your agent before committing."
    exit 1
fi

# 2. Load contract
CONTRACT=".claude/contracts/${AGENT}.toml"
if [ ! -f "$CONTRACT" ]; then
    echo "ERROR: No contract for agent '$AGENT'"
    exit 1
fi

# 3. Check changed files against write boundaries
CHANGED=$(git diff --cached --name-only)
WRITE_PATHS=$(grep -A 10 '\[boundaries\]' "$CONTRACT" | grep 'write_paths' | tr -d '[]" ' | cut -d= -f2 | tr ',' '\n')

# .claude/sessions/ is always writable
WRITE_PATHS="$WRITE_PATHS
.claude/sessions/"

for file in $CHANGED; do
    ALLOWED=false
    for wp in $WRITE_PATHS; do
        if echo "$file" | grep -q "^${wp}"; then
            ALLOWED=true
            break
        fi
    done
    if [ "$ALLOWED" = false ]; then
        echo "BOUNDARY VIOLATION: Agent '$AGENT' cannot write to '$file'"
        echo "Allowed paths: $WRITE_PATHS"
        exit 1
    fi
done

# 4. Verify session log exists for today
TODAY=$(date +%Y-%m-%d)
if ! ls .claude/sessions/"$TODAY"/*_${AGENT}_* 1>/dev/null 2>&1; then
    echo "ERROR: No session log for agent '$AGENT' on $TODAY"
    echo "Write your session log before committing."
    exit 1
fi

# 5. Run agent's tests
TEST_CMD=$(grep 'must_pass' "$CONTRACT" | cut -d'"' -f2)
if [ -n "$TEST_CMD" ]; then
    echo "Running: $TEST_CMD"
    eval "$TEST_CMD" || {
        echo "ERROR: Tests failed for agent '$AGENT'"
        exit 1
    }
fi

echo "✓ Agent '$AGENT' commit validated"
```

---

## SESSION START SCRIPT

```bash
#!/bin/bash
# start-session.sh <agent> <task-description>

AGENT=$1
TASK=$2
TODAY=$(date +%Y-%m-%d)

if [ -z "$AGENT" ] || [ -z "$TASK" ]; then
    echo "Usage: ./start-session.sh <agent> <task-description>"
    echo "Agents: architect protocol world entity redstone storage items core plugin"
    exit 1
fi

# Set current agent marker
echo "$AGENT" > .current-agent

# Determine next sequence number
mkdir -p ".claude/sessions/$TODAY"
LAST_SEQ=$(ls ".claude/sessions/$TODAY/" 2>/dev/null | grep -oP '^\d+' | sort -n | tail -1)
NEXT_SEQ=$(printf "%03d" $(( ${LAST_SEQ:-0} + 10#1 )))

# Build context file list for Claude Code
CONTEXT_FILES=(
    "ORCHESTRATOR.md"
    ".claude/contracts/${AGENT}.toml"
    ".claude/sessions/decisions/${AGENT}.md"
    ".claude/sessions/decisions/architect.md"
)

# Add today's logs
for f in .claude/sessions/"$TODAY"/*.md; do
    [ -f "$f" ] && CONTEXT_FILES+=("$f")
done

# Add yesterday's last 5 logs
YESTERDAY=$(date -d "yesterday" +%Y-%m-%d 2>/dev/null || date -v-1d +%Y-%m-%d)
if [ -d ".claude/sessions/$YESTERDAY" ]; then
    for f in $(ls ".claude/sessions/$YESTERDAY/"*.md | tail -5); do
        CONTEXT_FILES+=("$f")
    done
fi

echo "═══════════════════════════════════════════"
echo "  Agent:    $AGENT"
echo "  Task:     $TASK"
echo "  Log:      .claude/sessions/$TODAY/${NEXT_SEQ}_${AGENT}_*.md"
echo "  Context:  ${#CONTEXT_FILES[@]} files loaded"
echo "═══════════════════════════════════════════"

# Export for Claude Code
export MC_AGENT="$AGENT"
export MC_TASK="$TASK"
export MC_LOG_SEQ="$NEXT_SEQ"
export MC_TODAY="$TODAY"
```

---

## .claude/rules/session-protocol.md

This file is loaded into every Claude Code session automatically:

```markdown
# SESSION PROTOCOL — MANDATORY

You are an agent in the Pumpkin (fork) project. You have ONE job, ONE folder, 
and ONE set of rules. Follow them exactly.

## Your Identity

Read `.current-agent` to know which agent you are.
Read `.claude/contracts/{your-agent}.toml` for your boundaries.
Read `ORCHESTRATOR.md` if you need the full constitution.

## Before Writing Any Code

You MUST read:
1. Every file in `.claude/sessions/{today}/`
2. The last 5 files in `.claude/sessions/{yesterday}/`
3. `.claude/sessions/decisions/{your-agent}.md`
4. `.claude/sessions/decisions/architect.md`
5. Any log file from any day that mentions your agent name in its title or content

Then write your session preamble in your log file FIRST, proving you read these.
Only then may you begin coding.

## While Working

- Write ONLY to paths listed in your contract's write_paths
- Write ONLY to `.claude/sessions/` (always allowed)
- NEVER modify `pumpkin-util/` unless you are the Architect
- NEVER modify another agent's folder
- USE `.claude/reference/pumpkin/` for inspiration but do not copy blindly
- RUN your tests before finishing: check `must_pass` in your contract

## Before Finishing

1. Write your session log: `.claude/sessions/{today}/{seq}_{agent}_{description}.md`
2. Follow the log format in ORCHESTRATOR.md exactly
3. Update `.claude/sessions/decisions/{agent}.md` if you made any decisions
4. Ensure all tests pass
5. Commit with message: `[{agent}] {description}`

## Shared Interface Requests

If you need a new type or trait in `pumpkin-util/`:
- Do NOT create it yourself (unless you are Architect)
- Document what you need in "What I Need From Others → Architect"
- Propose the signature you want
- Continue your work using a TODO placeholder if needed

## Non-Negotiable Rules

1. READ BEFORE WRITE — always, no exceptions
2. STAY IN YOUR LANE — your folder, your tests, your logs
3. LOG EVERYTHING — no session without a log entry
4. TEST YOUR WORK — broken tests = invalid session
5. DECISIONS ARE PERMANENT — never relitigate without new evidence
```

---

## BOOTSTRAP SEQUENCE

The project comes to life in this order:

### Phase 0: Scaffold (you are here)
- Create repository structure
- Write all contracts
- Write ORCHESTRATOR.md
- Set up pre-commit hooks and session scripts

### Phase 1: Audit
- **Architect session 001**: Clone Pumpkin, explore structure, produce gap analysis
- **Architect session 002**: Ingest wiki.vg specs into `.claude/specs/protocol/`
- **Architect session 003**: Ingest MC data dumps into `.claude/specs/data/`
- **Architect session 004**: Design initial `pumpkin-util/` traits based on gap analysis

### Phase 2: Foundation
- **Core session 001**: Tick loop, server bootstrap, config loading
- **Storage session 001**: NBT parser, Anvil region file reader
- **Protocol session 001**: Handshake + Login sequence
- **Items session 001**: Load block/item registries from .claude/specs/data

### Phase 3: Playable
- **Protocol session 002-005**: Play state packets (chunk, entity, player)
- **WorldGen session 001-003**: Flat world → noise terrain → biomes
- **Entity session 001-003**: Player entity, basic mob spawning, collision
- **Storage session 002**: World save/load

### Phase 4: Complete
- **Redstone session 001-005**: Signal propagation, components, circuits
- **Entity session 004-008**: Full mob AI, pathfinding, combat
- **WorldGen session 004-006**: Structures, caves, decoration
- **Items session 002-004**: Crafting, enchanting, loot tables
- **PluginAPI session 001-003**: Event system, plugin loading, API surface

### Phase 5: Integration
- **Architect**: Cross-agent integration tests
- **Core**: Performance profiling, optimization
- **Protocol**: Multi-version support (if desired)
- **PluginAPI**: Mindcraft compatibility layer

---

## QUALITY GATES

Before any phase transition:

1. All agent tests pass (`cargo test --all`)
2. Server boots and accepts connections (Phase 2+)
3. Vanilla client can connect and see the world (Phase 3+)
4. No unresolved items in any agent's "What I Need From Others" older than 3 sessions
5. All shared trait changes acknowledged by consuming agents

---

## NOTES FOR HUMAN OPERATOR

- You assign sessions. Agents don't self-schedule.
- You can run multiple agents in parallel if their scopes don't overlap.
- If two agents need the same shared trait change, run Architect between them.
- The Architect agent is the most important — run it whenever there's cross-cutting work.
- Logs are your project management tool. Read them. They're written for you.
- When in doubt, run an Architect session to assess state and resolve blockers.
