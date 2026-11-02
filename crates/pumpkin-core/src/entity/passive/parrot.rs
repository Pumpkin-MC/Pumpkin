use std::sync::{Arc, Weak, atomic::Ordering};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::BlockState;
use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;

use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    player::Player,
};

/// Duration in ticks of the poison a parrot gets from eating a cookie, matching
/// vanilla `Parrot.mobInteract`.
const COOKIE_POISON_DURATION: i32 = 900;

/// Represents a Parrot, a passive flying mob that can mimic nearby mob sounds.
///
/// Wiki: <https://minecraft.wiki/w/Parrot>
pub struct ParrotEntity {
    pub mob_entity: MobEntity,
    flap_speed: AtomicCell<f32>,
    next_flap: AtomicCell<f32>,
}

impl ParrotEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let parrot = Self {
            mob_entity,
            flap_speed: AtomicCell::new(0.0),
            next_flap: AtomicCell::new(1.0),
        };
        let mob_arc = Arc::new(parrot);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

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

    /// Feeds the parrot a cookie: it is poisoned and then killed, as in vanilla
    /// `Parrot.mobInteract`.
    fn eat_cookie(&self, player: &Arc<Player>, item_stack: &mut ItemStack) {
        item_stack.decrement_unless_creative(player.gamemode.load(), 1);

        self.mob_entity
            .living_entity
            .add_effect(pumpkin_data::potion::Effect {
                effect_type: &StatusEffect::POISON,
                duration: COOKIE_POISON_DURATION,
                amplifier: 0,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: true,
            });

        // Vanilla guards this call with `player.isCreative() || !this.isInvulnerable()`,
        // but `hurt` re-checks invulnerability itself and `player_attack` doesn't bypass
        // it, so the guard only skips a call that would do nothing anyway.
        self.damage_with_context(
            self,
            f32::MAX,
            DamageType::PLAYER_ATTACK,
            None,
            Some(player.as_ref()),
            Some(player.as_ref()),
        );
    }
}

impl Mob for ParrotEntity {
    fn mob_ai_step(&self) {
        // Vanilla `calculateFlapping`.
        let entity = self.get_entity();
        let on_ground = entity.on_ground.load(Ordering::Relaxed);
        let flap_change: f32 = if on_ground || entity.has_vehicle() {
            -1.0
        } else {
            4.0
        };
        self.flap_speed.store(
            flap_change
                .mul_add(0.3, self.flap_speed.load())
                .clamp(0.0, 1.0),
        );
        let velocity = entity.velocity.load();
        if !on_ground && velocity.y < 0.0 {
            entity.set_velocity(velocity.multiply(1.0, 0.6, 1.0));
        }
    }

    fn mob_is_flapping(&self) -> bool {
        self.get_entity().fly_dist.load() > self.next_flap.load()
    }

    fn mob_on_flap(&self) {
        self.play_entity_sound(Sound::EntityParrotFly, 0.15, 1.0);
        self.next_flap
            .store(self.get_entity().fly_dist.load() + self.flap_speed.load() / 2.0);
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityParrotStep, 0.15, 1.0);
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        // Vanilla checks the poisonous food tag last, after taming, which isn't
        // implemented yet. Nothing in `parrot_food` is also in
        // `parrot_poisonous_food`, so the two branches can't be confused.
        if !item_stack
            .get_item()
            .has_tag(&tag::Item::MINECRAFT_PARROT_POISONOUS_FOOD)
        {
            return self
                .mob_entity
                .mob_interact(player, item_stack, self.can_be_leashed());
        }

        self.eat_cookie(player, item_stack);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::COOKIE_POISON_DURATION;
    use pumpkin_data::item::Item;
    use pumpkin_data::tag::{self, Taggable};

    /// The interaction is gated on the vanilla `parrot_poisonous_food` tag rather than
    /// on a hardcoded cookie id, so check the tag actually resolves the way the
    /// interaction assumes.
    #[test]
    fn cookie_is_poisonous_parrot_food() {
        assert!(Item::COOKIE.has_tag(&tag::Item::MINECRAFT_PARROT_POISONOUS_FOOD));
    }

    /// Seeds tame a parrot in vanilla and must not reach the poison branch.
    #[test]
    fn parrot_food_is_not_poisonous() {
        assert!(!Item::WHEAT_SEEDS.has_tag(&tag::Item::MINECRAFT_PARROT_POISONOUS_FOOD));
        assert!(!Item::COOKED_CHICKEN.has_tag(&tag::Item::MINECRAFT_PARROT_POISONOUS_FOOD));
    }

    #[test]
    fn poison_lasts_45_seconds() {
        assert_eq!(COOKIE_POISON_DURATION, 900);
    }
}
