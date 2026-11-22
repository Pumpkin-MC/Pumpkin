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
        // Vanilla sends the attack status even when the damage is rejected.
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
            self.reset_ambient_sound_time();
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

#[cfg(test)]
mod tests {
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_world::{chunk::ChunkData, world::BlockFlags};

    use super::*;
    use crate::{
        block::entities::BlockEntity,
        entity::{
            Entity,
            ai::test_support::world,
            passive::{cow::CowEntity, iron_golem::IronGolemEntity, wolf::WolfEntity},
            projectile::{ProjectileHit, arrow::ArrowEntity},
        },
    };

    fn add_linked_creaking(world: &Arc<World>) -> Arc<CreakingEntity> {
        let home = BlockPos::new(8, 64, 8);
        let chunk = home.chunk_position();
        world
            .level
            .loaded_chunks
            .insert(chunk, ChunkData::empty_sync(chunk.x, chunk.y));
        world.set_block_state(
            &home,
            Block::CREAKING_HEART.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );
        let creaking = CreakingEntity::new(Entity::new(
            world.clone(),
            Vector3::new(10.5, 64.0, 8.5),
            &EntityType::CREAKING,
        ));
        creaking.set_transient(home);
        let mut nbt = NbtCompound::new();
        nbt.put_uuid("creaking", creaking.get_entity().entity_uuid);
        world.add_block_entity(Arc::new(CreakingHeartBlockEntity::from_nbt(&nbt, home)));
        creaking
    }

    fn assert_protected_melee(creaking: &CreakingEntity, attacker: &dyn Mob) {
        let attack = || attacker.do_hurt_target(creaking);
        let health = &creaking.mob_entity.living_entity.health;
        let before = health.load();
        let accepted = attack();
        let after = health.load();
        let repeated = attack();
        assert_eq!(before, 1.0);
        assert!(accepted, "a living attacker should trigger a protected hit");
        assert_eq!(after, before);
        assert!(!repeated, "the accepted hit should start the cooldown");
    }

    #[tokio::test]
    async fn creaking_accepts_wolf_melee_as_a_protected_hit()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let creaking = add_linked_creaking(&world);
        let wolf = WolfEntity::new(Entity::new(
            world.clone(),
            Vector3::new(11.5, 64.0, 8.5),
            &EntityType::WOLF,
        ));
        assert_protected_melee(creaking.as_ref(), wolf.as_ref());
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn creaking_accepts_iron_golem_melee_as_a_protected_hit()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let creaking = add_linked_creaking(&world);
        let golem = IronGolemEntity::new(Entity::new(
            world.clone(),
            Vector3::new(11.5, 64.0, 8.5),
            &EntityType::IRON_GOLEM,
        ));
        assert_protected_melee(creaking.as_ref(), golem.as_ref());
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn creaking_punch_arrow_preserves_the_protected_hit_but_respects_freeze()
    -> Result<(), Box<dyn std::error::Error>> {
        for can_move in [true, false] {
            let folder = tempfile::tempdir()?;
            let world = world(&folder);
            let creaking = add_linked_creaking(&world);
            creaking.set_can_move(can_move);
            let arrow = ArrowEntity::new(
                Entity::new(
                    world.clone(),
                    Vector3::new(9.5, 65.0, 8.5),
                    &EntityType::ARROW,
                ),
                None,
            );
            arrow.punch_level.store(1, Relaxed);
            arrow
                .get_entity()
                .velocity
                .store(Vector3::new(1.0, 0.0, 0.0));
            // New entities start dirty before their first tracker update.
            creaking.get_entity().velocity_dirty.store(false, Relaxed);
            assert_eq!(creaking.get_entity().velocity.load(), Vector3::default());
            assert_eq!(
                creaking
                    .invulnerability_animation_remaining_ticks
                    .load(Relaxed),
                0
            );

            arrow.on_hit(ProjectileHit::Entity {
                entity: creaking.clone(),
                hit_pos: creaking.get_entity().pos.load(),
                normal: Vector3::new(-1.0, 0.0, 0.0),
            });

            assert_eq!(creaking.mob_entity.living_entity.health.load(), 1.0);
            assert_eq!(
                creaking
                    .invulnerability_animation_remaining_ticks
                    .load(Relaxed),
                INVULNERABILITY_ANIMATION_DURATION
            );
            let motion = creaking.get_entity().velocity.load();
            let dirty = creaking.get_entity().velocity_dirty.load(Relaxed);
            world.level.shutdown().await;
            if can_move {
                assert!(motion.x > 0.0 && motion.y > 0.0);
                assert!(dirty);
            } else {
                assert_eq!(motion, Vector3::default());
                assert!(!dirty);
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn creaking_mace_shockwave_respects_freeze() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let creaking = add_linked_creaking(&world);
        let make_cow = |x| {
            CowEntity::new(Entity::new(
                world.clone(),
                Vector3::new(x, 64.0, 8.5),
                &EntityType::COW,
            ))
        };
        let attacker = make_cow(6.5);
        let victim = make_cow(9.5);
        world.add_entity_silent(creaking.clone());
        let entity = creaking.get_entity();
        assert_eq!(entity.velocity.load(), Vector3::default());
        crate::entity::combat::mace_smash_knockback(&world, attacker.as_ref(), victim.as_ref());
        let moving = entity.velocity.load();
        entity.velocity.store(Vector3::default());
        creaking.set_can_move(false);
        crate::entity::combat::mace_smash_knockback(&world, attacker.as_ref(), victim.as_ref());
        let frozen = entity.velocity.load();
        world.level.shutdown().await;
        assert!(moving.x > 0.0 && moving.y > 0.0);
        assert_eq!(frozen, Vector3::default());
        Ok(())
    }

    #[tokio::test]
    async fn creaking_protected_hit_resets_ambient_sound_time()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let creaking = add_linked_creaking(&world);
        let attacker = WolfEntity::new(Entity::new(
            world.clone(),
            Vector3::new(11.5, 64.0, 8.5),
            &EntityType::WOLF,
        ));
        let timer = &creaking.mob_entity.ambient_sound_time;
        timer.store(20, Relaxed);
        let accepted = creaking.damage_with_context(
            creaking.as_ref(),
            1.0,
            DamageType::MOB_ATTACK,
            None,
            Some(attacker.as_ref()),
            Some(attacker.as_ref()),
        );
        let after = timer.load(Relaxed);
        world.level.shutdown().await;

        assert!(accepted);
        assert_eq!(after, -creaking.get_ambient_sound_interval());
        Ok(())
    }
}
