# Session 006 — ARCH-032 Scope Expansion Event Wiring

**Agent:** redstone
**Date:** 2026-02-07
**Branch:** `claude/redstone-signal-propagation-QKEoc`

## Preamble

Read all session logs (001-011 across all agents). Read decisions log (RED-001 through RED-005). Read ARCH-032 scope expansion assigning fire/, plant/, snow.rs, fluid/, note.rs, sponge.rs to Redstone agent. Read Plugin-009 audit identifying remaining unfired events.

## What Changed

### 1. BlockBurnEvent wired in `fire/fire.rs::try_spreading_fire` (line ~120)
- Fires BEFORE the flammable block at `pos` is consumed (replaced with fire or air)
- `igniting_block` = `&Block::FIRE`, `block` = the block being burned
- If cancelled by a plugin, `return` skips the entire burn (no fire placement, no TNT prime)
- `world.get_block(pos).await` returns `&'static Block` — no lifetime workaround needed

### 2. BlockGrowEvent wired in `plant/crop/mod.rs::CropBlockBase::random_tick` (line ~58)
- Fires BEFORE `set_block_state` advances the crop age
- `block` = current crop, `new_block` = same block type (crops don't change block type on grow)
- If cancelled, `return` prevents the age advancement
- `world.get_block_and_state_id(pos).await` returns `(&'static Block, u16)` — no lifetime workaround needed

### 3. BlockFadeEvent wired in `snow.rs::LayeredSnowBlock::on_scheduled_tick` (line ~93)
- Fires BEFORE `break_block` removes the snow layer
- `block` = `&Block::SNOW` (static reference, avoids `args.block` lifetime issue)
- `new_block` = `&Block::AIR` (snow fades to air)
- If cancelled, `return` prevents the break

### 4. BlockFromToEvent — NOT wired (no fire point)
- Liquid flow (water/lava spreading) is not implemented in Pumpkin
- Dragon egg teleportation is not implemented
- No `fluid/` directory exists
- Event struct is ready; will be wired when flow logic is added

### Registry updated
- `bukkit_api.toml`: BlockGrowEvent, BlockFadeEvent → `status = "implemented"` (34 total, 7 remaining redstone)
- BlockBurnEvent was already marked as implemented

## Decisions Made

- **RED-006:** BlockBurnEvent fires in `try_spreading_fire` only, not in the triple-nested spread loop (which only targets AIR positions, not flammable blocks).
- **RED-007:** BlockFromToEvent deferred — no liquid flow implementation exists to wire into.

## Tests

72 tests pass (59 redstone + 13 piston). Build clean. No warnings in pumpkin crate.

## What Others Should Know

- **Plugin:** BlockBurnEvent, BlockGrowEvent, BlockFadeEvent are now all fired. Plugins can cancel fire spread, crop growth, and snow decay.
- **Core/WorldGen:** BlockFromToEvent still needs a liquid flow system before it can be wired.
- **All agents owning crop-like blocks:** The `CropBlockBase::random_tick` default method now fires BlockGrowEvent. Any crop that overrides `random_tick` should fire the event independently.

## Perspectives Consulted
- **Plugin:** Event struct definitions (read block_burn.rs, block_grow.rs, block_fade.rs, block_from_to.rs)

## Vanilla Parity Notes
- BlockBurnEvent fires per-block, matching vanilla's per-neighbor fire spread check
- BlockGrowEvent fires before state change, matching Bukkit's cancellation semantics
- BlockFadeEvent for snow is triggered by structural invalidity (support block removed), not light level — light-based melting is not yet implemented in Pumpkin
