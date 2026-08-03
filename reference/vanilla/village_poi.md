# Village / POI system

Source: `net.minecraft.world.entity.ai.village.poi.{PoiManager,PoiRecord,PoiTypes}`,
`net.minecraft.server.level.ServerLevel` (`isCloseToVillage`), decompiled 26.2 mappings.

## What exists now

`pumpkin/src/world/village_poi.rs` (landed this session): a genuine POI density system.

- Reuses the *existing* region-file-backed `World::portal_poi` (`pumpkin_world::poi::PoiStorage`)
  — vanilla itself keeps all POI types (portals, beds, job sites, etc.) in one registry, so
  sharing storage is not a scope shortcut, it matches vanilla's own architecture.
- Classifies blocks into POI types using existing tags: `MINECRAFT_BEDS` → `home`, `Block::BELL`
  → `meeting`, `C_VILLAGER_JOB_SITES` → a generic `job_site` (this approximates vanilla's
  per-profession job-site POI types as one bucket — see "deferred" below).
- `section_chebyshev_distance`: vanilla's `SectionTracker` 26-neighbor BFS
  (`SectionTracker.checkNeighborsAfterUpdate`) computes **Chebyshev distance** in sections
  (max of |dx|,|dy|,|dz|), not Manhattan or Euclidean. This is the metric behind
  `sectionsToVillage`/`isCloseToVillage`. Get this wrong and village-density queries silently
  use the wrong radius shape.
- `in_sphere`: reproduces `PoiManager.getInRange`'s true 3D sphere filter (Y axis included,
  applied *after* a square/box prefilter for performance — vanilla does the same two-stage
  filter, don't skip the box prefilter and call it equivalent).
- `World::poi_count_in_range`, `sections_to_village`, `is_close_to_village` — the query API
  other subsystems call. `CatSpawner` (`world/custom_spawners.rs`) already uses the real vanilla
  two-part gate: `is_close_to_village(pos, 2)` then `poi_count_in_range(HOME, pos, 48) > 4`.
- `set_block_state` (the single block-mutation chokepoint) adds/removes POI entries on relevant
  block changes, mirroring vanilla's `onBlockStateChange`.

## Deliberately deferred (don't silently "fix" these without understanding why they're deferred)

1. **Occupancy.ANY instead of Occupancy.IS_OCCUPIED.** Pumpkin has no bed-claiming
   (`AcquirePoi` in vanilla — a villager claims a specific bed POI as its own). Faithfully
   filtering by `IS_OCCUPIED` would always return zero occupied beds and regress cat spawning
   (which currently relies on POI *existing*, not on being claimed). Fixing this properly
   requires building villager bed-claiming first — a bigger, separate piece of work, not a
   one-line change here.
2. **No backfill for world-generated structures.** Villages placed by worldgen write blocks
   directly into chunk sections during generation, bypassing `set_block_state` — so a
   freshly-generated village registers zero POIs until a bed/job-site block is placed or broken
   post-generation (e.g. by a player or a villager). Vanilla's equivalent is
   `PoiManager.checkConsistencyWithBlocks`, run at chunk load, which scans the chunk's blocks
   against the POI registry and backfills any that are missing. Implementing this properly means
   hooking into `pumpkin-world`'s chunk-load path, not `pumpkin`'s `set_block_state` — different
   crate, different lifecycle point.
3. **Both bed halves counted as `HOME`.** Vanilla only counts the head half
   (`BedPart.HEAD`) as the POI position; Pumpkin's tag-based classification currently doesn't
   distinguish bed halves, so both count. This is a constant 2x overcount on bed-POI density —
   it doesn't change which side of the `>4` threshold a real village falls on, but if you're
   adding a new consumer of `poi_count_in_range(HOME, ...)` with a tighter threshold, account for
   this, or fix it properly by checking the bed's `part` block-state property.

## Consumers still worth wiring (not done)

- Iron golem village-defense logic — needs `is_close_to_village`/density query, doesn't exist yet.
- `MoveThroughVillageGoal` for villagers — doesn't exist as a goal at all yet.
- Villager reputation propagation-by-witness (the gossip system already exists per-villager, but
  has no "who is nearby to witness this event" village-scoped mechanism using POI density).
