use std::sync::Arc;

use super::{EntityBase, NBTStorage, NBTStorageInit, player::Player};
use crate::entity::NbtFuture;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::Difficulty;

// Exhaustion and food constants
pub(crate) const MAX_FOOD: u8 = 20;
pub(crate) const EXHAUSTION_COST: f32 = 4.0;
pub(crate) const MAX_EXHAUSTION: f32 = 40.0;
pub(crate) const EXHAUSTION_HEAL: f32 = 6.0;
pub(crate) const EXHAUSTION_JUMP: f32 = 0.05;
pub(crate) const EXHAUSTION_SPRINT_JUMP: f32 = 0.2;
pub(crate) const EXHAUSTION_MINE: f32 = 0.005;
pub(crate) const EXHAUSTION_ATTACK: f32 = 0.1;
pub(crate) const EXHAUSTION_SPRINT: f32 = 0.1;
pub(crate) const EXHAUSTION_SWIM: f32 = 0.01;
pub(crate) const START_SATURATION: f32 = 5.0;

pub struct HungerManager {
    pub level: AtomicCell<u8>,
    pub saturation: AtomicCell<f32>,
    pub exhaustion: AtomicCell<f32>,
    pub tick_timer: AtomicCell<u32>,
}

impl Default for HungerManager {
    fn default() -> Self {
        Self {
            level: AtomicCell::new(MAX_FOOD),
            saturation: AtomicCell::new(START_SATURATION),
            exhaustion: AtomicCell::new(0.0),
            tick_timer: AtomicCell::new(0),
        }
    }
}

impl HungerManager {
    pub async fn tick(&self, player: &Arc<Player>) {
        let mut level = self.level.load();
        let mut saturation = self.saturation.load();
        let mut exhaustion = self.exhaustion.load();
        let mut timer = self.tick_timer.load();

        let difficulty = player.world().level_info.load().difficulty;
        let health = player.living_entity.health.load();
        let can_heal = player.can_food_heal();

        let mut needs_sync = false;
        let mut heal_amount = 0.0;
        let mut damage_amount = 0.0;

        if exhaustion > EXHAUSTION_COST {
            exhaustion -= EXHAUSTION_COST;
            if saturation > 0.0 {
                saturation = (saturation - 1.0).max(0.0);
            } else if difficulty != Difficulty::Peaceful {
                level = level.saturating_sub(1);
            }
            needs_sync = true;
        }

        let natural_regen = true; // TODO: GameRule check

        // In Peaceful difficulty hunger never depletes. If it's below max, slowly restore it.
        if difficulty == Difficulty::Peaceful {
            if level < MAX_FOOD || can_heal {
                timer += 1;
                if timer >= 20 {
                    // Restore Food
                    if level < MAX_FOOD {
                        level = (level + 1).min(MAX_FOOD);
                        saturation = START_SATURATION; // Full saturation
                    }

                    // Restore Health
                    if can_heal {
                        heal_amount = 1.0;
                    }

                    timer = 0;
                    needs_sync = true;
                }
            } else {
                timer = 0;
            }
        } else if natural_regen && saturation > 0.0 && can_heal && level >= 20 {
            // Saturation healing
            timer += 1;
            if timer >= 10 {
                let cost = saturation.min(EXHAUSTION_HEAL);
                saturation -= cost;
                exhaustion += cost;
                heal_amount = cost / EXHAUSTION_HEAL;
                timer = 0;
                needs_sync = true;
            }
        } else if natural_regen && level >= 18 && can_heal {
            // Standard healing
            timer += 1;
            if timer >= 80 {
                heal_amount = 1.0;
                exhaustion += EXHAUSTION_HEAL;
                timer = 0;
                needs_sync = true;
            }
        } else if level == 0 {
            // Starvation damage
            timer += 1;
            if timer >= 80 {
                timer = 0;
                let should_starve = match difficulty {
                    Difficulty::Peaceful => false,
                    Difficulty::Easy => health > 10.0,
                    Difficulty::Normal => health > 1.0,
                    Difficulty::Hard => true,
                };

                if should_starve {
                    damage_amount = 1.0;
                }
                //self.tick_timer.store(0);
            }
        } else {
            timer = 0;
        }

        // Write back changes
        if needs_sync || timer != self.tick_timer.load() {
            self.level.store(level);
            self.saturation.store(saturation);
            self.exhaustion.store(exhaustion);
            self.tick_timer.store(timer);
        }

        if needs_sync {
            player.send_health().await;
        }
        if heal_amount > 0.0 {
            player.heal(heal_amount).await;
        }
        if damage_amount > 0.0 {
            player
                .damage(&**player, damage_amount, DamageType::STARVE)
                .await;
        }
    }

    /// Apply eating effects.
    pub async fn eat(&self, player: &Player, food: u8, saturation: f32) {
        let added_saturation = saturation;

        let current_level = self.level.load();
        let current_sat = self.saturation.load();

        let new_level = (current_level + food).min(MAX_FOOD);

        // Saturation is clamped to the new food level
        let new_sat = (current_sat + added_saturation).min(f32::from(new_level));

        self.level.store(new_level);
        self.saturation.store(new_sat);

        player.send_health().await;
    }

    pub fn add_exhaustion(&self, exhaustion: f32) {
        let current = self.exhaustion.load();
        self.exhaustion
            .store((current + exhaustion).min(MAX_EXHAUSTION));
    }

    pub fn restart(&self) {
        self.level.store(MAX_FOOD);
        self.saturation.store(5.0);
        self.exhaustion.store(0.0);
        self.tick_timer.store(0);
    }
}

impl NBTStorage for HungerManager {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            nbt.put_int("foodLevel", self.level.load().into());
            nbt.put_float("foodSaturationLevel", self.saturation.load());
            nbt.put_float("foodExhaustionLevel", self.exhaustion.load());
            nbt.put_int("foodTickTimer", self.tick_timer.load() as i32);
        })
    }

    fn read_nbt<'a>(&'a mut self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.level
                .store(nbt.get_int("foodLevel").unwrap_or(20) as u8);
            self.saturation
                .store(nbt.get_float("foodSaturationLevel").unwrap_or(5.0));
            self.exhaustion
                .store(nbt.get_float("foodExhaustionLevel").unwrap_or(0.0));
            self.tick_timer
                .store(nbt.get_int("foodTickTimer").unwrap_or(0) as u32);
        })
    }
}

impl NBTStorageInit for HungerManager {}
