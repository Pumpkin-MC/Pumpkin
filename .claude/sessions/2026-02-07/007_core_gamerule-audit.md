# Session: core-007
**Agent:** core
**Date:** 2026-02-07
**Task:** Game rule audit, log_admin_commands enforcement, debug/perf commands

## Context Loaded
- Rebased onto origin/master (e2fa5e8)
- Read updated .claude/prompts/core.md — P1 priority: command audit completion
- Read .claude/sessions/decisions/core.md (CORE-001 through CORE-010)
- Linter applied formatting to debug.rs, perf.rs, dispatcher.rs, core.md — all accepted

## What I Did

### 1. Game Rule Completeness Audit

Full audit of Pumpkin's game rules vs vanilla Minecraft 1.21.4:
- **59 rules defined** in Pumpkin (via assets/game_rules.json → pumpkin-data codegen)
- **13 rules enforced** at runtime (22%)
- **46 rules defined but NOT enforced** (78%)
- **~11 vanilla rules missing** or named differently

**Enforced rules:** advance_time, advance_weather, command_blocks_work, send_command_feedback, players_nether_portal_creative_delay, players_nether_portal_default_delay, lava_source_conversion, water_source_conversion, max_block_modifications, mob_griefing, drowning_damage, show_death_messages, keep_inventory

**Critical unenforced rules (other agents' scope):**
- pvp, fall_damage, fire_damage, freeze_damage (Entity)
- random_tick_speed, spawn_mobs, spawn_monsters (WorldGen/Entity)
- block_drops, mob_drops, entity_drops (Entity/Items)
- natural_health_regeneration (Entity — has TODO in hunger.rs)
- immediate_respawn, limited_crafting, reduced_debug_info

### 2. Enforced `log_admin_commands` in Command Dispatcher

Added game rule check in `dispatcher.rs:dispatch()` — when `log_admin_commands` is true (default), all executed commands are logged to the server console with the sender name:
```
{sender} issued server command: /{cmd}
```
This fires after permission check but before execution, matching vanilla behavior.

### 3. `send_command_feedback` Assessment

Already partially enforced in:
- `gamemode.rs` — checks before sending "game mode changed" to target
- `command.rs` — used for command block output behavior

Full enforcement would require wrapping every command's `send_message()` calls — documented as future work rather than partial implementation.

## What I Changed

### Modified Files
- `pumpkin/src/command/dispatcher.rs` — added `log_admin_commands` enforcement in `dispatch()`

### Session Files
- `.claude/sessions/2026-02-07/007_core_gamerule-audit.md` — this file
- `.claude/sessions/decisions/core.md` — CORE-011

## Decisions Made

### CORE-011: log_admin_commands enforced at dispatcher level
**Date:** 2026-02-07
**Decision:** The `log_admin_commands` game rule is checked in `CommandDispatcher::dispatch()` after permission check. When enabled, logs `"{sender} issued server command: /{cmd}"` to the server console. This covers all command execution paths (player, console, RCON, command block).
**Rationale:** Centralized enforcement in the dispatcher ensures all commands are logged without per-command code. Matches vanilla behavior where all operator commands are logged when the rule is enabled.
**Affects:** Core
**Status:** active

## What Others Should Know
- **Game rule enforcement gap:** 78% of defined rules are not enforced. See audit above for categorization by agent scope. Entity agent has the most unenforced rules (damage types, spawning, drops).
- **`natural_health_regeneration`:** Entity's hunger.rs has an explicit `TODO: GameRule check` at line 57. The game rule exists in the registry.
- **`pvp` rule:** Defined but never checked. Entity scope.
- **`random_tick_speed`:** Defined but never checked. WorldGen/World scope.

## Tests
- `cargo test -p pumpkin` — **121 tests pass**, 0 failures
- `cargo check -p pumpkin` — compiles cleanly with RUSTFLAGS="-Dwarnings"
