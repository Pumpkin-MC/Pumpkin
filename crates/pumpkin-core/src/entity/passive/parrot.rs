mod jukebox;
#[cfg(test)]
#[path = "parrot/tests.rs"]
mod regression_tests;
mod shoulder;
use std::sync::{Arc, Weak, atomic::Ordering};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::BlockState;
use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::entity_variant::ParrotVariant;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        escape_danger::EscapeDangerGoal, follow_mob::FollowMobGoal, follow_owner::FollowOwnerGoal,
        look_at_entity::LookAtEntityGoal, sit_when_ordered_to::SitWhenOrderedToGoal,
        swim::SwimGoal, water_avoiding_random_flying::WaterAvoidingRandomFlyingGoal,
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
    pub tamable_data: super::tamable::TamableData,
    ageable_data: crate::entity::ageable::AgeableData,
    variant: AtomicCell<ParrotVariant>,
    flap_speed: AtomicCell<f32>,
    next_flap: AtomicCell<f32>,
    jukebox: AtomicCell<Option<BlockPos>>,
    party: std::sync::atomic::AtomicBool,
}

impl ParrotEntity {
    pub(crate) fn imitate_nearby(entity: &dyn EntityBase) -> bool {
        let e = entity.get_entity();
        if !e.is_alive() || e.is_silent() || rand::rng().random_range(0..2) != 0 {
            return false;
        }
        let candidates: Vec<_> = e
            .world
            .load()
            .get_entities_at_box(&e.bounding_box.load().expand(20.0, 20.0, 20.0))
            .into_iter()
            .filter(|m| {
                m.get_mob().is_some()
                    && IMITATIONS
                        .iter()
                        .any(|(kind, _)| *kind == m.get_entity().entity_type)
            })
            .collect();
        if candidates.is_empty() {
            return false;
        }
        let target = &candidates[rand::rng().random_range(0..candidates.len())];
        if target.get_entity().is_silent() {
            return false;
        }
        if let Some((_, sound)) = IMITATIONS
            .iter()
            .find(|(kind, _)| *kind == target.get_entity().entity_type)
        {
            e.world.load().play_sound_fine(
                *sound,
                entity.sound_source(),
                &e.pos.load(),
                0.7,
                Self::pitch(),
            );
            return true;
        }
        false
    }

    pub(crate) fn pitch() -> f32 {
        let mut rng = rand::rng();
        (rng.random::<f32>() - rng.random::<f32>()) * 0.2 + 1.0
    }

    pub(crate) fn ambient(world: &crate::world::World) -> Sound {
        let mut rng = rand::rng();
        if world.level_info.load().difficulty != pumpkin_util::Difficulty::Peaceful
            && rng.random_range(0..1000) == 0
        {
            IMITATIONS[rng.random_range(0..IMITATIONS.len())].1
        } else {
            Sound::EntityParrotAmbient
        }
    }
    pub fn new(entity: Entity) -> Arc<Self> {
        use crate::entity::ai::{
            control::flying_move_control::FlyingMoveControl,
            pathfinder::{Navigator, node::PathType},
        };
        let mut mob_entity = MobEntity::new(entity);
        *mob_entity
            .move_control
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(FlyingMoveControl::new(10, false));
        let navigator = mob_entity
            .navigator
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *navigator = Navigator::flying();
        navigator.set_can_open_doors(false);
        navigator.set_can_float(true);
        for path_type in [PathType::DangerFire, PathType::DamageFire, PathType::Cocoa] {
            navigator.set_pathfinding_malus(path_type, -1.0);
        }
        let parrot = Self {
            mob_entity,
            tamable_data: super::tamable::TamableData::default(),
            ageable_data: crate::entity::ageable::AgeableData::default(),
            variant: AtomicCell::new(ParrotVariant::DEFAULT),
            flap_speed: AtomicCell::new(0.0),
            next_flap: AtomicCell::new(1.0),
            jukebox: AtomicCell::new(None),
            party: std::sync::atomic::AtomicBool::new(false),
        };
        let mob_arc = Arc::new(parrot);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, EscapeDangerGoal::tamable(1.25));
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(
                1,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(2, Box::new(SitWhenOrderedToGoal::new()));
            goal_selector.add_goal(2, FollowOwnerGoal::new(1.0, 5.0, 1.0));
            goal_selector.add_goal(2, Box::new(WaterAvoidingRandomFlyingGoal::parrot(1.0)));
            goal_selector.add_goal(
                3,
                Box::new(shoulder::LandOnShoulderGoal::new(Arc::downgrade(&mob_arc))),
            );
            goal_selector.add_goal(3, Box::new(FollowMobGoal::new(1.0, 3.0, 7.0)));
        };

        mob_arc
    }

    pub fn set_variant(&self, variant: ParrotVariant) {
        self.variant.store(variant);
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::parrot::DATA_VARIANT_ID,
            VarInt(variant.id()),
        );
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

        // Damage handling re-checks invulnerability, including vanilla's
        // creative-player bypass. An invulnerable parrot fed by a survival
        // player still receives poison, but the damage call has no side effects.
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

impl super::animal::Animal for ParrotEntity {
    fn is_food(&self, _stack: &ItemStack) -> bool {
        false
    }
    fn can_mate(&self, _partner: &dyn EntityBase) -> bool {
        false
    }
}

impl super::tamable::TamableAnimal for ParrotEntity {
    fn get_tamable_data(&self) -> &super::tamable::TamableData {
        &self.tamable_data
    }
}

impl crate::entity::ageable::AgeableMob for ParrotEntity {
    fn get_ageable_data(&self) -> &crate::entity::ageable::AgeableData {
        &self.ageable_data
    }
    fn can_be_a_baby(&self) -> bool {
        false
    }
    fn default_should_spawn_baby(&self) -> bool {
        false
    }
}

impl crate::entity::custom_sound::CustomSound for ParrotEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityParrotDeath)
    }
    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityParrotHurt)
    }
}

impl Mob for ParrotEntity {
    fn can_fly_to_owner(&self) -> bool {
        true
    }
    fn mob_metadata(&self) -> Option<crate::entity::mob::MobMetadata> {
        use super::tamable::TamableAnimal;
        // Unconditional so a cleared flag reaches Bedrock clients as false.
        Some(
            crate::entity::mob::MobMetadata::new()
                .tamed(self.is_tame())
                .sitting(self.is_in_sitting_pose())
                .no_ai(self.get_mob_entity().is_no_ai()),
        )
    }
    fn for_each_game_event_listener(
        &self,
        visit: &mut dyn FnMut(&dyn crate::world::game_event::GameEventListener),
    ) {
        visit(&jukebox::JukeboxListener(self));
    }
    fn can_push_entity(&self, other: &dyn EntityBase) -> bool {
        other.get_player().is_none()
    }
    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(Self::ambient(&self.get_entity().world.load()))
    }
    fn has_global_tick_reach(&self) -> bool {
        true
    }
    fn as_tamable(&self) -> Option<&dyn super::tamable::TamableAnimal> {
        Some(self)
    }
    fn as_animal(&self) -> Option<&dyn super::animal::Animal> {
        Some(self)
    }
    fn as_ageable(&self) -> Option<&dyn crate::entity::ageable::AgeableMob> {
        Some(self)
    }
    fn as_custom_sound(&self) -> Option<&dyn crate::entity::custom_sound::CustomSound> {
        Some(self)
    }
    fn should_check_fall_damage(&self) -> bool {
        false
    }
    fn omnidirectional_air_mover(&self) -> bool {
        true
    }
    fn remove_when_far_away(&self, _distance_sq: f64) -> bool {
        false
    }
    fn mob_init_data_tracker(&self) {
        self.set_variant(self.variant.load());
    }
    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("Variant", self.variant.load().id());
    }
    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.set_variant(ParrotVariant::by_id(nbt.get_int("Variant").unwrap_or(0)));
    }
    fn finalize_spawn(
        &self,
        world: &Arc<crate::world::World>,
        _reason: crate::entity::mob::spawn::EntitySpawnReason,
        group: Option<crate::entity::mob::spawn::SpawnGroupData>,
    ) -> Option<crate::entity::mob::spawn::SpawnGroupData> {
        self.set_variant(
            ParrotVariant::ALL[self.get_random().random_range(0..ParrotVariant::ALL.len())],
        );
        self.default_finalize_spawn(world, group)
    }
    fn mob_set_variant_name(&self, name: &str) {
        if let Some(v) = ParrotVariant::from_name(name.strip_prefix("minecraft:").unwrap_or(name)) {
            self.set_variant(v);
        }
    }
    fn mob_apply_implicit_component(
        &self,
        component: pumpkin_data::data_component::DataComponent,
        value: &str,
    ) {
        if component == pumpkin_data::data_component::DataComponent::ParrotVariant {
            self.mob_set_variant_name(value);
        }
    }
    fn mob_get_component(
        &self,
        component: pumpkin_data::data_component::DataComponent,
    ) -> Option<Box<dyn pumpkin_data::data_component_impl::DataComponentImpl>> {
        (component == pumpkin_data::data_component::DataComponent::ParrotVariant).then(|| {
            Box::new(pumpkin_data::data_component_impl::ParrotVariantImpl {
                value: self.variant.load().name().into(),
            }) as Box<dyn pumpkin_data::data_component_impl::DataComponentImpl>
        })
    }
    fn pre_damage_with_context(
        &self,
        damage: DamageType,
        _source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        if !self.get_entity().is_invulnerable_to(&damage, cause) {
            super::tamable::TamableAnimal::set_ordered_to_sit(self, false);
        }
        true
    }
    fn mob_is_pushable(&self) -> bool {
        true
    }

    fn get_base_experience_reward(&self) -> u32 {
        crate::entity::passive::animal::experience_reward(self)
    }

    fn mob_ai_step(&self) {
        if let Some(pos) = self.jukebox.load() {
            let e = self.get_entity();
            if pos.to_centered_f64().squared_distance_to_vec(&e.pos.load()) >= 3.46 * 3.46
                || e.world.load().get_block(&pos).id != pumpkin_data::Block::JUKEBOX.id
            {
                self.party.store(false, Ordering::Relaxed);
                self.jukebox.store(None);
            }
        } else {
            self.party.store(false, Ordering::Relaxed);
        }
        if self.get_random().random_range(0..400) == 0 {
            Self::imitate_nearby(self);
        }
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
        use super::tamable::TamableAnimal;
        if !self.is_tame() && item_stack.item.has_tag(&tag::Item::MINECRAFT_PARROT_FOOD) {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            let pitch =
                1.0 + (self.get_random().random::<f32>() - self.get_random().random::<f32>()) * 0.2;
            self.play_entity_sound(Sound::EntityParrotEat, 1.0, pitch);
            let success = self.get_random().random_range(0..10) == 0 && self.approve_taming(player);
            if success {
                self.tame(player.gameprofile.id);
            }
            self.get_entity().world.load().send_entity_status(
                self.get_entity(),
                if success {
                    pumpkin_data::entity::EntityStatus::TamingSucceeded
                } else {
                    pumpkin_data::entity::EntityStatus::TamingFailed
                },
                if success {
                    pumpkin_protocol::bedrock::server::actor_event::ActorEventID::TamingSucceeded
                } else {
                    pumpkin_protocol::bedrock::server::actor_event::ActorEventID::TamingFailed
                },
            );
            return true;
        }
        if !item_stack
            .get_item()
            .has_tag(&tag::Item::MINECRAFT_PARROT_POISONOUS_FOOD)
        {
            if self.get_entity().on_ground.load(Ordering::Relaxed)
                && self.is_tame()
                && self.is_owned_by(&player.gameprofile.id)
            {
                self.order_to_sit(!self.is_ordered_to_sit());
                return true;
            }
            return self
                .mob_entity
                .mob_interact(player, item_stack, self.can_be_leashed());
        }

        self.eat_cookie(player, item_stack);
        true
    }
}

// Vanilla 26.3 Parrot.MOB_SOUND_MAP, including the deliberately empty happy ghast sound.
const IMITATIONS: &[(&EntityType, Sound)] = &[
    (&EntityType::BLAZE, Sound::EntityParrotImitateBlaze),
    (&EntityType::BOGGED, Sound::EntityParrotImitateBogged),
    (&EntityType::BREEZE, Sound::EntityParrotImitateBreeze),
    (&EntityType::CAMEL_HUSK, Sound::EntityParrotImitateCamelHusk),
    (&EntityType::CAVE_SPIDER, Sound::EntityParrotImitateSpider),
    (&EntityType::CREAKING, Sound::EntityParrotImitateCreaking),
    (&EntityType::CREEPER, Sound::EntityParrotImitateCreeper),
    (&EntityType::DROWNED, Sound::EntityParrotImitateDrowned),
    (
        &EntityType::ELDER_GUARDIAN,
        Sound::EntityParrotImitateElderGuardian,
    ),
    (
        &EntityType::ENDER_DRAGON,
        Sound::EntityParrotImitateEnderDragon,
    ),
    (&EntityType::ENDERMITE, Sound::EntityParrotImitateEndermite),
    (&EntityType::EVOKER, Sound::EntityParrotImitateEvoker),
    (&EntityType::GHAST, Sound::EntityParrotImitateGhast),
    (&EntityType::HAPPY_GHAST, Sound::IntentionallyEmpty),
    (&EntityType::GUARDIAN, Sound::EntityParrotImitateGuardian),
    (&EntityType::HOGLIN, Sound::EntityParrotImitateHoglin),
    (&EntityType::HUSK, Sound::EntityParrotImitateHusk),
    (
        &EntityType::ILLUSIONER,
        Sound::EntityParrotImitateIllusioner,
    ),
    (&EntityType::MAGMA_CUBE, Sound::EntityParrotImitateMagmaCube),
    (&EntityType::PARCHED, Sound::EntityParrotImitateParched),
    (&EntityType::PHANTOM, Sound::EntityParrotImitatePhantom),
    (&EntityType::PIGLIN, Sound::EntityParrotImitatePiglin),
    (
        &EntityType::PIGLIN_BRUTE,
        Sound::EntityParrotImitatePiglinBrute,
    ),
    (&EntityType::PILLAGER, Sound::EntityParrotImitatePillager),
    (&EntityType::RAVAGER, Sound::EntityParrotImitateRavager),
    (&EntityType::SHULKER, Sound::EntityParrotImitateShulker),
    (
        &EntityType::SILVERFISH,
        Sound::EntityParrotImitateSilverfish,
    ),
    (&EntityType::SKELETON, Sound::EntityParrotImitateSkeleton),
    (&EntityType::SLIME, Sound::EntityParrotImitateSlime),
    (&EntityType::SPIDER, Sound::EntityParrotImitateSpider),
    (&EntityType::STRAY, Sound::EntityParrotImitateStray),
    (&EntityType::VEX, Sound::EntityParrotImitateVex),
    (
        &EntityType::VINDICATOR,
        Sound::EntityParrotImitateVindicator,
    ),
    (&EntityType::WARDEN, Sound::EntityParrotImitateWarden),
    (&EntityType::WITCH, Sound::EntityParrotImitateWitch),
    (&EntityType::WITHER, Sound::EntityParrotImitateWither),
    (
        &EntityType::WITHER_SKELETON,
        Sound::EntityParrotImitateWitherSkeleton,
    ),
    (&EntityType::ZOGLIN, Sound::EntityParrotImitateZoglin),
    (&EntityType::ZOMBIE, Sound::EntityParrotImitateZombie),
    (
        &EntityType::ZOMBIE_HORSE,
        Sound::EntityParrotImitateZombieHorse,
    ),
    (
        &EntityType::ZOMBIE_NAUTILUS,
        Sound::EntityParrotImitateZombieNautilus,
    ),
    (
        &EntityType::ZOMBIE_VILLAGER,
        Sound::EntityParrotImitateZombieVillager,
    ),
];

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
