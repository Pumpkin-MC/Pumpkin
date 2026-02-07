# Sprint Reference Index

> Source of truth for all agents. Data extracted from vanilla MC 1.21.4, PrismarineJS, and Bukkit API.

## Data Version

| Source | Version | Protocol | Notes |
|--------|---------|----------|-------|
| Mojang vanilla (misode/mcmeta) | 1.21.4 | — | Blocks, recipes, loot tables, worldgen, damage types, tags |
| PrismarineJS (minecraft-data) | 1.21.4 | — | Entities, foods, effects, materials, enchantments |
| Bukkit/Spigot API | 1.21.11 | 774 | 318 events, plugin lifecycle, scheduler |
| Pumpkin target | 1.21.11 | 772–774 | Current supported range |

### Multi-Version Strategy

Pumpkin currently supports **1.21.7–1.21.11** (protocol 772–774). Only packet ID mapping exists for the range; all gameplay data is 1.21.11-only.

**Version cliff points in Minecraft history:**

| Version | Protocol | Breaking Change |
|---------|----------|----------------|
| 1.12.2 → 1.13 | 340 → 393 | **The Flattening** — numeric block IDs replaced by block states |
| 1.13 → 1.14 | 393 → 477 | New chunk format, heightmap changes |
| 1.15 → 1.16 | 578 → 735 | Dimension codec, new biome system |
| 1.17 → 1.18 | 756 → 757 | World height -64 to 320, chunk format v2 |
| 1.20.4 → 1.20.5 | 765 → 766 | Item components replaced NBT |

**Recommended multi-version tiers:**
- **Tier 0 (current):** 1.21.x minor range — packet ID mapping only
- **Tier 1:** Back to 1.20.5 — add item component translation
- **Tier 2:** Back to 1.18 — add chunk format + world height translation
- **Tier 3 (ViaVersion-scale):** Back to 1.12.2 — full packet rewriting

All reference packages below are tagged **MC 1.21.4** unless noted.

---

## Agent Reference Packages

| Agent | File | Key Contents | Raw Data Sources |
|-------|------|-------------|-----------------|
| **Entity** | [entity-data.md](entity-data.md) | 149 entities, hitboxes, metadata, damage types, effects, foods, AI | entities.json, effects.json, foods.json, damage_type/*.json |
| **World** | [world-data.md](world-data.md) | 1095 blocks, states, biomes, worldgen structures, mining speeds | blocks.json, block_definitions.json, materials.json, worldgen/ |
| **Items** | [items-data.md](items-data.md) | Item components, 1370 recipes, 1237 loot tables, enchantments | item_components.json, recipes.json, loot_table/, enchantments.json |
| **Protocol** | [protocol-data.md](protocol-data.md) | Registries, command tree, entity metadata, packet coverage | registries.json, commands.json, entities.json metadata |
| **Plugin** | [plugin-data.md](plugin-data.md) | 318 Bukkit events mapped to Pumpkin, lifecycle, scheduler | BUKKIT-API-REFERENCE.md, bukkit-api-ref.zip |
| **Redstone** | [redstone-data.md](redstone-data.md) | Redstone block states, component types, signal propagation | blocks.json (redstone subset), block_definitions.json |
| **Core** | [core-data.md](core-data.md) | Vanilla commands, tick loop, server lifecycle, config | commands.json, Bukkit.txt, BukkitScheduler.txt |
| **Storage** | *(covered by World)* | NBT format, Anvil regions | pumpkin-nbt/ (already implemented) |
| **Architect** | *(reads all)* | Cross-agent coordination, trait definitions | All of the above |

---

## How Agents Should Use This

1. **On session start:** Read YOUR reference file + this index
2. **Before implementing:** Check the Gap Analysis section — it tells you what exists vs what's missing
3. **Sprint priorities:** Each file ends with prioritized work items
4. **Version awareness:** All data is 1.21.4. If implementing version-sensitive features, note the version pinning point
5. **Raw data deep-dive:** File paths to original JSON/zip sources are included for when you need exact values

## Raw Data Locations

```
.claude/specs/data/
├── README.md                          # Original data source guide
├── mcdata-1.21.4.zip                  # 4544 files: recipes, loot, worldgen, damage, tags
├── 1.21.4/
│   ├── summary/
│   │   ├── blocks.json                # 1095 blocks with state properties
│   │   ├── block_definitions.json     # Block type classifications
│   │   ├── registries.json            # All game registries (650KB)
│   │   ├── item_components.json       # Item component data (507KB)
│   │   └── commands.json              # Vanilla command tree (124KB)
│   └── prismarine/
│       ├── prismarine-1.21.4.zip      # Full PrismarineJS dataset
│       ├── entities.json              # 149 entities with hitboxes + metadata
│       ├── effects.json               # 32+ status effects
│       ├── foods.json                 # ~50 foods with nutrition
│       └── materials.json             # Tool mining speed table
└── bukkit-api/
    ├── BUKKIT-API-REFERENCE.md        # 318 events, plugin lifecycle
    └── bukkit-api-ref.zip             # 23 scraped Javadoc files
```
