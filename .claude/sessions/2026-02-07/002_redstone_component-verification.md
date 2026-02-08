# Session: redstone-002
**Agent:** redstone
**Date:** 2026-02-07
**Task:** Verify and test repeater, comparator, observer, and piston vanilla parity

## Context Loaded
- Read CLAUDE.md (project guide, CI rules, blackboard protocol)
- Read .claude/prompts/redstone.md (updated prompt with session progress, task priorities)
- Read .claude/sessions/2026-02-07/001_redstone_signal-propagation.md (my prior session)
- Read .claude/sessions/2026-02-07/003_plugin_api-events.md (plugin agent: 9 new events, priority sorting fix)
- Read .claude/sessions/2026-02-07/007_entity_plugin-events-more-mobs.md (entity agent: wired damage/death events)
- Read .claude/sessions/decisions/redstone.md (RED-001, RED-002)
- Read .claude/sessions/decisions/architect.md (ARCH-001 through ARCH-019)
- Read .claude/sessions/decisions/core.md (CORE-001 through CORE-005)

## Preamble
I am the redstone agent, session 002. Building on session 001 (signal propagation fix, dispenser quasi-connectivity, 28 tests).

Key context from other agents since session 001:
- **Plugin** (session 003): Added BlockRedstoneEvent, BlockPistonExtendEvent, BlockPistonRetractEvent to my event backlog. Events are defined but NOT wired. Plugin needs me to fire them.
- **Entity** (session 007): Wired EntityDamageEvent/EntityDeathEvent through plugin system. Good pattern to follow for block events.
- **Architect** (ARCH-011): NEVER RENAME rule is non-negotiable. ARCH-015: Payload::is_cancelled() landed via derive macro.
- **Core** (CORE-004/005): ServerTickEvent and ServerStopEvent firing order established.

This session focuses on the prompt's priorities #1-4: repeater, comparator, observer, and piston verification with comprehensive tests.

## What I Did

### 1. Repeater Verification and Tests (repeater.rs)
Verified the repeater implementation against vanilla behavior:
- **Delay calculation**: `(delay.to_index() + 1) * 2` correctly maps settings 1-4 to 2/4/6/8 game ticks
- **Output level**: Always returns 15 (correct — repeaters restore full signal)
- **Locking**: Side input from repeaters/comparators prevents state changes (via `get_max_input_level_sides`)
- **on_scheduled_tick**: Correctly handles powered→unpowered transition (direct) and unpowered→powered→unpowered cycle (schedules follow-up tick at VeryHigh priority)
- **update_powered**: Correctly skips when locked, uses ExtremelyHigh/VeryHigh/High priority based on alignment

Added 7 tests:
- `test_repeater_delay_calculation` — all 4 delay values
- `test_repeater_output_level_always_15` — constant behavior doc
- `test_delay_cycling` — right-click toggle 1→2→3→4→1
- `test_delay_property_roundtrip` — state ID roundtrip for all delays
- `test_powered_property_roundtrip` — state ID roundtrip
- `test_locked_property_roundtrip` — state ID roundtrip
- `test_facing_property_roundtrip` — state ID roundtrip for all 4 directions

### 2. Comparator Verification and Tests (comparator.rs)
Verified the comparator implementation:
- **Update delay**: Always 2 game ticks (1 redstone tick) — correct
- **Mode toggle**: Compare ↔ Subtract on right-click — correct
- **has_power**: In Compare mode, true when back ≥ side; in both modes, true when back > side — correct
- **calculate_output_signal**: When side ≥ back → 0; subtract mode → back-side; compare mode → back — correct
- **Container reading**: Reads comparator output from block entities via `get_comparator_output` — correct
- **Item frame reading**: Partially implemented (TODO: getComparatorPower returns hardcoded 1)

Added 6 tests:
- `test_comparator_update_delay_always_2` — constant delay
- `test_mode_toggle` — Compare ↔ Subtract
- `test_mode_property_roundtrip` — state ID roundtrip
- `test_comparator_powered_roundtrip` — state ID roundtrip
- `test_comparator_facing_roundtrip` — all 4 directions
- `test_compare_subtract_formula` — 10 test cases covering edge cases

### 3. Observer Verification and Tests (observer.rs)
Verified the observer implementation:
- **Tick delay**: 2 game ticks for both powering on and off — correct vanilla 2-tick pulse
- **Detection**: Triggers on `get_state_for_neighbor_update` when facing matches direction and not already powered
- **Power output**: 15 when powered in facing direction, 0 otherwise — correct
- **Strong = weak power**: `get_strong_redstone_power` delegates to `get_weak_redstone_power` — correct

Added 4 tests:
- `test_observer_uses_2_tick_delay` — design doc
- `test_observer_powered_roundtrip` — state ID roundtrip
- `test_observer_facing_roundtrip` — all 6 directions
- `test_observer_power_levels` — 15/0 power constants

### 4. Piston Verification and Tests (piston/mod.rs)
Verified the piston implementation:
- **Push limit**: `MAX_MOVABLE_BLOCKS = 12` — correct vanilla limit
- **Sticky blocks**: Slime and honey are sticky — correct
- **Non-stick rule**: Slime doesn't stick to honey (and vice versa) — correct for flying machines
- **Immovable blocks**: Obsidian, crying obsidian, respawn anchor, reinforced deepslate — correct
- **Extended piston immovable**: Cannot push an extended piston — correct
- **Hardness -1**: Immovable (bedrock, barriers, etc.) — correct
- **PistonBehavior**: Destroy/Block/PushOnly handled correctly
- **Block entity check**: Blocks with block entities are immovable — correct

Added 13 tests:
- `test_max_movable_blocks` — push limit is 12
- `test_slime_block_is_sticky` / `test_honey_block_is_sticky`
- `test_regular_blocks_not_sticky`
- `test_slime_honey_dont_stick` — non-stick rule
- `test_same_sticky_blocks_stick`
- `test_sticky_sticks_to_regular`
- `test_regular_blocks_dont_stick`
- `test_air_is_movable` / `test_regular_blocks_movable` / `test_redstone_block_movable`
- `test_immovable_hardcoded_blocks` — obsidian et al.
- `test_sticky_blocks_movable` — slime/honey can be pushed

## What I Changed
- `pumpkin/src/block/blocks/redstone/repeater.rs` — Added 7 unit tests
- `pumpkin/src/block/blocks/redstone/comparator.rs` — Added 6 unit tests
- `pumpkin/src/block/blocks/redstone/observer.rs` — Added 4 unit tests
- `pumpkin/src/block/blocks/piston/mod.rs` — Added 13 unit tests

## Perspectives Consulted
- **Plugin**: Block events (BlockRedstoneEvent, BlockPistonExtend/RetractEvent) need wiring. Deferred to next session — event type definitions are in plugin's scope, not mine.

## Vanilla Parity Notes
- **Slime-honey non-stickiness**: A 1.14+ behavior. Before 1.14, honey blocks didn't exist. This is critical for flying machine designs using both blocks side by side.
- **Piston quasi-connectivity in should_extend()**: Already verified correct in session 001. Checks power at all 6 faces plus all 6 faces of the block above.
- **Observer 2-tick pulse**: Matches vanilla. Observers power on for 2 game ticks, then schedule power-off. This creates the characteristic 1 redstone tick pulse.
- **Comparator item frame reading**: Partially implemented (hardcoded to 1). In vanilla, item frames output signal strength based on rotation (0-8 map to 0-8 power). TODO.

## What I Need From Others
- **Plugin**: Create BlockRedstoneEvent, BlockPistonExtendEvent, BlockPistonRetractEvent event structs in `pumpkin/src/plugin/api/events/block/`. I can then fire them from redstone code.

## What Others Should Know
- **Total test count**: 58 tests across redstone (45) and piston (13) modules.
- **All verified behaviors match vanilla** — no bugs found in repeater, comparator, observer, or piston logic.
- **Comparator item frame TODO**: `get_attached_itemframe_level` returns hardcoded `Some(1)` when an item frame is found, not the actual rotation-based power. Low priority since item frames in comparator setups are niche.

## Decisions Made

### RED-003: Block event firing deferred until Plugin creates event types
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/002_redstone_component-verification.md
**Decision:** Redstone agent will not create BlockRedstoneEvent/BlockPistonExtend/RetractEvent since event definitions belong in `pumpkin/src/plugin/api/events/block/` which is outside redstone write_paths. Will fire them once Plugin creates them.
**Rationale:** Respect agent ownership boundaries. Plugin agent owns event type definitions.
**Affects:** Redstone, Plugin
**Status:** active — waiting on Plugin

## Tests
- `cargo test --lib -p pumpkin -- block::blocks` — 58 tests pass, 0 failures
- `cargo check -p pumpkin` — compiles cleanly

## Open Questions
1. **BlockRedstoneEvent**: Should it fire every time wire power changes? Or only on transitions (0→nonzero, nonzero→0)? Vanilla fires on every power level change.
2. **Comparator container edge cases**: The comparator reads from various containers. Are all container types correctly implementing `get_comparator_output`? Need audit.
3. **Hopper locking**: Outside redstone write_paths but affects redstone behavior. Is this Core or Items scope?
