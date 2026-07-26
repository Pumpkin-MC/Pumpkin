use super::Player;
use crate::plugin::player::exp_change::PlayerExpChangeEvent;
use pumpkin_data::Enchantment;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::java::client::play::CSetExperience;
use pumpkin_protocol::java::client::play::CSetPlayerInventory;
use pumpkin_util::math::experience;
use pumpkin_world::inventory::Inventory;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::sync::Mutex;

impl Player {
    pub async fn tick_experience(&self) {
        if !self.has_client_loaded() {
            return;
        }

        let level = self.experience_level.load(Ordering::Relaxed);
        if self.last_sent_xp.load(Ordering::Relaxed) != level {
            let progress = self.experience_progress.load();
            let points = self.experience_points.load(Ordering::Relaxed);

            self.last_sent_xp.store(level, Ordering::Relaxed);

            self.client
                .send_packet_now(&CSetExperience::new(
                    progress.clamp(0.0, 1.0),
                    level.into(),
                    points.into(),
                ))
                .await;
        }
    }

    /// Sets the player's experience level and notifies the client.
    pub async fn set_experience(&self, level: i32, progress: f32, points: i32) {
        // TODO: These should be atomic together, not isolated; make a struct containing these. can cause ABA issues
        self.experience_level.store(level, Ordering::Relaxed);
        self.experience_progress.store(progress.clamp(0.0, 1.0));
        self.experience_points.store(points, Ordering::Relaxed);
        self.last_sent_xp.store(-1, Ordering::Relaxed);
        self.tick_experience().await;

        if self.has_client_loaded() {
            self.client
                .enqueue_packet(&CSetExperience::new(
                    progress.clamp(0.0, 1.0),
                    level.into(),
                    points.into(),
                ))
                .await;
        }
    }

    /// Sets the player's experience level directly.
    pub async fn set_experience_level(&self, new_level: i32, keep_progress: bool) {
        let progress = self.experience_progress.load();
        let mut points = self.experience_points.load(Ordering::Relaxed);

        // If `keep_progress` is `true` then calculate the number of points needed to keep the same progress scaled.
        if keep_progress {
            // Get our current level
            let current_level = self.experience_level.load(Ordering::Relaxed);
            let current_max_points = experience::points_in_level(current_level);
            // Calculate the max value for the new level
            let new_max_points = experience::points_in_level(new_level);
            // Calculate the scaling factor
            let scale = new_max_points as f32 / current_max_points as f32;
            // Scale the points (Vanilla doesn't seem to recalculate progress so we won't)
            points = (points as f32 * scale) as i32;
        }

        self.set_experience(new_level, progress, points).await;
    }

    /// Add experience levels to the player.
    pub async fn add_experience_levels(&self, added_levels: i32) {
        let current_level = self.experience_level.load(Ordering::Relaxed);
        let new_level = current_level + added_levels;
        self.set_experience_level(new_level, true).await;
    }

    /// Set the player's experience points directly. Returns `true` if successful.
    pub async fn set_experience_points(&self, new_points: i32) -> bool {
        let current_points = self.experience_points.load(Ordering::Relaxed);

        if new_points == current_points {
            return true;
        }

        let current_level = self.experience_level.load(Ordering::Relaxed);
        let max_points = experience::points_in_level(current_level);

        if new_points < 0 || new_points > max_points {
            return false;
        }

        let progress = new_points as f32 / max_points as f32;
        self.set_experience(current_level, progress, new_points)
            .await;
        true
    }

    /// Add experience points to the player.
    pub async fn add_experience_points(self: &Arc<Self>, mut added_points: i32) {
        if let Some(server) = self.world().server.upgrade() {
            let event = PlayerExpChangeEvent::new(self.clone(), added_points);
            let event = server.plugin_manager.fire(event).await;
            added_points = event.amount;
        }

        let current_level = self.experience_level.load(Ordering::Relaxed);
        let current_points = self.experience_points.load(Ordering::Relaxed);

        let total_exp = experience::points_to_level(current_level) as i64 + current_points as i64;
        let new_total_exp = total_exp + added_points as i64;
        let safe_new_total = new_total_exp.clamp(0, i32::MAX as i64) as i32;

        let (new_level, new_points) = experience::total_to_level_and_points(safe_new_total);
        let progress = experience::progress_in_level(new_points, new_level);

        self.set_experience(new_level, progress, new_points).await;
    }

    pub async fn apply_mending_from_xp(&self, mut xp: i32) -> i32 {
        if xp <= 0 {
            return xp;
        }

        let mut candidates: Vec<(usize, EquipmentSlot, Arc<Mutex<ItemStack>>)> = Vec::new();

        let selected_slot = self.inventory.get_selected_slot() as usize;
        let mut slot_pairs: Vec<(usize, EquipmentSlot)> = vec![
            (selected_slot, EquipmentSlot::MAIN_HAND),
            (PlayerInventory::OFF_HAND_SLOT, EquipmentSlot::OFF_HAND),
        ];
        for (slot_index, slot) in self.inventory.equipment_slots.iter() {
            if slot.is_armor_slot() {
                slot_pairs.push((*slot_index, slot.clone()));
            }
        }

        for (slot_index, equipment_slot) in slot_pairs {
            let stack = self.inventory.get_stack(slot_index).await;
            let eligible = {
                let s = stack.lock().await;
                s.get_enchantment_level(&Enchantment::MENDING) > 0 && s.get_damage() > 0
            };
            if eligible {
                candidates.push((slot_index, equipment_slot, stack));
            }
        }

        if candidates.is_empty() {
            return xp;
        }

        let idx = rand::random::<u32>() as usize % candidates.len();
        let (slot_index, equipment_slot, stack) = candidates.swap_remove(idx);

        let (updated_stack, repaired) = {
            let mut stack = stack.lock().await;
            let repaired = stack.repair_item(xp.saturating_mul(2));
            (stack.clone(), repaired)
        };

        if repaired <= 0 {
            return xp;
        }

        let xp_used = (repaired + 1) / 2;
        xp = xp.saturating_sub(xp_used);

        self.enqueue_slot_set_packet(&CSetPlayerInventory::new(
            (slot_index as i32).into(),
            &ItemStackSerializer::from(updated_stack.clone()),
        ))
        .await;

        self.living_entity
            .send_equipment_changes(&[(equipment_slot, updated_stack)]);

        xp
    }
}
