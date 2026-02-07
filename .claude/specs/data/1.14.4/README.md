# MC 1.14.4 PrismarineJS Data

> **Village & Pillage** — Protocol 498. The version that overhauled villages, trading, and introduced pillagers.
> Data sourced from PrismarineJS minecraft-data (inheritance chain: 1.14 → 1.14.1 → 1.14.3 → 1.14.4)

## File Inventory

| File | Source Version | Entries | Size |
|------|---------------|---------|------|
| entities.json | 1.14.4 | 102 | 19KB |
| blocks.json | 1.14.4 | 676 | 432KB |
| items.json | 1.14.4 | 876 | 107KB |
| recipes.json | 1.14.4 | 501 cats (949) | 232KB |
| biomes.json | 1.14.4 | 75 | 20KB |
| enchantments.json | 1.14.4 | 30 | 12KB |
| effects.json | 1.14.4 | 33 | 3KB |
| foods.json | 1.14.4 | 39 | 8KB |
| materials.json | 1.14.4 | 8 categories | 1KB |
| blockLoot.json | 1.14.4 | 615 | 138KB |
| entityLoot.json | 1.14.4 | 66 | 17KB |
| attributes.json | 1.14 | 13 | 2KB |
| particles.json | 1.14.4 | 58 | 3KB |

## Version Delta: 1.14.4 → 1.21.4

| Category | 1.14.4 | 1.21.4 | Added | Removed |
|----------|--------|--------|-------|---------|
| **Entities** | 102 | 149 | +49 | -2 (boat → split, zombie_pigman → zombified_piglin) |
| **Blocks** | 676 | 1095 | +421 | -2 (grass→short_grass, grass_path→dirt_path) |
| **Items** | 876 | 1385 | +513 | -4 (grass, grass_path, scute→turtle_scute, zombie_pigman_spawn_egg) |
| **Effects** | 33 | 39 | +8 | -1 (BadLuck→Unluck) |
| **Enchantments** | 30 | 42* | +12 | 0 |
| **Biomes** | 75 | 135* | +60 | 0 |

### Key New Content (1.15–1.21)
- **1.15 (Buzzy Bees):** Bees, beehives, honey blocks
- **1.16 (Nether Update):** Piglins, hoglins, netherite, basalt, crimson/warped forests, soul speed
- **1.17 (Caves & Cliffs I):** Axolotl, goat, glow squid, amethyst, copper, deepslate, dripstone
- **1.18 (Caves & Cliffs II):** World height -64→320, new cave biomes
- **1.19 (Wild Update):** Warden, allay, frog, sculk, mangrove, mud
- **1.20 (Trails & Tales):** Camel, sniffer, cherry grove, bamboo wood, armor trims
- **1.21 (Tricky Trials):** Breeze, bogged, wind charge, trial spawner, vault, mace

### Notable Entity Changes
- `boat` removed → split into wood-specific boat types
- `zombie_pigman` renamed → `zombified_piglin` (1.16 Nether Update)
- 49 new entities across 7 major updates

### Notable Block Changes
- 421 new blocks (62% increase)
- The Flattening happened in 1.13, so 1.14.4 already uses modern block IDs
- Major additions: barrels, bells, blast furnaces, smokers, scaffolding, lanterns, campfires, composters, lecterns
- 2 renames: `grass` → `short_grass`, `grass_path` → `dirt_path`

### Notable Effect Changes
- +8 new effects: Darkness, Infested, Oozing, RaidOmen, TrialOmen, Unluck, Weaving, WindCharged
- BadLuck renamed → Unluck
- 1.14.4 already has: BadOmen, HeroOfTheVillage, ConduitPower, DolphinsGrace, SlowFalling

## Multi-Version Strategy Notes

This data represents the **Tier 2** baseline. Key architectural differences:
- 1.14.4 is post-Flattening (1.13) so block/item IDs are modern string-based
- 1.14.4 uses the old world height range (0→255)
- 1.14.4 uses pre-component item system (NBT-based)
- Protocol 498 vs 774 — extensive packet changes across 7 major versions
- 1.14.4 introduced the new villager trading system still used in 1.21.4
