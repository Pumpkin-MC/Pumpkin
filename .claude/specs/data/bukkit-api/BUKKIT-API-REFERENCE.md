# Bukkit/Spigot API Reference for Pumpkin Plugin Compatibility

Source: hub.spigotmc.org/javadocs/bukkit/ (Spigot-API 1.21.11-R0.2-SNAPSHOT)
Purpose: Reference for implementing Bukkit-compatible plugin API in Pumpkin

## Quick Reference URLs
- Javadoc: https://hub.spigotmc.org/javadocs/bukkit/
- Spigot API: https://hub.spigotmc.org/javadocs/spigot/
- Source (Stash): https://hub.spigotmc.org/stash/projects/SPIGOT/repos/bukkit

## Events (318 total across 11 packages)

Events are the core plugin API — plugins register listeners via @EventHandler.
Priority: LOWEST → LOW → NORMAL → HIGH → HIGHEST → MONITOR

### block (50 events)
**Key events (implement first):** BlockBreakEvent, BlockPlaceEvent, BlockPhysicsEvent, BlockRedstoneEvent, BlockPistonExtendEvent, BlockPistonRetractEvent, BlockFromToEvent, SignChangeEvent, BlockExplodeEvent, BlockGrowEvent
All: BellResonateEvent, BellRingEvent, BlockBreakEvent, BlockBrushEvent, BlockBurnEvent, BlockCanBuildEvent, BlockCookEvent, BlockDamageAbortEvent, BlockDamageEvent, BlockDispenseArmorEvent, BlockDispenseEvent, BlockDispenseLootEvent, BlockDropItemEvent, BlockEvent, BlockExpEvent, BlockExplodeEvent, BlockFadeEvent, BlockFertilizeEvent, BlockFormEvent, BlockFromToEvent, BlockGrowEvent, BlockIgniteEvent, BlockMultiPlaceEvent, BlockPhysicsEvent, BlockPistonEvent, BlockPistonExtendEvent, BlockPistonRetractEvent, BlockPlaceEvent, BlockReceiveGameEvent, BlockRedstoneEvent, BlockShearEntityEvent, BlockSpreadEvent, BrewingStartEvent, CampfireStartEvent, CauldronLevelChangeEvent, CrafterCraftEvent, EntityBlockFormEvent, FluidLevelChangeEvent, InventoryBlockStartEvent, LeavesDecayEvent, MoistureChangeEvent, NotePlayEvent, SculkBloomEvent, SignChangeEvent, SpongeAbsorbEvent, TNTPrimeEvent, VaultDisplayItemEvent

### enchantment (2 events)
All: EnchantItemEvent, PrepareItemEnchantEvent

### entity (92 events)
**Key events (implement first):** EntityDamageEvent, EntityDamageByEntityEvent, EntityDeathEvent, CreatureSpawnEvent, EntityExplodeEvent, ProjectileHitEvent, EntityTargetEvent, EntityTeleportEvent, FoodLevelChangeEvent
All: AreaEffectCloudApplyEvent, ArrowBodyCountChangeEvent, BatToggleSleepEvent, CreatureSpawnEvent, CreeperPowerEvent, EnderDragonChangePhaseEvent, EntityAirChangeEvent, EntityBreakDoorEvent, EntityBreedEvent, EntityChangeBlockEvent, EntityCombustByBlockEvent, EntityCombustByEntityEvent, EntityCombustEvent, EntityCreatePortalEvent, EntityDamageByBlockEvent, EntityDamageByEntityEvent, EntityDamageEvent, EntityDeathEvent, EntityDismountEvent, EntityDropItemEvent, EntityEnterBlockEvent, EntityEnterLoveModeEvent, EntityEvent, EntityExhaustionEvent, EntityExplodeEvent, EntityInteractEvent, EntityKnockbackByEntityEvent, EntityKnockbackEvent, EntityMountEvent, EntityPickupItemEvent, EntityPlaceEvent, EntityPortalEnterEvent, EntityPortalEvent, EntityPortalExitEvent, EntityPoseChangeEvent, EntityPotionEffectEvent, EntityRegainHealthEvent, EntityRemoveEvent, EntityResurrectEvent, EntityShootBowEvent, EntitySpawnEvent, EntitySpellCastEvent, EntityTameEvent, EntityTargetBlockEvent, EntityTargetEvent, EntityTargetLivingEntityEvent, EntityTeleportEvent, EntityToggleGlideEvent, EntityToggleSwimEvent, EntityTransformEvent, EntityUnleashEvent, ExpBottleEvent, ExplosionPrimeEvent, FireworkExplodeEvent, FoodLevelChangeEvent, HorseJumpEvent, ItemDespawnEvent, ItemMergeEvent, ItemSpawnEvent, LingeringPotionSplashEvent, PigZapEvent, PigZombieAngerEvent, PiglinBarterEvent, PlayerDeathEvent, PlayerLeashEntityEvent, PotionSplashEvent, ProjectileHitEvent, ProjectileLaunchEvent, SheepDyeWoolEvent, SheepRegrowWoolEvent, SlimeSplitEvent, SpawnerSpawnEvent, StriderTemperatureChangeEvent, TrialSpawnerSpawnEvent, VillagerAcquireTradeEvent, VillagerCareerChangeEvent, VillagerReplenishTradeEvent, VillagerReputationChangeEvent

### hanging (5 events)
All: HangingBreakByEntityEvent, HangingBreakEvent, HangingEvent, HangingPlaceEvent

### inventory (25 events)
**Key events (implement first):** InventoryClickEvent, InventoryOpenEvent, InventoryCloseEvent, CraftItemEvent, FurnaceSmeltEvent, InventoryMoveItemEvent
All: BrewEvent, BrewingStandFuelEvent, CraftItemEvent, FurnaceBurnEvent, FurnaceExtractEvent, FurnaceSmeltEvent, FurnaceStartSmeltEvent, HopperInventorySearchEvent, InventoryClickEvent, InventoryCloseEvent, InventoryCreativeEvent, InventoryDragEvent, InventoryEvent, InventoryInteractEvent, InventoryMoveItemEvent, InventoryOpenEvent, InventoryPickupItemEvent, PrepareAnvilEvent, PrepareGrindstoneEvent, PrepareInventoryResultEvent, PrepareItemCraftEvent, PrepareSmithingEvent, SmithItemEvent, TradeSelectEvent

### player (85 events)
**Key events (implement first):** PlayerJoinEvent, PlayerQuitEvent, PlayerMoveEvent, PlayerInteractEvent, PlayerChatEvent, AsyncPlayerChatEvent, PlayerCommandPreprocessEvent, PlayerLoginEvent, PlayerTeleportEvent, PlayerDeathEvent, PlayerRespawnEvent, PlayerDropItemEvent, PlayerItemConsumeEvent
All: AsyncPlayerChatEvent, AsyncPlayerChatPreviewEvent, AsyncPlayerPreLoginEvent, PlayerAdvancementDoneEvent, PlayerAnimationEvent, PlayerArmorStandManipulateEvent, PlayerBedEnterEvent, PlayerBedLeaveEvent, PlayerBucketEmptyEvent, PlayerBucketEntityEvent, PlayerBucketEvent, PlayerBucketFillEvent, PlayerBucketFishEvent, PlayerChangedMainHandEvent, PlayerChangedWorldEvent, PlayerChannelEvent, PlayerChatEvent, PlayerChatTabCompleteEvent, PlayerCommandPreprocessEvent, PlayerCommandSendEvent, PlayerCustomClickEvent, PlayerDropItemEvent, PlayerEditBookEvent, PlayerEggThrowEvent, PlayerEvent, PlayerExpChangeEvent, PlayerExpCooldownChangeEvent, PlayerFishEvent, PlayerGameModeChangeEvent, PlayerHarvestBlockEvent, PlayerHideEntityEvent, PlayerInputEvent, PlayerInteractAtEntityEvent, PlayerInteractEntityEvent, PlayerInteractEvent, PlayerItemBreakEvent, PlayerItemConsumeEvent, PlayerItemDamageEvent, PlayerItemHeldEvent, PlayerItemMendEvent, PlayerJoinEvent, PlayerKickEvent, PlayerLevelChangeEvent, PlayerLinksSendEvent, PlayerLocaleChangeEvent, PlayerLoginEvent, PlayerMoveEvent, PlayerPickupArrowEvent, PlayerPickupItemEvent, PlayerPortalEvent, PlayerPreLoginEvent, PlayerQuitEvent, PlayerRecipeBookClickEvent, PlayerRecipeBookSettingsChangeEvent, PlayerRecipeDiscoverEvent, PlayerRegisterChannelEvent, PlayerResourcePackStatusEvent, PlayerRespawnEvent, PlayerRiptideEvent, PlayerShearEntityEvent, PlayerShowEntityEvent, PlayerSignOpenEvent, PlayerSpawnChangeEvent, PlayerStatisticIncrementEvent, PlayerSwapHandItemsEvent, PlayerTakeLecternBookEvent, PlayerTeleportEvent, PlayerToggleFlightEvent, PlayerToggleSneakEvent, PlayerToggleSprintEvent, PlayerUnleashEntityEvent, PlayerUnregisterChannelEvent, PlayerVelocityEvent

### raid (6 events)
All: RaidEvent, RaidFinishEvent, RaidSpawnWaveEvent, RaidStopEvent, RaidTriggerEvent

### server (15 events)
**Key events (implement first):** PluginEnableEvent, PluginDisableEvent, ServerLoadEvent
All: BroadcastMessageEvent, MapInitializeEvent, PluginDisableEvent, PluginEnableEvent, PluginEvent, RemoteServerCommandEvent, ServerCommandEvent, ServerEvent, ServerListPingEvent, ServerLoadEvent, ServiceEvent, ServiceRegisterEvent, ServiceUnregisterEvent, TabCompleteEvent

### vehicle (11 events)
All: VehicleBlockCollisionEvent, VehicleCollisionEvent, VehicleCreateEvent, VehicleDamageEvent, VehicleDestroyEvent, VehicleEnterEvent, VehicleEntityCollisionEvent, VehicleEvent, VehicleExitEvent, VehicleMoveEvent, VehicleUpdateEvent

### weather (5 events)
All: LightningStrikeEvent, ThunderChangeEvent, WeatherChangeEvent, WeatherEvent

### world (22 events)
**Key events (implement first):** ChunkLoadEvent, ChunkUnloadEvent, WorldLoadEvent, StructureGrowEvent
All: AsyncStructureGenerateEvent, AsyncStructureSpawnEvent, ChunkEvent, ChunkLoadEvent, ChunkPopulateEvent, ChunkUnloadEvent, EntitiesLoadEvent, EntitiesUnloadEvent, GenericGameEvent, LootGenerateEvent, PortalCreateEvent, SpawnChangeEvent, StructureGrowEvent, TimeSkipEvent, WorldEvent, WorldInitEvent, WorldLoadEvent, WorldSaveEvent, WorldUnloadEvent

## Key Interfaces & Classes

### Player (extends LivingEntity, HumanEntity)
Most-used methods (from Javadoc scrape):
```
  player.sendMessage()
  player.getLocation()
  player.teleport()
  player.getInventory()
  player.getHealth()
  player.setHealth()
  player.getGameMode()
  player.setGameMode()
  player.kickPlayer()
  player.getUniqueId()
  player.getName()
  player.getDisplayName()
  player.setDisplayName()
  player.isOp()
  player.hasPermission()
  player.getWorld()
  player.isSneaking()
  player.isSprinting()
  player.isFlying()
  player.setFlying()
  player.getAllowFlight()
  player.setAllowFlight()
  player.getLevel()
  player.setLevel()
  player.getExp()
  player.setExp()
  player.getFoodLevel()
  player.setFoodLevel()
  player.setBedSpawnLocation()
  player.sendTitle()
  player.sendActionBar()
  player.showTitle()
  player.playSound()
  player.spawnParticle()
  player.openInventory()
  player.closeInventory()
  player.updateInventory()
  player.getAddress()
  player.chat()
  player.performCommand()
  player.hidePlayer()
  player.showPlayer()
```

### World
Key methods:
```
  world.getBlockAt()
  world.spawnEntity()
  world.dropItem()
  world.getEntities()
  world.getLivingEntities()
  world.getPlayers()
  world.generateTree()
  world.createExplosion()
  world.strikeLightning()
  world.getSpawnLocation()
  world.setSpawnLocation()
  world.getTime()
  world.setTime()
  world.getFullTime()
  world.setFullTime()
  world.hasStorm()
  world.setStorm()
  world.isThundering()
  world.setThundering()
  world.getChunkAt()
  world.loadChunk()
  world.unloadChunk()
  world.isChunkLoaded()
  world.getHighestBlockAt()
  world.getName()
  world.getEnvironment()
  world.getSeed()
  world.getDifficulty()
  world.playSound()
  world.spawnParticle()
  world.getWorldBorder()
```

### Block
Key methods: getType(), setType(Material), getData(), getState(), getLocation(),
getWorld(), getRelative(BlockFace), isLiquid(), isEmpty(), getDrops(),
breakNaturally(), getBlockData(), setBlockData()

### ItemStack
Key methods: ItemStack(Material, amount), getType(), setType(), getAmount(),
setAmount(), getItemMeta(), setItemMeta(), getEnchantments(), addEnchantment(),
getMaxStackSize(), clone(), isSimilar()

### JavaPlugin (plugin lifecycle)
```java
public class MyPlugin extends JavaPlugin {
    @Override public void onEnable() { }   // Called on startup/reload
    @Override public void onDisable() { }  // Called on shutdown/reload
    @Override public void onLoad() { }     // Called before onEnable
}
```
Key methods: getConfig(), saveConfig(), reloadConfig(), getServer(),
getLogger(), getDataFolder(), getResource(), getCommand(), registerEvents()

### BukkitScheduler
```java
// Sync tasks (main thread)
scheduler.runTask(plugin, runnable)
scheduler.runTaskLater(plugin, runnable, delayTicks)
scheduler.runTaskTimer(plugin, runnable, delayTicks, periodTicks)
// Async tasks (off main thread)
scheduler.runTaskAsynchronously(plugin, runnable)
scheduler.runTaskLaterAsynchronously(plugin, runnable, delayTicks)
scheduler.runTaskTimerAsynchronously(plugin, runnable, delayTicks, periodTicks)
```

## Key Enums

### Material (blocks + items)
~1500 constants. See Material.txt or PrismarineJS items.json + blocks.json
Key methods: isBlock(), isItem(), isEdible(), isFuel(), isRecord(), isSolid(),
isTransparent(), isFlammable(), isBurnable(), isOccluding(), hasGravity()

### EntityType
149 entity types. See EntityType.txt or PrismarineJS entities.json

### GameMode
SURVIVAL, CREATIVE, ADVENTURE, SPECTATOR

### Attribute
Attributes: MAX_ABSORPTION, MAX_HEALTH

### Enchantment
See PrismarineJS enchantments.json for full list with levels/applicability

### PotionEffectType
See PrismarineJS effects.json for full list with durations

## Agent → Bukkit API Mapping

| Agent | Primary Bukkit Packages |
|-------|----------------------|
| Entity | org.bukkit.entity.*, event.entity.*, attribute.* |
| Items | org.bukkit.inventory.*, Material, event.inventory.* |
| WorldGen | org.bukkit.World, generator.*, event.world.* |
| Storage | org.bukkit.block.*, inventory.*, event.block.* |
| Redstone | event.block.BlockRedstoneEvent, BlockPiston*, block.data.* |
| Net | org.bukkit.event.player.*, Server, BanList |
| Core | org.bukkit.plugin.*, scheduler.*, command.* |

## Raw Javadoc Files
The following scraped Javadoc pages are included alongside this document:
- Material.txt (8333 lines) — every Material constant
- Player.txt — full Player interface
- World.txt — full World interface
- Bukkit.txt — static server accessor
- LivingEntity.txt — base mob interface
- EntityType.txt — all entity types
- Block.txt, ItemStack.txt, Inventory.txt
- JavaPlugin.txt, BukkitScheduler.txt, FileConfiguration.txt
- Event.txt, CommandExecutor.txt, Attribute.txt
- AllClasses.txt — complete class index