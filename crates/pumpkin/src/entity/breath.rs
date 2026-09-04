use crate::entity::EntityBase;
use crate::entity::player::Player;
use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::GameMode;
use std::sync::atomic::{AtomicI32, Ordering};

pub const MAX_AIR: i32 = 300;
pub const AIR_RECOVERY_RATE: i32 = 4;
pub const AIR_DEPLETION_RATE: i32 = 1;
pub const DROWNING_INTERVAL: i32 = 20;
pub const DROWNING_DAMAGE: f32 = 2.0;

pub struct BreathManager {
    pub air_supply: AtomicI32,
    pub drowning_tick: AtomicI32,
}

impl Default for BreathManager {
    fn default() -> Self {
        Self {
            air_supply: AtomicI32::new(MAX_AIR),
            drowning_tick: AtomicI32::new(0),
        }
    }
}

impl BreathManager {
    pub fn tick(&self, player: &Player) {
        let mode = player.gamemode.load();

        if matches!(mode, GameMode::Creative | GameMode::Spectator) {
            if self.air_supply.load(Ordering::Relaxed) != MAX_AIR {
                self.air_supply.store(MAX_AIR, Ordering::Relaxed);
                self.send_air_supply(player);
            }
            self.drowning_tick.store(0, Ordering::Relaxed);
            return;
        }

        if !player.world().level_info.load().game_rules.drowning_damage {
            return;
        }

        if player
            .living_entity
            .has_effect(&StatusEffect::WATER_BREATHING)
        {
            if self.air_supply.swap(MAX_AIR, Ordering::Relaxed) != MAX_AIR {
                self.send_air_supply(player);
            }
            self.drowning_tick.store(0, Ordering::Relaxed);
            return;
        }

        let in_water = Self::is_eye_in_water(player);
        let prev = self.air_supply.load(Ordering::Relaxed);

        if in_water {
            let mut new_air = (prev - AIR_DEPLETION_RATE).max(0);
            if new_air != prev {
                let server = player.world().server.upgrade();
                if let Some(server) = server {
                    let mut event = crate::plugin::api::events::entity::entity_air_change::EntityAirChangeEvent::new(
                        player.entity_id(),
                        new_air,
                    );
                    server.plugin_manager.fire_blocking(&server, &mut event);
                    if event.cancelled {
                        return;
                    }
                    new_air = event.amount.clamp(0, MAX_AIR);
                }
                self.air_supply.store(new_air, Ordering::Relaxed);
                self.send_air_supply(player);
            }

            if new_air <= 0 {
                let t = self.drowning_tick.fetch_add(1, Ordering::Relaxed) + 1;

                if t >= DROWNING_INTERVAL {
                    self.drowning_tick.store(0, Ordering::Relaxed);
                    player
                        .living_entity
                        .damage(player, DROWNING_DAMAGE, DamageType::DROWN);
                }
            }
        } else {
            let mut new_air = (prev + AIR_RECOVERY_RATE).min(MAX_AIR);
            if new_air != prev {
                let server = player.world().server.upgrade();
                if let Some(server) = server {
                    let mut event = crate::plugin::api::events::entity::entity_air_change::EntityAirChangeEvent::new(
                        player.entity_id(),
                        new_air,
                    );
                    server.plugin_manager.fire_blocking(&server, &mut event);
                    if event.cancelled {
                        return;
                    }
                    new_air = event.amount.clamp(0, MAX_AIR);
                }
                self.air_supply.store(new_air, Ordering::Relaxed);
                self.send_air_supply(player);
            }
            self.drowning_tick.store(0, Ordering::Relaxed);
        }
    }

    fn is_eye_in_water(player: &Player) -> bool {
        player.get_entity().is_submerged_in_water()
    }

    pub fn send_air_supply(&self, player: &Player) {
        let air = self.air_supply.load(Ordering::Relaxed).clamp(0, MAX_AIR);

        let mut bedrock_meta =
            pumpkin_protocol::bedrock::client::set_actor_data::SyncedActorDataList::new();
        bedrock_meta.set(
            pumpkin_protocol::bedrock::client::set_actor_data::entity_data_key::AIR_SUPPLY,
            pumpkin_protocol::bedrock::client::set_actor_data::MetadataValue::Short(air as i16),
        );

        player.get_entity().set_synced_data(
            pumpkin_data::tracked_data::entity::DATA_AIR_SUPPLY_ID,
            VarInt(air),
        );
        player.get_entity().send_bedrock_actor_data(&bedrock_meta);
    }

    pub fn reset(&self, player: &Player) {
        self.air_supply.store(MAX_AIR, Ordering::Relaxed);
        self.send_air_supply(player);
        self.drowning_tick.store(0, Ordering::Relaxed);
    }
}
