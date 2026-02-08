# Storage — Decisions

## STOR-001: Player data helpers are pure NBT converters
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/002_storage_player-data-and-hardening.md
**Decision:** `pumpkin_nbt::player_data` provides only NBT ↔ Rust type conversions (UUID encoding, position/rotation serialization, entity/ability structs). No file I/O, no GZip compression. Consumers (pumpkin-world/) handle persistence.
**Rationale:** Clean separation between serialization format (our scope) and persistence logic (WorldGen/world scope).
**Affects:** Storage, WorldGen
**Status:** active

## STOR-002: Struct-based API for entity/ability NBT helpers
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/002_storage_player-data-and-hardening.md
**Decision:** `EntityBase` and `PlayerAbilities` are structs with `write_to()` and `read_from()` methods, not free functions with tuple returns. Avoids clippy complaints about complex return types and too many arguments.
**Rationale:** Clippy pedantic/nursery lints are enforced project-wide. Structs are more ergonomic and self-documenting than 6-element tuples.
**Affects:** Storage
**Status:** active
