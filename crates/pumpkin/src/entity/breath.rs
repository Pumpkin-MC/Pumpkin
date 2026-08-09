use crate::entity::EntityBase;
use crate::entity::player::Player;
use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityStatus;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
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
    pub async fn tick(&self, player: &Player) {
        let mode = player.gamemode.load();

        if matches!(mode, GameMode::Creative | GameMode::Spectator) {
            let previous = self.air_supply.load(Ordering::Relaxed);
            let air = (previous + AIR_RECOVERY_RATE).min(MAX_AIR);
            if air != previous {
                self.air_supply.store(air, Ordering::Relaxed);
                self.send_air_supply(player);
            }
            self.drowning_tick.store(0, Ordering::Relaxed);
            return;
        }

        let world = player.world();
        let eye_block = {
            let entity = &player.get_entity();
            let pos = entity.pos.load();
            BlockPos::floored(pos.x, entity.get_eye_y(), pos.z)
        };
        let in_water = player.living_entity.is_eye_in_water(&world)
            && world.get_block(&eye_block) != &pumpkin_data::Block::BUBBLE_COLUMN;

        if in_water {
            let has_water_breathing = player
                .living_entity
                .has_effect(&StatusEffect::WATER_BREATHING)
                .await;
            let has_conduit_power = player
                .living_entity
                .has_effect(&StatusEffect::CONDUIT_POWER)
                .await;
            let has_breath_of_the_nautilus = player
                .living_entity
                .has_effect(&StatusEffect::BREATH_OF_THE_NAUTILUS)
                .await;
            let player_invulnerable = player.abilities.lock().await.invulnerable;
            if player.living_entity.dead.load(Ordering::Relaxed)
                || player.living_entity.health.load() <= 0.0
                || player.get_entity().is_removed()
            {
                return;
            }
            let can_breathe = has_water_breathing
                || has_conduit_power
                || has_breath_of_the_nautilus
                || player_invulnerable;
            let refill_air =
                !has_breath_of_the_nautilus || has_water_breathing || has_conduit_power;

            let prev = self.air_supply.load(Ordering::Relaxed);
            let new_air = if can_breathe {
                if refill_air {
                    (prev + AIR_RECOVERY_RATE).min(MAX_AIR)
                } else {
                    prev
                }
            } else {
                player.living_entity.decrease_air_supply(prev)
            };
            if new_air != prev {
                self.air_supply.store(new_air, Ordering::Relaxed);
                if let Some(server) = player.world().server.upgrade() {
                    let mut event = crate::plugin::api::events::entity::entity_air_change::EntityAirChangeEvent::new(
                        player.entity_id(),
                        new_air,
                    );
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current()
                            .block_on(server.plugin_manager.fire(&server, &mut event));
                    });
                    if event.cancelled {
                        self.air_supply.store(prev, Ordering::Relaxed);
                        return;
                    }
                    if player.living_entity.dead.load(Ordering::Relaxed)
                        || player.living_entity.health.load() <= 0.0
                        || player.get_entity().is_removed()
                    {
                        return;
                    }
                }
                self.send_air_supply(player);
            }

            if !can_breathe && new_air <= -20 {
                self.air_supply.store(0, Ordering::Relaxed);
                self.send_air_supply(player);
                world.send_entity_status(player.get_entity(), EntityStatus::DrownParticles);
                player
                    .living_entity
                    .damage(player, DROWNING_DAMAGE, DamageType::DROWN)
                    .await;
            }
        } else {
            let prev = self.air_supply.load(Ordering::Relaxed);
            let new_air = (prev + AIR_RECOVERY_RATE).min(MAX_AIR);
            if new_air != prev {
                self.air_supply.store(new_air, Ordering::Relaxed);
                self.send_air_supply(player);
            }
            self.drowning_tick.store(0, Ordering::Relaxed);
        }
    }

    pub fn send_air_supply(&self, player: &Player) {
        let air = self.air_supply.load(Ordering::Relaxed);

        let mut bedrock_meta =
            pumpkin_protocol::bedrock::client::set_actor_data::EntityMetadata::new();
        bedrock_meta.set(
            pumpkin_protocol::bedrock::client::set_actor_data::entity_data_key::AIR_SUPPLY,
            pumpkin_protocol::bedrock::client::set_actor_data::MetadataValue::Short(
                air.clamp(0, i32::from(i16::MAX)) as i16,
            ),
        );

        player.get_entity().send_meta_data(
            &[Metadata::new(
                TrackedData::AIR_SUPPLY_ID,
                MetaDataType::INT,
                VarInt(air),
            )],
            Some(&bedrock_meta),
        );
    }

    pub fn reset(&self, player: &Player) {
        self.air_supply.store(MAX_AIR, Ordering::Relaxed);
        self.send_air_supply(player);
        self.drowning_tick.store(0, Ordering::Relaxed);
    }
}
