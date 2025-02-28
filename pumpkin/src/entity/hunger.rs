use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;

use super::{EntityBase, player::Player};

pub struct HungerManager {
    /// The current hunger level.
    pub level: AtomicCell<u32>,
    /// The food saturation level.
    pub saturation: AtomicCell<f32>,
    exhaustion: AtomicCell<f32>,
    tick_timer: AtomicCell<u32>,
}

impl Default for HungerManager {
    fn default() -> Self {
        Self {
            level: AtomicCell::new(20),
            saturation: AtomicCell::new(5.0),
            exhaustion: AtomicCell::new(0.0),
            tick_timer: AtomicCell::new(0),
        }
    }
}

impl HungerManager {
    pub async fn tick(&self, player: &Player) {
        let saturation = self.saturation.load();
        let level = self.level.load();
        let exhaustion = self.exhaustion.load();
        // Decrease hunger level on exhaustion
        if level != 0 && exhaustion > 4.0 {
            self.exhaustion.store(exhaustion - 4.0);
            if saturation > 0.0 {
                self.saturation.store((saturation - 1.0).max(0.0));
            } else {
                self.level.store(level - 1);
                player.send_health().await;
            }
        }
        // Heal when hunger is full
        if saturation > 0.0 && player.can_food_heal() && level >= 20 {
            self.tick_timer.fetch_add(1);
            if self.tick_timer.load() >= 10 {
                let saturation = saturation.min(6.0);
                player.heal(saturation / 6.0).await;
                self.add_exhausten(saturation);
                self.tick_timer.store(0);
            }
        } else if level == 0 {
            self.tick_timer.fetch_add(1);
            if self.tick_timer.load() >= 80 {
                player.damage(1.0, DamageType::STARVE).await;
                self.tick_timer.store(0);
            }
        } else {
            self.tick_timer.store(0);
        }
    }

    pub fn add_exhausten(&self, exhaustion: f32) {
        self.exhaustion
            .store((self.exhaustion.load() + exhaustion).min(40.0));
    }
}
