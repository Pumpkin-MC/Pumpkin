# World — Decisions

## WORLD-001: Acknowledge ARCH-009 — Anvil deduplication
**Date:** 2026-02-07
**In response to:** ARCH-009
**Decision:** WorldGen acknowledges Storage's `pumpkin_nbt::anvil::RegionFile` as the canonical Anvil implementation. WorldGen will adopt it for region file I/O in `pumpkin-world/src/chunk/format/anvil.rs` and `pumpkin-world/src/chunk/io/` when the migration is scheduled. WorldGen retains chunk-level convenience wrappers but will not fork or reimplement the low-level Anvil format logic.
**Constraint:** Migration must NOT rename or delete the existing `chunk/format/anvil.rs` module — it will be refactored to delegate to `pumpkin_nbt::anvil::RegionFile` internally while preserving its public API.
**Status:** acknowledged — migration not yet scheduled
