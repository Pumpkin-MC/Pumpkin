# Items — Decisions

## ITEMS-001: Stonecutter slot layout matches vanilla
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/002_items_recipe-audit-and-screen-handlers.md
**Decision:** StonecutterScreenHandler uses slot 0 for input, slot 1 for output, slots 2-37 for player inventory. Quick-move routes: output→player, input→player, player→input, intra-player.
**Rationale:** Matches vanilla StonecutterMenu.java slot indices exactly.
**Affects:** Items
**Status:** active

## ITEMS-002: Smithing slot layout matches vanilla
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/002_items_recipe-audit-and-screen-handlers.md
**Decision:** SmithingScreenHandler uses slots 0-2 for template/base/addition, slot 3 for output, slots 4-39 for player inventory.
**Rationale:** Matches vanilla SmithingMenu.java slot indices exactly.
**Affects:** Items
**Status:** active

## ITEMS-003: Recipe coverage prioritization
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/002_items_recipe-audit-and-screen-handlers.md
**Decision:** Stonecutting recipes (254) are the highest priority gap because they're the largest count of unused recipes. Smithing (30) is second. Special crafting (11) is third.
**Rationale:** Data is already in assets/recipes.json. Only the build codegen is missing (Architect territory).
**Affects:** Items, Architect
**Status:** resolved — stonecutting and smithing recipe matching wired in session 008

## ITEMS-004: Stonecutting recipe matching uses static filtering
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/008_items_recipe-matching.md
**Decision:** `get_stonecutting_recipes_for()` iterates `RECIPES_STONECUTTING` and filters by `ingredient.match_item()`. Results are cached in the handler's `available_recipes` field and refreshed on input change.
**Rationale:** Simple, correct, and fast enough for 254 recipes. No index structure needed.
**Affects:** Items
**Status:** active

## ITEMS-005: Smithing transform takes priority over trim
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/008_items_recipe-matching.md
**Decision:** `find_smithing_recipe()` checks transform recipes before trim recipes. If both match, transform wins.
**Rationale:** Transform produces a different item (diamond→netherite), which is the more significant operation. Trims are cosmetic.
**Affects:** Items
**Status:** active

## ITEMS-006: Trim recipes produce base item copy (pending component system)
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/008_items_recipe-matching.md
**Decision:** When a smithing trim recipe matches, the output is a copy of the base item. The trim pattern is stored in `SmithingTrimRecipe::pattern` but not yet applied as a data component.
**Rationale:** The data component system for armor trims isn't available yet. This produces a functionally correct output while the visual trim is deferred.
**Affects:** Items
**Status:** active

## ITEMS-007: Special recipes implemented as procedural code in pumpkin-inventory
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/009_items_special-recipes-and-blocking-assessment.md
**Decision:** All 11 `crafting_special_*` recipes live in `pumpkin-inventory/src/crafting/special_recipes.rs` as procedural Rust functions. They are NOT generated from data. Each recipe type has its own matching function.
**Rationale:** Special recipes can't be represented as static data — their results depend on input item components (dye colors, potion effects, book contents). Procedural code is the correct approach.
**Affects:** Items
**Status:** active

## ITEMS-008: Special recipes checked after RECIPES_CRAFTING as fallback
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/009_items_special-recipes-and-blocking-assessment.md
**Decision:** `try_special_recipe()` is called in `ResultSlot::refill_output()` only when the normal `match_recipe()` returns None. Special recipes never override data-driven recipes.
**Rationale:** Data-driven recipes are authoritative. Special recipes are fallback for the 11 types that have no data representation. This ordering prevents conflicts.
**Affects:** Items
**Status:** active
