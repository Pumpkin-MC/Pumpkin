use std::collections::HashMap;

use crossbeam::atomic::AtomicCell;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Statistic;
use tokio::sync::Mutex;

use super::{NBTStorage, NBTStorageInit, NbtFuture};

/// Category IDs as defined by the Minecraft protocol (Award Statistics packet).
pub mod category {
    pub const MINED: i32 = 0;
    pub const CRAFTED: i32 = 1;
    pub const USED: i32 = 2;
    pub const BROKEN: i32 = 3;
    pub const PICKED_UP: i32 = 4;
    pub const DROPPED: i32 = 5;
    pub const KILLED: i32 = 6;
    pub const KILLED_BY: i32 = 7;
    pub const CUSTOM: i32 = 8;
}

/// Keys for `minecraft:custom` statistics (category 8).
/// The `statistic_id` for custom stats is the registry ID in `minecraft:custom_stat`.
pub mod custom {
    // Time
    pub const LEAVE_GAME: &str = "minecraft:leave_game";
    pub const PLAY_TIME: &str = "minecraft:play_time";
    pub const TOTAL_WORLD_TIME: &str = "minecraft:total_world_time";
    pub const TIME_SINCE_DEATH: &str = "minecraft:time_since_death";
    pub const TIME_SINCE_REST: &str = "minecraft:time_since_rest";
    pub const SNEAK_TIME: &str = "minecraft:sneak_time";
    // Movement
    pub const WALK_ONE_CM: &str = "minecraft:walk_one_cm";
    pub const CROUCH_ONE_CM: &str = "minecraft:crouch_one_cm";
    pub const SPRINT_ONE_CM: &str = "minecraft:sprint_one_cm";
    pub const WALK_ON_WATER_ONE_CM: &str = "minecraft:walk_on_water_one_cm";
    pub const FALL_ONE_CM: &str = "minecraft:fall_one_cm";
    pub const CLIMB_ONE_CM: &str = "minecraft:climb_one_cm";
    pub const FLY_ONE_CM: &str = "minecraft:fly_one_cm";
    pub const WALK_UNDER_WATER_ONE_CM: &str = "minecraft:walk_under_water_one_cm";
    pub const MINECART_ONE_CM: &str = "minecraft:minecart_one_cm";
    pub const BOAT_ONE_CM: &str = "minecraft:boat_one_cm";
    pub const PIG_ONE_CM: &str = "minecraft:pig_one_cm";
    pub const HAPPY_GHAST_ONE_CM: &str = "minecraft:happy_ghast_one_cm";
    pub const HORSE_ONE_CM: &str = "minecraft:horse_one_cm";
    pub const AVIATE_ONE_CM: &str = "minecraft:aviate_one_cm";
    pub const SWIM_ONE_CM: &str = "minecraft:swim_one_cm";
    pub const STRIDER_ONE_CM: &str = "minecraft:strider_one_cm";
    pub const NAUTILUS_ONE_CM: &str = "minecraft:nautilus_one_cm";
    // Gameplay
    pub const JUMP: &str = "minecraft:jump";
    pub const DROP: &str = "minecraft:drop";
    // Combat
    pub const DAMAGE_DEALT: &str = "minecraft:damage_dealt";
    pub const DAMAGE_DEALT_ABSORBED: &str = "minecraft:damage_dealt_absorbed";
    pub const DAMAGE_DEALT_RESISTED: &str = "minecraft:damage_dealt_resisted";
    pub const DAMAGE_TAKEN: &str = "minecraft:damage_taken";
    pub const DAMAGE_BLOCKED_BY_SHIELD: &str = "minecraft:damage_blocked_by_shield";
    pub const DAMAGE_ABSORBED: &str = "minecraft:damage_absorbed";
    pub const DAMAGE_RESISTED: &str = "minecraft:damage_resisted";
    pub const DEATHS: &str = "minecraft:deaths";
    pub const MOB_KILLS: &str = "minecraft:mob_kills";
    pub const ANIMALS_BRED: &str = "minecraft:animals_bred";
    pub const PLAYER_KILLS: &str = "minecraft:player_kills";
    // Interactions
    pub const FISH_CAUGHT: &str = "minecraft:fish_caught";
    pub const TALKED_TO_VILLAGER: &str = "minecraft:talked_to_villager";
    pub const TRADED_WITH_VILLAGER: &str = "minecraft:traded_with_villager";
    pub const EAT_CAKE_SLICE: &str = "minecraft:eat_cake_slice";
    pub const FILL_CAULDRON: &str = "minecraft:fill_cauldron";
    pub const USE_CAULDRON: &str = "minecraft:use_cauldron";
    pub const CLEAN_ARMOR: &str = "minecraft:clean_armor";
    pub const CLEAN_BANNER: &str = "minecraft:clean_banner";
    pub const CLEAN_SHULKER_BOX: &str = "minecraft:clean_shulker_box";
    pub const INTERACT_WITH_BREWINGSTAND: &str = "minecraft:interact_with_brewingstand";
    pub const INTERACT_WITH_BEACON: &str = "minecraft:interact_with_beacon";
    pub const INSPECT_DROPPER: &str = "minecraft:inspect_dropper";
    pub const INSPECT_HOPPER: &str = "minecraft:inspect_hopper";
    pub const INSPECT_DISPENSER: &str = "minecraft:inspect_dispenser";
    pub const PLAY_NOTEBLOCK: &str = "minecraft:play_noteblock";
    pub const TUNE_NOTEBLOCK: &str = "minecraft:tune_noteblock";
    pub const POT_FLOWER: &str = "minecraft:pot_flower";
    pub const TRIGGER_TRAPPED_CHEST: &str = "minecraft:trigger_trapped_chest";
    pub const OPEN_ENDERCHEST: &str = "minecraft:open_enderchest";
    pub const ENCHANT_ITEM: &str = "minecraft:enchant_item";
    pub const PLAY_RECORD: &str = "minecraft:play_record";
    pub const INTERACT_WITH_FURNACE: &str = "minecraft:interact_with_furnace";
    pub const INTERACT_WITH_CRAFTING_TABLE: &str = "minecraft:interact_with_crafting_table";
    pub const OPEN_CHEST: &str = "minecraft:open_chest";
    pub const SLEEP_IN_BED: &str = "minecraft:sleep_in_bed";
    pub const OPEN_SHULKER_BOX: &str = "minecraft:open_shulker_box";
    pub const OPEN_BARREL: &str = "minecraft:open_barrel";
    pub const INTERACT_WITH_BLAST_FURNACE: &str = "minecraft:interact_with_blast_furnace";
    pub const INTERACT_WITH_SMOKER: &str = "minecraft:interact_with_smoker";
    pub const INTERACT_WITH_LECTERN: &str = "minecraft:interact_with_lectern";
    pub const INTERACT_WITH_CAMPFIRE: &str = "minecraft:interact_with_campfire";
    pub const INTERACT_WITH_CARTOGRAPHY_TABLE: &str = "minecraft:interact_with_cartography_table";
    pub const INTERACT_WITH_LOOM: &str = "minecraft:interact_with_loom";
    pub const INTERACT_WITH_STONECUTTER: &str = "minecraft:interact_with_stonecutter";
    pub const BELL_RING: &str = "minecraft:bell_ring";
    pub const RAID_TRIGGER: &str = "minecraft:raid_trigger";
    pub const RAID_WIN: &str = "minecraft:raid_win";
    pub const INTERACT_WITH_ANVIL: &str = "minecraft:interact_with_anvil";
    pub const INTERACT_WITH_GRINDSTONE: &str = "minecraft:interact_with_grindstone";
    pub const TARGET_HIT: &str = "minecraft:target_hit";
    pub const INTERACT_WITH_SMITHING_TABLE: &str = "minecraft:interact_with_smithing_table";
}

/// Manages all statistics for a single player.
///
/// Statistics are stored as a map from a composite key `"<category>:<id>"` to an `i32` value.
/// For custom stats the key is just the custom stat name (e.g. `"minecraft:jump"`).
/// For per-object stats the key is `"<category>:<registry_key>"` (e.g. `"mined:minecraft:stone"`).
pub struct StatisticsManager {
    /// Inner map protected by a Mutex so it can be mutated from async contexts.
    stats: Mutex<HashMap<String, i32>>,
    /// Tracks whether any stat changed since the last flush to the client.
    dirty: AtomicCell<bool>,
}

impl Default for StatisticsManager {
    fn default() -> Self {
        Self {
            stats: Mutex::new(HashMap::new()),
            dirty: AtomicCell::new(false),
        }
    }
}

impl StatisticsManager {
    /// Increment a statistic by `amount` and mark dirty.
    pub async fn increment(&self, key: &str, amount: i32) {
        let mut map = self.stats.lock().await;
        let entry = map.entry(key.to_string()).or_insert(0);
        *entry = entry.saturating_add(amount);
        self.dirty.store(true);
    }

    /// Set a statistic to an exact value and mark dirty.
    pub async fn set(&self, key: &str, value: i32) {
        let mut map = self.stats.lock().await;
        map.insert(key.to_string(), value);
        self.dirty.store(true);
    }

    /// Get the current value of a statistic (0 if not set).
    pub async fn get(&self, key: &str) -> i32 {
        *self.stats.lock().await.get(key).unwrap_or(&0)
    }

    /// Returns true if any stat changed since the last `build_packet` call.
    pub fn is_dirty(&self) -> bool {
        self.dirty.load()
    }

    /// Build a [`pumpkin_protocol::java::client::play::CAwardStats`] packet containing all current statistics.
    /// Clears the dirty flag.
    pub async fn build_full_packet(&self) -> pumpkin_protocol::java::client::play::CAwardStats {
        self.dirty.store(false);
        let map = self.stats.lock().await;
        let statistics = map
            .iter()
            .filter_map(|(key, &value)| parse_stat_entry(key, value))
            .collect();
        pumpkin_protocol::java::client::play::CAwardStats::new(statistics)
    }
}

/// Parse a stored stat key back into a `Statistic` for the protocol packet.
///
/// Key formats:
/// - `"custom:<name>"` → category 8, id = custom stat registry id
/// - `"mined:<block>"` → category 0, id = block registry id
/// - `"crafted:<item>"` → category 1
/// - `"used:<item>"` → category 2
/// - `"broken:<item>"` → category 3
/// - `"picked_up:<item>"` → category 4
/// - `"dropped:<item>"` → category 5
/// - `"killed:<entity>"` → category 6
/// - `"killed_by:<entity>"` → category 7
fn parse_stat_entry(key: &str, value: i32) -> Option<Statistic> {
    let (prefix, name) = key.split_once(':')?;
    // The name after the first colon may itself contain colons (e.g. "minecraft:stone")
    // so we need the full remainder.
    let full_name = &key[prefix.len() + 1..];

    let (cat_id, stat_id) = match prefix {
        "custom" => (category::CUSTOM, custom_stat_id(full_name)?),
        "mined" => (category::MINED, block_id_from_name(full_name)?),
        "crafted" => (category::CRAFTED, item_id_from_name(full_name)?),
        "used" => (category::USED, item_id_from_name(full_name)?),
        "broken" => (category::BROKEN, item_id_from_name(full_name)?),
        "picked_up" => (category::PICKED_UP, item_id_from_name(full_name)?),
        "dropped" => (category::DROPPED, item_id_from_name(full_name)?),
        "killed" => (category::KILLED, entity_id_from_name(full_name)?),
        "killed_by" => (category::KILLED_BY, entity_id_from_name(full_name)?),
        _ => return None,
    };
    let _ = name; // suppress unused warning
    Some(Statistic::new(cat_id.into(), stat_id.into(), value.into()))
}

/// Look up the protocol registry ID for a custom stat by its resource name.
/// The client uses the `minecraft:custom_stat` registry ordering.
/// Order matches Minestom's `StatisticTypes.java` (latest vanilla 1.21.x).
fn custom_stat_id(name: &str) -> Option<i32> {
    const CUSTOM_STATS: &[&str] = &[
        "minecraft:leave_game",                      // 0
        "minecraft:play_time",                       // 1
        "minecraft:total_world_time",                // 2
        "minecraft:time_since_death",                // 3
        "minecraft:time_since_rest",                 // 4
        "minecraft:sneak_time",                      // 5
        "minecraft:walk_one_cm",                     // 6
        "minecraft:crouch_one_cm",                   // 7
        "minecraft:sprint_one_cm",                   // 8
        "minecraft:walk_on_water_one_cm",            // 9
        "minecraft:fall_one_cm",                     // 10
        "minecraft:climb_one_cm",                    // 11
        "minecraft:fly_one_cm",                      // 12
        "minecraft:walk_under_water_one_cm",         // 13
        "minecraft:minecart_one_cm",                 // 14
        "minecraft:boat_one_cm",                     // 15
        "minecraft:pig_one_cm",                      // 16
        "minecraft:happy_ghast_one_cm",              // 17
        "minecraft:horse_one_cm",                    // 18
        "minecraft:aviate_one_cm",                   // 19
        "minecraft:swim_one_cm",                     // 20
        "minecraft:strider_one_cm",                  // 21
        "minecraft:nautilus_one_cm",                 // 22
        "minecraft:jump",                            // 23
        "minecraft:drop",                            // 24
        "minecraft:damage_dealt",                    // 25
        "minecraft:damage_dealt_absorbed",           // 26
        "minecraft:damage_dealt_resisted",           // 27
        "minecraft:damage_taken",                    // 28
        "minecraft:damage_blocked_by_shield",        // 29
        "minecraft:damage_absorbed",                 // 30
        "minecraft:damage_resisted",                 // 31
        "minecraft:deaths",                          // 32
        "minecraft:mob_kills",                       // 33
        "minecraft:animals_bred",                    // 34
        "minecraft:player_kills",                    // 35
        "minecraft:fish_caught",                     // 36
        "minecraft:talked_to_villager",              // 37
        "minecraft:traded_with_villager",            // 38
        "minecraft:eat_cake_slice",                  // 39
        "minecraft:fill_cauldron",                   // 40
        "minecraft:use_cauldron",                    // 41
        "minecraft:clean_armor",                     // 42
        "minecraft:clean_banner",                    // 43
        "minecraft:clean_shulker_box",               // 44
        "minecraft:interact_with_brewingstand",      // 45
        "minecraft:interact_with_beacon",            // 46
        "minecraft:inspect_dropper",                 // 47
        "minecraft:inspect_hopper",                  // 48
        "minecraft:inspect_dispenser",               // 49
        "minecraft:play_noteblock",                  // 50
        "minecraft:tune_noteblock",                  // 51
        "minecraft:pot_flower",                      // 52
        "minecraft:trigger_trapped_chest",           // 53
        "minecraft:open_enderchest",                 // 54
        "minecraft:enchant_item",                    // 55
        "minecraft:play_record",                     // 56
        "minecraft:interact_with_furnace",           // 57
        "minecraft:interact_with_crafting_table",    // 58
        "minecraft:open_chest",                      // 59
        "minecraft:sleep_in_bed",                    // 60
        "minecraft:open_shulker_box",                // 61
        "minecraft:open_barrel",                     // 62
        "minecraft:interact_with_blast_furnace",     // 63
        "minecraft:interact_with_smoker",            // 64
        "minecraft:interact_with_lectern",           // 65
        "minecraft:interact_with_campfire",          // 66
        "minecraft:interact_with_cartography_table", // 67
        "minecraft:interact_with_loom",              // 68
        "minecraft:interact_with_stonecutter",       // 69
        "minecraft:bell_ring",                       // 70
        "minecraft:raid_trigger",                    // 71
        "minecraft:raid_win",                        // 72
        "minecraft:interact_with_anvil",             // 73
        "minecraft:interact_with_grindstone",        // 74
        "minecraft:target_hit",                      // 75
        "minecraft:interact_with_smithing_table",    // 76
    ];
    CUSTOM_STATS
        .iter()
        .position(|&s| s == name)
        .map(|i| i as i32)
}

fn block_id_from_name(name: &str) -> Option<i32> {
    // Strip "minecraft:" prefix if present
    let key = name.strip_prefix("minecraft:").unwrap_or(name);
    pumpkin_data::Block::from_name(key).map(|b| i32::from(b.id))
}

fn item_id_from_name(name: &str) -> Option<i32> {
    let key = name.strip_prefix("minecraft:").unwrap_or(name);
    pumpkin_data::item::Item::from_registry_key(key)
        .or_else(|| pumpkin_data::item::Item::from_registry_key(name))
        .map(|i| i32::from(i.id))
}

fn entity_id_from_name(name: &str) -> Option<i32> {
    let key = name.strip_prefix("minecraft:").unwrap_or(name);
    pumpkin_data::entity::EntityType::from_name(key).map(|e| i32::from(e.id))
}

impl NBTStorage for StatisticsManager {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            let map = self.stats.lock().await;
            let mut stats_nbt = NbtCompound::new();
            for (key, &value) in map.iter() {
                // Use insert directly to always overwrite
                stats_nbt
                    .child_tags
                    .insert(key.as_str().into(), pumpkin_nbt::tag::NbtTag::Int(value));
            }
            // Insert into parent, overwriting any previous value
            nbt.child_tags.insert(
                "Statistics".into(),
                pumpkin_nbt::tag::NbtTag::Compound(stats_nbt),
            );
        })
    }

    fn read_nbt<'a>(&'a mut self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            if let Some(stats_nbt) = nbt.get_compound("Statistics") {
                let mut map = self.stats.lock().await;
                map.clear();
                for (key, tag) in &stats_nbt.child_tags {
                    if let Some(value) = tag.extract_int() {
                        map.insert(key.to_string(), value);
                    }
                }
            }
        })
    }
}

impl NBTStorageInit for StatisticsManager {}

/// Helper: build the stat key for a per-block stat.
#[must_use]
pub fn mined_key(block_name: &str) -> String {
    format!("mined:{block_name}")
}

/// Helper: build the stat key for a per-item stat.
#[must_use]
pub fn item_key(category: &str, item_name: &str) -> String {
    format!("{category}:{item_name}")
}

/// Helper: build the stat key for a per-entity stat.
#[must_use]
pub fn entity_key(category: &str, entity_name: &str) -> String {
    format!("{category}:{entity_name}")
}

/// Helper: build the stat key for a custom stat.
#[must_use]
pub fn custom_key(stat_name: &str) -> String {
    format!("custom:{stat_name}")
}
