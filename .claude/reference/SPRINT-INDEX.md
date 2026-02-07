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

## TOML Registries (machine-queryable)

The source of truth. Agents grep these to find their backlog.

| Registry | Path | Key Counts |
|----------|------|-----------|
| **Bukkit API** | `.claude/registry/bukkit_api.toml` | 283 events (28 implemented, 252 missing), 9 interfaces, 5 enums |
| **Entities** | `.claude/registry/entities.toml` | 149 entities, 39 effects, 40 foods, 49 damage types |
| **Items** | `.claude/registry/items.toml` | 1385 items, 42 enchantments, 1370 recipes, 1237 loot tables |
| **Blocks** | `.claude/registry/blocks.toml` | 1095 blocks (198 types), 135 biomes, 47 structures |

**How to query your backlog:**
```bash
# Entity agent: what events do I need to fire?
grep -B5 'owner = "entity"' .claude/registry/bukkit_api.toml | grep 'name ='

# What entities are missing implementations?
grep -B2 'pumpkin_status = "missing"' .claude/registry/entities.toml | grep 'name ='

# Block type categories:
grep -v '#' .claude/registry/blocks.toml | grep -A1 '\[block_types\]'
```

---

## Markdown References (human-readable)

Detailed briefings with gap analysis. Read YOUR file on session start.

| Agent | File | Key Contents |
|-------|------|-------------|
| **Entity** | [entity-data.md](entity-data.md) | 149 entities, hitboxes, metadata, damage types, effects, foods, gap analysis |
| **World** | [world-data.md](world-data.md) | 1095 blocks, biomes, worldgen, mining speeds, gap analysis |
| **Items** | [items-data.md](items-data.md) | Item components, 1370 recipes, 1237 loot tables, enchantments, gap analysis |
| **Protocol** | [protocol-data.md](protocol-data.md) | Registries, command tree, entity metadata, packet coverage |
| **Plugin** | [plugin-data.md](plugin-data.md) | 283 Bukkit events mapped to Pumpkin, lifecycle, scheduler |
| **Redstone** | [redstone-data.md](redstone-data.md) | Redstone block states, component types, signal propagation |
| **Core** | [core-data.md](core-data.md) | Vanilla commands, tick loop, server lifecycle, config |
| **Storage** | *(covered by World)* | NBT format, Anvil regions — pumpkin-nbt/ already implemented |
| **Architect** | *(reads all)* | Cross-agent coordination, trait definitions |

---

## How Agents Should Use This

1. **On session start:** Read YOUR reference `.md` file + this index
2. **Query your backlog:** `grep 'owner = "YOUR_AGENT"' .claude/registry/bukkit_api.toml`
3. **Before implementing:** Check the Gap Analysis section in your `.md` file
4. **Sprint priorities:** Each `.md` file ends with prioritized work items
5. **Version awareness:** All data is 1.21.4. If implementing version-sensitive features, note the version pinning point
6. **Raw data deep-dive:** File paths to original JSON/zip sources are included for when you need exact values
7. **Update registries:** When you implement something, update the `status` field in the relevant `.toml`

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
