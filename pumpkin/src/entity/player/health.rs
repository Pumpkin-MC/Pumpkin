use super::Player;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::potion::Effect;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::CSetHealth;
use pumpkin_protocol::java::client::play::CUpdateMobEffect;
use std::sync::atomic::Ordering;

impl Player {
    pub fn can_food_heal(&self) -> bool {
        let health = self.living_entity.health.load();
        let max_health = self.living_entity.get_max_health();
        health > 0.0 && health < max_health
    }

    pub async fn add_exhaustion(&self, exhaustion: f32) {
        if self.abilities.lock().await.invulnerable {
            return;
        }
        self.hunger_manager.add_exhaustion(exhaustion);
    }

    pub async fn heal(&self, additional_health: f32) {
        self.living_entity.heal(additional_health);
        self.send_health().await;
    }

    pub async fn send_health(&self) {
        if !self.has_client_loaded() {
            return;
        }

        self.client
            .enqueue_packet_editioned(
                &CSetHealth::new(
                    self.living_entity.health.load(),
                    self.hunger_manager.level.load().into(),
                    self.hunger_manager.saturation.load(),
                ),
                &pumpkin_protocol::bedrock::client::set_health::CSetHealth::new(
                    self.living_entity.health.load() as i32,
                ),
            )
            .await;
    }

    pub async fn tick_health(&self) {
        if !self.has_client_loaded() {
            return;
        }

        let health = self.living_entity.health.load() as i32;
        let food = self.hunger_manager.level.load();
        let saturation = self.hunger_manager.saturation.load();

        let last_health = self.last_sent_health.load(Ordering::Relaxed);
        let last_food = self.last_sent_food.load(Ordering::Relaxed);
        let last_saturation = self.last_food_saturation.load(Ordering::Relaxed);

        if health != last_health || food != last_food || (saturation == 0.0) != last_saturation {
            self.last_sent_health.store(health, Ordering::Relaxed);
            self.last_sent_food.store(food, Ordering::Relaxed);
            self.last_food_saturation
                .store(saturation == 0.0, Ordering::Relaxed);
            self.send_health().await;
        }
    }

    pub async fn set_health(&self, health: f32) {
        self.living_entity.set_health(health);
        self.send_health().await;
    }

    pub async fn set_max_health(&self, max_health: f32) {
        self.living_entity.set_max_health(max_health).await;
        self.send_health().await;
    }

    pub async fn set_food_level(&self, food_level: u8) {
        self.hunger_manager.set_level(food_level);
        self.send_health().await;
    }

    pub async fn set_saturation(&self, saturation: f32) {
        self.hunger_manager.set_saturation(saturation);
        self.send_health().await;
    }

    pub fn get_exhaustion(&self) -> f32 {
        self.hunger_manager.get_exhaustion()
    }

    pub async fn set_exhaustion(&self, exhaustion: f32) {
        self.hunger_manager.set_exhaustion(exhaustion);
        self.send_health().await;
    }

    pub fn get_absorption(&self) -> f32 {
        self.living_entity.get_absorption()
    }

    pub async fn set_absorption(&self, absorption: f32) {
        self.living_entity.set_absorption(absorption).await;
    }

    pub async fn add_effect(&self, effect: Effect) {
        self.living_entity.add_effect(effect).await;
    }

    pub async fn send_active_effects(&self) {
        let effects = self.living_entity.active_effects.lock().await;
        for effect in effects.values() {
            self.send_effect(effect.clone()).await;
        }
    }

    /**
     * Send a clientside only effect to the player.
     * It won't be tracked on the server.
     */
    pub async fn send_effect(&self, effect: Effect) {
        let mut flag: i8 = 0;

        if effect.ambient {
            flag |= 1;
        }
        if effect.show_particles {
            flag |= 2;
        }
        if effect.show_icon {
            flag |= 4;
        }
        if effect.blend {
            flag |= 8;
        }

        let effect_id = VarInt(i32::from(effect.effect_type.id));
        self.client
            .enqueue_packet(&CUpdateMobEffect::new(
                self.entity_id().into(),
                effect_id,
                effect.amplifier.into(),
                effect.duration.into(),
                flag,
            ))
            .await;
    }

    pub async fn remove_effect(&self, effect_type: &'static StatusEffect) -> bool {
        let effect_id = VarInt(i32::from(effect_type.id));
        self.client
            .enqueue_packet(
                &pumpkin_protocol::java::client::play::CRemoveMobEffect::new(
                    self.entity_id().into(),
                    effect_id,
                ),
            )
            .await;

        self.living_entity.remove_effect(effect_type).await

        // TODO broadcast metadata
    }

    pub async fn remove_all_effects(&self) -> bool {
        let mut succeeded = false;
        let mut effect_list = vec![];
        for effect in self.living_entity.active_effects.lock().await.keys() {
            effect_list.push(*effect);
            let effect_id = VarInt(i32::from(effect.id));
            self.client
                .enqueue_packet(
                    &pumpkin_protocol::java::client::play::CRemoveMobEffect::new(
                        self.entity_id().into(),
                        effect_id,
                    ),
                )
                .await;
            succeeded = true;
        }

        // Need to remove effects afterward here because there would be a deadlock if this is done in the for loop.
        for effect in effect_list {
            self.living_entity.remove_effect(effect).await;
        }

        succeeded
    }
}
