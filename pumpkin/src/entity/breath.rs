use crate::entity::EntityBase;
use crate::entity::player::Player;
use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_data::tracked_data::{TrackedData, TrackedId};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

pub const MAX_AIR: i32 = 300;
pub const AIR_RECOVERY_RATE: i32 = 4;
pub const AIR_DEPLETION_RATE: i32 = 1;
pub const DROWNING_INTERVAL: i32 = 20;
pub const DROWNING_DAMAGE: f32 = 2.0;

const AIR_SUPPLY_TRACKED_DATA: TrackedId = TrackedId {
    v1_21: TrackedData::AIR.v1_21,
    v1_21_2: TrackedData::AIR.v1_21_2,
    v1_21_4: TrackedData::AIR.v1_21_4,
    v1_21_5: TrackedData::AIR.v1_21_5,
    v1_21_6: TrackedData::AIR.v1_21_6,
    v1_21_7: TrackedData::AIR.v1_21_7,
    v1_21_9: TrackedData::AIR.v1_21_9,
    v1_21_11: TrackedData::AIR.v1_21_11,
    v26_1: TrackedData::AIR_SUPPLY_ID.v26_1,
};

const fn air_supply_metadata(air: i32) -> [Metadata<VarInt>; 2] {
    [
        Metadata::new(AIR_SUPPLY_TRACKED_DATA, MetaDataType::INTEGER, VarInt(air)),
        Metadata::new(AIR_SUPPLY_TRACKED_DATA, MetaDataType::INT, VarInt(air)),
    ]
}

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
    pub async fn tick(&self, player: &Arc<Player>) {
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
            .await
        {
            if self.air_supply.swap(MAX_AIR, Ordering::Relaxed) != MAX_AIR {
                self.send_air_supply(player);
            }
            self.drowning_tick.store(0, Ordering::Relaxed);
            return;
        }

        let in_water = Self::is_eye_in_water(player);

        if in_water {
            let prev = self
                .air_supply
                .fetch_sub(AIR_DEPLETION_RATE, Ordering::Relaxed);
            let new_air = (prev - AIR_DEPLETION_RATE).max(0);
            if new_air != prev {
                self.air_supply.store(new_air, Ordering::Relaxed);
                self.send_air_supply(player);
            }

            if new_air <= 0 {
                let t = self.drowning_tick.fetch_add(1, Ordering::Relaxed) + 1;

                if t >= DROWNING_INTERVAL {
                    self.drowning_tick.store(0, Ordering::Relaxed);
                    player
                        .living_entity
                        .damage(player.as_ref(), DROWNING_DAMAGE, DamageType::DROWN)
                        .await;
                }
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

    fn is_eye_in_water(player: &Player) -> bool {
        let e = &player.get_entity();
        let pos = e.pos.load();
        let eye_y = e.get_eye_y();

        let bp = BlockPos::new(
            pos.x.floor() as i32,
            eye_y.floor() as i32,
            pos.z.floor() as i32,
        );
        let world = player.world();

        let (fluid, state) = world.get_fluid_and_fluid_state(&bp);

        let mut in_water_fluid = fluid.has_tag(&tag::Fluid::MINECRAFT_WATER);

        if !in_water_fluid {
            let state_here = world.get_block_state(&bp);
            if !state_here.is_solid() {
                let above = BlockPos::new(bp.0.x, bp.0.y + 1, bp.0.z);
                let fluid_above_x = world.get_fluid(&above);
                if fluid_above_x.has_tag(&tag::Fluid::MINECRAFT_WATER) {
                    in_water_fluid = true;
                }
            }
        }

        if !in_water_fluid {
            return false;
        }

        let above = BlockPos::new(bp.0.x, bp.0.y + 1, bp.0.z);
        let fluid_above = world.get_fluid(&above);

        let surface_y = if fluid_above.has_tag(&tag::Fluid::MINECRAFT_WATER) {
            f64::from(bp.0.y as f32 + 1.0)
        } else {
            let height: f32 = if state.is_still {
                1.0
            } else {
                let lvl = i32::from(state.level);
                if lvl >= 8 {
                    1.0
                } else {
                    ((8 - lvl).clamp(1, 8) as f32) / 8.0
                }
            };
            f64::from(bp.0.y as f32 + height)
        };

        surface_y > eye_y
    }

    pub fn send_air_supply(&self, player: &Player) {
        let air = self.air_supply.load(Ordering::Relaxed).clamp(0, MAX_AIR);

        player
            .get_entity()
            .send_meta_data(&air_supply_metadata(air));
    }

    pub fn reset(&self, player: &Player) {
        self.air_supply.store(MAX_AIR, Ordering::Relaxed);
        self.send_air_supply(player);
        self.drowning_tick.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::version::JavaMinecraftVersion;

    #[test]
    fn air_supply_metadata_writes_entity_air_index() {
        let versions = [
            JavaMinecraftVersion::V_1_21,
            JavaMinecraftVersion::V_1_21_2,
            JavaMinecraftVersion::V_1_21_4,
            JavaMinecraftVersion::V_1_21_5,
            JavaMinecraftVersion::V_1_21_6,
            JavaMinecraftVersion::V_1_21_7,
            JavaMinecraftVersion::V_1_21_9,
            JavaMinecraftVersion::V_1_21_11,
            JavaMinecraftVersion::V_26_1,
        ];

        for version in versions {
            assert_eq!(AIR_SUPPLY_TRACKED_DATA.get(&version), 1);

            let mut buf = Vec::new();
            for metadata in air_supply_metadata(MAX_AIR) {
                metadata.write(&mut buf, &version).unwrap();
            }

            assert_eq!(buf, vec![1, 1, 0xac, 0x02]);
        }
    }
}
