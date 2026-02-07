# Session 004 — Redstone Comparator/Observer Completion Audit

**Agent:** redstone
**Date:** 2026-02-07
**Branch:** `claude/redstone-signal-propagation-QKEoc`

## Preamble

Read all session logs for today (001-009 across all agents). Read updated prompt (P1 priority: comparator and observer completion). Read ARCH-031 benchmark target. All decisions reviewed.

## Audit Results

### Comparator — Functionally Complete
The comparator implementation in `comparator.rs` correctly handles:
- Compare mode: outputs back power when back > side, outputs back when back == side
- Subtract mode: outputs (back - side) when back > side, 0 when side >= back
- Container signal reading via `get_comparator_output` trait (composter, command block, chiseled bookshelf implement it; chests/barrels/furnaces are Items agent scope)
- Item frame detection (partial — TODO returns `Some(1)` instead of actual rotation value, Entity scope)
- Block entity stores output signal level (not in block state — correct vanilla behavior)
- Scheduled tick updates with correct priority (High when target not aligned, Normal otherwise)
- 2-tick update delay (1 redstone tick) — always constant

### Observer — Functionally Complete
The observer implementation in `observer.rs` correctly handles:
- Block state change detection via `get_state_for_neighbor_update` (triggers when facing direction matches)
- 2-tick pulse: first scheduled tick powers on + schedules second tick; second tick powers off
- Power output only from back face (opposite of observed face), level 15
- Strong == weak power (delegates, correct — observers strongly power blocks)
- Cooldown during pulse (won't re-trigger while powered)
- Handles state replacement when powered (on_state_replaced notifies neighbors)

### Hopper Locking — Already Implemented
`hopper.rs` correctly locks when powered: `enabled = !block_receives_redstone_power()`.

## What Changed

### 1. Comparator tests expanded (comparator.rs)
- **test_compare_subtract_formula_exhaustive**: All 256 (16×16) back×side power combinations for both modes
- **test_has_power_logic**: Named edge cases for the `has_power` boolean (back=0, back>side, back==side per mode, side>back)
- **test_has_power_exhaustive**: All 256×2 combinations verifying correct powered/unpowered state
- **test_comparator_full_state_roundtrip**: All facing × mode × powered combinations (4×2×2 = 16 states)

### 2. Observer tests expanded (observer.rs)
- **test_emits_power_direction_specificity**: All 36 facing × query direction combinations
- **test_weak_power_truth_table**: All 72 facing × direction × powered combinations
- **test_detection_trigger_condition**: Full truth table for observation trigger (facing match + not powered)
- **test_strong_equals_weak_power**: Confirms strong delegates to weak for all 6 facings
- **test_observer_full_state_roundtrip**: All 12 facing × powered states
- **test_pulse_state_machine**: Simulates the on_scheduled_tick 2-step cycle without async

### 3. Signal propagation tests (mod.rs)
- **test_solid_block_power_propagation_rule**: Documents the solid block pass-through rule
- **test_wire_excluded_from_non_dust_queries**: Verifies the dust_power guard preventing wire loops
- **test_block_receives_power_checks_all_six**: Confirms all 6 directions checked
- **test_diode_exhaustive_redstone_blocks**: Tests 23+ blocks confirming only repeater/comparator are diodes

## Tests

72 tests pass (59 redstone + 13 piston). No warnings. Build clean.

## Implementation Gaps (for other agents)

| Gap | Owner | Notes |
|-----|-------|-------|
| Container comparator output (chests, barrels, furnaces, hoppers) | Items | Implement `get_comparator_output` on container blocks |
| Item frame `getComparatorPower()` | Entity | Currently hardcoded to `Some(1)` in comparator.rs:361 |
| Jukebox, lectern, beehive, cauldron, end portal frame comparator output | Items/World | Missing `get_comparator_output` implementations |

## Decisions Made

No new decisions. Comparator and observer verified as functionally complete per vanilla semantics.

## Perspectives Consulted
- **Items:** Container blocks need `get_comparator_output` implementations
- **Entity:** Item frame comparator power needs actual rotation-based calculation

## Vanilla Parity Notes
- Comparator `has_power` correctly implements the asymmetry: Compare mode considers equal signals as "powered" (back==side → true), Subtract does not (back==side → false)
- Observer pulse is exactly 2 game ticks (1 redstone tick), matching vanilla
- Observer won't re-trigger during cooldown (while powered), matching vanilla
