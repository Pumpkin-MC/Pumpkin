use crate::entity::EntityBase;
use crate::entity::player::Player;
use pumpkin_data::Block;
use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

pub struct BreathManager {
    pub air_supply: AtomicI32,
    pub drowning_tick: AtomicI32,
}

impl Default for BreathManager {
    fn default() -> Self {
        Self {
            air_supply: AtomicI32::new(300),
            drowning_tick: AtomicI32::new(0),
        }
    }
}

impl BreathManager {
    pub async fn tick(&self, player: &Arc<Player>) {
        let mode = player.gamemode.load();

        if matches!(mode, GameMode::Creative | GameMode::Spectator) {
            let prev = self.air_supply.load(Ordering::Relaxed);
            let new_air = (prev + 4).min(300);
            if new_air != prev {
                self.air_supply.store(new_air, Ordering::Relaxed);
                self.send_air_supply(player).await;
            }
            self.drowning_tick.store(0, Ordering::Relaxed);
            return;
        }

        if !player
            .world()
            .level_info
            .read()
            .await
            .game_rules
            .drowning_damage
        {
            return;
        }

        if player
            .living_entity
            .has_effect(&StatusEffect::WATER_BREATHING)
            .await
        {
            if self.air_supply.swap(300, Ordering::Relaxed) != 300 {
                self.send_air_supply(player).await;
            }
            self.drowning_tick.store(0, Ordering::Relaxed);
            return;
        }

        if self.is_eye_in_water(player).await {
            let prev = self.air_supply.fetch_sub(1, Ordering::Relaxed);
            let new_air = (prev - 1).max(0);
            if new_air != prev {
                self.send_air_supply(player).await;
            }

            if new_air <= 0 {
                let t = self.drowning_tick.fetch_add(1, Ordering::Relaxed) + 1;

                if t >= 20 {
                    self.drowning_tick.store(0, Ordering::Relaxed);
                    player
                        .living_entity
                        .damage(player.as_ref(), 2.0, DamageType::DROWN)
                        .await;
                }
            }
        } else {
            let prev = self.air_supply.load(Ordering::Relaxed);
            let new_air = (prev + 4).min(300);
            if new_air != prev {
                self.air_supply.store(new_air, Ordering::Relaxed);
                self.send_air_supply(player).await;
            }
            self.drowning_tick.store(0, Ordering::Relaxed);
        }
    }

    async fn is_eye_in_water(&self, player: &Player) -> bool {
        let e = &player.living_entity.entity;
        let pos = e.pos.load();

        let eye_y = e.get_eye_y();

        let bp = BlockPos::new(
            pos.x.floor() as i32,
            eye_y.floor() as i32,
            pos.z.floor() as i32,
        );

        let world = player.world();
        let (fluid, state) = world.get_fluid_and_fluid_state(&bp).await;

        let is_water_fluid = fluid.id == Fluid::WATER.id || fluid.id == Fluid::FLOWING_WATER.id;

        if !is_water_fluid {
            let block_here = world.get_block(&bp).await;

            let is_water_plant = block_here == &Block::SEAGRASS
                || block_here == &Block::TALL_SEAGRASS
                || block_here == &Block::KELP
                || block_here == &Block::KELP_PLANT;

            if !is_water_plant {
                return false;
            }
        }

        let above = BlockPos::new(bp.0.x, bp.0.y + 1, bp.0.z);
        let (fluid_above, _) = world.get_fluid_and_fluid_state(&above).await;

        let surface_y = if fluid_above.id == fluid.id {
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

    pub async fn send_air_supply(&self, player: &Player) {
        let air = self.air_supply.load(Ordering::Relaxed).clamp(0, 300);

        player
            .living_entity
            .entity
            .send_meta_data(&[Metadata::new(
                TrackedData::DATA_AIR,
                MetaDataType::Integer,
                VarInt(air),
            )])
            .await;
    }

    pub async fn reset(&self, player: &Player) {
        self.air_supply.store(300, Ordering::Relaxed);
        self.send_air_supply(player).await;
        self.drowning_tick.store(0, Ordering::Relaxed);
    }
}
