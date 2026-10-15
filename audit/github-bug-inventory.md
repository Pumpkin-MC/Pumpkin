# Pumpkin Bug Inventory & Root-Cause Analysis

This inventory documents all analyzed and addressed defects, categorizing them by architectural subsystem and verifying fix integrity against Minecraft Java Vanilla parity.

## I. Locally Resolved Defects in Current Audit Cycle

### 1. Issue #4075: Empty command in console triggers 'Unknown command: '
- **Subsystem:** Console & Command Dispatcher (`pumpkin-command`, `pumpkin-core`)
- **Root Cause:** Command dispatcher attempted to process whitespace/empty strings without an early guard, falling through to the fallback error message. In addition, stdin console setup was not trimming/ignoring empty enter keypresses.
- **Resolution:**
  - Added empty/whitespace guard in `CommandDispatcher::handle_command`: `if input.trim().is_empty() { return; }`.
  - Added unit test `empty_command_handling` verifying empty commands produce no errors.
  - Added empty line suppression in `setup_stdin_console` and `setup_console`.
- **Status:** **RESOLVED & VERIFIED** (Test passed in `pumpkin-command`).

### 2. Issue #4064: Sleeping in bed causes suffocation damage
- **Subsystem:** Player Entity & World Simulation (`pumpkin-core`)
- **Root Causes:**
  - `player.sleep()` set `set_pose(EntityPose::Sleeping)` prior to positioning at bed coordinates, misaligning the bounding box.
  - Movement packets during sleeping allowed client interpolation to shift player position into adjacent block colliders.
  - `update_player_pose` continuously checked `can_fit_pose` and demoted sleeping players to crouching/swimming, removing sleeping pose.
  - Suffocation check in `living.rs` lacked an explicit guard against sleeping players.
- **Resolution:**
  - Position is set before pose assignment in `sleep()`.
  - Sleeping players lock pose to `EntityPose::Sleeping` in `update_player_pose()`.
  - Position packets for sleeping players discard delta shifts and snap back if client delta exceeds 1.0.
  - Suffocation loop explicitly exempts sleeping players.
- **Status:** **RESOLVED & VERIFIED**.

### 3. Issue #4063: Fish go wild on land (Mob AI navigation out of water)
- **Subsystem:** Entity AI & Pathfinding (`pumpkin-core`)
- **Root Cause:** `WaterBoundPathNavigation::tick` continued updating paths and computing steering velocities even when the entity had exited water onto dry land.
- **Resolution:** Added water-contact check in `WaterBoundPathNavigation::tick`. When an entity with `!allow_breaching` is not touching water, pathfinding is halted and horizontal movement input is zeroed.
- **Status:** **RESOLVED & VERIFIED**.

### 4. Issue #4070: Parts of trees generating across chunk borders get replaced with grass
- **Subsystem:** World Generation & Feature Placement (`pumpkin-world`)
- **Root Cause:** `SimpleBlockFeature::generate()` invoked `block_registry.can_place_at(block, state, chunk, pos)` without testing if the destination block `pos` was replaceable. In vanilla, simple block features are only placed in air or replaceable blocks (`state.replaceable()`).
- **Resolution:** Added `if !chunk.get_block_state(&pos).replaceable() { return false; }` in `SimpleBlockFeature::generate()`.
- **Status:** **RESOLVED & VERIFIED**.

### 5. Issue #4076: Geodes do not generate (Amethyst Geode feature failure)
- **Subsystem:** World Generation & Configured Features (`pumpkin-world`, `pumpkin-data`)
- **Root Causes:**
  - In `configured_features_generated.rs`, missing fields for `distribution_points` and `point_offset` defaulted to `Constant(0)` instead of vanilla defaults `Uniform(3, 4)` and `Uniform(1, 2)`. With 0 distribution points, the distance shell sum remained 0 and no blocks were ever placed.
  - In `GeodeFeature::generate`, `invalid_blocks` and `cannot_replace` checked `Block::from_name(s.as_str())`, which failed for block tags starting with `#` like `#minecraft:geode_invalid_blocks`.
- **Resolution:**
  - Added block tag resolution via `pumpkin_data::tag::get_tag_ids(RegistryKey::Block, tag_str)`.
  - Added vanilla fallback defaults in `GeodeFeature::generate` for `distribution_points` (Uniform 3..=4) and `point_offset` (Uniform 1..=2).
  - Configured `AmethystGeode` with vanilla uniform providers in `configured_features_generated.rs`.
  - Added unit test `test_geode_feature_generates_blocks` verifying geode generation.
- **Status:** **RESOLVED & VERIFIED**.

### 6. Issue #4048: Memory grows by ~3 MB with every player join and leave
- **Subsystem:** Chunk Management & Memory Lifecycle (`pumpkin-world`, `pumpkin-core`)
- **Root Cause:** `Level::clean_memory()` pruned `chunk_watchers` and `loaded_entity_chunks` but completely omitted pruning unwatched chunks from `self.loaded_chunks`. As a result, chunks loaded around every joining player remained in memory indefinitely after the player left.
- **Resolution:** In `Level::clean_memory()`, unwatched chunks are now pruned from `self.loaded_chunks` (saving dirty ones asynchronously to disk), and `loaded_chunks.shrink_to_fit()` is invoked to release excess heap capacity. Added unit test `clean_memory_prunes_unwatched_chunks`.
- **Status:** **RESOLVED & VERIFIED**.

## II. Historical & Master Parity Fixes Preserved

1. **Villager & Structure Generation Parity:** Complete resolution of village structures generating without villagers, restoring vanilla structure entity spawners.
2. **Dimension & Biome Mob Spawning Caps:** Restoration of accurate mob caps across Overworld, Nether, and The End.
3. **Entity Lifecycle & Data Preservation:** Safe chunk unloading and entity serialization parity.
