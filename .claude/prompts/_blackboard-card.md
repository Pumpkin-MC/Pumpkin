<!-- Include this card in every agent prompt -->

## Blackboard Protocol (Upstash Redis A2A Orchestration)

Every session uses the Redis blackboard. You MUST hydrate at start and persist at end.

### Session Lifecycle

```python
from blackboard import Blackboard
bb = Blackboard("pumpkin", agent_id="{YOUR_AGENT_ID}")
state = await bb.hydrate()          # FIRST — always
# ... work ...
await bb.ice_cake(decision)         # MID — FLOW decisions go to Redis immediately
state["summary"] = "What you did"
await bb.persist(state)             # LAST — always
await bb.close()
```

### Session Specialist Roles

Within each session, you can activate specialist sub-roles as the situation demands. These are not separate agents — they are lenses you apply to your own work:

| Role | When to Activate | What It Does |
|---|---|---|
| **Savant** | Deep domain problem requiring specialized knowledge | Focuses on correctness over speed. Reads specs, cross-references vanilla behavior, validates edge cases. Produces authoritative implementation. |
| **Contract Specialist** | Cross-agent boundary question or shared interface change | Reviews all agent contracts, checks write_paths/forbidden boundaries, validates that proposed changes don't violate ownership. Produces boundary-safe implementation. |
| **Scout** | Starting a new feature area with unknown scope | Reads broadly across codebase and specs before writing any code. Maps dependencies, identifies blockers, produces a scoped task list before implementation. |
| **Integrator** | Wiring events, connecting subsystems, firing cross-agent hooks | Focuses on the seams between agents. Reads both sides of an interface, ensures events fire correctly, validates the handoff works end-to-end. |
| **Auditor** | Reviewing previous session's work or validating correctness | Reads all session logs, re-runs tests, checks for regressions. Does NOT write new code — only validates and documents findings. |
| **Upstash Coordinator** | Multi-agent orchestration, handover routing, state synchronization | Manages the Redis blackboard. Posts handovers to the right agents, monitors inbox responses, ice-cakes cross-agent decisions, resolves stale handovers. Activates when a task requires work from 2+ agents. |

Announce which role you're activating in your session log preamble:
```
Activating: Savant (deep Anvil format analysis needed)
```

### A2A Agent Directory (for Handovers)

When you need work done outside your scope, post a handover via the blackboard. Here's who does what:

| Agent | Scope | Handover For |
|---|---|---|
| **architect** | pumpkin-util/, pumpkin-data/, pumpkin-macros/, .claude/, Cargo.toml | Shared traits, cross-agent conflicts, spec ingestion, data codegen, macro changes |
| **protocol** | pumpkin-protocol/, pumpkin/src/net/ | Packet serialization, connection lifecycle, encryption, compression, recipe book packets |
| **worldgen** | pumpkin-world/, pumpkin/src/world/ | Terrain generation, structures, biomes, chunk management, lighting, chunk events |
| **entity** | pumpkin/src/entity/ | Mobs, players, AI goals, pathfinding, combat, entity events, spawning |
| **redstone** | pumpkin/src/block/blocks/redstone/, .../piston/ | Signal propagation, components, quasi-connectivity, piston, hopper, block events |
| **storage** | pumpkin-nbt/ | NBT format, Anvil region files, SNBT parser, player data persistence |
| **items** | pumpkin-inventory/, pumpkin/src/item/, pumpkin/src/data/ | Recipes, loot tables, crafting, enchantments, inventory events |
| **core** | pumpkin/src/server/, pumpkin/src/command/, lib.rs, pumpkin-config/ | Tick loop, commands, server lifecycle, configuration, server events |
| **plugin** | pumpkin/src/plugin/, pumpkin-api-macros/ | Event bus, plugin loading, API surface, Bukkit compatibility, Mindcraft |

### Posting a Handover

```python
hid = await bb.post_handover(
    to_agent="entity",                              # target agent
    task="Fire EntitySpawnEvent in mob spawn path",  # what you need
    context={"related_decisions": ["PLUGIN-001"]},   # relevant context
    expected_output="Event wired into spawn code",   # what success looks like
    constraints=["ARCH-011: don't rename"],           # rules to follow
)
```

### Receiving a Handover

```python
handover = await bb.receive_handover()
if handover:
    # Another agent needs something from you
    print(f"From: {handover['from_agent']}")
    print(f"Task: {handover['request']['task']}")
    # Do the work, then report back
    await bb.post_result(handover, {
        "status": "done",
        "files_changed": ["src/entity/mob.rs"],
        "decisions": ["ENT-006"],
    })
```

### Ice-Caking Decisions

Every FLOW decision goes to Redis immediately (don't wait for persist):

```python
await bb.ice_cake({
    "id": "YOUR-NEXT-ID",
    "task": "What was decided",
    "rationale": "Why this was the right call",
    "gate": "FLOW",
    "gate_sd": 0.05,
})
```

### Quick Commands

- `/blackboard status` — current blackboard state
- `/blackboard decisions` — all ice-caked decisions
- `/blackboard inbox` — check for pending handovers
