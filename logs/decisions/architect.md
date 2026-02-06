# Architect — Decisions

## ARCH-001: Block module ownership split
**Date:** 2026-02-06
**Session:** logs/2026-02-06/001_architect_gap-analysis.md
**Decision:** Redstone owns `block/blocks/redstone/` and `block/blocks/piston/`. WorldGen owns `block/registry.rs`. Architect resolves `block/mod.rs` and `block/blocks/mod.rs` conflicts. All other block files assigned per-domain in `contracts/block-ownership.toml`.
**Rationale:** The block/ module serves multiple agents. Clean ownership prevents merge conflicts.
**Affects:** Redstone, WorldGen, Architect
**Status:** active

## ARCH-002: Storage vs WorldGen boundary for Anvil files
**Date:** 2026-02-06
**Session:** logs/2026-02-06/001_architect_gap-analysis.md
**Decision:** Storage owns NBT wire format (`pumpkin-nbt/`). WorldGen owns chunk IO that uses NBT (`pumpkin-world/src/level.rs` and related). Storage format changes require WorldGen acknowledgment.
**Rationale:** NBT is serialization (Storage). Chunk persistence is world management (WorldGen). Shared interface, not implementation.
**Affects:** Storage, WorldGen
**Status:** active

## ARCH-003: Data loading ownership
**Date:** 2026-02-06
**Session:** logs/2026-02-06/001_architect_gap-analysis.md
**Decision:** Items owns runtime data loading (`pumpkin/src/data/`). Architect owns compile-time generated data (`pumpkin-data/`). Items must not modify `pumpkin-data/build.rs` or generated output.
**Rationale:** Generated data is build artifact. Runtime loading is gameplay logic.
**Affects:** Items, Architect
**Status:** active

## ARCH-004: lib.rs decomposition authority
**Date:** 2026-02-06
**Session:** logs/2026-02-06/001_architect_gap-analysis.md
**Decision:** Core owns `lib.rs` but any refactor moving code into another agent's module requires that agent to acknowledge before merge. Core must publish decomposition plan as its first session.
**Rationale:** lib.rs touches every subsystem. Uncoordinated decomposition breaks everyone.
**Affects:** All agents
**Status:** active
