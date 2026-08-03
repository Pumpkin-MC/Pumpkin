# GameEvent / vibration engine

Source: `net.minecraft.world.level.gameevent.{GameEvent,GameEventDispatcher,GameEventListener,
PositionSource,BlockPositionSource,EntityPositionSource}`,
`net.minecraft.world.level.gameevent.vibrations.VibrationSystem`, decompiled 26.2 mappings.

## What exists (`pumpkin/src/world/game_event/{mod.rs,vibration.rs}`)

Was **zero** at the start of this session (only the unrelated client-protocol `CGameEvent`
packet enum existed — a different thing entirely, don't confuse the two).

- `GameEventContext` (mirrors `GameEvent.Context` — but note it currently has only one field,
  `source_entity: Option<Arc<dyn EntityBase>>`; vanilla's `Context` can also carry an affected
  `BlockState`, which Pumpkin's version can't represent yet — see "known limitation" below).
- `PositionSource` enum (`Block`/`Entity` variants, mirrors `PositionSource`/
  `BlockPositionSource`/`EntityPositionSource`).
- `GameEventListener` trait: `listener_source`, `listener_radius`, async `handle_game_event`.
- Notification-radius / vibration-frequency lookup tables, cited against `GameEvent.java`'s
  bootstrap and `VibrationSystem.VIBRATION_FREQUENCY_FOR_EVENT`.
- `redstone_strength_for_distance`, ported from `VibrationSystem.getRedstoneStrengthForDistance`
  (used by sculk sensors).
- `VibrationSelector` — faithful port of `VibrationSelector.java`'s closest/highest-frequency-wins
  logic (7 unit tests).
- `emit_game_event` — ported from `GameEventDispatcher.post`: radius-filter, closest-first sort,
  occlusion gate, dispatch to registered listeners.
- `World::register_game_event_listener` / `unregister_game_event_listener_at` /
  `unregister_game_event_listener_for_entity` (the last one added when Warden/Allay landed, since
  entity-backed listeners had no removal path — a despawned mob's listener would otherwise leak
  in the registry forever).

## Emission call sites wired so far (small list — most of vanilla's are still missing)

- `GameEvent::BlockDestroy` — from `break_block()`, right after `set_block_state`
  (`Level.destroyBlock`, line 298).
- `GameEvent::BlockPlace` — from `block::registry::place_block`, right after `set_block_state`
  succeeds (`BlockItem.place`, line 88).
- `GameEvent::NoteBlockPlay` — from `note.rs`'s `play_note`, both call sites.
- `GameEvent::JukeboxPlay` (every 20 ticks while playing) / `JukeboxStopPlay` (natural end and
  manual stop) — from `block/entities/jukebox.rs` + `block/blocks/jukebox.rs`.
- `GameEvent::Eat` / `GameEvent::Drink` for the **item-consumable path only** — from
  `LivingEntity::tick`'s item-in-use completion block (`pumpkin/src/entity/living.rs`), right
  after the consumable's sound plays. This is the same shared completion point that already
  applies `FoodImpl` hunger, potion effects, and `consumable_clears_all_effects` — it was NOT a
  missing infrastructure gap, just a missing call (Pumpkin has no instant-consume branch
  separate from this tick-driven path, so this is the only completion site that needs it).
  Matches `Consumable.onConsume` (`Consumable.java:90`): `user.gameEvent(this.animation ==
  ItemUseAnimation.DRINK ? GameEvent.DRINK : GameEvent.EAT)`, fired unconditionally for any item
  with a `Consumable` data component via `Item.finishUsingItem` (`Item.java:216`) — gated on
  `ConsumableImpl::animation == ConsumeAnimation::Drink` vs. anything else (not on whether the
  item happens to carry `FoodImpl`/`PotionContentsImpl`).
  **Still missing**: several other vanilla EAT/DRINK sites unrelated to the item-consumable
  flow — `CakeBlock.java:102` (`level.gameEvent(player, GameEvent.EAT, pos)`, block-based, not
  through `Consumable` at all), mob eating (`Mob.java:265`, `Panda.java:409`,
  `AbstractHorse.java:489`, `Camel.java:466`), and `Witch.java:130`'s `GameEvent.DRINK`. These
  are separate, out-of-scope units of work — file them individually rather than assuming
  this entry covers them.

## Still missing (most of vanilla's emission call sites) — good units of work, pick any

Footsteps, container open/close, entity place/kill/damage, projectile shoot/land,
cake/mob eating and witch drinking (see note above — distinct from the item-consumable EAT/DRINK
path, which is now wired), splash, sculk-related events beyond the sensor itself, and many more.
Grep vanilla's
`GameEvent.java` bootstrap for the full enumerated list (dozens of named events); cross-reference
each against where the corresponding action already happens in Pumpkin (e.g. container-open
almost certainly has an existing hook point in the screen-handler open path — find it rather than
inventing a new one).

## Related architectural gap found this session: no inventory-to-player callback hook

Two known bugs turned out to be the same underlying structural gap, not independent fixes:

- `ArmorSlot::set_stack_prev` (`pumpkin-inventory/src/slot.rs`) has a `// TODO:
  this.entity.onEquipStack(...)` stub — vanilla's `LivingEntity.onEquipItem`
  (`LivingEntity.java:689-711`) plays an equip sound (from the stack's `Equippable` data
  component) and emits `GameEvent.EQUIP`/`GameEvent.UNEQUIP`, gated on the item actually
  changing (`!isSameItemSameComponents`) and not firing on the entity's first tick.
- `ResultSlot` (`pumpkin-inventory/src/crafting/crafting_screen_handler.rs`) never triggers
  recipe-book unlock tracking on craft.

Both need to call back into per-player/per-entity state (unlocked recipes, or `world.play_sound`
+ `emit_game_event`) that only exists on `Player`/`LivingEntity` in the `pumpkin` crate — but
`ArmorSlot`/`ResultSlot` live in the lower-level `pumpkin-inventory` crate, which `pumpkin`
depends on, not the reverse. `ArmorSlot`'s only fields are `inventory: Arc<dyn Inventory>` (the
`PlayerInventory`, which itself has no back-reference to the owning `Player`) plus bookkeeping —
there is no path from slot code up to the entity that owns it. Compounding this, `ArmorSlot` is
constructed inside `Player::new` (`pumpkin/src/entity/player.rs:649`, via
`PlayerScreenHandler::new`) *before* the `Arc<Player>` itself exists, so even a naive
"pass an `Arc<Player>` at construction time" fix doesn't work without restructuring `Player`
construction (e.g. `Arc::new_cyclic`, or a two-phase init that sets a back-reference after the
`Arc` exists).

**Correct fix shape** (not attempted this session — this is its own scoped task): add a generic
trait in `pumpkin-inventory` (something like `EquipListener`/`SlotOwnerCallback`) that
`ArmorSlot`/`ResultSlot` hold an `Option<Arc<dyn ...>>` of, with methods like
`on_equip(slot, old, new)` / `on_crafted(recipe, output)`. `Player` (and any other `InventoryPlayer`
impl) implements it in the `pumpkin` crate and gets wired in after construction, not at
`ArmorSlot::new` call time. This is the same shape of problem `Mob`'s existing `ContainerUser`-less
gap already documented for Copper Golem — a recurring pattern of "inventory-adjacent code needs
to call back into entity/world state it structurally can't see."

## Known architectural limitations (don't try to "fix" these without a bigger redesign)

- **Flat `Vec` listener registry, not per-chunk-section sharded.** Vanilla shards listeners by
  chunk section for lookup performance at scale. Pumpkin's registry is a flat list scanned
  linearly. Fine for correctness, not for performance at high listener counts — only worth fixing
  if profiling shows it matters.
- **Straight-line block-sampling for occlusion, not a full 6-direction raycast.** Vanilla's real
  occlusion check is more geometrically precise. Pumpkin's approximation will have false
  positives/negatives at the margins of complex block shapes.
- **`VibrationSelector` is tested but not tick-driven yet.** Vanilla's `VibrationSystem.Ticker`
  simulates travel time and spawns travel particles over several ticks before the vibration
  actually resolves at its destination. Pumpkin currently resolves synchronously — a vibration is
  processed instantly rather than "traveling." This is a real, user-visible gap (no vibration
  particle trail) but is its own scoped feature, not a quick fix.
- **`GameEventContext` has no block-state field.** Some vanilla listeners key behavior off the
  block state involved in the event (e.g. `BlockPlace`'s context carries `placedState`).
  Pumpkin's context can't carry this yet — if you need it for a new listener, you'll need to
  extend `GameEventContext`, not work around its absence with a hack.

## Warden and Allay (landed on top of this engine)

- **Warden** (`pumpkin/src/entity/mob/{warden.rs,warden_anger.rs}`): full `AngerManagement`/
  `AngerLevel` port (thresholds 0/40/80, `MAX_ANGER`=150, `DEFAULT_ANGER`=35,
  `ON_HURT_ANGER_BOOST`=20 — all vanilla constants, 13 unit tests). A `GameEventListener` impl
  gated by the real `GameEvent::MINECRAFT_WARDEN_CAN_LISTEN` tag. Deferred: Pose-driven
  emerge/dig/roar animations (Pumpkin's `Entity` has no `Pose` enum at all — this needs a new
  concept, not just wiring), `doPush` touch-anger (no entity-collision hook exists), anger
  persistence across unload (needs entity-NBT custom-data, a `Codec`-equivalent Pumpkin doesn't
  have yet).
- **Allay** (`pumpkin/src/entity/passive/allay.rs`): jukebox/note-block "liked position" memory
  and item duplication, matching `Allay.mobInteract` exactly. Deferred: client-synced
  dancing/animation state (no tracked-data/animation channel exists), the item-carrying/delivery
  goal AI (`GoToWantedItem`/`GoAndGiveItemsToTarget`/`StayCloseToTarget` — no item-seeking or
  deliver-to-position goal exists in Pumpkin's Goal system at all yet).
