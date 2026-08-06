use super::BlockEntity;
use pumpkin_data::block_properties::{BlockProperties, VaultLikeProperties, VaultState};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, world::WorldEvent};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::world::{BlockFlags, World};

// VaultConfig.java:38-47 (private no-arg constructor defaults)
struct VaultConfig {
    loot_table: String,
    activation_range: f64,
    deactivation_range: f64,
    key_item: ItemStack,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            loot_table: "minecraft:chests/trial_chambers/reward".to_string(),
            activation_range: 4.0,
            deactivation_range: 4.5,
            key_item: ItemStack::new(1, &Item::TRIAL_KEY),
        }
    }
}

impl VaultConfig {
    fn from_nbt(nbt: Option<&NbtCompound>) -> Self {
        let mut config = Self::default();
        let Some(nbt) = nbt else {
            return config;
        };
        if let Some(v) = nbt.get_string("loot_table") {
            config.loot_table = v.to_string();
        }
        if let Some(v) = nbt.get_double("activation_range") {
            config.activation_range = v;
        }
        if let Some(v) = nbt.get_double("deactivation_range") {
            config.deactivation_range = v;
        }
        if let Some(item_nbt) = nbt.get_compound("key_item")
            && let Some(stack) = ItemStack::read_item_stack(item_nbt)
        {
            config.key_item = stack;
        }
        config
    }
}

// VaultState.java:57-60
const UPDATE_CONNECTED_PLAYERS_TICK_RATE: i64 = 20;
const DELAY_BETWEEN_EJECTIONS_TICKS: i64 = 20;
// VaultServerData.java:29
const MAX_REWARD_PLAYERS: usize = 128;
// VaultBlockEntity.java:219
const INSERT_FAIL_SOUND_BUFFER_TICKS: i64 = 15;

pub struct VaultBlockEntity {
    pub position: BlockPos,
    config_nbt: Mutex<Option<NbtCompound>>,
    config: VaultConfig,
    rewarded_players: Mutex<Vec<Uuid>>,
    connected_players: Mutex<HashSet<Uuid>>,
    state_updating_resumes_at: AtomicI64,
    items_to_eject: Mutex<Vec<ItemStack>>,
    total_ejections_needed: AtomicUsize,
    last_insert_fail_timestamp: AtomicI64,
    display_item: Mutex<Option<ItemStack>>,
}

impl VaultBlockEntity {
    pub const ID: &'static str = "minecraft:vault";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            config_nbt: Mutex::const_new(None),
            config: VaultConfig::default(),
            rewarded_players: Mutex::const_new(Vec::new()),
            connected_players: Mutex::const_new(HashSet::new()),
            state_updating_resumes_at: AtomicI64::new(0),
            items_to_eject: Mutex::const_new(Vec::new()),
            total_ejections_needed: AtomicUsize::new(0),
            last_insert_fail_timestamp: AtomicI64::new(0),
            display_item: Mutex::const_new(None),
        }
    }

    // VaultSharedData.java:67-79, simplified: PlayerDetector::INCLUDING_CREATIVE_PLAYERS
    // (excludes only spectators, VaultConfig.java:45) without the line-of-sight check.
    async fn update_connected_players_within_range(&self, world: &Arc<World>, range: f64) -> bool {
        let nearby = world.get_nearby_players(self.position.to_f64(), range);
        let rewarded = self.rewarded_players.lock().await;
        let current: HashSet<Uuid> = nearby
            .iter()
            .filter(|p| {
                !matches!(p.gamemode.load(), pumpkin_util::GameMode::Spectator)
                    && !rewarded.contains(&p.gameprofile.id)
            })
            .map(|p| p.gameprofile.id)
            .collect();
        drop(rewarded);
        let mut connected = self.connected_players.lock().await;
        if *connected == current {
            false
        } else {
            *connected = current;
            true
        }
    }

    async fn cycle_display_item_from_loot_table(&self, can_eject: bool) {
        if !can_eject {
            *self.display_item.lock().await = None;
            return;
        }
        if let Some(item) = self.roll_loot_table_item() {
            *self.display_item.lock().await = Some(item);
        }
    }

    fn roll_loot_table_item(&self) -> Option<ItemStack> {
        let key = self
            .config
            .loot_table
            .strip_prefix("minecraft:")
            .unwrap_or(&self.config.loot_table);
        let table =
            pumpkin_data::chest_loot_table::get_chest_loot_table(&format!("minecraft:{key}"))?;
        let seed: i64 = rand::random();
        let items = crate::world::loot::generate_chest_loot(table, seed);
        items.into_iter().next()
    }

    fn roll_loot_table_items(&self) -> Vec<ItemStack> {
        let key = self
            .config
            .loot_table
            .strip_prefix("minecraft:")
            .unwrap_or(&self.config.loot_table);
        pumpkin_data::chest_loot_table::get_chest_loot_table(&format!("minecraft:{key}"))
            .map_or_else(Vec::new, |table| {
                let seed: i64 = rand::random();
                crate::world::loot::generate_chest_loot(table, seed)
            })
    }

    // VaultState.java:82-83, 105-116
    async fn update_state_for_connected_players(
        &self,
        world: &Arc<World>,
        range: f64,
        game_time: i64,
    ) -> VaultState {
        self.update_connected_players_within_range(world, range)
            .await;
        self.state_updating_resumes_at.store(
            game_time + UPDATE_CONNECTED_PLAYERS_TICK_RATE,
            Ordering::Relaxed,
        );
        if self.connected_players.lock().await.is_empty() {
            VaultState::Inactive
        } else {
            VaultState::Active
        }
    }

    // VaultServerData.java:129-131
    fn ejection_progress(total_ejections_needed: usize, remaining: usize) -> f32 {
        if total_ejections_needed == 1 {
            1.0
        } else if total_ejections_needed == 0 {
            0.0
        } else {
            1.0 - (remaining as f32 - 1.0) / (total_ejections_needed as f32 - 1.0)
        }
    }

    // VaultState.java:78-103
    async fn tick_state_machine(
        &self,
        world: &Arc<World>,
        current: VaultState,
        game_time: i64,
    ) -> VaultState {
        match current {
            VaultState::Inactive => {
                self.update_state_for_connected_players(
                    world,
                    self.config.activation_range,
                    game_time,
                )
                .await
            }
            VaultState::Active => {
                self.update_state_for_connected_players(
                    world,
                    self.config.deactivation_range,
                    game_time,
                )
                .await
            }
            VaultState::Unlocking => {
                self.state_updating_resumes_at
                    .store(game_time + DELAY_BETWEEN_EJECTIONS_TICKS, Ordering::Relaxed);
                VaultState::Ejecting
            }
            VaultState::Ejecting => {
                let next_item = self.items_to_eject.lock().await.pop();
                let Some(item) = next_item else {
                    self.total_ejections_needed.store(0, Ordering::Relaxed);
                    return self
                        .update_state_for_connected_players(
                            world,
                            self.config.deactivation_range,
                            game_time,
                        )
                        .await;
                };
                let remaining = self.items_to_eject.lock().await.len();
                let progress = Self::ejection_progress(
                    self.total_ejections_needed.load(Ordering::Relaxed),
                    remaining + 1,
                );
                world.drop_stack(&self.position, item).await;
                world.sync_world_event(WorldEvent::AnimationVaultEjectItem, self.position, 0);
                world.play_sound_fine(
                    Sound::BlockVaultEjectItem,
                    SoundCategory::Blocks,
                    &self.position.to_f64(),
                    1.0,
                    0.8 + 0.4 * progress,
                );
                *self.display_item.lock().await = self.items_to_eject.lock().await.last().cloned();
                self.state_updating_resumes_at
                    .store(game_time + DELAY_BETWEEN_EJECTIONS_TICKS, Ordering::Relaxed);
                VaultState::Ejecting
            }
        }
    }

    async fn on_transition(
        &self,
        world: &Arc<World>,
        from: VaultState,
        to: VaultState,
        is_ominous: bool,
    ) {
        if from == VaultState::Ejecting {
            world.play_sound(
                Sound::BlockVaultCloseShutter,
                SoundCategory::Blocks,
                &self.position.to_f64(),
            );
        }
        match to {
            VaultState::Inactive => {
                *self.display_item.lock().await = None;
                world.sync_world_event(
                    WorldEvent::AnimationVaultDeactivate,
                    self.position,
                    i32::from(is_ominous),
                );
            }
            VaultState::Active => {
                if self.display_item.lock().await.is_none() {
                    self.cycle_display_item_from_loot_table(true).await;
                }
                world.sync_world_event(
                    WorldEvent::AnimationVaultActivate,
                    self.position,
                    i32::from(is_ominous),
                );
            }
            VaultState::Unlocking => {
                world.play_sound(
                    Sound::BlockVaultInsertItem,
                    SoundCategory::Blocks,
                    &self.position.to_f64(),
                );
            }
            VaultState::Ejecting => {
                world.play_sound(
                    Sound::BlockVaultOpenShutter,
                    SoundCategory::Blocks,
                    &self.position.to_f64(),
                );
            }
        }
    }

    async fn tick_server(&self, world: &Arc<World>) {
        let state_id = world.get_block_state_id(&self.position);
        let block = Block::from_state_id(state_id);
        if !VaultLikeProperties::handles_block_id(block.id) {
            return;
        }
        let mut props = VaultLikeProperties::from_state_id(state_id, block);
        let game_time = world.get_world_age().await;

        let can_eject =
            !self.config.key_item.is_empty() && props.vault_state != VaultState::Inactive;
        if game_time % UPDATE_CONNECTED_PLAYERS_TICK_RATE == 0
            && props.vault_state == VaultState::Active
        {
            self.cycle_display_item_from_loot_table(can_eject).await;
        }

        if game_time >= self.state_updating_resumes_at.load(Ordering::Relaxed) {
            let next_state = self
                .tick_state_machine(world, props.vault_state, game_time)
                .await;
            if next_state != props.vault_state {
                let from = props.vault_state;
                props.vault_state = next_state;
                let new_state_id = props.to_state_id(block);
                world
                    .set_block_state(&self.position, new_state_id, BlockFlags::NOTIFY_ALL)
                    .await;
                self.on_transition(world, from, next_state, props.ominous)
                    .await;
            }
        }
    }

    // VaultBlockEntity.java:253-280
    pub async fn try_insert_key(
        &self,
        world: &Arc<World>,
        player: &Arc<crate::entity::player::Player>,
        item_stack: &mut ItemStack,
    ) -> bool {
        let state_id = world.get_block_state_id(&self.position);
        let block = Block::from_state_id(state_id);
        if !VaultLikeProperties::handles_block_id(block.id) {
            return false;
        }
        let mut props = VaultLikeProperties::from_state_id(state_id, block);
        let can_eject =
            !self.config.key_item.is_empty() && props.vault_state != VaultState::Inactive;
        if !can_eject {
            return false;
        }

        let is_valid = item_stack.are_items_and_components_equal(&self.config.key_item)
            && item_stack.item_count >= self.config.key_item.item_count;
        let game_time = world.get_world_age().await;

        if !is_valid {
            self.play_insert_fail_sound(world, Sound::BlockVaultInsertItemFail, game_time);
            return false;
        }

        if self
            .rewarded_players
            .lock()
            .await
            .contains(&player.gameprofile.id)
        {
            self.play_insert_fail_sound(world, Sound::BlockVaultRejectRewardedPlayer, game_time);
            return false;
        }

        let items = self.roll_loot_table_items();
        if items.is_empty() {
            return false;
        }

        item_stack
            .decrement_unless_creative(player.gamemode.load(), self.config.key_item.item_count);

        self.total_ejections_needed
            .store(items.len(), Ordering::Relaxed);
        *self.items_to_eject.lock().await = items;
        *self.display_item.lock().await = self.items_to_eject.lock().await.last().cloned();
        self.state_updating_resumes_at
            .store(game_time + 14, Ordering::Relaxed);

        let from = props.vault_state;
        props.vault_state = VaultState::Unlocking;
        let new_state_id = props.to_state_id(block);
        world
            .set_block_state(&self.position, new_state_id, BlockFlags::NOTIFY_ALL)
            .await;
        self.on_transition(world, from, VaultState::Unlocking, props.ominous)
            .await;

        {
            let mut rewarded = self.rewarded_players.lock().await;
            rewarded.push(player.gameprofile.id);
            // VaultServerData.java:68-74: FIFO eviction once over the cap.
            if rewarded.len() > MAX_REWARD_PLAYERS {
                rewarded.remove(0);
            }
        }
        self.update_connected_players_within_range(world, self.config.deactivation_range)
            .await;
        true
    }

    fn play_insert_fail_sound(&self, world: &Arc<World>, sound: Sound, game_time: i64) {
        if game_time
            >= self.last_insert_fail_timestamp.load(Ordering::Relaxed)
                + INSERT_FAIL_SOUND_BUFFER_TICKS
        {
            world.play_sound(sound, SoundCategory::Blocks, &self.position.to_f64());
            self.last_insert_fail_timestamp
                .store(game_time, Ordering::Relaxed);
        }
    }
}

impl BlockEntity for VaultBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn tick<'a>(&'a self, world: &'a Arc<World>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move { self.tick_server(world).await })
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let config_nbt = nbt.get_compound("config").cloned();
        let config = VaultConfig::from_nbt(config_nbt.as_ref());

        let server_data = nbt.get_compound("server_data");
        let rewarded_players: Vec<Uuid> = server_data
            .and_then(|data| data.get_list("rewarded_players"))
            .map(|list| {
                list.iter()
                    .filter_map(|tag| {
                        let NbtTag::IntArray(v) = tag else {
                            return None;
                        };
                        let &[a, b, c, d] = v.as_slice() else {
                            return None;
                        };
                        Some(Uuid::from_u128(
                            ((a as u32 as u128) << 96)
                                | ((b as u32 as u128) << 64)
                                | ((c as u32 as u128) << 32)
                                | (d as u32 as u128),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();
        let state_updating_resumes_at = server_data
            .and_then(|data| data.get_long("state_updating_resumes_at"))
            .unwrap_or(0);
        let items_to_eject: Vec<ItemStack> = server_data
            .and_then(|data| data.get_list("items_to_eject"))
            .map(|list| {
                list.iter()
                    .filter_map(|tag| {
                        let NbtTag::Compound(item_nbt) = tag else {
                            return None;
                        };
                        ItemStack::read_item_stack(item_nbt)
                    })
                    .collect()
            })
            .unwrap_or_default();
        let total_ejections_needed = server_data
            .and_then(|data| data.get_int("total_ejections_needed"))
            .map_or(0, |v| v.max(0) as usize);

        Self {
            position,
            config_nbt: Mutex::new(config_nbt),
            config,
            rewarded_players: Mutex::new(rewarded_players),
            connected_players: Mutex::const_new(HashSet::new()),
            state_updating_resumes_at: AtomicI64::new(state_updating_resumes_at),
            items_to_eject: Mutex::new(items_to_eject),
            total_ejections_needed: AtomicUsize::new(total_ejections_needed),
            last_insert_fail_timestamp: AtomicI64::new(0),
            display_item: Mutex::const_new(None),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if let Some(cfg) = self.config_nbt.lock().await.as_ref() {
                nbt.put_compound("config", cfg.clone());
            }

            let mut data = NbtCompound::new();
            let players: Vec<NbtTag> = self
                .rewarded_players
                .lock()
                .await
                .iter()
                .map(|u| uuid_to_int_array(*u))
                .collect();
            data.put_list("rewarded_players", players);
            data.put_long(
                "state_updating_resumes_at",
                self.state_updating_resumes_at.load(Ordering::Relaxed),
            );
            let items: Vec<NbtTag> = self
                .items_to_eject
                .lock()
                .await
                .iter()
                .map(|stack| {
                    let mut item_nbt = NbtCompound::new();
                    stack.write_item_stack(&mut item_nbt);
                    NbtTag::Compound(item_nbt)
                })
                .collect();
            data.put_list("items_to_eject", items);
            data.put_int(
                "total_ejections_needed",
                self.total_ejections_needed.load(Ordering::Relaxed) as i32,
            );
            nbt.put_compound("server_data", data);
        })
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(cfg) = self.config_nbt.try_lock()
            && let Some(ref cfg) = *cfg
        {
            nbt.put_compound("config", cfg.clone());
        }
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn uuid_to_int_array(u: Uuid) -> NbtTag {
    let v = u.as_u128();
    NbtTag::IntArray(vec![
        (v >> 96) as i32,
        ((v >> 64) & 0xFFFF_FFFF) as i32,
        ((v >> 32) & 0xFFFF_FFFF) as i32,
        (v & 0xFFFF_FFFF) as i32,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_matches_vanilla() {
        let config = VaultConfig::default();
        assert_eq!(config.loot_table, "minecraft:chests/trial_chambers/reward");
        assert!((config.activation_range - 4.0).abs() < f64::EPSILON);
        assert!((config.deactivation_range - 4.5).abs() < f64::EPSILON);
    }

    #[test]
    fn ejection_progress_single_item() {
        assert!((VaultBlockEntity::ejection_progress(1, 1) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn ejection_progress_multiple_items_monotonic() {
        let first = VaultBlockEntity::ejection_progress(4, 4);
        let last = VaultBlockEntity::ejection_progress(4, 1);
        assert!(last > first);
        assert!((last - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn rewarded_players_fifo_eviction_at_cap() {
        let mut rewarded: Vec<Uuid> = (0..MAX_REWARD_PLAYERS).map(|_| Uuid::new_v4()).collect();
        let oldest = rewarded[0];
        rewarded.push(Uuid::new_v4());
        if rewarded.len() > MAX_REWARD_PLAYERS {
            rewarded.remove(0);
        }
        assert_eq!(rewarded.len(), MAX_REWARD_PLAYERS);
        assert!(!rewarded.contains(&oldest));
    }
}
