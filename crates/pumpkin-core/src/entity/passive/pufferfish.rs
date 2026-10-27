use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering::Relaxed},
};

use pumpkin_data::{
    damage::DamageType,
    effect::StatusEffect,
    item::Item,
    potion::Effect,
    sound::Sound,
    tag::{self, Taggable},
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::{CGameEvent, GameEvent};

use super::abstract_fish::{AbstractFish, FishData};
use crate::entity::bucketable::Bucketable;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::Goal,
    ai::target_predicate::TargetPredicate,
    mob::{Mob, MobEntity},
    player::Player,
};

const STATE_SMALL: i32 = 0;
const STATE_MID: i32 = 1;
const STATE_FULL: i32 = 2;
const FULL_INFLATE_DELAY: i32 = 40;
const MID_DEFLATE_DELAY: i32 = 60;
const SMALL_DEFLATE_DELAY: i32 = 100;
const PUFF_RANGE: f64 = 2.0;
const STING_RANGE: f64 = 0.3;
const POISON_TICKS_PER_STATE: i32 = 60;

pub struct PufferfishEntity {
    pub mob_entity: MobEntity,
    fish_data: FishData,
    puff_state: AtomicI32,
    inflate_counter: AtomicI32,
    deflate_timer: AtomicI32,
    targeting_conditions: TargetPredicate,
}

impl PufferfishEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        super::abstract_fish::initialize_fish_movement(&mut mob_entity);
        let mob_arc = Arc::new(Self {
            mob_entity,
            fish_data: FishData::default(),
            puff_state: AtomicI32::new(STATE_SMALL),
            inflate_counter: AtomicI32::new(0),
            deflate_timer: AtomicI32::new(0),
            targeting_conditions: Self::targeting_conditions(),
        });
        mob_arc.set_dimension_scale(Self::get_scale(STATE_SMALL));
        super::abstract_fish::register_fish_goals(&mob_arc.mob_entity);
        mob_arc
            .mob_entity
            .goals_selector
            .add_goal(1, Box::new(PufferfishPuffGoal));

        mob_arc
    }

    pub fn get_puff_state(&self) -> i32 {
        self.puff_state.load(Relaxed)
    }

    pub fn set_puff_state(&self, state: i32) {
        self.puff_state.store(state, Relaxed);
        self.get_entity()
            .set_synced_data(pumpkin_data::tracked_data::pufferfish::PUFF_STATE, state);
        self.set_dimension_scale(Self::get_scale(state));
    }

    const fn get_scale(state: i32) -> f32 {
        match state {
            STATE_SMALL => 0.5,
            STATE_MID => 0.7,
            _ => 1.0,
        }
    }

    fn targeting_conditions() -> TargetPredicate {
        let mut conditions = TargetPredicate::create_non_attackable().ignore_visibility();
        conditions.set_predicate(|target, _| {
            !target
                .entity
                .entity_type
                .has_tag(&tag::EntityType::MINECRAFT_NOT_SCARY_FOR_PUFFERFISH)
        });
        conditions
    }

    fn is_scary(&self, target: &dyn EntityBase) -> bool {
        !target.get_player().is_some_and(Player::is_creative)
            && self
                .targeting_conditions
                .test(&self.get_entity().world.load(), Some(self), target)
    }

    const fn poison(state: i32) -> Effect {
        Effect {
            effect_type: &StatusEffect::POISON,
            duration: POISON_TICKS_PER_STATE * state,
            amplifier: 0,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }
    }

    fn touch(&self, mob: &dyn EntityBase) {
        let this: &dyn EntityBase = self;
        if !this.do_teams_allow_damage(mob) {
            return;
        }
        let state = self.get_puff_state();
        if mob.damage_with_context(
            mob,
            (1 + state) as f32,
            DamageType::MOB_ATTACK,
            None,
            Some(self),
            Some(self),
        ) {
            if let Some(living) = mob.get_living_entity() {
                living.add_effect(Self::poison(state));
            }
            let entity = self.get_entity();
            if !entity.is_silent() {
                entity.play_sound(Sound::EntityPufferFishSting);
            }
        }
    }
}

impl Mob for PufferfishEntity {
    crate::fish_mob_hooks!();

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        if !self.get_entity().is_alive()
            || self.mob_entity.living_entity.is_dead_or_dying()
            || self.mob_entity.is_no_ai()
        {
            return;
        }
        let state = self.get_puff_state();
        if self.inflate_counter.load(Relaxed) > 0 {
            if state == STATE_SMALL {
                self.make_sound(Sound::EntityPufferFishBlowUp);
                self.set_puff_state(STATE_MID);
            } else if self.inflate_counter.load(Relaxed) > FULL_INFLATE_DELAY && state == STATE_MID
            {
                self.make_sound(Sound::EntityPufferFishBlowUp);
                self.set_puff_state(STATE_FULL);
            }
            self.inflate_counter.fetch_add(1, Relaxed);
        } else if state != STATE_SMALL {
            if self.deflate_timer.load(Relaxed) > MID_DEFLATE_DELAY && state == STATE_FULL {
                self.make_sound(Sound::EntityPufferFishBlowOut);
                self.set_puff_state(STATE_MID);
            } else if self.deflate_timer.load(Relaxed) > SMALL_DEFLATE_DELAY && state == STATE_MID {
                self.make_sound(Sound::EntityPufferFishBlowOut);
                self.set_puff_state(STATE_SMALL);
            }
            self.deflate_timer.fetch_add(1, Relaxed);
        }
    }

    fn mob_ai_step(&self) {
        let entity = self.get_entity();
        if !entity.is_alive()
            || self.mob_entity.living_entity.is_dead_or_dying()
            || self.get_puff_state() <= STATE_SMALL
        {
            return;
        }
        let bounds = entity
            .bounding_box
            .load()
            .expand(STING_RANGE, STING_RANGE, STING_RANGE);
        for mob in entity.world.load().get_entities_at_box(&bounds) {
            if mob.get_mob().is_some() && self.is_scary(mob.as_ref()) {
                self.touch(mob.as_ref());
            }
        }
    }

    fn mob_player_collision(&self, player: &Arc<Player>) {
        let this: &dyn EntityBase = self;
        if !this.do_teams_allow_damage(player.as_ref()) {
            return;
        }
        let state = self.get_puff_state();
        if state > STATE_SMALL
            && player.damage_with_context(
                player.as_ref(),
                (1 + state) as f32,
                DamageType::MOB_ATTACK,
                None,
                Some(self),
                Some(self),
            )
        {
            if !self.get_entity().is_silent() {
                player.try_send_client_packet(&CGameEvent::new(
                    GameEvent::PlayPufferfishStringSound,
                    0.0,
                ));
            }
            player.living_entity.add_effect(Self::poison(state));
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.fish_write_nbt(nbt);
        nbt.put_int("PuffState", self.get_puff_state());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.fish_read_nbt(nbt);
        self.set_puff_state(
            nbt.get_int("PuffState")
                .unwrap_or(STATE_SMALL)
                .min(STATE_FULL),
        );
    }

    fn mob_init_data_tracker(&self) {
        self.set_from_bucket(self.is_from_bucket());
        self.set_puff_state(self.get_puff_state());
    }
}

crate::impl_abstract_fish!(
    PufferfishEntity,
    bucket: &Item::PUFFERFISH_BUCKET,
    flop: Sound::EntityPufferFishFlop,
    ambient: None
);

struct PufferfishPuffGoal;

impl PufferfishPuffGoal {
    fn fish(mob: &dyn Mob) -> Option<&PufferfishEntity> {
        mob.cast_any().downcast_ref::<PufferfishEntity>()
    }
}

impl Goal for PufferfishPuffGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fish) = Self::fish(mob) else {
            return false;
        };
        let entity = fish.get_entity();
        let bounds = entity
            .bounding_box
            .load()
            .expand(PUFF_RANGE, PUFF_RANGE, PUFF_RANGE);
        let world = entity.world.load();
        world
            .get_entities_at_box(&bounds)
            .iter()
            .any(|target| fish.is_scary(target.as_ref()))
            || world
                .get_players_at_box(&bounds)
                .iter()
                .any(|player| fish.is_scary(player.as_ref()))
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(fish) = Self::fish(mob) {
            fish.inflate_counter.store(1, Relaxed);
            fish.deflate_timer.store(0, Relaxed);
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        if let Some(fish) = Self::fish(mob) {
            fish.inflate_counter.store(0, Relaxed);
        }
    }
}
