use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tracked_data;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;

use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::custom_sound::CustomSound;
use crate::entity::{
    Entity, EntityBase,
    ai::control::smooth_swimming_move_control::SmoothSwimmingMoveControl,
    ai::goal::{
        Goal, look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        random_swimming::RandomSwimmingGoal,
    },
    mob::{Mob, MobEntity},
};

/// Represents a Pufferfish, a passive aquatic mob that can inflate when threatened.
///
/// Wiki: <https://minecraft.wiki/w/Pufferfish>
pub struct PufferfishEntity {
    pub mob_entity: MobEntity,
    pub air_supply: AtomicI32,
    puff_state: AtomicI32,
    inflate_counter: AtomicI32,
    deflate_timer: AtomicI32,
}

impl PufferfishEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        *mob_entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(SmoothSwimmingMoveControl::new(85, 10, 1.0, 0.1, true));

        let pufferfish = Self {
            mob_entity,
            air_supply: AtomicI32::new(300),
            puff_state: AtomicI32::new(0),
            inflate_counter: AtomicI32::new(0),
            deflate_timer: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(pufferfish);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(0, Box::new(RandomSwimmingGoal::new(1.0)));
            goal_selector.add_goal(1, Box::new(PufferfishPuffGoal::new(mob_arc.clone())));
            goal_selector.add_goal(
                2,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(3, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    #[must_use]
    pub fn get_puff_state(&self) -> i32 {
        self.puff_state.load(Ordering::Relaxed)
    }

    pub fn set_puff_state(&self, state: i32) {
        let clamped = state.clamp(0, 2);
        self.puff_state.store(clamped, Ordering::Relaxed);
        self.mob_entity
            .living_entity
            .entity
            .set_synced_data(tracked_data::pufferfish::PUFF_STATE, VarInt(clamped));
    }
}

pub struct PufferfishPuffGoal {
    fish: Arc<PufferfishEntity>,
}

impl PufferfishPuffGoal {
    #[must_use]
    pub const fn new(fish: Arc<PufferfishEntity>) -> Self {
        Self { fish }
    }
}

impl Goal for PufferfishPuffGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let entity = &mob.get_mob_entity().living_entity.entity;
        let world = entity.world.load();
        let my_id = entity.entity_id;
        let bb = entity.bounding_box.load().expand_all(2.0);
        let entities = world.get_entities_at_box(&bb);

        entities.iter().any(|e| {
            let other_entity = e.get_entity();
            if other_entity.entity_id == my_id || !other_entity.is_alive() {
                return false;
            }
            if let Some(player) = e.get_player() {
                return player.gamemode.load() != pumpkin_util::GameMode::Creative
                    && player.gamemode.load() != pumpkin_util::GameMode::Spectator;
            }
            other_entity.entity_type != &EntityType::PUFFERFISH
                && other_entity.entity_type != &EntityType::COD
                && other_entity.entity_type != &EntityType::SALMON
                && other_entity.entity_type != &EntityType::TROPICAL_FISH
        })
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.fish.inflate_counter.store(1, Ordering::Relaxed);
        self.fish.deflate_timer.store(0, Ordering::Relaxed);
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.fish.inflate_counter.store(0, Ordering::Relaxed);
    }
}

impl CustomSound for PufferfishEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityPufferFishDeath)
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityPufferFishHurt)
    }
}

impl Mob for PufferfishEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn is_pushed_by_fluids(&self) -> bool {
        false
    }

    fn mob_tick(&self, caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }

        let world = entity.world.load_full();
        let in_water = entity.touching_water.load(Ordering::Relaxed);

        // Air supply / drowning on land (matching vanilla AbstractFish / WaterAnimal)
        if in_water {
            self.air_supply.store(300, Ordering::Relaxed);
        } else {
            let air = self.air_supply.fetch_sub(1, Ordering::Relaxed) - 1;
            if air <= -20 {
                self.air_supply.store(0, Ordering::Relaxed);
                caller.damage(caller, 2.0, DamageType::DROWN);
            }
        }

        // Flop on land (matching vanilla AbstractFish.aiStep)
        if !in_water && entity.on_ground.load(Ordering::Relaxed) {
            let mut rng = rand::rng();
            let dx = (rng.random::<f32>() * 2.0 - 1.0) * 0.05;
            let dz = (rng.random::<f32>() * 2.0 - 1.0) * 0.05;
            let velo = entity.velocity.load();
            entity.set_velocity(Vector3::new(velo.x + dx as f64, 0.4, velo.z + dz as f64));
            entity.on_ground.store(false, Ordering::Relaxed);
            if !entity.silent.load(Ordering::Relaxed) {
                world.play_sound_fine(
                    Sound::EntityPufferFishFlop,
                    SoundCategory::Neutral,
                    &entity.pos.load(),
                    1.0,
                    1.0,
                );
            }
        }

        let inflate = self.inflate_counter.load(Ordering::Relaxed);
        let state = self.get_puff_state();

        // Vanilla inflation sequence
        if inflate > 0 {
            if state == 0 {
                world.play_sound(
                    Sound::EntityPufferFishBlowUp,
                    SoundCategory::Neutral,
                    &entity.pos.load(),
                );
                self.set_puff_state(1);
            } else if inflate > 40 && state == 1 {
                world.play_sound(
                    Sound::EntityPufferFishBlowUp,
                    SoundCategory::Neutral,
                    &entity.pos.load(),
                );
                self.set_puff_state(2);
            }
            self.inflate_counter.fetch_add(1, Ordering::Relaxed);
        } else if state != 0 {
            // Vanilla deflation sequence
            let deflate = self.deflate_timer.load(Ordering::Relaxed);
            if deflate > 60 && state == 2 {
                world.play_sound(
                    Sound::EntityPufferFishBlowOut,
                    SoundCategory::Neutral,
                    &entity.pos.load(),
                );
                self.set_puff_state(1);
            } else if deflate > 100 && state == 1 {
                world.play_sound(
                    Sound::EntityPufferFishBlowOut,
                    SoundCategory::Neutral,
                    &entity.pos.load(),
                );
                self.set_puff_state(0);
            }
            self.deflate_timer.fetch_add(1, Ordering::Relaxed);
        }

        // Vanilla sting & poison aura when inflated
        if state > 0 {
            let bb = entity.bounding_box.load().expand_all(0.3);
            let targets = world.get_entities_at_box(&bb);
            let my_id = entity.entity_id;

            for target in targets {
                let target_entity = target.get_entity();
                if target_entity.entity_id == my_id || !target_entity.is_alive() {
                    continue;
                }
                if target_entity.entity_type == &EntityType::PUFFERFISH {
                    continue;
                }
                if let Some(player) = target.get_player() {
                    if player.gamemode.load() == pumpkin_util::GameMode::Creative
                        || player.gamemode.load() == pumpkin_util::GameMode::Spectator
                    {
                        continue;
                    }
                }

                if let Some(living) = target.get_living_entity() {
                    let damage = (1 + state) as f32;
                    let damaged = target.damage(target.as_ref(), damage, DamageType::MOB_ATTACK);
                    if damaged {
                        living.add_effect(Effect {
                            effect_type: &StatusEffect::POISON,
                            duration: 60 * state,
                            amplifier: 0,
                            ambient: false,
                            show_particles: true,
                            show_icon: true,
                            blend: true,
                        });
                        world.play_sound(
                            Sound::EntityPufferFishSting,
                            SoundCategory::Neutral,
                            &entity.pos.load(),
                        );
                    }
                }
            }
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("PuffState", self.get_puff_state());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(state) = nbt.get_int("PuffState") {
            self.set_puff_state(state);
        }
    }
}
