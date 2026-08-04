use std::sync::atomic::{AtomicI32, AtomicU8, Ordering, Ordering::Relaxed};
use std::sync::{Arc, Weak};

use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        avoid_entity::AvoidEntityGoal, escape_danger::EscapeDangerGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    player::Player,
};

/// Pufferfish.java: `inflateCounter > 40` gates the deflated->half transition.
const HALF_PUFF_INFLATE_TICKS: i32 = 40;
/// Pufferfish.java: `deflateTimer > 60` gates the full->half transition.
const HALF_DEFLATE_TICKS: i32 = 60;
/// Pufferfish.java: `deflateTimer > 100` gates the half->deflated transition.
const FULL_DEFLATE_TICKS: i32 = 100;

/// Pufferfish.java `tick()`, folded together with `PufferfishPuffGoal.start`/`stop` (which set
/// `inflateCounter` to 1/0 and reset `deflateTimer` on state entry/exit). Returns the new
/// `(puff_state, inflate_counter, deflate_timer, played_blow_up, played_blow_out)`.
const fn advance_puff_state(
    threat_present: bool,
    mut puff_state: u8,
    mut inflate_counter: i32,
    mut deflate_timer: i32,
) -> (u8, i32, i32, bool, bool) {
    if threat_present && inflate_counter == 0 {
        inflate_counter = 1;
        deflate_timer = 0;
    } else if !threat_present && inflate_counter > 0 {
        inflate_counter = 0;
    }

    let mut blow_up = false;
    let mut blow_out = false;

    if inflate_counter > 0 {
        if puff_state == 0 {
            blow_up = true;
            puff_state = 1;
        } else if inflate_counter > HALF_PUFF_INFLATE_TICKS && puff_state == 1 {
            blow_up = true;
            puff_state = 2;
        }
        inflate_counter += 1;
    } else if puff_state != 0 {
        if deflate_timer > HALF_DEFLATE_TICKS && puff_state == 2 {
            blow_out = true;
            puff_state = 1;
        } else if deflate_timer > FULL_DEFLATE_TICKS && puff_state == 1 {
            blow_out = true;
            puff_state = 0;
        }
        deflate_timer += 1;
    }

    (
        puff_state,
        inflate_counter,
        deflate_timer,
        blow_up,
        blow_out,
    )
}

/// Pufferfish.java `touch`/`playerTouch`: `1 + puffState` damage, `Poison` for `60 * puffState`
/// ticks at amplifier 0.
fn sting_damage(puff_state: u8) -> f32 {
    1.0 + f32::from(puff_state)
}

fn sting_poison_duration(puff_state: u8) -> i32 {
    60 * i32::from(puff_state)
}

/// Pufferfish.java `SCARY_MOB` selector: a creative player is never scary; otherwise anything
/// not tagged `minecraft:not_scary_for_pufferfish` is.
fn is_scary(candidate: &dyn EntityBase) -> bool {
    if candidate.get_player().is_some_and(Player::is_creative) {
        return false;
    }
    !candidate
        .get_entity()
        .entity_type
        .has_tag(&tag::EntityType::MINECRAFT_NOT_SCARY_FOR_PUFFERFISH)
}

/// Represents a Pufferfish, a passive aquatic mob that can inflate when threatened.
///
/// Wiki: <https://minecraft.wiki/w/Pufferfish>
pub struct PufferfishEntity {
    pub mob_entity: MobEntity,
    puff_state: AtomicU8,
    inflate_counter: AtomicI32,
    deflate_timer: AtomicI32,
}

impl PufferfishEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let pufferfish = Self {
            mob_entity,
            puff_state: AtomicU8::new(0),
            inflate_counter: AtomicI32::new(0),
            deflate_timer: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(pufferfish);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(0, EscapeDangerGoal::new(1.25));
            // Vanilla `AbstractFish.registerGoals`: flee players within 8 blocks.
            // The vanilla goal also skips spectators, which `AvoidEntityGoal` cannot do yet.
            goal_selector.add_goal(
                2,
                Box::new(AvoidEntityGoal::new(&EntityType::PLAYER, 8.0, 1.6, 1.4)),
            );
            goal_selector.add_goal(4, Box::new(WanderAroundGoal::new_with_interval(1.0, 40)));
            goal_selector.add_goal(
                2,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(3, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    fn send_puff_state(&self, state: u8) {
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::PUFF_STATE,
                MetaDataType::INT,
                VarInt(i32::from(state)),
            )],
            None,
        );
    }

    /// Pufferfish.java `PufferfishPuffGoal.canUse`: any non-scary-immune living entity within the
    /// bounding box inflated by 2.0. Approximated here with a spherical distance check, matching
    /// this codebase's other proximity goals (e.g. `BreedGoal`).
    fn threat_nearby(&self) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        let pos = entity.pos.load();
        let world = entity.world.load();
        let my_uuid = entity.entity_uuid;

        world.get_nearby_entities(pos, 2.0).values().any(|e| {
            e.get_entity().entity_uuid != my_uuid
                && e.get_living_entity().is_some()
                && e.get_entity().is_alive()
                && is_scary(e.as_ref())
        })
    }

    async fn sting(&self, target: &dyn EntityBase) {
        let puff_state = self.puff_state.load(Relaxed);
        if puff_state == 0 {
            return;
        }

        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();

        let damaged = target
            .damage_with_context(
                target,
                sting_damage(puff_state),
                DamageType::MOB_ATTACK,
                Some(pos),
                Some(self as &dyn EntityBase),
                Some(self as &dyn EntityBase),
            )
            .await;

        if damaged {
            if let Some(living) = target.get_living_entity() {
                living
                    .add_effect(Effect {
                        effect_type: &StatusEffect::POISON,
                        duration: sting_poison_duration(puff_state),
                        amplifier: 0,
                        ambient: false,
                        show_particles: true,
                        show_icon: true,
                        blend: false,
                    })
                    .await;
            }
            world.play_sound(Sound::EntityPufferFishSting, SoundCategory::Neutral, &pos);
        }
    }

    /// Pufferfish.java `aiStep`: contact-damages non-player `Mob`s within `boundingBox.inflate(0.3)`
    /// while puffed. Players are handled separately via `mob_player_collision`, which mirrors
    /// vanilla's `playerTouch` (invoked by the generic entity/player collision check).
    async fn sting_nearby_mobs(&self) {
        if self.puff_state.load(Relaxed) == 0 {
            return;
        }

        let entity = &self.mob_entity.living_entity.entity;
        let pos = entity.pos.load();
        let world = entity.world.load();
        let my_uuid = entity.entity_uuid;

        let nearby: Vec<_> = world
            .get_nearby_entities(pos, 1.0)
            .values()
            .filter(|e| {
                e.get_entity().entity_uuid != my_uuid
                    && e.get_living_entity().is_some()
                    && e.get_entity().is_alive()
                    && e.get_player().is_none()
            })
            .cloned()
            .collect();

        for target in nearby {
            self.sting(target.as_ref()).await;
        }
    }
}

impl NBTStorage for PufferfishEntity {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
    ) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            nbt.put_int("PuffState", i32::from(self.puff_state.load(Relaxed)));
        })
    }

    fn read_nbt_non_mut<'a>(
        &'a self,
        nbt: &'a pumpkin_nbt::compound::NbtCompound,
    ) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            let state = nbt.get_int("PuffState").unwrap_or(0).clamp(0, 2) as u8;
            self.puff_state.store(state, Relaxed);
        })
    }
}

impl Mob for PufferfishEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.send_puff_state(self.puff_state.load(Relaxed));
        })
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self.mob_entity.living_entity.dead.load(Ordering::Relaxed) {
                return;
            }

            let threat_present = self.threat_nearby();
            let (new_state, new_inflate, new_deflate, blow_up, blow_out) = advance_puff_state(
                threat_present,
                self.puff_state.load(Relaxed),
                self.inflate_counter.load(Relaxed),
                self.deflate_timer.load(Relaxed),
            );

            self.inflate_counter.store(new_inflate, Relaxed);
            self.deflate_timer.store(new_deflate, Relaxed);

            if new_state != self.puff_state.load(Relaxed) {
                self.puff_state.store(new_state, Relaxed);
                self.send_puff_state(new_state);
            }

            if blow_up || blow_out {
                let entity = &self.mob_entity.living_entity.entity;
                let world = entity.world.load();
                let pos = entity.pos.load();
                let sound = if blow_up {
                    Sound::EntityPufferFishBlowUp
                } else {
                    Sound::EntityPufferFishBlowOut
                };
                world.play_sound(sound, SoundCategory::Neutral, &pos);
            }

            self.sting_nearby_mobs().await;
        })
    }

    fn mob_player_collision<'a>(&'a self, player: &'a Arc<Player>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.sting(&**player).await;
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_threat_stays_deflated() {
        let (state, inflate, deflate, blow_up, blow_out) = advance_puff_state(false, 0, 0, 0);
        assert_eq!(
            (state, inflate, deflate, blow_up, blow_out),
            (0, 0, 0, false, false)
        );
    }

    #[test]
    fn threat_inflates_to_half_immediately() {
        let (state, inflate, _deflate, blow_up, blow_out) = advance_puff_state(true, 0, 0, 0);
        assert_eq!(state, 1);
        assert_eq!(inflate, 2);
        assert!(blow_up);
        assert!(!blow_out);
    }

    #[test]
    fn sustained_threat_reaches_full_puff_after_40_ticks() {
        let mut state = 0u8;
        let mut inflate = 0i32;
        let mut deflate = 0i32;
        let mut reached_full_at = None;
        for tick in 1..=45 {
            let (s, i, d, _, _) = advance_puff_state(true, state, inflate, deflate);
            state = s;
            inflate = i;
            deflate = d;
            if state == 2 && reached_full_at.is_none() {
                reached_full_at = Some(tick);
            }
        }
        assert_eq!(reached_full_at, Some(41));
    }

    #[test]
    fn threat_removed_deflates_over_time() {
        let mut state = 2u8;
        let mut inflate = 0i32;
        let mut deflate = 0i32;
        let mut half_at = None;
        let mut deflated_at = None;
        for tick in 1..=105 {
            let (s, i, d, _, _) = advance_puff_state(false, state, inflate, deflate);
            state = s;
            inflate = i;
            deflate = d;
            if state == 1 && half_at.is_none() {
                half_at = Some(tick);
            }
            if state == 0 && deflated_at.is_none() {
                deflated_at = Some(tick);
            }
        }
        assert_eq!(half_at, Some(62));
        assert_eq!(deflated_at, Some(102));
    }

    #[test]
    fn sting_damage_and_poison_scale_with_puff_state() {
        assert_eq!(sting_damage(1), 2.0);
        assert_eq!(sting_damage(2), 3.0);
        assert_eq!(sting_poison_duration(1), 60);
        assert_eq!(sting_poison_duration(2), 120);
    }
}
