# Bukkit API Reference for Pumpkin Plugin Agent

> Source: Spigot-API 1.21.11-R0.2-SNAPSHOT Javadoc
> Generated: 2026-02-07
> Purpose: Structured reference for implementing Bukkit-compatible plugin API in Pumpkin

---

## Table of Contents

1. [Event System Architecture](#1-event-system-architecture)
2. [Complete Event Listing by Package](#2-complete-event-listing-by-package)
3. [Plugin Lifecycle](#3-plugin-lifecycle)
4. [Scheduler API](#4-scheduler-api)
5. [Command System](#5-command-system)
6. [Configuration API](#6-configuration-api)
7. [Pumpkin Current Implementation](#7-pumpkin-current-implementation)
8. [Bukkit-to-Pumpkin Mapping (Top 50)](#8-bukkit-to-pumpkin-mapping-top-50)
9. [Sprint Priorities](#9-sprint-priorities)

---

## 1. Event System Architecture

### Bukkit Base Event Class

```java
public abstract class Event {
    Event()                    // Synchronous by default
    Event(boolean isAsync)     // Explicitly async
    String getEventName()      // Returns class simple name
    abstract HandlerList getHandlers()
    final boolean isAsynchronous()
}
```

**Key patterns:**
- Every event must have a `static HandlerList getHandlerList()` method
- Events are fired via `PluginManager.callEvent(Event)`
- Listeners registered via `PluginManager.registerEvents(Listener, Plugin)`
- Async events: never fired from sync context; handlers may block; not tracked in timing system

### Bukkit Event Priority (execution order)

| Priority | Order | Purpose |
|----------|-------|---------|
| LOWEST   | 1st   | First to run, can be overridden by all others |
| LOW      | 2nd   | Early processing |
| NORMAL   | 3rd   | Default for most handlers |
| HIGH     | 4th   | Late processing |
| HIGHEST  | 5th   | Last modifying handler, overrides all below |
| MONITOR  | 6th   | Read-only observation, MUST NOT modify the event |

### Pumpkin Event Priority (current implementation)

| Priority | Order | Notes |
|----------|-------|-------|
| Highest  | 1st   | Executed first (can override all) |
| High     | 2nd   | |
| Normal   | 3rd   | Default |
| Low      | 4th   | |
| Lowest   | 5th   | Last modifying handler |
| Monitor  | 6th   | Must not modify (matches Bukkit) |

**NOTE:** Pumpkin's ordering is inverted from Bukkit's naming convention. In Bukkit, LOWEST runs first (lowest priority = least important). In Pumpkin, `Highest` runs first. The net effect is the same: `Highest`/`LOWEST` runs first, `Monitor` runs last.

### Cancellable Interface

Bukkit:
```java
public interface Cancellable {
    boolean isCancelled();
    void setCancelled(boolean cancel);
}
```

Pumpkin equivalent:
```rust
pub trait Cancellable: Send + Sync {
    fn cancelled(&self) -> bool;
    fn set_cancelled(&mut self, cancelled: bool);
}
```

### Pumpkin Payload Trait (= Bukkit Event base)

```rust
pub trait Payload: Send + Sync {
    fn get_name_static() -> &'static str where Self: Sized;
    fn get_name(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn is_cancelled(&self) -> bool { false }  // default for non-cancellable
}
```

- `#[derive(Event)]` macro (in `pumpkin-macros`) generates `Payload` impl
- `#[cancellable]` macro adds `cancelled: bool` field + `Cancellable` impl + overrides `is_cancelled()`
- Downcasting uses name-based matching (safe across compilation boundaries for plugin compat)

---

## 2. Complete Event Listing by Package

### 2.1 block (50 events)

**Priority implement (from Bukkit docs):** BlockBreakEvent, BlockPlaceEvent, BlockPhysicsEvent, BlockRedstoneEvent, BlockPistonExtendEvent, BlockPistonRetractEvent, BlockFromToEvent, SignChangeEvent, BlockExplodeEvent, BlockGrowEvent

| Event | Description | Cancellable |
|-------|-------------|-------------|
| BellResonateEvent | Bell resonated after ringing, highlights raiders | No |
| BellRingEvent | Bell is being rung | Yes |
| BlockBreakEvent | Block broken by player | Yes |
| BlockBrushEvent | Block brushed by player | Yes |
| BlockBurnEvent | Block destroyed by fire | Yes |
| BlockCanBuildEvent | Check if block can be placed | Yes |
| BlockCookEvent | ItemStack cooked in a block | Yes |
| BlockDamageAbortEvent | Player stops damaging a block | No |
| BlockDamageEvent | Block damaged by player | Yes |
| BlockDispenseArmorEvent | Equippable item dispensed onto entity | Yes |
| BlockDispenseEvent | Item dispensed from block | Yes |
| BlockDispenseLootEvent | Block dispenses loot from LootTable | Yes |
| BlockDropItemEvent | Block broken by player drops item | Yes |
| BlockExpEvent | Block yields experience | No |
| BlockExplodeEvent | Block explodes | Yes |
| BlockFadeEvent | Block fades/melts/disappears | Yes |
| BlockFertilizeEvent | Player fertilizes block with bonemeal | Yes |
| BlockFormEvent | Block forms based on world conditions | Yes |
| BlockFromToEvent | Liquid flow / dragon egg teleport | Yes |
| BlockGrowEvent | Block grows naturally | Yes |
| BlockIgniteEvent | Block ignited (IgniteCause enum) | Yes |
| BlockMultiPlaceEvent | Single placement creates multiple blocks | Yes |
| BlockPhysicsEvent | Block physics check | Yes |
| BlockPistonExtendEvent | Piston extends | Yes |
| BlockPistonRetractEvent | Piston retracts | Yes |
| BlockPlaceEvent | Block placed by player | Yes |
| BlockReceiveGameEvent | Sculk sensor receives game event | Yes |
| BlockRedstoneEvent | Redstone current changes | No |
| BlockShearEntityEvent | Dispenser shears nearby sheep | Yes |
| BlockSpreadEvent | Block spreads (fire, mushroom, etc.) | Yes |
| BrewingStartEvent | Brewing stand starts brewing | No |
| CampfireStartEvent | Campfire starts cooking | No |
| CauldronLevelChangeEvent | Cauldron level changes (ChangeReason enum) | Yes |
| CrafterCraftEvent | Crafter about to craft | Yes |
| EntityBlockFormEvent | Block formed by entities (e.g. snowman snow) | Yes |
| FluidLevelChangeEvent | Fluid level changes due to adjacent blocks | Yes |
| InventoryBlockStartEvent | Furnace/brewing/campfire starts processing | No |
| LeavesDecayEvent | Leaves decaying naturally | Yes |
| MoistureChangeEvent | Soil moisture level changes | Yes |
| NotePlayEvent | Note block played | Yes |
| SculkBloomEvent | New sculk cursor from SculkCatalyst | Yes |
| SignChangeEvent | Sign changed by player | Yes |
| SpongeAbsorbEvent | Sponge absorbs water | Yes |
| TNTPrimeEvent | TNT block becomes primed (PrimeCause enum) | Yes |
| VaultDisplayItemEvent | Vault in trial chamber displays item | Yes |

### 2.2 enchantment (2 events)

| Event | Description | Cancellable |
|-------|-------------|-------------|
| EnchantItemEvent | Item enchanted at enchanting table | Yes |
| PrepareItemEnchantEvent | Item placed in enchanting table | Yes |

### 2.3 entity (92 events)

**Priority implement:** EntityDamageEvent, EntityDamageByEntityEvent, EntityDeathEvent, CreatureSpawnEvent, EntityExplodeEvent, ProjectileHitEvent, EntityTargetEvent, EntityTeleportEvent, FoodLevelChangeEvent

| Event | Description | Key Fields |
|-------|-------------|------------|
| AreaEffectCloudApplyEvent | Lingering potion applies effects | entity, affectedEntities |
| ArrowBodyCountChangeEvent | Arrow enters/exits entity body | entity, oldAmount, newAmount |
| BatToggleSleepEvent | Bat sleeps/wakes | entity, isAwake |
| CreatureSpawnEvent | Creature spawned (SpawnReason enum) | entity, spawnReason |
| CreeperPowerEvent | Creeper struck by lightning (PowerCause) | entity, cause |
| EnderDragonChangePhaseEvent | Dragon switches phase | entity, newPhase |
| EntityAirChangeEvent | Entity air remaining changes | entity, amount |
| EntityBreakDoorEvent | Entity breaks door | entity, block |
| EntityBreedEvent | Entity breeds with another | entity, mother, father |
| EntityChangeBlockEvent | Entity changes a block | entity, block, to |
| EntityCombustByBlockEvent | Block causes entity to combust | entity, combuster |
| EntityCombustByEntityEvent | Entity causes another to combust | entity, combuster |
| EntityCombustEvent | Entity combusts | entity, duration |
| EntityCreatePortalEvent | (Deprecated: use PortalCreateEvent) | |
| EntityDamageByBlockEvent | Entity damaged by block | entity, damager, cause, damage |
| EntityDamageByEntityEvent | Entity damaged by entity | entity, damager, cause, damage |
| EntityDamageEvent | Entity takes damage (DamageCause enum) | entity, cause, damage |
| EntityDeathEvent | LivingEntity dies | entity, drops, droppedExp |
| EntityDismountEvent | Entity stops riding | entity, dismounted |
| EntityDropItemEvent | Entity creates item drop | entity, itemDrop |
| EntityEnterBlockEvent | Entity enters and is stored in block | entity, block |
| EntityEnterLoveModeEvent | Entity enters love mode | entity, humanEntity |
| EntityExhaustionEvent | Human exhaustion (ExhaustionReason) | entity, reason, exhaustion |
| EntityExplodeEvent | Entity explodes | entity, blockList, yield |
| EntityInteractEvent | Entity interacts with object | entity, block |
| EntityKnockbackByEntityEvent | Entity knocked back by entity | entity, sourceEntity |
| EntityKnockbackEvent | Entity receives knockback (KnockbackCause) | entity, cause, knockback |
| EntityMountEvent | Entity attempts to ride another | entity, mount |
| EntityPickupItemEvent | Entity picks up item | entity, item, remaining |
| EntityPlaceEvent | Entity placed by player | entity, player, block |
| EntityPortalEnterEvent | Entity contacts portal | entity, location |
| EntityPortalEvent | Non-player entity teleports via portal | entity, from, to |
| EntityPortalExitEvent | Entity exits portal | entity, from, to |
| EntityPoseChangeEvent | Entity changes pose | entity, pose |
| EntityPotionEffectEvent | Potion effect modified (Action, Cause enums) | entity, action, cause |
| EntityRegainHealthEvent | Entity regains health (RegainReason) | entity, amount, reason |
| EntityRemoveEvent | Entity removed (Cause enum) | entity, cause |
| EntityResurrectEvent | Entity may be resurrected (totem) | entity |
| EntityShootBowEvent | LivingEntity shoots bow | entity, bow, projectile |
| EntitySpawnEvent | Entity spawned into world | entity, location |
| EntitySpellCastEvent | Spellcaster casts spell | entity, spell |
| EntityTameEvent | LivingEntity tamed | entity, owner |
| EntityTargetBlockEvent | Creature targets/untargets block | entity, target |
| EntityTargetEvent | Creature targets/untargets entity (TargetReason) | entity, target, reason |
| EntityTargetLivingEntityEvent | Entity targets LivingEntity | entity, target |
| EntityTeleportEvent | Non-player entity teleports | entity, from, to |
| EntityToggleGlideEvent | Entity gliding toggled (elytra) | entity, isGliding |
| EntityToggleSwimEvent | Entity swimming toggled | entity, isSwimming |
| EntityTransformEvent | Entity replaced by another (TransformReason) | entity, transformedEntities |
| EntityUnleashEvent | Entity unleashed (UnleashReason) | entity, reason |
| ExpBottleEvent | ThrownExpBottle hits and releases exp | entity, experience |
| ExplosionPrimeEvent | Entity decides to explode | entity, radius, fire |
| FireworkExplodeEvent | Firework explodes | entity |
| FoodLevelChangeEvent | Human food level changes | entity, foodLevel, item |
| HorseJumpEvent | Horse jumps | entity, power |
| ItemDespawnEvent | Item removed after 5 minutes | entity, location |
| ItemMergeEvent | Two item entities merge | entity, target |
| ItemSpawnEvent | Item spawned into world | entity, location |
| LingeringPotionSplashEvent | Splash potion hits area (lingering) | entity, areaEffectCloud |
| PigZapEvent | Pig zapped by lightning | entity, lightning, pigZombie |
| PigZombieAngerEvent | Pig Zombie angered | entity, newAnger, target |
| PiglinBarterEvent | Piglin barter interaction | entity, input, outcome |
| PlayerDeathEvent | Player dies | entity, deathMessage, keepLevel, keepInventory |
| PlayerLeashEntityEvent | Player leashes creature | entity, player, leashHolder |
| PotionSplashEvent | Splash potion hits area | entity, affectedEntities |
| ProjectileHitEvent | Projectile hits object | entity, hitBlock, hitBlockFace, hitEntity |
| ProjectileLaunchEvent | Projectile launched | entity |
| SheepDyeWoolEvent | Sheep wool dyed | entity, color |
| SheepRegrowWoolEvent | Sheep regrows wool | entity |
| SlimeSplitEvent | Slime splits on death | entity, count |
| SpawnerSpawnEvent | Entity spawned by spawner | entity, spawner |
| StriderTemperatureChangeEvent | Strider temperature changes | entity |
| TrialSpawnerSpawnEvent | Entity spawned by trial spawner | entity, trialSpawner |
| VillagerAcquireTradeEvent | Villager acquires new trade | entity, recipe |
| VillagerCareerChangeEvent | Villager career changes (ChangeReason) | entity, profession, reason |
| VillagerReplenishTradeEvent | Villager restocks trade | entity, recipe |
| VillagerReputationChangeEvent | Entity reputation with villager changes | entity, reputation |

### 2.4 hanging (5 events)

| Event | Description |
|-------|-------------|
| HangingBreakByEntityEvent | Hanging entity broken by entity |
| HangingBreakEvent | Hanging entity breaks |
| HangingEvent | Base hanging entity event |
| HangingPlaceEvent | Hanging entity placed |

### 2.5 inventory (25 events)

**Priority implement:** InventoryClickEvent, InventoryOpenEvent, InventoryCloseEvent, CraftItemEvent, FurnaceSmeltEvent, InventoryMoveItemEvent

| Event | Description | Cancellable |
|-------|-------------|-------------|
| BrewEvent | Brewing complete | Yes |
| BrewingStandFuelEvent | ItemStack about to fuel brewing stand | Yes |
| CraftItemEvent | Recipe completed in crafting matrix | Yes |
| FurnaceBurnEvent | ItemStack burned as fuel | Yes |
| FurnaceExtractEvent | Player takes items from furnace | No |
| FurnaceSmeltEvent | ItemStack smelted | Yes |
| FurnaceStartSmeltEvent | Furnace starts smelting | No |
| HopperInventorySearchEvent | Hopper searches for containers | No |
| InventoryClickEvent | Player clicks in inventory | Yes |
| InventoryCloseEvent | Player closes inventory | No |
| InventoryCreativeEvent | Creative mode inventory manipulation | Yes |
| InventoryDragEvent | Player drags item across inventory | Yes |
| InventoryInteractEvent | Base interaction event | Yes |
| InventoryMoveItemEvent | Block/entity moves item between inventories | Yes |
| InventoryOpenEvent | Player opens inventory | Yes |
| InventoryPickupItemEvent | Hopper picks up dropped item | Yes |
| PrepareAnvilEvent | Item placed in anvil | No |
| PrepareGrindstoneEvent | Item placed in grindstone | No |
| PrepareInventoryResultEvent | Result calculated for preparation | No |
| PrepareItemCraftEvent | Crafting result preview | No |
| PrepareSmithingEvent | Item placed in smithing table | No |
| SmithItemEvent | Smithing table recipe completed | Yes |
| TradeSelectEvent | Player clicks new trade in sidebar | No |

### 2.6 player (85 events)

**Priority implement:** PlayerJoinEvent, PlayerQuitEvent, PlayerMoveEvent, PlayerInteractEvent, PlayerChatEvent/AsyncPlayerChatEvent, PlayerCommandPreprocessEvent, PlayerLoginEvent, PlayerTeleportEvent, PlayerDeathEvent, PlayerRespawnEvent, PlayerDropItemEvent, PlayerItemConsumeEvent

| Event | Description | Cancellable |
|-------|-------------|-------------|
| AsyncPlayerChatEvent | Player chat (may fire async) | Yes |
| AsyncPlayerChatPreviewEvent | (Deprecated) | Yes |
| AsyncPlayerPreLoginEvent | Pre-login async (Result enum) | Yes (via disallow) |
| PlayerAdvancementDoneEvent | Player completes advancement | No |
| PlayerAnimationEvent | Player animation | Yes |
| PlayerArmorStandManipulateEvent | Player interacts with armor stand | Yes |
| PlayerBedEnterEvent | Player entering bed (BedEnterResult) | Yes |
| PlayerBedLeaveEvent | Player leaving bed | No |
| PlayerBucketEmptyEvent | Player empties bucket | Yes |
| PlayerBucketEntityEvent | Player captures entity in bucket | Yes |
| PlayerBucketFillEvent | Player fills bucket | Yes |
| PlayerChangedMainHandEvent | Player changes main hand setting | No |
| PlayerChangedWorldEvent | Player switches world | No |
| PlayerChannelEvent | Player registers/unregisters plugin channel | No |
| PlayerChatEvent | (Deprecated: use AsyncPlayerChatEvent) | Yes |
| PlayerChatTabCompleteEvent | (Deprecated: no longer fired) | No |
| PlayerCommandPreprocessEvent | Player runs command (slash prefix) | Yes |
| PlayerCommandSendEvent | Available commands sent to player | No |
| PlayerCustomClickEvent | Player runs custom action from chat/form | No |
| PlayerDropItemEvent | Player drops item from inventory | Yes |
| PlayerEditBookEvent | Player edits/signs book | Yes |
| PlayerEggThrowEvent | Player throws egg | No |
| PlayerExpChangeEvent | Player experience changes | No |
| PlayerExpCooldownChangeEvent | Player exp cooldown changes | No |
| PlayerFishEvent | Player fishing (State enum) | Yes |
| PlayerGameModeChangeEvent | Player gamemode changes | Yes |
| PlayerHarvestBlockEvent | Player harvests block | Yes |
| PlayerHideEntityEvent | Visible entity hidden from player | No |
| PlayerInputEvent | Player sends updated input | No |
| PlayerInteractAtEntityEvent | Player right-clicks entity at location | Yes |
| PlayerInteractEntityEvent | Player right-clicks entity | Yes |
| PlayerInteractEvent | Player interacts with object/air | Yes |
| PlayerItemBreakEvent | Player item breaks (durability) | No |
| PlayerItemConsumeEvent | Player consumes item (food/potion/milk) | Yes |
| PlayerItemDamageEvent | Player item takes durability damage | Yes |
| PlayerItemHeldEvent | Player changes held item | Yes |
| PlayerItemMendEvent | Item repaired via Mending | Yes |
| PlayerJoinEvent | Player joins server | No |
| PlayerKickEvent | Player kicked from server | Yes |
| PlayerLevelChangeEvent | Player level changes | No |
| PlayerLinksSendEvent | Links list sent to player | No |
| PlayerLocaleChangeEvent | Player changes locale setting | No |
| PlayerLoginEvent | Player login (Result enum) | Yes (via disallow) |
| PlayerMoveEvent | Player movement | Yes |
| PlayerPickupArrowEvent | Player picks up arrow | Yes |
| PlayerPickupItemEvent | (Deprecated: use EntityPickupItemEvent) | Yes |
| PlayerPortalEvent | Player teleports via portal | Yes |
| PlayerPreLoginEvent | (Deprecated: use AsyncPlayerPreLoginEvent) | Yes |
| PlayerQuitEvent | Player leaves server | No |
| PlayerRecipeBookClickEvent | Player clicks recipe in recipe book | Yes |
| PlayerRecipeBookSettingsChangeEvent | Player changes recipe book settings | No |
| PlayerRecipeDiscoverEvent | Player discovers recipe | Yes |
| PlayerRegisterChannelEvent | Player registers plugin channel | No |
| PlayerResourcePackStatusEvent | Player acts on resource pack (Status enum) | No |
| PlayerRespawnEvent | Player respawns (RespawnReason enum) | No |
| PlayerRiptideEvent | Player activates riptide enchantment | No |
| PlayerShearEntityEvent | Player shears entity | Yes |
| PlayerShowEntityEvent | Hidden entity shown to player | No |
| PlayerSignOpenEvent | Player opens sign (Cause enum) | Yes |
| PlayerSpawnChangeEvent | Player spawn point changes (Cause enum) | Yes |
| PlayerStatisticIncrementEvent | Player statistic incremented | Yes |
| PlayerSwapHandItemsEvent | Player swaps items between hands | Yes |
| PlayerTakeLecternBookEvent | Player takes book from lectern | Yes |
| PlayerTeleportEvent | Player teleports (TeleportCause enum) | Yes |
| PlayerToggleFlightEvent | Player toggles flying | Yes |
| PlayerToggleSneakEvent | Player toggles sneaking | Yes |
| PlayerToggleSprintEvent | Player toggles sprinting | Yes |
| PlayerUnleashEntityEvent | Player unleashes entity | Yes |
| PlayerUnregisterChannelEvent | Player unregisters plugin channel | No |
| PlayerVelocityEvent | Player velocity changes | Yes |

### 2.7 raid (6 events)

| Event | Description |
|-------|-------------|
| RaidEvent | Base raid event |
| RaidFinishEvent | Raid finishes |
| RaidSpawnWaveEvent | Raid wave spawns |
| RaidStopEvent | Raid stops |
| RaidTriggerEvent | Raid triggered |

### 2.8 server (15 events)

**Priority implement:** PluginEnableEvent, PluginDisableEvent, ServerLoadEvent

| Event | Description | Cancellable |
|-------|-------------|-------------|
| BroadcastMessageEvent | Message broadcast to server | Yes |
| MapInitializeEvent | Map initialized | No |
| PluginDisableEvent | Plugin disabled | No |
| PluginEnableEvent | Plugin enabled | No |
| RemoteServerCommandEvent | Remote command (RCON) | Yes |
| ServerCommandEvent | Server console command | Yes |
| ServerListPingEvent | Server list ping | No |
| ServerLoadEvent | Server load/reload complete | No |
| ServiceRegisterEvent | Service registered | No |
| ServiceUnregisterEvent | Service unregistered | No |
| TabCompleteEvent | Tab completion | Yes |

### 2.9 vehicle (11 events)

| Event | Description |
|-------|-------------|
| VehicleBlockCollisionEvent | Vehicle collides with block |
| VehicleCreateEvent | Vehicle created |
| VehicleDamageEvent | Vehicle damaged |
| VehicleDestroyEvent | Vehicle destroyed |
| VehicleEnterEvent | Entity enters vehicle |
| VehicleEntityCollisionEvent | Vehicle collides with entity |
| VehicleExitEvent | Entity exits vehicle |
| VehicleMoveEvent | Vehicle moves |
| VehicleUpdateEvent | Vehicle updated |

### 2.10 weather (5 events)

| Event | Description | Cancellable |
|-------|-------------|-------------|
| LightningStrikeEvent | Lightning strikes | Yes |
| ThunderChangeEvent | Thunder state changes | Yes |
| WeatherChangeEvent | Weather state changes | Yes |

### 2.11 world (22 events)

**Priority implement:** ChunkLoadEvent, ChunkUnloadEvent, WorldLoadEvent, StructureGrowEvent

| Event | Description | Cancellable |
|-------|-------------|-------------|
| AsyncStructureGenerateEvent | Structure generated (async) | No |
| AsyncStructureSpawnEvent | Structure spawned (async) | Yes |
| ChunkLoadEvent | Chunk loaded | No |
| ChunkPopulateEvent | Chunk populated | No |
| ChunkUnloadEvent | Chunk unloaded | Yes |
| EntitiesLoadEvent | Entities loaded in chunk | No |
| EntitiesUnloadEvent | Entities unloaded from chunk | No |
| GenericGameEvent | Generic game event | Yes |
| LootGenerateEvent | Loot generated | Yes |
| PortalCreateEvent | Portal created | Yes |
| SpawnChangeEvent | World spawn changes | No |
| StructureGrowEvent | Structure grows (tree, mushroom) | Yes |
| TimeSkipEvent | World time skipped | Yes |
| WorldInitEvent | World initialized | No |
| WorldLoadEvent | World loaded | No |
| WorldSaveEvent | World saved | No |
| WorldUnloadEvent | World unloaded | Yes |

---

## 3. Plugin Lifecycle

### Bukkit JavaPlugin

```java
public class MyPlugin extends JavaPlugin {
    void onLoad()     // Called after plugin loaded, before onEnable
    void onEnable()   // Called on startup/reload - register listeners/commands here
    void onDisable()  // Called on shutdown/reload - cleanup here
}
```

**Key JavaPlugin methods:**

| Method | Description |
|--------|-------------|
| `getDataFolder()` | Returns plugin-specific data folder (may not exist yet) |
| `getServer()` | Returns Server instance |
| `isEnabled()` | Whether plugin is currently enabled |
| `getConfig()` | Gets FileConfiguration for config.yml |
| `reloadConfig()` | Reloads config.yml from disk |
| `saveConfig()` | Saves config to disk |
| `saveDefaultConfig()` | Extracts default config.yml from JAR if not present |
| `saveResource(path, replace)` | Extracts resource from JAR to data folder |
| `getResource(filename)` | Gets embedded resource InputStream |
| `getLogger()` | Returns plugin-specific logger |
| `getCommand(name)` | Gets command by name (must be in plugin.yml) |
| `onCommand(sender, cmd, label, args)` | Default command handler |
| `onTabComplete(sender, cmd, alias, args)` | Default tab completion handler |
| `getDescription()` | Returns plugin.yml contents |
| `getPluginLoader()` | Returns the PluginLoader |

**Lifecycle order:**
1. All plugins: `onLoad()` called
2. All plugins: `onEnable()` called
3. Server runs
4. On shutdown/reload: `onDisable()` called for each plugin

---

## 4. Scheduler API

### BukkitScheduler

**Synchronous (main thread) methods:**

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `runTask(plugin, runnable)` | Plugin, Runnable | BukkitTask | Next tick |
| `runTaskLater(plugin, runnable, delay)` | Plugin, Runnable, long | BukkitTask | After delay ticks |
| `runTaskTimer(plugin, runnable, delay, period)` | Plugin, Runnable, long, long | BukkitTask | Repeating from delay |
| `scheduleSyncDelayedTask(plugin, runnable, delay)` | Plugin, Runnable, long | int (taskId) | After delay ticks |
| `scheduleSyncDelayedTask(plugin, runnable)` | Plugin, Runnable | int (taskId) | Next tick |
| `scheduleSyncRepeatingTask(plugin, runnable, delay, period)` | Plugin, Runnable, long, long | int (taskId) | Repeating |
| `callSyncMethod(plugin, callable)` | Plugin, Callable<T> | Future<T> | Call on main thread, get Future |

**Asynchronous (worker thread) methods:**

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `runTaskAsynchronously(plugin, runnable)` | Plugin, Runnable | BukkitTask | Immediately on worker |
| `runTaskLaterAsynchronously(plugin, runnable, delay)` | Plugin, Runnable, long | BukkitTask | After delay ticks on worker |
| `runTaskTimerAsynchronously(plugin, runnable, delay, period)` | Plugin, Runnable, long, long | BukkitTask | Repeating on worker |

**Management methods:**

| Method | Description |
|--------|-------------|
| `cancelTask(taskId)` | Remove task by ID |
| `cancelTasks(plugin)` | Remove all tasks for plugin |
| `isCurrentlyRunning(taskId)` | Check if task thread is alive |
| `isQueued(taskId)` | Check if task is queued |
| `getActiveWorkers()` | List async tasks with running threads |
| `getPendingTasks()` | List all pending tasks |

**IMPORTANT:** Async tasks must NEVER access Bukkit API. Thread-safety is the plugin's responsibility.

---

## 5. Command System

### CommandExecutor Interface

```java
public interface CommandExecutor {
    boolean onCommand(CommandSender sender, Command command, String label, String[] args);
}
```

- Returns `true` if valid command, `false` shows usage from plugin.yml
- `sender` = source of command (Player, ConsoleCommandSender, etc.)
- `command` = Command object
- `label` = alias used
- `args` = arguments passed

### TabCompleter Interface

```java
public interface TabCompleter {
    List<String> onTabComplete(CommandSender sender, Command command, String alias, String[] args);
}
```

### Command Registration

Commands must be declared in `plugin.yml`:
```yaml
commands:
  mycommand:
    description: Does something
    usage: /<command> [args]
    permission: myplugin.mycommand
    aliases: [mc, mycmd]
```

Then in `onEnable()`:
```java
getCommand("mycommand").setExecutor(new MyCommandExecutor());
getCommand("mycommand").setTabCompleter(new MyTabCompleter());
```

---

## 6. Configuration API

### FileConfiguration (extends MemorySection)

**File operations:**

| Method | Description |
|--------|-------------|
| `save(File)` | Save to file (creates if needed, overwrites if exists) |
| `save(String)` | Save to file path |
| `saveToString()` | Serialize to string |
| `load(File)` | Load from file (clears existing values) |
| `load(Reader)` | Load from reader |
| `load(String)` | Load from file path |
| `loadFromString(String)` | Load from string |
| `options()` | Get FileConfigurationOptions |

**Data access methods (inherited from MemorySection/ConfigurationSection):**

| Method | Returns |
|--------|---------|
| `get(path)`, `get(path, default)` | Object |
| `getString(path)`, `getString(path, default)` | String |
| `getInt(path)`, `getInt(path, default)` | int |
| `getLong(path)`, `getLong(path, default)` | long |
| `getDouble(path)`, `getDouble(path, default)` | double |
| `getBoolean(path)`, `getBoolean(path, default)` | boolean |
| `getStringList(path)` | List<String> |
| `getIntegerList(path)` | List<Integer> |
| `getDoubleList(path)` | List<Double> |
| `getLongList(path)` | List<Long> |
| `getBooleanList(path)` | List<Boolean> |
| `getMapList(path)` | List<Map<?,?>> |
| `getList(path)`, `getList(path, default)` | List<?> |
| `getItemStack(path)` | ItemStack |
| `getLocation(path)` | Location |
| `getVector(path)` | Vector |
| `getColor(path)` | Color |
| `getConfigurationSection(path)` | ConfigurationSection |
| `getObject(path, clazz)` | T |
| `getSerializable(path, clazz)` | T |

**Structure methods:**

| Method | Description |
|--------|-------------|
| `set(path, value)` | Set value at path |
| `contains(path)` | Check if path exists |
| `isSet(path)` | Check if value is set (not default) |
| `getKeys(deep)` | Get all keys |
| `getValues(deep)` | Get all values |
| `createSection(path)` | Create new section |
| `getCurrentPath()` | Get current path |
| `getName()` | Get section name |
| `getRoot()` | Get root configuration |
| `getDefaultSection()` | Get defaults section |
| `setComments(path, comments)` | Set comments for path |
| `getComments(path)` | Get comments for path |
| `setInlineComments(path, comments)` | Set inline comments |
| `getInlineComments(path)` | Get inline comments |

---

## 7. Pumpkin Current Implementation

### 7.1 Implemented Events (28 total)

#### Block Events (4)

| Pumpkin Event | Fields | Cancellable | Bukkit Equivalent |
|---------------|--------|-------------|-------------------|
| `BlockBreakEvent` | player: Option<Arc<Player>>, block: &Block, block_position: BlockPos, exp: u32, drop: bool | Yes | BlockBreakEvent |
| `BlockBurnEvent` | igniting_block: &Block, block: &Block | Yes | BlockBurnEvent |
| `BlockCanBuildEvent` | block_to_build: &Block, buildable: bool, player: Arc<Player>, block: &Block | Yes | BlockCanBuildEvent |
| `BlockPlaceEvent` | player: Arc<Player>, block_placed: &Block, block_placed_against: &Block, can_build: bool | Yes | BlockPlaceEvent |

#### Player Events (12)

| Pumpkin Event | Fields | Cancellable | Bukkit Equivalent |
|---------------|--------|-------------|-------------------|
| `PlayerJoinEvent` | player: Arc<Player>, join_message: TextComponent | Yes | PlayerJoinEvent |
| `PlayerLeaveEvent` | player: Arc<Player>, leave_message: TextComponent | Yes | PlayerQuitEvent |
| `PlayerChatEvent` | player: Arc<Player>, message: String, recipients: Vec<Arc<Player>> | Yes | AsyncPlayerChatEvent |
| `PlayerMoveEvent` | player: Arc<Player>, from: Vector3<f64>, to: Vector3<f64> | Yes | PlayerMoveEvent |
| `PlayerCommandSendEvent` | player: Arc<Player>, command: String | Yes | PlayerCommandPreprocessEvent |
| `PlayerLoginEvent` | player: Arc<Player>, kick_message: TextComponent | Yes | PlayerLoginEvent |
| `PlayerTeleportEvent` | player: Arc<Player>, from: Vector3<f64>, to: Vector3<f64> | Yes | PlayerTeleportEvent |
| `PlayerChangeWorldEvent` | player, previous_world, new_world, position, yaw, pitch | Yes | PlayerChangedWorldEvent |
| `PlayerGamemodeChangeEvent` | player: Arc<Player>, previous_gamemode: GameMode, new_gamemode: GameMode | Yes | PlayerGameModeChangeEvent |
| `PlayerInteractEvent` | player, action: InteractAction, clicked_pos, item, block | Yes | PlayerInteractEvent |
| `PlayerDeathEvent` | player: Arc<Player>, death_message: TextComponent, keep_inventory: bool | Yes | PlayerDeathEvent |
| `PlayerRespawnEvent` | player: Arc<Player>, respawn_position: Vector3<f64> | Yes | PlayerRespawnEvent |

#### Entity Events (4)

| Pumpkin Event | Fields | Cancellable | Bukkit Equivalent |
|---------------|--------|-------------|-------------------|
| `EntitySpawnEvent` | entity_id: i32, entity_type: &EntityType, position: Vector3<f64>, world: Arc<World> | Yes | EntitySpawnEvent |
| `EntityDamageEvent` | entity_id, entity_type, damage: f32, damage_type: &DamageType, world | Yes | EntityDamageEvent |
| `EntityDamageByEntityEvent` | entity_id, entity_type, attacker_id, attacker_type, damage, damage_type, world | Yes | EntityDamageByEntityEvent |
| `EntityDeathEvent` | entity_id, entity_type, position: Vector3<f64>, world | Yes | EntityDeathEvent |

#### Server Events (5)

| Pumpkin Event | Fields | Cancellable | Bukkit Equivalent |
|---------------|--------|-------------|-------------------|
| `ServerCommandEvent` | command: String | Yes | ServerCommandEvent |
| `ServerBroadcastEvent` | message: TextComponent, sender: TextComponent | Yes | BroadcastMessageEvent |
| `ServerStartedEvent` | world_count: usize, plugin_count: usize | No | ServerLoadEvent |
| `ServerStopEvent` | reason: String | No | (no direct equivalent) |
| `ServerTickEvent` | tick_count: i64 | No | (no direct equivalent) |

#### World Events (3)

| Pumpkin Event | Fields | Cancellable | Bukkit Equivalent |
|---------------|--------|-------------|-------------------|
| `ChunkLoad` | world: Arc<World>, chunk: Arc<RwLock<ChunkData>> | Yes | ChunkLoadEvent |
| `ChunkSave` | world: Arc<World>, chunk: Arc<RwLock<ChunkData>> | Yes | WorldSaveEvent (partial) |
| `ChunkSend` | world: Arc<World>, chunk: Arc<RwLock<ChunkData>> | Yes | (no direct equivalent) |

### 7.2 Coverage Summary

| Package | Bukkit Count | Pumpkin Count | Coverage |
|---------|-------------|---------------|----------|
| block | 50 | 4 | 8% |
| enchantment | 2 | 0 | 0% |
| entity | 92 | 4 | 4% |
| hanging | 5 | 0 | 0% |
| inventory | 25 | 0 | 0% |
| player | 85 | 12 | 14% |
| raid | 6 | 0 | 0% |
| server | 15 | 5 | 33% |
| vehicle | 11 | 0 | 0% |
| weather | 5 | 0 | 0% |
| world | 22 | 3 | 14% |
| **TOTAL** | **318** | **28** | **8.8%** |

---

## 8. Bukkit-to-Pumpkin Mapping (Top 50 Most Used Events)

Events ranked by real-world plugin usage frequency (based on analysis of top Bukkit/Spigot plugins).

| # | Bukkit Event | Pumpkin Equivalent | Status |
|---|-------------|-------------------|--------|
| 1 | PlayerJoinEvent | `PlayerJoinEvent` | IMPLEMENTED |
| 2 | PlayerQuitEvent | `PlayerLeaveEvent` | IMPLEMENTED |
| 3 | PlayerInteractEvent | `PlayerInteractEvent` | IMPLEMENTED |
| 4 | PlayerMoveEvent | `PlayerMoveEvent` | IMPLEMENTED |
| 5 | AsyncPlayerChatEvent | `PlayerChatEvent` | IMPLEMENTED (sync only) |
| 6 | PlayerCommandPreprocessEvent | `PlayerCommandSendEvent` | IMPLEMENTED |
| 7 | EntityDamageByEntityEvent | `EntityDamageByEntityEvent` | IMPLEMENTED |
| 8 | EntityDamageEvent | `EntityDamageEvent` | IMPLEMENTED |
| 9 | BlockBreakEvent | `BlockBreakEvent` | IMPLEMENTED |
| 10 | BlockPlaceEvent | `BlockPlaceEvent` | IMPLEMENTED |
| 11 | PlayerLoginEvent | `PlayerLoginEvent` | IMPLEMENTED |
| 12 | PlayerTeleportEvent | `PlayerTeleportEvent` | IMPLEMENTED |
| 13 | EntityDeathEvent | `EntityDeathEvent` | IMPLEMENTED |
| 14 | PlayerDeathEvent | `PlayerDeathEvent` | IMPLEMENTED |
| 15 | PlayerRespawnEvent | `PlayerRespawnEvent` | IMPLEMENTED |
| 16 | InventoryClickEvent | -- | NOT IMPLEMENTED |
| 17 | InventoryCloseEvent | -- | NOT IMPLEMENTED |
| 18 | InventoryOpenEvent | -- | NOT IMPLEMENTED |
| 19 | PlayerDropItemEvent | -- | NOT IMPLEMENTED |
| 20 | PlayerItemHeldEvent | -- | NOT IMPLEMENTED |
| 21 | EntitySpawnEvent | `EntitySpawnEvent` | IMPLEMENTED |
| 22 | CreatureSpawnEvent | -- | NOT IMPLEMENTED |
| 23 | PlayerToggleSneakEvent | -- | NOT IMPLEMENTED |
| 24 | PlayerToggleSprintEvent | -- | NOT IMPLEMENTED |
| 25 | PlayerGameModeChangeEvent | `PlayerGamemodeChangeEvent` | IMPLEMENTED |
| 26 | PlayerChangedWorldEvent | `PlayerChangeWorldEvent` | IMPLEMENTED |
| 27 | ProjectileHitEvent | -- | NOT IMPLEMENTED |
| 28 | ProjectileLaunchEvent | -- | NOT IMPLEMENTED |
| 29 | EntityExplodeEvent | -- | NOT IMPLEMENTED |
| 30 | BlockExplodeEvent | -- | NOT IMPLEMENTED |
| 31 | FoodLevelChangeEvent | -- | NOT IMPLEMENTED |
| 32 | EntityRegainHealthEvent | -- | NOT IMPLEMENTED |
| 33 | PlayerItemConsumeEvent | -- | NOT IMPLEMENTED |
| 34 | ServerCommandEvent | `ServerCommandEvent` | IMPLEMENTED |
| 35 | PlayerKickEvent | -- | NOT IMPLEMENTED |
| 36 | PlayerBedEnterEvent | -- | NOT IMPLEMENTED |
| 37 | PlayerBedLeaveEvent | -- | NOT IMPLEMENTED |
| 38 | SignChangeEvent | -- | NOT IMPLEMENTED |
| 39 | BlockPhysicsEvent | -- | NOT IMPLEMENTED |
| 40 | BlockFromToEvent | -- | NOT IMPLEMENTED |
| 41 | PlayerPickupArrowEvent | -- | NOT IMPLEMENTED |
| 42 | EntityPickupItemEvent | -- | NOT IMPLEMENTED |
| 43 | CraftItemEvent | -- | NOT IMPLEMENTED |
| 44 | FurnaceSmeltEvent | -- | NOT IMPLEMENTED |
| 45 | PlayerInteractEntityEvent | -- | NOT IMPLEMENTED |
| 46 | EntityTargetEvent | -- | NOT IMPLEMENTED |
| 47 | EntityTeleportEvent | -- | NOT IMPLEMENTED |
| 48 | ChunkLoadEvent | `ChunkLoad` | IMPLEMENTED |
| 49 | ChunkUnloadEvent | -- | NOT IMPLEMENTED |
| 50 | WeatherChangeEvent | -- | NOT IMPLEMENTED |

**Score: 20/50 implemented (40% of high-impact events)**

---

## 9. Sprint Priorities

### Tier 1 -- Critical (enables 80% of plugins)

These events are used by nearly every protection/economy/gameplay plugin. Implementing these unlocks the largest number of plugins.

| Priority | Event | Bukkit Package | Why |
|----------|-------|---------------|-----|
| P0 | InventoryClickEvent | inventory | Every GUI plugin needs this (menus, shops, kits) |
| P0 | InventoryOpenEvent | inventory | Required for custom inventory GUIs |
| P0 | InventoryCloseEvent | inventory | Cleanup for custom inventory GUIs |
| P0 | PlayerDropItemEvent | player | Anti-cheat, item protection, economy plugins |
| P0 | PlayerItemHeldEvent | player | Weapon switching detection, ability plugins |
| P0 | CreatureSpawnEvent | entity | Mob control (spawner plugins, world guard) |
| P0 | PlayerToggleSneakEvent | player | Vanish plugins, ability triggers, double-sneak menus |
| P0 | PlayerToggleSprintEvent | player | Movement detection, anti-cheat |

### Tier 2 -- High Impact (enables protection and combat plugins)

| Priority | Event | Bukkit Package | Why |
|----------|-------|---------------|-----|
| P1 | ProjectileHitEvent | entity | Combat plugins, arrow mechanics |
| P1 | ProjectileLaunchEvent | entity | Projectile control, ability plugins |
| P1 | EntityExplodeEvent | entity | Grief protection (WorldGuard, etc.) |
| P1 | BlockExplodeEvent | block | Grief protection |
| P1 | FoodLevelChangeEvent | entity | Hunger management, game modes |
| P1 | EntityRegainHealthEvent | entity | Custom healing mechanics |
| P1 | PlayerItemConsumeEvent | player | Custom food effects, potion handling |
| P1 | PlayerKickEvent | player | Ban/punishment plugins |
| P1 | SignChangeEvent | block | Sign shops, information systems |
| P1 | BlockPhysicsEvent | block | World protection, anti-grief |

### Tier 3 -- Medium Impact (enables niche but popular plugins)

| Priority | Event | Bukkit Package | Why |
|----------|-------|---------------|-----|
| P2 | PlayerInteractEntityEvent | player | NPC plugins, villager trading |
| P2 | EntityTargetEvent | entity | Custom mob AI, boss fights |
| P2 | EntityTeleportEvent | entity | Enderman grief protection |
| P2 | ChunkUnloadEvent | world | Custom world management |
| P2 | WeatherChangeEvent | weather | Weather control plugins |
| P2 | BlockFromToEvent | block | Liquid flow control |
| P2 | CraftItemEvent | inventory | Custom recipes, crafting restrictions |
| P2 | FurnaceSmeltEvent | inventory | Custom smelting |
| P2 | PlayerBedEnterEvent | player | Sleep voting, time skip plugins |
| P2 | EntityPickupItemEvent | entity | Item protection, custom loot |

### Tier 4 -- Low Priority (niche functionality)

| Priority | Event | Bukkit Package | Why |
|----------|-------|---------------|-----|
| P3 | BlockGrowEvent | block | Farming plugins |
| P3 | LeavesDecayEvent | block | Tree management |
| P3 | BlockRedstoneEvent | block | Redstone control |
| P3 | BlockPistonExtendEvent | block | Piston protection |
| P3 | BlockPistonRetractEvent | block | Piston protection |
| P3 | PlayerFishEvent | player | Fishing plugins |
| P3 | EntityBreedEvent | entity | Breeding control |
| P3 | VillagerAcquireTradeEvent | entity | Economy plugins |
| P3 | WorldLoadEvent | world | Multi-world management |
| P3 | WorldUnloadEvent | world | Multi-world management |

### Implementation Effort Estimates

| Tier | Event Count | Estimated Effort | Impact |
|------|-------------|-----------------|--------|
| Tier 1 (P0) | 8 events | 2-3 sessions | Unlocks GUI plugins, anti-cheat, mob control |
| Tier 2 (P1) | 10 events | 3-4 sessions | Unlocks combat, protection, punishment plugins |
| Tier 3 (P2) | 10 events | 3-4 sessions | Unlocks NPC, crafting, weather plugins |
| Tier 4 (P3) | 10 events | 2-3 sessions | Unlocks farming, redstone, breeding plugins |

### Dependencies and Blockers

- **Inventory events (P0):** Require Items agent to expose inventory interaction hooks
- **CreatureSpawnEvent:** Requires Entity agent to fire event from spawn logic
- **ProjectileHitEvent/LaunchEvent:** Requires Entity agent projectile system
- **BlockPhysicsEvent:** Requires Core/WorldGen agent block update hooks
- **BlockFromToEvent:** Requires Core liquid flow system
- **All entity events:** Require Entity agent to wire `fire()` calls into game loop

### What the Plugin Agent Can Do Alone

The Plugin agent can define all event structs and their fields without dependencies. However, the events will not fire until the relevant agent wires `fire()` calls into the appropriate game logic. The recommended workflow is:

1. **Plugin agent:** Define event structs (already done for 28 events)
2. **Plugin agent:** Document required fire points for each event
3. **Other agents:** Wire `fire()` calls at documented points
4. **Plugin agent:** Write integration tests confirming events fire correctly

### Naming Conventions to Maintain

| Convention | Example |
|------------|---------|
| Struct suffix | `Event` (e.g., `BlockBreakEvent`) |
| Module file | snake_case of struct (e.g., `block_break.rs`) |
| Category trait | `BlockEvent`, `PlayerEvent`, `EntityEvent` |
| Cancellable decorator | `#[cancellable]` before `#[derive(Event, Clone)]` |
| Non-cancellable | Only `#[derive(Event, Clone)]` |
| Constructor | `pub fn new(...)` or `pub const fn new(...)` |

---

## Appendix A: Bukkit Event Hierarchy

```
Event (abstract)
  +-- BlockEvent
  |     +-- BlockBreakEvent, BlockPlaceEvent, BlockBurnEvent, ...
  +-- EntityEvent
  |     +-- EntityDamageEvent
  |     |     +-- EntityDamageByEntityEvent
  |     |     +-- EntityDamageByBlockEvent
  |     +-- EntityDeathEvent
  |     |     +-- PlayerDeathEvent
  |     +-- EntitySpawnEvent
  |     |     +-- CreatureSpawnEvent
  |     |     +-- SpawnerSpawnEvent
  |     |     +-- ItemSpawnEvent
  |     +-- ProjectileHitEvent, ProjectileLaunchEvent, ...
  +-- PlayerEvent
  |     +-- PlayerJoinEvent, PlayerQuitEvent, PlayerMoveEvent, ...
  |     +-- PlayerBucketEvent
  |     |     +-- PlayerBucketEmptyEvent
  |     |     +-- PlayerBucketFillEvent
  |     +-- PlayerInteractEvent
  |     +-- PlayerTeleportEvent
  |           +-- PlayerPortalEvent
  +-- InventoryEvent
  |     +-- InventoryClickEvent, InventoryDragEvent, ...
  |     +-- InventoryInteractEvent
  |           +-- CraftItemEvent
  |           +-- InventoryCreativeEvent
  +-- InventoryMoveItemEvent (direct Event subclass)
  +-- InventoryPickupItemEvent (direct Event subclass)
  +-- ServerEvent
  |     +-- PluginEnableEvent, PluginDisableEvent, ServerLoadEvent, ...
  +-- WeatherEvent
  |     +-- WeatherChangeEvent, ThunderChangeEvent, LightningStrikeEvent
  +-- WorldEvent
  |     +-- ChunkEvent
  |     |     +-- ChunkLoadEvent, ChunkUnloadEvent, ChunkPopulateEvent
  |     +-- WorldLoadEvent, WorldSaveEvent, WorldUnloadEvent, ...
  +-- VehicleEvent
  |     +-- VehicleMoveEvent, VehicleDamageEvent, ...
  +-- HangingEvent
  |     +-- HangingBreakEvent, HangingPlaceEvent
  +-- TabCompleteEvent (direct Event subclass)
  +-- AsyncPlayerPreLoginEvent (direct Event subclass)
  +-- PlayerLeashEntityEvent (direct Event subclass)
```

## Appendix B: Pumpkin vs Bukkit Structural Differences

| Aspect | Bukkit | Pumpkin |
|--------|--------|---------|
| Base class | `abstract class Event` | `trait Payload` |
| Cancellation | `interface Cancellable` | `trait Cancellable` + `#[cancellable]` macro |
| HandlerList | Static per-event class, `getHandlerList()` | Name-based dispatch via `fire()` |
| Type checking | Java class hierarchy + `instanceof` | Name string matching + unsafe downcast |
| Async events | `Event(true)` constructor | Not yet implemented |
| Event hierarchy | Deep inheritance (EntityDamageByEntityEvent extends EntityDamageEvent) | Flat structs, no inheritance (trait composition) |
| Priority order | LOWEST=first, HIGHEST=last, MONITOR=observe | Highest=first, Lowest=last, Monitor=observe |
| Event names | Class name via `getEventName()` | `get_name_static()` / `get_name()` |
| Registration | `@EventHandler` annotation + `registerEvents()` | Plugin registers closures in `fire()` system |
| Plugin lifecycle | `onLoad()` / `onEnable()` / `onDisable()` | Similar (needs verification of current impl) |

## Appendix C: Key Enums Needed for Future Events

These Bukkit enums will be needed as Pumpkin enums when implementing the corresponding events:

| Enum | Used By | Variants |
|------|---------|----------|
| `Action` (block) | PlayerInteractEvent | LEFT_CLICK_BLOCK, RIGHT_CLICK_BLOCK, LEFT_CLICK_AIR, RIGHT_CLICK_AIR, PHYSICAL |
| `BlockIgniteEvent.IgniteCause` | BlockIgniteEvent | LAVA, FLINT_AND_STEEL, SPREAD, LIGHTNING, FIREBALL, ENDER_CRYSTAL, EXPLOSION, ARROW |
| `CreatureSpawnEvent.SpawnReason` | CreatureSpawnEvent | NATURAL, JOCKEY, SPAWNER, EGG, SPAWNER_EGG, LIGHTNING, BUILD_SNOWMAN, BUILD_IRONGOLEM, BUILD_WITHER, VILLAGE_DEFENSE, VILLAGE_INVASION, BREEDING, SLIME_SPLIT, REINFORCEMENTS, NETHER_PORTAL, DISPENSE_EGG, INFECTION, CURED, OCELOT_BABY, SILVERFISH_BLOCK, MOUNT, TRAP, ENDER_PEARL, SHOULDER_ENTITY, DROWNED, SHEARED, EXPLOSION, RAID, PATROL, BEEHIVE, PIGLIN_ZOMBIFIED, COMMAND, CUSTOM, DEFAULT |
| `EntityDamageEvent.DamageCause` | EntityDamageEvent | CONTACT, ENTITY_ATTACK, ENTITY_SWEEP_ATTACK, PROJECTILE, SUFFOCATION, FALL, FIRE, FIRE_TICK, MELTING, LAVA, DROWNING, BLOCK_EXPLOSION, ENTITY_EXPLOSION, VOID, LIGHTNING, SUICIDE, STARVATION, POISON, MAGIC, WITHER, FALLING_BLOCK, THORNS, DRAGON_BREATH, CUSTOM, FLY_INTO_WALL, HOT_FLOOR, CRAMMING, DRYOUT, FREEZE, SONIC_BOOM |
| `EntityTargetEvent.TargetReason` | EntityTargetEvent | TARGET_DIED, CLOSEST_PLAYER, TARGET_ATTACKED_ENTITY, PIG_ZOMBIE_TARGET, FORGOT_TARGET, TARGET_ATTACKED_NEARBY_ENTITY, DEFEND_VILLAGE, TARGET_ATTACKED_OWNER, OWNER_ATTACKED_TARGET, RANDOM_TARGET, COLLISION, CUSTOM, CLOSEST_ENTITY, FOLLOW_LEADER, TEMPT, UNKNOWN |
| `PlayerTeleportEvent.TeleportCause` | PlayerTeleportEvent | ENDER_PEARL, COMMAND, PLUGIN, NETHER_PORTAL, END_PORTAL, SPECTATE, END_GATEWAY, CHORUS_FRUIT, DISMOUNT, EXIT_BED, UNKNOWN |
| `PlayerRespawnEvent.RespawnReason` | PlayerRespawnEvent | DEATH, END_PORTAL, PLUGIN |
| `ClickType` (inventory) | InventoryClickEvent | LEFT, SHIFT_LEFT, RIGHT, SHIFT_RIGHT, WINDOW_BORDER_LEFT, WINDOW_BORDER_RIGHT, MIDDLE, NUMBER_KEY, DOUBLE_CLICK, DROP, CONTROL_DROP, CREATIVE, SWAP_OFFHAND, UNKNOWN |
| `InventoryAction` | InventoryClickEvent | NOTHING, PICKUP_ALL, PICKUP_SOME, PICKUP_HALF, PICKUP_ONE, PLACE_ALL, PLACE_SOME, PLACE_ONE, SWAP_WITH_CURSOR, DROP_ALL_CURSOR, DROP_ONE_CURSOR, DROP_ALL_SLOT, DROP_ONE_SLOT, MOVE_TO_OTHER_INVENTORY, HOTBAR_MOVE_AND_READD, HOTBAR_SWAP, CLONE_STACK, COLLECT_TO_CURSOR, UNKNOWN |

**Note:** Pumpkin already has `InteractAction` enum with LEFT_CLICK_BLOCK, LEFT_CLICK_AIR, RIGHT_CLICK_AIR, RIGHT_CLICK_BLOCK (missing PHYSICAL from Bukkit's Action enum).
