# MC 1.18.2 PrismarineJS Data

> **Caves & Cliffs Part II** — Protocol 758. The version that changed world height to -64→320 and overhauled cave generation.
> Data sourced from PrismarineJS minecraft-data (inheritance chain: 1.18 → 1.18.1 → 1.18.2)

## File Inventory

| File | Source Version | Entries | Size |
|------|---------------|---------|------|
| entities.json | 1.18.2 | 113 | 21KB |
| blocks.json | 1.18.2 | 898 | 599KB |
| items.json | 1.18.2 | 1100 | 135KB |
| recipes.json | 1.18.2 | 639 cats (1297) | 318KB |
| biomes.json | 1.18.2 | 61 | 17KB |
| enchantments.json | 1.18.2 | 38 | 15KB |
| effects.json | 1.18.2 | 33 | 3KB |
| foods.json | 1.18.2 | 40 | 8KB |
| materials.json | 1.18.2 | 16 categories | 2KB |
| blockLoot.json | 1.18.2 | 835 | 189KB |
| entityLoot.json | 1.18.2 | 75 | 18KB |
| attributes.json | 1.18 | 13 | 2KB |
| particles.json | 1.18.2 | 88 | 4KB |

## Version Delta: 1.18.2 → 1.21.4

| Category | 1.18.2 | 1.21.4 | Added | Removed |
|----------|--------|--------|-------|---------|
| **Entities** | 113 | 149 | +37 | -1 (boat → split into wood-specific boats) |
| **Blocks** | 898 | 1095 | +198 | -1 (grass → short_grass) |
| **Items** | 1100 | 1385 | +287 | -2 (grass→short_grass, scute→turtle_scute) |
| **Effects** | 33 | 39 | +8 | -1 (BadLuck→Unluck) |
| **Enchantments** | 38 | 42* | +4 | 0 |
| **Biomes** | 61 | 135* | +74 | 0 |

*1.21.4 enchantments/biomes counts from 1.16.5 registry (no 1.21.4 prismarine data for these)

### Key New Content (1.19–1.21)
- **1.19 (Wild Update):** Warden, allay, frog, sculk, mangrove, mud, ancient city
- **1.20 (Trails & Tales):** Camel, sniffer, cherry grove, bamboo wood, armor trims, pottery
- **1.21 (Tricky Trials):** Breeze, bogged, wind charge, trial spawner, vault, mace, armadillo

### Notable Entity Changes
- `boat` removed → split into `acacia_boat`, `birch_boat`, `cherry_boat`, etc. (6 wood types x 2 variants)
- 37 new entities including: allay, armadillo, axolotl→already in 1.18.2, breeze, camel, creaking, frog, sniffer, warden

### Notable Block Changes
- 198 new blocks (22% increase)
- Major additions: hanging signs, candles→already in 1.18.2, sculk family→partial, cherry wood, bamboo wood
- 1 rename: `grass` → `short_grass`

### Notable Effect Changes
- +8 new effects: Darkness, Infested, Oozing, RaidOmen, TrialOmen, Unluck, Weaving, WindCharged
- 1 rename: BadLuck → Unluck

## Multi-Version Strategy Notes

This data represents the **Tier 2** baseline from the sprint index. Key architectural differences:
- 1.18.2 introduced the new world height (-64→320) and cave generation
- 1.18.2 still uses pre-component item system (NBT-based, not item components)
- 1.18.2 chunk format uses the new height range but simpler section encoding than 1.21.4
- Protocol 758 vs 774 — significant packet changes across 3 major versions
