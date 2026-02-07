#!/usr/bin/env python3
"""
Initialize the Pumpkin project blackboard in Upstash Redis.

Run once to seed the blackboard with:
- Project state (all 9 agents, current progress)
- All 15 ARCH decisions as ice-caked entries
- Agent registry
- Session log entry

Usage:
    python3 init_blackboard.py
"""

import asyncio
import sys
import os

# Ensure we can import blackboard from same directory
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from blackboard import Blackboard


AGENTS = {
    "architect": {"progress": 0.85, "status": "active", "scope": "pumpkin-data/, pumpkin-util/, .claude/"},
    "protocol": {"progress": 0.75, "status": "active", "scope": "pumpkin-protocol/"},
    "worldgen": {"progress": 0.70, "status": "active", "scope": "pumpkin-world/"},
    "entity": {"progress": 0.30, "status": "active", "scope": "pumpkin/src/entity/"},
    "redstone": {"progress": 0.60, "status": "active", "scope": "pumpkin/src/block/blocks/redstone/"},
    "storage": {"progress": 0.80, "status": "active", "scope": "pumpkin-nbt/"},
    "items": {"progress": 0.50, "status": "active", "scope": "pumpkin-inventory/, pumpkin/src/item/"},
    "core": {"progress": 0.65, "status": "active", "scope": "pumpkin/src/server/, pumpkin/src/command/"},
    "plugin": {"progress": 0.25, "status": "active", "scope": "pumpkin/src/plugin/, pumpkin-api-macros/"},
}

DECISIONS = [
    {
        "id": "ARCH-001",
        "task": "Block module ownership split",
        "rationale": "The block/ module serves multiple agents. Clean ownership prevents merge conflicts.",
        "affects": ["redstone", "worldgen", "architect"],
        "gate": "FLOW",
        "gate_sd": 0.05,
    },
    {
        "id": "ARCH-002",
        "task": "Storage vs WorldGen boundary for Anvil files",
        "rationale": "NBT is serialization (Storage). Chunk persistence is world management (WorldGen).",
        "affects": ["storage", "worldgen"],
        "gate": "FLOW",
        "gate_sd": 0.04,
    },
    {
        "id": "ARCH-003",
        "task": "Data loading ownership — Items owns runtime, Architect owns compile-time pumpkin-data",
        "rationale": "Generated data is build artifact. Runtime loading is gameplay logic.",
        "affects": ["items", "architect"],
        "gate": "FLOW",
        "gate_sd": 0.06,
    },
    {
        "id": "ARCH-004",
        "task": "lib.rs decomposition authority — Core owns, confirmed 607 lines not 23K",
        "rationale": "lib.rs touches every subsystem. Uncoordinated decomposition breaks everyone. CORE-001 confirms not needed.",
        "affects": ["core", "all"],
        "gate": "FLOW",
        "gate_sd": 0.03,
    },
    {
        "id": "ARCH-005",
        "task": "Session logs live in .claude/sessions/ (TRACKED, not gitignored)",
        "rationale": "Gitignoring sessions broke cross-session visibility. Logs must be committed.",
        "affects": ["all"],
        "gate": "FLOW",
        "gate_sd": 0.07,
    },
    {
        "id": "ARCH-006",
        "task": "All orchestration lives under .claude/",
        "rationale": "Fork source tree should be indistinguishable from upstream plus code changes.",
        "affects": ["all"],
        "gate": "FLOW",
        "gate_sd": 0.02,
    },
    {
        "id": "ARCH-007",
        "task": "All .claude/ is tracked — nothing gitignored",
        "rationale": "Gitignoring sessions broke the read-before-write protocol entirely.",
        "affects": ["all"],
        "gate": "FLOW",
        "gate_sd": 0.04,
    },
    {
        "id": "ARCH-008",
        "task": "Navigator::is_idle() fix ownership — Entity authorized",
        "rationale": "Entity goal system depends on Navigator cycling correctly.",
        "affects": ["entity"],
        "gate": "FLOW",
        "gate_sd": 0.06,
    },
    {
        "id": "ARCH-009",
        "task": "Anvil deduplication — Storage provides, WorldGen consumes",
        "rationale": "Storage has clean 420-line Anvil with 17 tests. No duplication in pumpkin-world/.",
        "affects": ["storage", "worldgen"],
        "gate": "FLOW",
        "gate_sd": 0.05,
    },
    {
        "id": "ARCH-010",
        "task": "Enderman teleportation is Entity scope",
        "rationale": "Teleportation is mob AI. Block validity uses existing world query interfaces.",
        "affects": ["entity"],
        "gate": "FLOW",
        "gate_sd": 0.03,
    },
    {
        "id": "ARCH-011",
        "task": "NEVER RENAME existing Pumpkin code — NON-NEGOTIABLE",
        "rationale": "Public fork for upstream PRs. Renaming breaks contributors and creates merge conflicts.",
        "affects": ["all"],
        "gate": "FLOW",
        "gate_sd": 0.01,
    },
    {
        "id": "ARCH-012",
        "task": "Vanilla Data Import — MC 1.21.4 data from misode/mcmeta",
        "rationale": "Data-driven agents need canonical vanilla JSON. 1370 recipes, 1237 loot tables.",
        "affects": ["items", "entity", "worldgen", "storage", "redstone"],
        "gate": "FLOW",
        "gate_sd": 0.04,
    },
    {
        "id": "ARCH-013",
        "task": "PrismarineJS + Bukkit API Reference Data added to specs",
        "rationale": "Agents need behavioral data (hitboxes, food values) and Bukkit event mapping.",
        "affects": ["entity", "items", "core", "plugin"],
        "gate": "FLOW",
        "gate_sd": 0.05,
    },
    {
        "id": "ARCH-014",
        "task": "Stonecutting/smithing recipes generated in pumpkin-data build.rs",
        "rationale": "Compile-time data from MC JSON dumps. Unblocks Items agent ITEMS-003.",
        "affects": ["items", "architect"],
        "gate": "FLOW",
        "gate_sd": 0.06,
    },
    {
        "id": "ARCH-015",
        "task": "Payload::is_cancelled() via Event derive field detection — Bukkit isCancelled() convention",
        "rationale": "Enables Bukkit-compatible ignore_cancelled filtering without trait object downcasting.",
        "affects": ["plugin", "all"],
        "gate": "FLOW",
        "gate_sd": 0.05,
    },
]

AGENT_DECISIONS = {
    "protocol": [
        {"id": "PROTO-001", "task": "VarInt overflow detection", "rationale": "Prevent malformed packets from crashing server", "gate": "FLOW", "gate_sd": 0.04},
        {"id": "PROTO-002", "task": "BitSet serialization for chunk data", "rationale": "Required for light data in chunk packets", "gate": "FLOW", "gate_sd": 0.05},
    ],
    "worldgen": [
        {"id": "WORLD-001", "task": "Acknowledge ARCH-009 Anvil adoption from Storage", "rationale": "Storage's RegionFile is canonical. Migration not yet scheduled.", "gate": "FLOW", "gate_sd": 0.08},
    ],
    "entity": [
        {"id": "ENT-001", "task": "Goal system uses priority queue", "rationale": "Vanilla behavior: higher priority goals preempt lower ones", "gate": "FLOW", "gate_sd": 0.05},
        {"id": "ENT-002", "task": "Mob struct wraps MobEntity", "rationale": "Consistent pattern: struct Zombie { mob: MobEntity }", "gate": "FLOW", "gate_sd": 0.03},
        {"id": "ENT-003", "task": "Navigator::is_idle() fix", "rationale": "Return correct state based on path active status", "gate": "FLOW", "gate_sd": 0.04},
    ],
    "redstone": [
        {"id": "RED-001", "task": "Vanilla update order for redstone", "rationale": "Match vanilla tick ordering for deterministic behavior", "gate": "FLOW", "gate_sd": 0.06},
        {"id": "RED-002", "task": "Dispenser quasi-connectivity", "rationale": "Vanilla mechanic: dispensers powered by block above", "gate": "FLOW", "gate_sd": 0.07},
    ],
    "items": [
        {"id": "ITEMS-001", "task": "Stonecutter slot layout matches vanilla", "rationale": "Slot 0 input, slot 1 output", "gate": "FLOW", "gate_sd": 0.03},
        {"id": "ITEMS-002", "task": "Smithing slot layout matches vanilla", "rationale": "Slots 0-2 template/base/addition, slot 3 output", "gate": "FLOW", "gate_sd": 0.03},
        {"id": "ITEMS-003", "task": "Priority: stonecutting first, smithing second, special crafting third", "rationale": "Based on data availability and dependency chain", "gate": "FLOW", "gate_sd": 0.05},
    ],
    "core": [
        {"id": "CORE-001", "task": "lib.rs not decomposed — 607 lines, well-structured", "rationale": "Gap analysis was wrong (claimed 23K). No action needed.", "gate": "FLOW", "gate_sd": 0.02},
        {"id": "CORE-002", "task": "server/mod.rs decomposition deferred until >1200 lines", "rationale": "Currently ~940 lines with 5 extracted submodules. Healthy.", "gate": "FLOW", "gate_sd": 0.04},
        {"id": "CORE-003", "task": "Tick profiler uses lock-free AtomicU64/AtomicBool", "rationale": "No Mutex in hot path. Lock-free atomics for per-tick timing.", "gate": "FLOW", "gate_sd": 0.03},
    ],
    "plugin": [
        {"id": "PLUGIN-001", "task": "Entity events use primitive entity_id: i32", "rationale": "Don't expose Arc<LivingEntity> to plugins", "gate": "FLOW", "gate_sd": 0.05},
        {"id": "PLUGIN-002", "task": "Monitor priority is Bukkit-compatible observe-only", "rationale": "Handlers MUST NOT modify event state", "gate": "FLOW", "gate_sd": 0.04},
        {"id": "PLUGIN-003", "task": "Server lifecycle events NOT cancellable", "rationale": "ServerStarted, ServerStop, ServerTick are fire-and-forget", "gate": "FLOW", "gate_sd": 0.02},
        {"id": "PLUGIN-004", "task": "ignore_cancelled filtering — UNBLOCKED by ARCH-015", "rationale": "Payload::is_cancelled() now available for fire() filtering", "gate": "FLOW", "gate_sd": 0.06},
    ],
    "storage": [
        {"id": "STOR-001", "task": "Anvil RegionFile is canonical implementation", "rationale": "420 lines, 17 tests. WorldGen will adopt per ARCH-009.", "gate": "FLOW", "gate_sd": 0.03},
    ],
}


async def main():
    bb = Blackboard("pumpkin", agent_id="architect")

    print("=== Pumpkin Blackboard Initialization ===\n")

    # 1. Hydrate (create fresh or resume)
    state = await bb.hydrate()
    print(f"Session: {bb.session_id}")
    print(f"Status: {state['status']}")
    print(f"Previous sessions: {len(state.get('previous_sessions', []))}\n")

    # 2. Set project state
    state["summary"] = "Blackboard initialization — seeded all 15 ARCH decisions, 9 agents, and cross-agent decisions"
    state["current_task"] = {
        "description": "Initialize Redis blackboard with full project state",
        "phase": "seeding",
        "progress": 1.0,
    }
    state["team"] = {
        "active": list(AGENTS.keys()),
        "parked": [],
    }
    state["project_stats"] = {
        "total_tests": 479,
        "total_commits": 112,
        "crates": 13,
        "lines_of_code": "~151K",
        "bukkit_events_catalogued": 283,
        "bukkit_events_implemented": 28,
    }
    state["agent_progress"] = {
        name: info["progress"] for name, info in AGENTS.items()
    }

    # 3. Ice-cake all ARCH decisions
    print("Ice-caking ARCH decisions...")
    for d in DECISIONS:
        await bb.ice_cake(d)
        print(f"  {d['id']}: {d['task'][:60]}...")

    # 4. Ice-cake all agent-specific decisions
    print("\nIce-caking agent decisions...")
    for agent_name, decisions in AGENT_DECISIONS.items():
        for d in decisions:
            d["agent_source"] = agent_name
            await bb.ice_cake(d)
            print(f"  {d['id']}: {d['task'][:60]}...")

    # 5. Register all agents
    print("\nRegistering agents...")
    cmds = []
    for name, info in AGENTS.items():
        cmds.append(["HSET", "ada:bb:pumpkin:agents", name, info["status"]])
    await bb.redis.pipeline(cmds)
    for name in AGENTS:
        print(f"  {name}: registered")

    # 6. Store agent progress on hot cache
    print("\nCaching agent details to hot Redis...")
    for name, info in AGENTS.items():
        await bb.redis_hot.set_json(f"ada:agent:{name}:status", {
            "agent": name,
            "progress": info["progress"],
            "scope": info["scope"],
            "status": info["status"],
        }, ex=86400)  # 24h TTL

    # 7. Persist final state
    await bb.persist(state)
    print(f"\nPersisted to Redis.")

    # 8. Verify
    count = await bb.get_ice_cake_count()
    decisions = await bb.get_recent_decisions(5)
    log = await bb.get_session_log(3)

    print(f"\n=== Verification ===")
    print(f"Total ice-caked decisions: {count}")
    print(f"Last 5 decisions:")
    for d in decisions:
        print(f"  - {d.get('id', '?')}: {d.get('task', '?')[:50]}")
    print(f"Session log entries: {len(log)}")

    await bb.close()
    print("\n=== Blackboard initialization complete! ===")


if __name__ == "__main__":
    asyncio.run(main())
