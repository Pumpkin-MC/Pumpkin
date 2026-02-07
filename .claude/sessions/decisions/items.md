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
**Status:** active
