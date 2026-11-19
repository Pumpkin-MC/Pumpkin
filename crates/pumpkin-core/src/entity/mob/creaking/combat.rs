use std::sync::{Arc, atomic::Ordering::Relaxed};

use pumpkin_data::Block;
use pumpkin_data::block_properties::{CreakingHeartLikeProperties, CreakingHeartState};
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::entity_status::EntityStatus;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use super::{
    ATTACK_ANIMATION_DURATION, CreakingEntity, INVULNERABILITY_ANIMATION_DURATION,
    TWITCH_DEATH_DURATION,
};
use crate::block::entities::creaking_heart::CreakingHeartBlockEntity;
use crate::entity::custom_sound::CustomSound;
use crate::entity::{EntityBase, living::LivingEntity, mob::Mob};
use crate::world::{BlockEntityLookup, World};

// Reserve the hit during plugin callbacks without starting the animation timer.
const DAMAGE_IN_PROGRESS: i32 = -1;

#[derive(Clone, Copy)]
enum PlayerDamageCredit {
    Retain,
    Replace(Option<Uuid>),
}

impl PlayerDamageCredit {
    fn from_attacker(attacker: Option<&dyn EntityBase>) -> Self {
        let Some(attacker) = attacker else {
            return Self::Retain;
        };
        if let Some(player) = attacker.get_player() {
            return Self::Replace(Some(player.gameprofile.id));
        }
        if attacker.get_entity().entity_type == &EntityType::WOLF
            && let Some(wolf) = attacker.get_mob().and_then(Mob::as_tamable)
            && wolf.is_tame()
        {
            return Self::Replace(wolf.get_owner());
        }
        Self::Retain
    }

    fn resolves_to_player(&self, living: &LivingEntity) -> bool {
        let player = match *self {
            Self::Retain => living.last_hurt_by_player.load(),
            Self::Replace(owner) => owner,
        };
        player.is_some_and(|uuid| {
            let world = living.entity.world.load();
            world.server.upgrade().map_or_else(
                || world.get_player_by_uuid(uuid).is_some(),
                |server| server.get_player_by_uuid(uuid).is_some(),
            )
        })
    }
}

impl CreakingEntity {
    pub(super) fn attack_target(&self, target: &dyn EntityBase) -> bool {
        if target.get_living_entity().is_none() {
            return false;
        }
        self.attack_animation_remaining_ticks
            .store(ATTACK_ANIMATION_DURATION, Relaxed);
        let entity = self.get_entity();
        // The vanilla client plays the attack sound when handling this status.
        entity.world.load().broadcast_entity_event(
            entity,
            EntityStatus::StartAttacking,
            ActorEventID::None,
        );
        self.mob_entity.try_attack(self, target)
    }

    pub(super) fn hurt_transient(
        &self,
        amount: f32,
        damage_type: DamageType,
        position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> Option<bool> {
        let home_pos = self.get_home_pos()?;
        if damage_type.has_tag(&tag::DamageType::MINECRAFT_BYPASSES_INVULNERABILITY) {
            return None;
        }
        let living = &self.mob_entity.living_entity;
        let entity = &living.entity;
        let world = entity.world.load_full();
        if entity.is_invulnerable_to(&damage_type, cause)
            || living.is_dead_or_dying()
            || self
                .invulnerability_animation_remaining_ticks
                .compare_exchange(0, DAMAGE_IN_PROGRESS, Relaxed, Relaxed)
                .is_err()
        {
            return Some(false);
        }

        let attacker = cause.or(source);
        let credit = PlayerDamageCredit::from_attacker(attacker);
        let direct_hit = source.is_some_and(|direct| {
            direct.get_living_entity().is_some() || direct.get_projectile_owner().is_some()
        });
        if !direct_hit && !credit.resolves_to_player(living) {
            self.blame_source_for_damage(damage_type, attacker, credit);
            self.invulnerability_animation_remaining_ticks
                .store(0, Relaxed);
            return Some(false);
        }

        if living
            .fire_damage_events(amount, damage_type, position, source, cause)
            .is_none()
            || !self.is_live_in_world(&world)
            || entity.is_invulnerable_to(&damage_type, cause)
            || self.get_home_pos() != Some(home_pos)
        {
            self.invulnerability_animation_remaining_ticks
                .store(0, Relaxed);
            return Some(false);
        }

        self.blame_source_for_damage(damage_type, attacker, credit);
        self.invulnerability_animation_remaining_ticks
            .store(INVULNERABILITY_ANIMATION_DURATION, Relaxed);
        world.broadcast_entity_event(entity, EntityStatus::Shake, ActorEventID::None);
        if !self.is_live_in_world(&world) || self.get_home_pos() != Some(home_pos) {
            return Some(true);
        }
        entity.emit_own_game_event(self, GameEvent::EntityAction);
        if !self.is_live_in_world(&world) || self.get_home_pos() != Some(home_pos) {
            return Some(true);
        }
        let hit_by_player = credit.resolves_to_player(living);
        match world.get_block_entity_if_chunk_loaded(&home_pos) {
            BlockEntityLookup::Unloaded => {
                self.request_home_chunk(&world, home_pos, Some(hit_by_player));
            }
            BlockEntityLookup::Loaded(block_entity) => {
                if let Some(heart) = block_entity
                    .as_deref()
                    .and_then(|block_entity| self.linked_heart(block_entity))
                {
                    self.play_heart_feedback(&world, heart, hit_by_player);
                }
            }
        }
        Some(true)
    }

    pub(super) fn play_heart_feedback(
        &self,
        world: &Arc<World>,
        heart: &CreakingHeartBlockEntity,
        hit_by_player: bool,
    ) {
        if !heart.can_notify_protector(world, self) {
            return;
        }
        if hit_by_player {
            heart.creaking_hurt(world, self);
        }
        if heart.can_notify_protector(world, self) {
            self.make_sound(Sound::EntityCreakingSway);
        }
    }

    fn blame_source_for_damage(
        &self,
        damage_type: DamageType,
        attacker: Option<&dyn EntityBase>,
        credit: PlayerDamageCredit,
    ) {
        let living = &self.mob_entity.living_entity;
        living.resolve_mob_responsible_for_damage(damage_type, attacker);
        if let PlayerDamageCredit::Replace(player) = credit {
            living.set_last_hurt_by_player(player);
        }
    }

    pub fn start_teardown(
        &self,
        damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) {
        let living = &self.mob_entity.living_entity;
        let attacker = cause.or(source);
        self.blame_source_for_damage(
            damage_type,
            attacker,
            PlayerDamageCredit::from_attacker(attacker),
        );
        living.on_death_with_sound(damage_type, source, cause, false);
        self.make_sound(Sound::EntityCreakingTwitch);
        self.set_tearing_down();
        living.set_health(0.0);
    }

    pub(super) fn tick_teardown(&self, death_time: u8) -> bool {
        if !self.is_heart_bound() || !self.is_tearing_down() {
            return false;
        }
        if death_time > TWITCH_DEATH_DURATION {
            self.tear_down();
        }
        true
    }

    pub fn tear_down(&self) {
        let entity = self.get_entity();
        if entity.is_removed() {
            return;
        }
        let world = entity.world.load();
        let bounds = entity.bounding_box.load();
        let position = bounds.get_center();
        let spread = Vector3::new(
            ((bounds.max.x - bounds.min.x) * 0.3) as f32,
            ((bounds.max.y - bounds.min.y) * 0.3) as f32,
            ((bounds.max.z - bounds.min.z) * 0.3) as f32,
        );
        world.spawn_block_crumble(position, spread, 100, Block::PALE_OAK_WOOD.default_state.id);
        let mut properties =
            CreakingHeartLikeProperties::from_state_id(Block::CREAKING_HEART.default_state.id);
        properties.creaking_heart_state = CreakingHeartState::Awake;
        world.spawn_block_crumble(
            position,
            spread,
            10,
            properties.to_state_id(&Block::CREAKING_HEART),
        );
        self.make_sound(Sound::EntityCreakingDeath);
        entity.remove();
    }
}

impl CustomSound for CreakingEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityCreakingDeath)
    }

    fn hurt_sound(&self) -> Option<Sound> {
        self.is_heart_bound().then_some(Sound::EntityCreakingSway)
    }
}
