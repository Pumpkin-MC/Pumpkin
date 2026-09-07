use crate::entity::EntityBase;
use crate::entity::living::LivingEntity;
use crate::entity::{Entity, mob::Mob};
use pumpkin_data::Block;
use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use std::sync::atomic::{AtomicI32, Ordering};

/// `LivingEntity.getMaxAirSupply`.
pub const MAX_AIR: i32 = 300;
pub const AIR_RECOVERY_RATE: i32 = 4;
pub const AIR_DEPLETION_RATE: i32 = 1;
pub const DROWNING_INTERVAL: i32 = 20;
pub const DROWNING_DAMAGE: f32 = 2.0;

/// Air supply, from `LivingEntity.baseTick` and `WaterAnimal.baseTick`.
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
    /// `LivingEntity.getMaxAirSupply`.
    #[must_use]
    pub fn max_air_supply(caller: &dyn EntityBase) -> i32 {
        caller.get_mob().map_or(MAX_AIR, Mob::max_air_supply)
    }

    /// `MobEffectUtil.hasWaterBreathing`.
    #[must_use]
    pub fn has_water_breathing(living: &LivingEntity) -> bool {
        living.has_effect(&StatusEffect::WATER_BREATHING)
            || living.has_effect(&StatusEffect::CONDUIT_POWER)
    }

    /// One tick of air change and suffocation damage.
    pub fn tick(&self, living: &LivingEntity, caller: &dyn EntityBase) {
        let entity = &living.entity;
        let mob = caller.get_mob();
        let max_air = Self::max_air_supply(caller);

        if let Some(player) = caller.get_player()
            && matches!(
                player.gamemode.load(),
                GameMode::Creative | GameMode::Spectator
            )
        {
            self.refill(entity, max_air);
            return;
        }

        if !entity
            .world
            .load()
            .level_info
            .load()
            .game_rules
            .drowning_damage
        {
            return;
        }

        let breathes_underwater =
            mob.is_some_and(Mob::can_breathe_underwater) || Self::has_water_breathing(living);
        let drowning = !breathes_underwater
            && Self::is_eye_in_water(entity)
            && !Self::is_eye_in_bubble_column(entity);
        let dries_out = mob.is_some_and(Mob::dries_out_on_land);
        let hydrated = if mob.is_some_and(Mob::rehydrates_in_rain) {
            entity.is_in_water_rain_or_bubble()
        } else {
            entity.is_in_water_or_bubble()
        };
        let drying_out = dries_out && !hydrated;

        let (suffocating, damage_type) = if drying_out {
            (true, DamageType::DRY_OUT)
        } else {
            (drowning, DamageType::DROWN)
        };

        if !suffocating {
            let prev = self.air_supply.load(Ordering::Relaxed);
            let new_air = if dries_out {
                max_air
            } else {
                mob.map_or_else(
                    || (prev + AIR_RECOVERY_RATE).min(max_air),
                    |mob| mob.increase_air_supply(prev),
                )
                .clamp(0, max_air)
            };
            if new_air != prev {
                self.set_air(entity, new_air, max_air);
            }
            self.drowning_tick.store(0, Ordering::Relaxed);
            return;
        }

        let prev = self.air_supply.load(Ordering::Relaxed);
        let new_air = (prev - AIR_DEPLETION_RATE).max(0);
        if new_air != prev {
            self.set_air(entity, new_air, max_air);
        }

        if self.air_supply.load(Ordering::Relaxed) <= 0 {
            let t = self.drowning_tick.fetch_add(1, Ordering::Relaxed) + 1;
            if t >= DROWNING_INTERVAL {
                self.drowning_tick.store(0, Ordering::Relaxed);
                living.damage(caller, DROWNING_DAMAGE, damage_type);
            }
        }
    }

    /// Applies an air change through `EntityAirChangeEvent`.
    fn set_air(&self, entity: &Entity, new_air: i32, max_air: i32) {
        let mut new_air = new_air;
        if let Some(server) = entity.world.load().server.upgrade() {
            let mut event =
                crate::plugin::api::events::entity::entity_air_change::EntityAirChangeEvent::new(
                    entity.entity_id,
                    new_air,
                );
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                return;
            }
            new_air = event.amount.clamp(0, max_air);
        }
        self.air_supply.store(new_air, Ordering::Relaxed);
        self.send_air_supply(entity);
    }

    /// Sets the air supply, clamped, through `EntityAirChangeEvent`.
    pub fn set_air_supply(&self, caller: &dyn EntityBase, air: i32) {
        let max_air = Self::max_air_supply(caller);
        self.set_air(caller.get_entity(), air.clamp(0, max_air), max_air);
    }

    /// Writes `Air`, `AirSupply` and `DrowningTick`.
    pub fn write_nbt(&self, nbt: &mut NbtCompound) {
        let air = self.air_supply.load(Ordering::Relaxed);
        nbt.put_short("Air", air.clamp(0, i32::from(i16::MAX)) as i16);
        nbt.put_int("AirSupply", air.max(0));
        nbt.put_int(
            "DrowningTick",
            self.drowning_tick
                .load(Ordering::Relaxed)
                .clamp(0, DROWNING_INTERVAL - 1),
        );
    }

    /// Reads [`Self::write_nbt`] tags, clamped to `max_air`.
    pub fn read_nbt(&self, nbt: &NbtCompound, max_air: i32) {
        if let Some(air) = nbt
            .get_short("Air")
            .map(i32::from)
            .or_else(|| nbt.get_int("AirSupply"))
        {
            self.air_supply
                .store(air.clamp(0, max_air), Ordering::Relaxed);
        }
        if let Some(tick) = nbt.get_int("DrowningTick") {
            self.drowning_tick
                .store(tick.clamp(0, DROWNING_INTERVAL - 1), Ordering::Relaxed);
        }
    }

    /// Sets the air to `max_air` without the event.
    pub fn refill(&self, entity: &Entity, max_air: i32) {
        if self.air_supply.swap(max_air, Ordering::Relaxed) != max_air {
            self.send_air_supply(entity);
        }
        self.drowning_tick.store(0, Ordering::Relaxed);
    }

    /// `Entity.isEyeInFluid(FluidTags.WATER)`.
    #[must_use]
    pub fn is_eye_in_water(entity: &Entity) -> bool {
        let pos = entity.pos.load();
        let eye_y = entity.get_eye_y();

        let bp = BlockPos::new(
            pos.x.floor() as i32,
            eye_y.floor() as i32,
            pos.z.floor() as i32,
        );
        let world = entity.world.load();

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

    /// Eye block is a bubble column.
    fn is_eye_in_bubble_column(entity: &Entity) -> bool {
        let pos = entity.pos.load();
        let bp = BlockPos::new(
            pos.x.floor() as i32,
            entity.get_eye_y().floor() as i32,
            pos.z.floor() as i32,
        );
        entity.world.load().get_block(&bp) == &Block::BUBBLE_COLUMN
    }

    /// Syncs the air supply to clients.
    pub fn send_air_supply(&self, entity: &Entity) {
        let air = self.air_supply.load(Ordering::Relaxed).max(0);

        let mut bedrock_meta =
            pumpkin_protocol::bedrock::client::set_actor_data::SyncedActorDataList::new();
        bedrock_meta.set(
            pumpkin_protocol::bedrock::client::set_actor_data::entity_data_key::AIR_SUPPLY,
            pumpkin_protocol::bedrock::client::set_actor_data::MetadataValue::Short(air as i16),
        );

        entity.set_synced_data(
            pumpkin_data::tracked_data::entity::DATA_AIR_SUPPLY_ID,
            VarInt(air),
        );
        entity.send_bedrock_actor_data(&bedrock_meta);
    }

    /// Refills the air to the maximum.
    pub fn reset(&self, caller: &dyn EntityBase) {
        self.refill(caller.get_entity(), Self::max_air_supply(caller));
    }
}
