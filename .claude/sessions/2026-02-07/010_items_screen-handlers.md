# Session: items-004
**Agent:** items
**Date:** 2026-02-07
**Task:** Implement 6 inventory screen handlers (anvil, grindstone, enchanting table, brewing stand, loom, cartography table)

## Context Loaded
- Read all session logs 001-009 from today
- Read .claude/sessions/decisions/{items,architect}.md
- Read .claude/registry/items.toml, items_1_16_5.toml, bukkit_api.toml
- Rebased on origin/master (d61d6d3), clean rebase

## Preamble
I am the Items agent, session 004. Rebased cleanly on latest master (9 new commits from world, redstone, core, protocol, plugin agents). This session implements the P1 priority: inventory screen handlers. Six new handlers created, all following the established pattern from stonecutter/smithing.

## What I Did

### 1. AnvilScreenHandler (`anvil.rs`, ~570 lines)

**New Types:**
- `AnvilInventory` — 2-slot inventory (left + right input)
- `AnvilOutputSlot` — output slot, cannot insert directly
- `AnvilScreenHandler` — full handler with slot layout 0-1 input, 2 output, 3-38 player

**New Functions:**
- `compute_anvil_result(left, right) -> Option<(ItemStack, i32)>` — computes result + level cost
  - Case 1: Same item type → repair + enchantment merge (12% durability bonus)
  - Case 2: Enchanted book → apply enchantments to target
  - Case 3: Material repair → 25% per material using repair tag system
- `merge_enchantments(target, source) -> i32` — merges enchantments with level combining
- `is_repair_material(item, material) -> bool` — checks repair tags via material.has_tag() + item registry_key prefix matching

**Bug Fix:** Repair tags contain MATERIALS (e.g., `REPAIRS_DIAMOND_ARMOR` → "diamond"), not the items being repaired. Fixed by checking `material.has_tag()` and matching item via `registry_key` prefix.

**10 tests** covering repair, enchant combine, material repair, edge cases.

### 2. GrindstoneScreenHandler (`grindstone.rs`, ~515 lines)

**New Types:**
- `GrindstoneInventory` — 2-slot inventory (top + bottom input)
- `GrindstoneOutputSlot` — output slot
- `GrindstoneScreenHandler` — slots 0-1 input, 2 output, 3-38 player

**New Functions:**
- `compute_grindstone_result(top, bottom) -> Option<ItemStack>` — enchantment removal + durability combine
  - Single item: strip non-curse enchantments
  - Two same-type: combine durability (5% bonus) + strip non-curse enchantments
- `strip_enchantments(stack) -> Option<ItemStack>` — removes non-curse enchantments via DataComponent patch
- `transfer_curses_only(target, source)` — copies only Binding/Vanishing curses
- `is_curse(enchantment) -> bool` — checks for curse enchantments

**8 tests** covering strip, curse preservation, durability combine, edge cases.

### 3. EnchantingTableScreenHandler (`enchanting_table.rs`, ~340 lines)

**New Types:**
- `EnchantingTableInventory` — 2-slot (item + lapis)
- `LapisSlot` — custom slot that only accepts lapis lazuli
- `EnchantingTableScreenHandler` — slots 0 item, 1 lapis, 2-37 player

**Window properties tracked:** seed, level_requirements[3], enchantment_ids[3], enchantment_levels[3]

**Helper:** `can_enchant(item) -> bool` — checks Book or `ENCHANTABLE_DURABILITY` tag

**5 tests** for inventory, slot validation, enchantability checks.

**TODO:** Enchantment generation requires bookshelf counting + seed-based randomization (block entity/world access).

### 4. BrewingStandScreenHandler (`brewing_stand.rs`, ~375 lines)

**New Types:**
- `BrewingStandInventory` — 5-slot (3 bottles + ingredient + fuel)
- `PotionSlot` — accepts potions, splash potions, lingering potions, glass bottles (max 1)
- `FuelSlot` — only accepts blaze powder
- `BrewingStandScreenHandler` — slots 0-2 bottles, 3 ingredient, 4 fuel, 5-40 player

**Helper:** `is_potion_slot_item(item) -> bool`

**7 tests** for inventory, slot validation, max count.

**TODO:** Brewing is tick-driven (400 ticks per batch). Block entity integration needed. Recipe data available in `pumpkin_data::potion_brewing::POTION_RECIPES`.

### 5. LoomScreenHandler (`loom.rs`, ~380 lines)

**New Types:**
- `LoomInventory` — 3-slot (banner + dye + pattern item)
- `LoomOutputSlot` — output slot
- `LoomScreenHandler` — slots 0 banner, 1 dye, 2 pattern, 3 output, 4-39 player

**Helpers:**
- `is_banner(item) -> bool` — uses `MINECRAFT_BANNERS` tag
- `is_dye(item) -> bool` — checks all 16 dye items
- `is_banner_pattern_item(item) -> bool` — checks 10 special pattern items

**4 tests** for inventory, output slot, banner/pattern detection.

**TODO:** Pattern application requires `BannerPatternsImpl` (currently stub).

### 6. CartographyTableScreenHandler (`cartography_table.rs`, ~350 lines)

**New Types:**
- `CartographyTableInventory` — 2-slot (map + additional)
- `CartographyOutputSlot` — output slot
- `CartographyTableScreenHandler` — slots 0 map, 1 additional, 2 output, 3-38 player

**Helper:** `compute_cartography_result(map, additional) -> Option<ItemStack>`
- Paper → extend (produces 1 filled map)
- Empty map → clone (produces 2 filled maps)
- Glass pane → lock (produces 1 filled map)

**7 tests** for inventory, all 3 operations, edge cases.

**TODO:** All operations require `MapIdImpl` (stub). Currently produces placeholder filled maps.

## What I Changed
- `pumpkin-inventory/src/anvil.rs` — NEW (570 lines, 10 tests)
- `pumpkin-inventory/src/grindstone.rs` — NEW (515 lines, 8 tests)
- `pumpkin-inventory/src/enchanting_table.rs` — NEW (340 lines, 5 tests)
- `pumpkin-inventory/src/brewing_stand.rs` — NEW (375 lines, 7 tests)
- `pumpkin-inventory/src/loom.rs` — NEW (380 lines, 4 tests)
- `pumpkin-inventory/src/cartography_table.rs` — NEW (350 lines, 7 tests)
- `pumpkin-inventory/src/lib.rs` — added 4 new module registrations

## Screen Handler Coverage

| Handler | Status | WindowType | Tests |
|---------|--------|------------|-------|
| CraftingTable | Pre-existing | Generic9x3 | Yes |
| FurnaceLike (Furnace/BlastFurnace/Smoker) | Pre-existing | Furnace/BlastFurnace/Smoker | Yes |
| GenericContainer (Chest/Barrel/Hopper/Shulker) | Pre-existing | Generic* | Yes |
| Player | Pre-existing | — | Yes |
| Smithing | Session 001 | SmithingTable | Yes |
| Stonecutter | Session 001 | Stonecutter | Yes |
| **Anvil** | **This session** | Anvil | 10 |
| **Grindstone** | **This session** | Grindstone | 8 |
| **EnchantingTable** | **This session** | Enchantment | 5 |
| **BrewingStand** | **This session** | BrewingStand | 7 |
| **Loom** | **This session** | Loom | 4 |
| **CartographyTable** | **This session** | CartographyTable | 7 |

**Remaining NOT implemented:** Beacon, Lectern, Merchant, Crafter3x3

## What I Need From Others
- **Protocol**: Window property syncing for enchanting table (10 properties), brewing stand (2 properties), loom (1 property)
- **Block entity integration**: Enchanting (bookshelf counting, seed generation), Brewing (tick-driven processing), Loom (pattern listing)
- **Data component impls**: BannerPatternsImpl, MapIdImpl, RepairableImpl, RepairCostImpl (currently stubs)

## What Others Should Know
- **136 tests now pass** in pumpkin-inventory (up from 94). All 6 new handlers compile and test cleanly.
- Quick-move routing follows vanilla slot index patterns exactly.
- All handlers are structurally complete — only processing logic that requires block entity or server-side tick management is deferred.
- Repair tag logic: `material.has_tag()` checks the tag, `item.registry_key` prefix matching determines the item category. This is a workaround for `RepairableImpl` being a stub.

## Decisions Made

### ITEMS-009: Anvil material repair uses tag + name prefix matching
**Date:** 2026-02-07
**Decision:** `is_repair_material()` checks `material.has_tag(&REPAIRS_*_ARMOR)` combined with `item.registry_key.starts_with("diamond_")` etc. This is a workaround because `RepairableImpl` is a stub.
**Rationale:** Repair tags contain materials, not items. Without the repairable component, name prefix matching is the most reliable heuristic.
**Affects:** Items
**Status:** active (supersede when RepairableImpl is implemented)

### ITEMS-010: Screen handlers defer processing logic to block entity
**Date:** 2026-02-07
**Decision:** Enchanting, brewing, and loom screen handlers provide correct slot layouts and quick-move routing but defer actual processing (enchantment generation, brewing ticks, pattern application) to block entity integration.
**Rationale:** These operations require world access (bookshelf counting), tick-driven processing (brewing), or component system support (banner patterns) that is outside pumpkin-inventory scope.
**Affects:** Items
**Status:** active

## Tests
- `cargo test -p pumpkin-inventory` — **136 tests pass**, 0 failures
- `RUSTFLAGS="-Dwarnings" cargo check -p pumpkin-inventory` — 0 warnings
- `cargo check -p pumpkin` — full binary compiles, 0 warnings

## Open Questions
1. **Beacon screen handler**: Needs effect selection logic and beacon power level tracking. Lower priority than the 6 implemented this session.
2. **Merchant/Villager trading**: Complex — needs villager entity integration, trade offer system. Blocked on Entity agent.
3. **Crafter3x3**: New in 1.21 — needs redstone integration for auto-crafting. Blocked on Redstone agent.
