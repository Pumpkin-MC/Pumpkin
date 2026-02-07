# Session: items-001
**Agent:** items
**Date:** 2026-02-07
**Task:** Recipe coverage audit, stonecutter/smithing screen handlers, first test suite for pumpkin-inventory

## Context Loaded
- Read .claude/sessions/2026-02-07/001_redstone_signal-propagation.md (redstone fixes, 28 tests, no items impact)
- Read .claude/sessions/2026-02-07/001_storage_anvil-and-snbt.md (anvil + snbt in pumpkin-nbt, no items impact)
- Read .claude/sessions/2026-02-06/001_architect_gap-analysis.md (items ~50%, first task: recipe audit)
- Read .claude/sessions/2026-02-06/002_architect_restructure-sessions.md (sessions under .claude/)
- Read .claude/sessions/2026-02-06/003_architect_consolidate-claude.md (orchestration under .claude/)
- Read .claude/sessions/2026-02-06/004_architect_setup-validation.md (gitignore policy, workspace compiles)
- Read .claude/sessions/2026-02-06/004_protocol_packets-serialization.md (VarInt overflow, no items impact)
- Read .claude/sessions/2026-02-06/005_entity_spawning-ai-goals-mobs.md (6 mobs, no items impact)
- Read .claude/sessions/decisions/items.md (no prior decisions)
- Read .claude/sessions/decisions/architect.md (ARCH-001 through ARCH-007)
- Read .claude/contracts/items.toml

## Preamble
I am the Items agent. This is the first items session. I acknowledge the architect's gap analysis (session 001) which estimates Items at ~50% complete with gaps in recipe coverage, enchantment system, loot tables, smithing/brewing, durability/repair, bundle, and newer items. The architect's first task recommendation was a "recipe coverage audit against MC 1.21."

I acknowledge ARCH-003: Items owns runtime data loading (`pumpkin/src/data/`), Architect owns compile-time generated data (`pumpkin-data/`). I will not modify `pumpkin-data/`.

I acknowledge ARCH-006 and ARCH-007 (orchestration under .claude/, selective gitignore).

No other agents have requested anything from Items. The redstone agent noted that dispensers need a `DispenserBlockEntity` — this is outside my write paths (that's Entity/Core territory).

## What I Did

### 1. Recipe Coverage Audit

Analyzed `assets/recipes.json` (1470 recipes total) and the build system in `pumpkin-data/build/recipes.rs`:

| Recipe Type | Count | Status |
|---|---|---|
| crafting_shaped | 706 | ✅ Generated + matched |
| crafting_shapeless | 320 | ✅ Generated + matched |
| crafting_transmute | 32 | ✅ Generated + matched |
| crafting_decorated_pot | 1 | ✅ Generated + matched |
| smelting | 73 | ✅ Generated (cooking) |
| blasting | 25 | ✅ Generated (cooking) |
| smoking | 9 | ✅ Generated (cooking) |
| campfire_cooking | 9 | ✅ Generated (cooking) |
| **stonecutting** | **254** | ❌ Parsed but skipped |
| **smithing_transform** | **12** | ❌ Parsed but skipped |
| **smithing_trim** | **18** | ❌ Parsed but skipped |
| **crafting_special_*** | **11** | ❌ Parsed as CraftingSpecial → None |

**Summary:** 1175/1470 recipes (80%) are generated and usable. 295 recipes (20%) are parsed but skipped — stonecutting (254), smithing (30), special (11).

The gap is primarily in `pumpkin-data/build/recipes.rs` where `SmithingTransform`, `SmithingTrim`, `Stonecutting`, and `CraftingSpecial` match arms produce empty `{}` or no output. Since `pumpkin-data/` is Architect territory (ARCH-003), I cannot fix this directly.

### 2. Implemented StonecutterScreenHandler (stonecutter.rs, ~310 lines)

Created a complete stonecutter screen handler in `pumpkin-inventory/`:
- `StonecutterInventory` — 1-slot backing inventory for the input item
- `StonecutterOutputSlot` — output slot that consumes 1 input item on take
- `StonecutterScreenHandler` — full screen handler with:
  - WindowType::Stonecutter
  - Slot 0: Input, Slot 1: Output, Slots 2-37: Player inventory
  - `quick_move()` with correct slot routing
  - `on_closed()` drops input back to player

Recipe selection (choosing which stonecutting recipe to produce) will be functional once the Architect generates `RECIPES_STONECUTTING` data.

### 3. Implemented SmithingScreenHandler (smithing.rs, ~330 lines)

Created a complete smithing table screen handler in `pumpkin-inventory/`:
- `SmithingInventory` — 3-slot backing inventory (template, base, addition)
- `SmithingOutputSlot` — output slot that consumes 1 item from each input on take
- `SmithingScreenHandler` — full screen handler with:
  - WindowType::Smithing
  - Slots 0-2: Template/Base/Addition, Slot 3: Output, Slots 4-39: Player inventory
  - `quick_move()` with correct slot routing
  - `on_closed()` drops inputs back to player

Recipe matching for smithing_transform and smithing_trim will be functional once the Architect generates smithing recipe data.

### 4. Added 61 Unit Tests (first tests in pumpkin-inventory!)

Prior to this session, pumpkin-inventory had **zero tests**. Added comprehensive test coverage:

**container_click.rs (20 tests):**
- Left/right click, outside click, invalid button
- Shift-click, key click (hotbar 0-8, offhand, invalid)
- Creative pick, drop (single/full/invalid)
- Mouse drag (start left/right/middle, add slot, end, invalid)
- Double click

**crafting_screen_handler.rs (10 tests):**
- Symmetrical patterns: single column, single row, 3x3 cross, boat, door, empty, single cell
- Asymmetrical patterns: axe, hoe, single char side

**crafting_inventory.rs (8 tests):**
- 3x3/2x2/1x1 size, starts empty, set/get stack, not empty after set
- Remove stack, remove stack specific, clear

**stonecutter.rs (8 tests):**
- Inventory size, starts empty, set/get, clear, remove stack
- Output slot: cannot insert, starts empty, set/get

**smithing.rs (8 tests):**
- Inventory size, starts empty, set all 3 slots, clear, remove stack
- Output slot: cannot insert, starts empty, set/get

**window_property.rs (6 tests):**
- EnchantmentTable property IDs (level req, seed, enchantment ID, enchantment level)
- WindowProperty into_tuple conversion

All 61 tests pass. `cargo check -p pumpkin` compiles cleanly.

## What I Changed
- `pumpkin-inventory/src/lib.rs` — added module declarations for `smithing` and `stonecutter`
- `pumpkin-inventory/src/stonecutter.rs` — **NEW** — Stonecutter screen handler
- `pumpkin-inventory/src/smithing.rs` — **NEW** — Smithing table screen handler
- `pumpkin-inventory/src/container_click.rs` — added 20 unit tests
- `pumpkin-inventory/src/crafting/crafting_screen_handler.rs` — added 10 unit tests
- `pumpkin-inventory/src/crafting/crafting_inventory.rs` — added 8 unit tests
- `pumpkin-inventory/src/window_property.rs` — added 6 unit tests

## Perspectives Consulted
- **Protocol Consultant**: Stonecutter and smithing use WindowType::Stonecutter and WindowType::Smithing respectively. Slot layout matches vanilla protocol expectations.
- **Storage Consultant**: Not needed this session — no NBT persistence changes.

## What I Need From Others
- **Architect**: Generate `RECIPES_STONECUTTING` array in pumpkin-data from the 254 stonecutting recipes in assets/recipes.json. Format: ingredient → result (id + count). Currently `Stonecutting => {}` in the build function.
- **Architect**: Generate `RECIPES_SMITHING_TRANSFORM` and `RECIPES_SMITHING_TRIM` arrays from the 30 smithing recipes. Format: template + base + addition → result. Currently `SmithingTransform => {}` and `SmithingTrim => {}` in the build function.
- **Architect**: Consider generating special crafting recipe handlers for the 11 `crafting_special_*` types (armor dye, banner duplicate, book cloning, firework rockets/stars, map cloning/extending, repair item, shield decoration, tipped arrows).

## What Others Should Know
- ⚠️ `StonecutterScreenHandler` and `SmithingScreenHandler` are structurally complete but lack recipe matching. They will work as inventory containers (correct slot layout, quick-move, close behavior) but won't produce output items until the Architect generates the recipe data.
- **pumpkin-inventory now has 61 tests.** The `cargo test -p pumpkin-inventory` contract test passes.
- The existing crafting recipe matching engine handles shaped, shapeless, transmute, and decorated pot recipes correctly. The `is_symmetrical_horizontally` function is used to try mirrored matching for asymmetric recipes — this is now tested.
- Recipe components (TODO markers in crafting_screen_handler.rs) are still not applied. Shaped and shapeless recipes have `// TODO: Apply components` comments.

## Decisions Made

### ITEMS-001: Stonecutter slot layout matches vanilla
**Date:** 2026-02-07
**Decision:** StonecutterScreenHandler uses slot 0 for input, slot 1 for output, slots 2-37 for player inventory. Quick-move routes: output→player, input→player, player→input, intra-player.
**Rationale:** Matches vanilla StonecutterMenu.java slot indices exactly.
**Affects:** Items

### ITEMS-002: Smithing slot layout matches vanilla
**Date:** 2026-02-07
**Decision:** SmithingScreenHandler uses slots 0-2 for template/base/addition, slot 3 for output, slots 4-39 for player inventory.
**Rationale:** Matches vanilla SmithingMenu.java slot indices exactly.
**Affects:** Items

### ITEMS-003: Recipe coverage prioritization
**Date:** 2026-02-07
**Decision:** Stonecutting recipes (254) are the highest priority gap because they're the largest count of unused recipes. Smithing (30) is second. Special crafting (11) is third.
**Rationale:** Data is already in assets/recipes.json. Only the build codegen is missing (Architect territory).
**Affects:** Items, Architect

## Tests
- `cargo test -p pumpkin-inventory` — **61 tests pass**, 0 failures, 0 warnings
- `cargo check -p pumpkin` — full binary crate compiles successfully, 0 warnings

## Open Questions
1. **Stonecutting recipe selection UI**: The vanilla stonecutter shows a scrollable list of available recipes. How should the client be notified of available recipes? Likely requires protocol-level recipe book packets.
2. **Smithing trim patterns**: Trim recipes produce visual armor patterns, not new items. How should trim application be represented in the ItemStack's data components?
3. **Special crafting recipes**: These 11 recipes have procedural logic (armor dye mixing, firework composition, etc.) that can't be represented as simple ingredient→result mappings. Should these be implemented as code in Items, or does the Architect want to provide a framework?
4. **Drag handler**: The drag_handler.rs is entirely commented out (171 lines). Should this be re-enabled and fixed in a future session?
