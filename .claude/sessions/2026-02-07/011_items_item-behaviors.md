# Session 011 — Items Agent: Item Behaviors
**Date:** 2026-02-07
**Agent:** Items
**Branch:** `claude/items-agent-setup-cgzPo`

## Preamble

Read session logs 010 (screen handlers), 009 (special recipes), 008 (recipe matching),
and decisions/items.md. No new requests from other agents targeting Items.

Previous session (010) completed 6 screen handlers (anvil, grindstone, enchanting,
brewing, loom, cartography) bringing test count to 136. P1 screen handlers are done.

This session focuses on P2: item behaviors (`pumpkin/src/item/items/`).

## What I Did

### 1. Implemented ShearsItem behavior (`pumpkin/src/item/items/shears.rs`)
- **Pumpkin carving**: Using shears on a pumpkin block converts it to a carved pumpkin
  facing the clicked face (or player direction for top/bottom clicks), drops 4 pumpkin
  seeds as an item entity, plays `BlockPumpkinCarve` sound
- **Beehive honey harvesting**: Using shears on a beehive/bee_nest with honey_level=5
  resets honey to 0 and drops 3 honeycombs as an item entity
- Damages shears by 1 in survival mode
- Uses `WallTorchLikeProperties` for carved pumpkin facing state
- Uses `BeeNestLikeProperties` for honey level tracking

### 2. Implemented BoatItem behavior (`pumpkin/src/item/items/boat.rs`)
- Maps all 20 boat/raft items to their entity types via `item_to_entity()`
- Uses `tag::Item::MINECRAFT_BOATS` tag for item ID registration (handles all variants)
- Spawns boat entity at the clicked block position
- Decrements item count in non-creative mode
- TODO: Bamboo raft entities mapped to oak_boat as placeholder (entity types not yet defined)

### 3. Implemented BoneMealItem behavior (`pumpkin/src/item/items/bone_meal.rs`)
- Checks `is_fertilizable()` for valid target blocks (crops, grass, moss, vines, etc.)
- For crops, advances growth stage by 3 (simplified from vanilla random 2-5)
- Handles different max ages per crop type (wheat/carrots/potatoes=7, beetroots=3, torchflower=2)
- Emits `WorldEvent::BoneMealUsed` particles
- Decrements item count in non-creative mode

### 4. Registered all 3 behaviors in `items/mod.rs`
- Added module declarations and use statements
- Added `manager.register()` calls in `default_registry()`

### 5. Fixed compilation errors (7 total)
- `Block` type missing in boat.rs → added to imports
- `GameMode` unused in bone_meal.rs/boat.rs → removed (type inferred)
- `WorldEvent::BoneMealUse` → `WorldEvent::BoneMealUsed`
- `block.properties` (field access) → `WallTorchLikeProperties` property struct
- `Integer0To5::_5`/`_0` → `Integer0To5::L5`/`L0`

## Test Results

- **136 tests pass** (`cargo test -p pumpkin-inventory`)
- Full workspace tests pass
- Clean compilation with `cargo check -p pumpkin`
- Pre-existing clippy failures in `pumpkin-protocol` (not Items scope)

## Files Modified

| File | Change |
|------|--------|
| `pumpkin/src/item/items/shears.rs` | New: ShearsItem behavior |
| `pumpkin/src/item/items/boat.rs` | New: BoatItem behavior |
| `pumpkin/src/item/items/bone_meal.rs` | New: BoneMealItem behavior |
| `pumpkin/src/item/items/mod.rs` | Added 3 modules + registrations |

## What Others Should Know

- **Entity agent**: BoatItem spawns boat entities but bamboo raft entity types don't exist
  yet. Currently mapped to oak_boat/oak_chest_boat as placeholders.
- **Block agent**: BoneMealItem uses simplified growth (always +3 stages). Vanilla has
  per-crop random growth with different ranges. Full behavior needs block entity tick support.
- `WallTorchLikeProperties` handles carved pumpkin facing (same as wall torches, buttons, etc.)
- `BeeNestLikeProperties` handles beehive honey levels (shared between bee_nest and beehive)

## Decisions Made

None new — these follow established patterns from existing item behaviors.

## What I Need From Others

- **Entity**: Bamboo raft entity types (BAMBOO_RAFT, BAMBOO_CHEST_RAFT)
- **Block**: Full crop growth logic integration with block entity ticks
