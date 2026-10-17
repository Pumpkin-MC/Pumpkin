use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering},
};

use pumpkin_data::{
    damage::DamageType,
    effect::StatusEffect,
    entity::EntityType,
    potion::Effect,
    sound::Sound,
    tag::{self, Taggable},
    tracked_data,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::GameEvent;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};

use crate::entity::{
    Entity, EntityBase,
    ai::{
        goal::{
            look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
            pufferfish_puff::PufferfishPuffGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
        },
        target_predicate::TargetPredicate,
    },
    custom_sound::CustomSound,
    mob::{Mob, MobEntity},
    player::Player,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
enum PuffState {
    #[default]
    Small = 0,
    Mid = 1,
    Full = 2,
}

impl PuffState {
    const fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Mid,
            2.. => Self::Full,
            _ => Self::Small,
        }
    }

    const fn id(self) -> i32 {
        self as i32
    }
}

/// Represents a Pufferfish, a passive aquatic mob that can inflate when threatened.
///
/// Wiki: <https://minecraft.wiki/w/Pufferfish>
pub struct PufferfishEntity {
    pub mob_entity: MobEntity,
    puff_state: AtomicI32,
    inflate_counter: AtomicI32,
    deflate_timer: AtomicI32,
}

impl PufferfishEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let pufferfish = Self {
            mob_entity,
            puff_state: AtomicI32::new(PuffState::Small.id()),
            inflate_counter: AtomicI32::new(0),
            deflate_timer: AtomicI32::new(0),
        };

        let mob_arc = Arc::new(pufferfish);
        let mob_weak = Arc::downgrade(&mob_arc);

        mob_arc.refresh_dimensions();

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(PufferfishPuffGoal::new(mob_weak.clone())));
            goal_selector.add_goal(1, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                2,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );

            goal_selector.add_goal(3, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    fn get_puff_state(&self) -> PuffState {
        PuffState::from_id(self.puff_state.load(Ordering::Relaxed))
    }

    fn set_puff_state(&self, puff_state: PuffState) {
        if self.puff_state.swap(puff_state.id(), Ordering::Relaxed) == puff_state.id() {
            return;
        }

        self.mob_entity
            .living_entity
            .entity
            .set_synced_data(tracked_data::pufferfish::PUFF_STATE, puff_state.id());

        self.refresh_dimensions();
    }

    fn refresh_dimensions(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let scale = match self.get_puff_state() {
            PuffState::Small => 0.5,
            PuffState::Mid => 0.7,
            PuffState::Full => 1.0,
        };

        let dimensions = EntityDimensions {
            width: entity.entity_type.dimension[0] * scale,
            height: entity.entity_type.dimension[1] * scale,
            eye_height: entity.entity_type.eye_height * scale,
        };

        let pos = entity.pos.load();
        entity
            .bounding_box
            .store(BoundingBox::new_from_pos(pos.x, pos.y, pos.z, &dimensions));
        entity.entity_dimension.store(dimensions);
    }

    pub(crate) fn start_inflating(&self) {
        self.inflate_counter.store(1, Ordering::Relaxed);
        self.deflate_timer.store(0, Ordering::Relaxed);
    }

    pub(crate) fn stop_inflating(&self) {
        self.inflate_counter.store(0, Ordering::Relaxed);
    }

    fn is_alive(&self) -> bool {
        let living = &self.mob_entity.living_entity;
        living.entity.is_alive() && !living.dead.load(Ordering::Relaxed)
    }

    fn make_sound(&self, sound: Sound) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_silent() {
            entity.play_sound(sound);
        }
    }

    fn tick_puff(&self) {
        let puff_state = self.get_puff_state();
        let inflate_counter = self.inflate_counter.load(Ordering::Relaxed);

        if inflate_counter > 0 {
            if puff_state == PuffState::Small {
                self.make_sound(Sound::EntityPufferFishBlowUp);
                self.set_puff_state(PuffState::Mid);
            } else if inflate_counter > 40 && puff_state == PuffState::Mid {
                self.make_sound(Sound::EntityPufferFishBlowUp);
                self.set_puff_state(PuffState::Full);
            }

            self.inflate_counter.fetch_add(1, Ordering::Relaxed);
        } else if puff_state != PuffState::Small {
            let deflate_timer = self.deflate_timer.load(Ordering::Relaxed);

            if deflate_timer > 60 && puff_state == PuffState::Full {
                self.make_sound(Sound::EntityPufferFishBlowOut);
                self.set_puff_state(PuffState::Mid);
            } else if deflate_timer > 100 && puff_state == PuffState::Mid {
                self.make_sound(Sound::EntityPufferFishBlowOut);
                self.set_puff_state(PuffState::Small);
            }

            self.deflate_timer.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn touch_nearby_mobs(&self, caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let bounding_box = entity.bounding_box.load().expand_all(0.3);

        for target in world.get_entities_at_box(&bounding_box) {
            if target.get_mob().is_some()
                && self.is_scary(caller, target.as_ref())
                && target.get_entity().is_alive()
            {
                self.touch(caller, target.as_ref());
            }
        }
    }

    fn touch(&self, caller: &dyn EntityBase, mob: &dyn EntityBase) {
        let puff_state = self.get_puff_state();
        if mob.damage_with_context(
            mob,
            (1 + puff_state.id()) as f32,
            DamageType::MOB_ATTACK,
            None,
            Some(caller),
            Some(caller),
        ) {
            if let Some(living) = mob.get_living_entity() {
                living.add_effect(poison(puff_state));
            }
            self.make_sound(Sound::EntityPufferFishSting);
        }
    }

    pub fn is_scary(&self, tester: &dyn EntityBase, target: &dyn EntityBase) -> bool {
        let world = self.mob_entity.living_entity.entity.world.load();

        TargetPredicate::create_non_attackable()
            .ignore_visibility()
            .test(&world, Some(tester), target)
            && !target.get_player().is_some_and(Player::is_creative)
            && !target
                .get_entity()
                .entity_type
                .has_tag(&tag::EntityType::MINECRAFT_NOT_SCARY_FOR_PUFFERFISH)
    }
}

impl Mob for PufferfishEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick(&self, caller: &dyn EntityBase) {
        if self.is_alive() && !self.mob_entity.is_no_ai() {
            self.tick_puff();
        }

        if self.is_alive() && self.get_puff_state() != PuffState::Small {
            self.touch_nearby_mobs(caller);
        }
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("PuffState", self.puff_state.load(Ordering::Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(state_id) = nbt.get_int("PuffState") {
            self.set_puff_state(PuffState::from_id(state_id));
        }
    }

    fn mob_player_collision(&self, player: &Arc<Player>) {
        let puff_state = self.get_puff_state();
        if puff_state == PuffState::Small {
            return;
        }

        let caller: &dyn EntityBase = self;
        if player.damage_with_context(
            &**player,
            (1 + puff_state.id()) as f32,
            DamageType::MOB_ATTACK,
            None,
            Some(caller),
            Some(caller),
        ) {
            if !self.mob_entity.living_entity.entity.is_silent() {
                player.send_game_event(GameEvent::PlayPufferfishStringSound, 0.0);
            }
            player.add_effect(poison(puff_state));
        }
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

const fn poison(puff_state: PuffState) -> Effect {
    Effect {
        effect_type: &StatusEffect::POISON,
        duration: 60 * puff_state.id(),
        amplifier: 0,
        ambient: false,
        show_particles: true,
        show_icon: true,
        blend: false,
    }
}
