use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, AtomicU8, Ordering::Relaxed},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{effect::StatusEffect, potion::Effect};
use pumpkin_data::{entity::EntityType, meta_data_type::MetaDataType, tracked_data::TrackedData};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::Difficulty;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// `Bee.FLAG_ROLL`, `Bee.FLAG_HAS_STUNG`, `Bee.FLAG_HAS_NECTAR`.
const FLAG_ROLL: u8 = 2;
const FLAG_HAS_STUNG: u8 = 4;
const FLAG_HAS_NECTAR: u8 = 8;

/// `Bee.STING_DEATH_COUNTDOWN`.
const STING_DEATH_COUNTDOWN: i32 = 1200;
/// `Bee.TICKS_WITHOUT_NECTAR_BEFORE_GOING_HOME`.
const TICKS_WITHOUT_NECTAR_BEFORE_GOING_HOME: i32 = 3600;

/// `Bee.doHurtTarget`: `POISON_SECONDS_NORMAL` / `POISON_SECONDS_HARD`.
const fn poison_duration(difficulty: Difficulty) -> Option<i32> {
    match difficulty {
        Difficulty::Normal => Some(10 * 20),
        Difficulty::Hard => Some(18 * 20),
        Difficulty::Peaceful | Difficulty::Easy => None,
    }
}

/// `Bee.customServerAiStep`: `random.nextInt(Mth.clamp(1200 - timeSinceSting, 1, 1200)) == 0`.
const fn sting_death_roll_bound(time_since_sting: i32) -> i32 {
    let remaining = STING_DEATH_COUNTDOWN - time_since_sting;
    if remaining < 1 {
        1
    } else if remaining > STING_DEATH_COUNTDOWN {
        STING_DEATH_COUNTDOWN
    } else {
        remaining
    }
}

/// `Bee.isTiredOfLookingForNectar`.
const fn is_tired_of_looking_for_nectar(ticks_without_nectar: i32) -> bool {
    ticks_without_nectar > TICKS_WITHOUT_NECTAR_BEFORE_GOING_HOME
}

/// Represents a Bee, a neutral flying mob that can pollinate crops and sting attackers.
///
/// Wiki: <https://minecraft.wiki/w/Bee>
pub struct BeeEntity {
    pub mob_entity: MobEntity,
    /// `Bee.DATA_FLAGS_ID`.
    flags: AtomicU8,
    /// `Bee.hivePos`.
    pub hive_pos: AtomicCell<Option<BlockPos>>,
    /// `Bee.savedFlowerPos`.
    pub flower_pos: AtomicCell<Option<BlockPos>>,
    /// `Bee.ticksWithoutNectarSinceExitingHive`.
    ticks_without_nectar: AtomicI32,
    /// `Bee.stayOutOfHiveCountdown`.
    stay_out_of_hive_countdown: AtomicI32,
    /// `Bee.numCropsGrownSincePollination`.
    crops_grown_since_pollination: AtomicI32,
    /// `Bee.timeSinceSting`.
    time_since_sting: AtomicI32,
    /// `Bee.underWaterTicks`.
    under_water_ticks: AtomicI32,
}

impl BeeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let bee = Self {
            mob_entity,
            flags: AtomicU8::new(0),
            hive_pos: AtomicCell::new(None),
            flower_pos: AtomicCell::new(None),
            ticks_without_nectar: AtomicI32::new(0),
            stay_out_of_hive_countdown: AtomicI32::new(0),
            crops_grown_since_pollination: AtomicI32::new(0),
            time_since_sting: AtomicI32::new(0),
            under_water_ticks: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(bee);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                2,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(3, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    /// `Bee.setFlag`: read-modify-write on the shared flag byte; vanilla's synched data only
    /// broadcasts when the value actually changes.
    fn set_flag(&self, flag: u8, value: bool) {
        let previous = if value {
            self.flags.fetch_or(flag, Relaxed)
        } else {
            self.flags.fetch_and(!flag, Relaxed)
        };
        let new = if value {
            previous | flag
        } else {
            previous & !flag
        };
        if previous == new {
            return;
        }
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::BEE_FLAGS,
                MetaDataType::BYTE,
                new as i8,
            )],
            None,
        );
    }

    fn get_flag(&self, flag: u8) -> bool {
        self.flags.load(Relaxed) & flag != 0
    }

    #[must_use]
    pub fn has_nectar(&self) -> bool {
        self.get_flag(FLAG_HAS_NECTAR)
    }

    /// `Bee.setHasNectar`.
    pub fn set_has_nectar(&self, has_nectar: bool) {
        if has_nectar {
            self.ticks_without_nectar.store(0, Relaxed);
        }
        self.set_flag(FLAG_HAS_NECTAR, has_nectar);
    }

    #[must_use]
    pub fn has_stung(&self) -> bool {
        self.get_flag(FLAG_HAS_STUNG)
    }

    fn set_has_stung(&self, has_stung: bool) {
        self.set_flag(FLAG_HAS_STUNG, has_stung);
    }

    #[must_use]
    pub fn is_rolling(&self) -> bool {
        self.get_flag(FLAG_ROLL)
    }

    /// `Bee.dropOffNectar`, called by the hive when a bee is released.
    pub fn drop_off_nectar(&self) {
        self.set_has_nectar(false);
        self.crops_grown_since_pollination.store(0, Relaxed);
    }

    /// `Bee.isTiredOfLookingForNectar`.
    #[must_use]
    pub fn is_tired_of_looking_for_nectar(&self) -> bool {
        is_tired_of_looking_for_nectar(self.ticks_without_nectar.load(Relaxed))
    }

    /// `Bee.setStayOutOfHiveCountdown`.
    pub fn set_stay_out_of_hive_countdown(&self, ticks: i32) {
        self.stay_out_of_hive_countdown.store(ticks, Relaxed);
    }
}

/// `BlockPos.CODEC` serializes as an `[x, y, z]` int array.
fn block_pos_to_nbt(pos: BlockPos) -> NbtTag {
    NbtTag::IntArray(vec![pos.0.x, pos.0.y, pos.0.z])
}

fn block_pos_from_nbt(nbt: &NbtCompound, name: &str) -> Option<BlockPos> {
    let &[x, y, z] = nbt.get_int_array(name)? else {
        return None;
    };
    Some(BlockPos::new(x, y, z))
}

impl NBTStorage for BeeEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            if let Some(hive_pos) = self.hive_pos.load() {
                nbt.put("hive_pos", block_pos_to_nbt(hive_pos));
            }
            if let Some(flower_pos) = self.flower_pos.load() {
                nbt.put("flower_pos", block_pos_to_nbt(flower_pos));
            }
            nbt.put_bool("HasNectar", self.has_nectar());
            nbt.put_bool("HasStung", self.has_stung());
            nbt.put_int(
                "TicksSincePollination",
                self.ticks_without_nectar.load(Relaxed),
            );
            nbt.put_int(
                "CannotEnterHiveTicks",
                self.stay_out_of_hive_countdown.load(Relaxed),
            );
            nbt.put_int(
                "CropsGrownSincePollination",
                self.crops_grown_since_pollination.load(Relaxed),
            );
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.set_has_nectar(nbt.get_bool("HasNectar").unwrap_or(false));
            self.set_has_stung(nbt.get_bool("HasStung").unwrap_or(false));
            self.ticks_without_nectar
                .store(nbt.get_int("TicksSincePollination").unwrap_or(0), Relaxed);
            self.stay_out_of_hive_countdown
                .store(nbt.get_int("CannotEnterHiveTicks").unwrap_or(0), Relaxed);
            self.crops_grown_since_pollination.store(
                nbt.get_int("CropsGrownSincePollination").unwrap_or(0),
                Relaxed,
            );
            self.hive_pos.store(block_pos_from_nbt(nbt, "hive_pos"));
            self.flower_pos.store(block_pos_from_nbt(nbt, "flower_pos"));
        })
    }
}

impl Mob for BeeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// `Bee.doHurtTarget`: the poison, the stung flag and the sting sound are all gated on the
    /// hit actually landing, which is what `Mob::try_attack` gates this hook on.
    fn on_successful_attack<'a>(&'a self, target: &'a dyn EntityBase) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if let Some(living) = target.get_living_entity()
                && let Some(duration) =
                    poison_duration(self.get_entity().world.load().level_info.load().difficulty)
            {
                living
                    .add_effect(Effect {
                        effect_type: &StatusEffect::POISON,
                        duration,
                        amplifier: 0,
                        ambient: false,
                        show_particles: true,
                        show_icon: true,
                        blend: false,
                    })
                    .await;
            }

            self.set_has_stung(true);
            let entity = &self.mob_entity.living_entity.entity;
            entity.world.load().play_sound(
                Sound::EntityBeeSting,
                SoundCategory::Neutral,
                &entity.pos.load(),
            );
        })
    }

    /// `Bee.aiStep` and `Bee.customServerAiStep`.
    fn mob_tick<'a>(&'a self, caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let living = &self.mob_entity.living_entity;
            if living.dead.load(Relaxed) {
                return;
            }

            if self.stay_out_of_hive_countdown.load(Relaxed) > 0 {
                self.stay_out_of_hive_countdown.fetch_sub(1, Relaxed);
            }

            if living.is_in_water() {
                self.under_water_ticks.fetch_add(1, Relaxed);
            } else {
                self.under_water_ticks.store(0, Relaxed);
            }

            if self.under_water_ticks.load(Relaxed) > 20 {
                caller.damage(caller.as_ref(), 1.0, DamageType::DROWN).await;
            }

            if self.has_stung() {
                let time_since_sting = self.time_since_sting.fetch_add(1, Relaxed) + 1;
                if time_since_sting % 5 == 0
                    && rand::rng().random_range(0..sting_death_roll_bound(time_since_sting)) == 0
                {
                    let health = living.health.load();
                    caller
                        .damage(caller.as_ref(), health, DamageType::GENERIC)
                        .await;
                }
            }

            if !self.has_nectar() {
                self.ticks_without_nectar.fetch_add(1, Relaxed);
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{is_tired_of_looking_for_nectar, poison_duration, sting_death_roll_bound};
    use pumpkin_util::Difficulty;

    #[test]
    fn bee_sting_poison_matches_vanilla_difficulty_durations() {
        assert_eq!(poison_duration(Difficulty::Peaceful), None);
        assert_eq!(poison_duration(Difficulty::Easy), None);
        assert_eq!(poison_duration(Difficulty::Normal), Some(200));
        assert_eq!(poison_duration(Difficulty::Hard), Some(360));
    }

    #[test]
    fn bee_sting_death_roll_bound_shrinks_and_clamps() {
        assert_eq!(sting_death_roll_bound(0), 1200);
        assert_eq!(sting_death_roll_bound(600), 600);
        assert_eq!(sting_death_roll_bound(1199), 1);
        assert_eq!(sting_death_roll_bound(1200), 1);
        assert_eq!(sting_death_roll_bound(5000), 1);
    }

    #[test]
    fn bee_is_tired_of_looking_for_nectar_after_3600_ticks() {
        assert!(!is_tired_of_looking_for_nectar(3600));
        assert!(is_tired_of_looking_for_nectar(3601));
    }
}
