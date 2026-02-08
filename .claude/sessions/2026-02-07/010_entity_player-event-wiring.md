# Session 010 — Entity: Player Event Wiring (PLUGIN-009 Blocker)

**Agent:** entity
**Date:** 2026-02-07
**Branch:** claude/entity-spawning-ai-V7oqj

## Preamble

Read session 009 log (AI goal expansion — 6 goals, melee+tempt wired).
Read architect session 010 (status update, P0 priorities).
Read plugin session 008 (PLUGIN-009 audit: 25/39 events fired at 64%).
Received cross-agent coordination request: 4 entity-owned player events need wiring.

## What I Did

### 1. Wired PlayerDeathEvent (player.rs `handle_killed`)
- Fires before inventory drop and death packet
- Looks up `Arc<Player>` from `world.players` by entity_id
- Passes game rule `keep_inventory` as event default
- If cancelled: restores health to 1.0 and returns (prevents death)
- Plugins can modify `keep_inventory` and `death_message`

### 2. Wired PlayerDropItemEvent (player.rs `drop_held_item`)
- Fires before `drop_item()` call
- Looks up `Arc<Player>` from `world.players` by entity_id
- If cancelled: returns without dropping (item stays in inventory)

### 3. Wired PlayerItemConsumeEvent (living.rs food consumption tick)
- Fires when `item_use_time` reaches 0 (before eating)
- Looks up `Arc<Player>` from `world.players` by entity_id
- If cancelled: skips hunger/saturation application and item decrement
- Always clears active hand (stops eating animation) regardless

### 4. Wired PlayerRespawnEvent (world/mod.rs `respawn_player`) — ARCH-023
- Cross-agent write: single fire() call, no logic changes
- Fires after respawn position computed, before respawn packet sent
- Plugins can modify `respawn_position`
- Uses `mut position` destructuring to allow event modification

## Files Modified (3)

### Entity-owned:
- `pumpkin/src/entity/player.rs` — PlayerDeathEvent + PlayerDropItemEvent imports and fire()
- `pumpkin/src/entity/living.rs` — PlayerItemConsumeEvent import and fire()

### Cross-agent (ARCH-023):
- `pumpkin/src/world/mod.rs` — PlayerRespawnEvent import and fire()

## Pattern Used: `Arc<Player>` Lookup

All 3 entity-file events use the same pattern to get `Arc<Player>` from `&self`:
```rust
world.players.load().iter()
    .find(|p| p.entity_id() == self.entity_id())
    .cloned()
```
This is necessary because `handle_killed` and `drop_held_item` take `&self` (not `self: &Arc<Self>`), and plugin events require `Arc<Player>`.

## Event Coverage After This Session

- **Before:** 25/39 events fired (64%)
- **After:** 29/39 events fired (74%) — 4 new player events
- **Remaining:** 10 events needing other agents (5 block/world, 5 redstone)

## Decisions Made
- ENT-012: Use world player list lookup for Arc<Player> in &self contexts rather than changing method signatures (avoids trait signature changes in EntityBase)

## What Others Should Know
- **Plugin agent:** All 4 entity-owned player events from PLUGIN-009 are now wired and firing
- **Core/WorldGen agent:** 5 events still need wiring (BlockPlace, BlockCanBuild, ChunkLoad/Save/Send)
- **Redstone/Block agent:** 5 events still need wiring (BlockBurn, BlockPhysics, BlockFromTo, BlockGrow, BlockFade)

## Build Status
- `cargo build` — clean, no warnings
