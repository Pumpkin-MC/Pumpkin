# Session 005 — BlockPhysicsEvent Wiring + Event Audit

**Agent:** redstone
**Date:** 2026-02-07
**Branch:** `claude/redstone-signal-propagation-QKEoc`

## Preamble

Read all session logs and decisions. Read Plugin-009 audit identifying 5 "Redstone/Block" events needing fire() calls. Assessed write path coverage for each event.

## Audit: 5 Block Events from Plugin-009

| Event | Fire Location | In Redstone scope? | Action |
|-------|---------------|---------------------|--------|
| **BlockPhysicsEvent** | `world/mod.rs::update_neighbors` | Yes (ARCH-023) | **Wired this session** |
| BlockBurnEvent | `block/blocks/fire/fire.rs` | No | Needs fire block owner |
| BlockFromToEvent | Liquid flow (not implemented) | No | Needs world/fluid owner |
| BlockGrowEvent | `block/blocks/plant/crop/mod.rs` | No | Needs plant block owner |
| BlockFadeEvent | `block/blocks/snow.rs` | No | Needs snow/ice block owner |

Only BlockPhysicsEvent was actionable from the Redstone agent. The other 4 events need agents who own fire.rs, crop/mod.rs, snow.rs, and liquid flow code.

## What Changed

### BlockPhysicsEvent wired in `world/mod.rs::update_neighbors` (line ~3282)
- Fires BEFORE each neighbor's `on_neighbor_update` dispatch
- If cancelled by a plugin, the `continue` skips that specific neighbor update
- Has full context: `neighbor_block` (target), `neighbor_pos` (target pos), `source_block` (source type), `block_pos` (source pos)
- Uses `server.plugin_manager.fire(event).await` pattern matching other event wiring

### Not wired: `update_neighbor` (single dispatch)
- This function receives `source_block: &Block` but NOT `source_position: BlockPos`
- Cannot construct `BlockPhysicsEvent` without source position
- Would require signature change across all callers — outside scope

### Registry updated
- `bukkit_api.toml`: BlockPhysicsEvent → `status = "implemented"` (32 total, 9 remaining redstone)

## Decisions Made

- **RED-005:** BlockPhysicsEvent wired in `update_neighbors` only, not `update_neighbor`.

## Tests

72 tests pass (59 redstone + 13 piston). Build clean. No warnings.

## What Others Should Know

- **Plugin:** BlockPhysicsEvent is now fired. Plugins can cancel individual neighbor updates during multi-direction propagation.
- **Core/All:** The `update_neighbor` single-dispatch does NOT fire BlockPhysicsEvent. If we want full coverage, `update_neighbor` needs a `source_position: BlockPos` parameter added. This is a future enhancement.
- **All agents owning block files:** The remaining 4 events (BlockBurnEvent, BlockFromToEvent, BlockGrowEvent, BlockFadeEvent) need fire() calls added in their respective files. Event structs are ready in `plugin/api/events/block/`.

## Perspectives Consulted
- **Plugin:** Event struct definition for BlockPhysicsEvent (read block_physics.rs)

## Vanilla Parity Notes
- Bukkit's BlockPhysicsEvent does NOT have source_position — Pumpkin adds it as an enhancement
- Cancellation skips individual neighbor updates within the 6-direction loop, matching vanilla tick behavior
