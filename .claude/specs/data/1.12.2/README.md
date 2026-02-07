# MC 1.12.2 PrismarineJS Data

> **World of Color Update** — Protocol 340. The last version before The Flattening (1.13). Uses numeric block/item IDs.
> Data sourced from PrismarineJS minecraft-data (inheritance chain: 1.12 → 1.12.1 → 1.12.2)

## File Inventory

| File | Source Version | Entries | Size |
|------|---------------|---------|------|
| entities.json | 1.12 | 84 | 21KB |
| blocks.json | 1.12 | 254 | 249KB |
| items.json | 1.12 | 410 | 57KB |
| recipes.json | 1.12 | 202 cats (346) | 105KB |
| biomes.json | 1.12 | 62 | 17KB |
| enchantments.json | 1.12 | 30 | 12KB |
| effects.json | 1.12 | 27 | 3KB |
| foods.json | 1.12 | 31 | 7KB |
| materials.json | 1.12 | 8 categories | 1KB |
| attributes.json | 1.12 | 13 | 2KB |
| particles.json | 1.12 | 49 | 2KB |

Note: No blockLoot.json or entityLoot.json available for 1.12.2.

## Version Delta: 1.12.2 → 1.21.4

| Category | 1.12.2 | 1.21.4 | Added | Removed/Renamed |
|----------|--------|--------|-------|-----------------|
| **Entities** | 84 | 149 | +80 | -15 (many pre-Flattening renames) |
| **Blocks** | 254 | 1095 | +911 | -70 (pre-Flattening numeric IDs) |
| **Items** | 410 | 1385 | +1049 | -74 (pre-Flattening numeric IDs) |
| **Effects** | 27 | 39 | +13 | -1 (BadLuck→Unluck) |
| **Enchantments** | 30 | 42* | +12 | 0 |
| **Biomes** | 62 | 135* | +73 | 0 |

### The Flattening Impact (1.12.2 → 1.13)

1.12.2 is the **last pre-Flattening version**. This is the most significant version cliff in Minecraft history:
- **Block IDs:** 254 blocks with numeric IDs and metadata variants → 1.13+ uses string IDs with block states
- **Item IDs:** 410 items with numeric IDs and damage values → 1.13+ uses string IDs
- **Entity names:** Many entities have different names (e.g., `villager_golem` → `iron_golem`, `snowman` → `snow_golem`)
- **Data values:** Block metadata (0-15) replaced by block state properties

### Removed/Renamed Entities (15 total)
- `boat` → split into wood-specific boats (1.19+)
- `commandblock_minecart` → `command_block_minecart`
- `ender_crystal` → `end_crystal`
- `evocation_fangs` → `evoker_fangs`
- `evocation_illager` → `evoker`
- `eye_of_ender_signal` → `eye_of_ender`
- `fireworks_rocket` → `firework_rocket`
- `Fishing Hook` → removed as entity type
- `illusion_illager` → `illusioner`
- `snowman` → `snow_golem`
- `villager_golem` → `iron_golem`
- `vindication_illager` → `vindicator`
- `xp_bottle` → `experience_bottle`
- `xp_orb` → `experience_orb`
- `zombie_pigman` → `zombified_piglin`

### Key New Content (1.13–1.21)
- **1.13 (The Flattening/Update Aquatic):** String IDs, block states, drowned, phantoms, tridents, coral, kelp
- **1.14 (Village & Pillage):** Pillagers, villager rework, new blocks (barrels, bells, etc.)
- **1.15 (Buzzy Bees):** Bees, beehives, honey blocks
- **1.16 (Nether Update):** Piglins, hoglins, netherite, basalt, crimson/warped
- **1.17 (Caves & Cliffs I):** Axolotl, goat, glow squid, amethyst, copper, deepslate
- **1.18 (Caves & Cliffs II):** World height -64→320, new cave biomes
- **1.19 (Wild Update):** Warden, allay, frog, sculk, mangrove
- **1.20 (Trails & Tales):** Camel, sniffer, cherry grove, bamboo wood
- **1.21 (Tricky Trials):** Breeze, bogged, wind charge, trial spawner, vault, mace

## Multi-Version Strategy Notes

This data represents the **Tier 3** baseline from the sprint index. Key architectural differences:
- 1.12.2 uses **numeric block/item IDs** — completely different from 1.21.4's string-based system
- 1.12.2 world height is 0→255 (vs -64→320 in 1.21.4)
- 1.12.2 chunk format is drastically different (no palette, no height map changes)
- Protocol 340 vs 774 — the most extensive protocol gap of any version pair
- Supporting 1.12.2 requires a full protocol translation layer (ViaVersion-style)
- Block metadata system (0-15 per block) has no equivalent in modern Minecraft
