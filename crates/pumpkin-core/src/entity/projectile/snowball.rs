use crate::entity::projectile::owner::ProjectileOwner;
use std::sync::atomic::AtomicBool;

use crate::entity::projectile::ProjectileHit;
use crate::world::game_event::GameEventContext;
use crate::{
    entity::{Entity, EntityBase, projectile::ThrownItemEntity},
    server::Server,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::game_event::GameEvent;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;

const GRAVITY: f64 = 0.03;

pub struct SnowballEntity {
    pub thrown: ThrownItemEntity,
}

impl SnowballEntity {
    pub const fn new(entity: Entity) -> Self {
        let thrown = ThrownItemEntity {
            entity,
            owner: ProjectileOwner::none(),
            collides_with_projectiles: false,
            has_hit: AtomicBool::new(false),
            gravity: GRAVITY,
        };

        Self { thrown }
    }

    pub fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        let thrown = ThrownItemEntity::new(entity, shooter, GRAVITY);
        Self { thrown }
    }
}

impl EntityBase for SnowballEntity {
    fn get_projectile_owner(&self) -> Option<&ProjectileOwner> {
        Some(&self.thrown.owner)
    }

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        self.thrown.process_tick(caller);
    }

    fn get_entity(&self) -> &Entity {
        self.thrown.get_entity()
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }
    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, hit: crate::entity::projectile::ProjectileHit) {
        let world = self.get_entity().world.load();

        if let ProjectileHit::Entity { ref entity, .. } = hit {
            let is_blaze = entity.get_entity().entity_type.id == EntityType::BLAZE.id;
            let damage = if is_blaze { 3.0 } else { 0.0 }; // Only damage blazes

            let owner = self.thrown.owner.get(&world);
            entity.damage_with_context(
                entity.as_ref(),
                damage,
                DamageType::THROWN,
                None,
                Some(self),
                owner.as_deref(),
            );
        }
        let event_pos = match &hit {
            ProjectileHit::Block { pos, .. } => pos.to_centered_f64(),
            ProjectileHit::Entity { hit_pos, .. } => *hit_pos,
        };
        world.emit_game_event_with_context(
            GameEvent::ProjectileLand.name(),
            event_pos,
            &GameEventContext {
                source_entity: Some(self),
            },
        );
        world.send_entity_status(self.get_entity(), EntityStatus::Death, ActorEventID::Death);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;
    use crate::entity::mob::blaze::BlazeEntity;
    use crate::entity::mob::{Mob, MobEntity};
    use crate::entity::passive::snow_golem::SnowGolemEntity;
    use crate::world::World;
    use crate::world::game_event::GameEventListener;
    use pumpkin_data::Block;
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_util::math::{position::BlockPos, vector2::Vector2, vector3::Vector3};
    use pumpkin_world::{chunk::ChunkData, world::BlockFlags};
    use std::sync::{Arc, Mutex};

    type TestResult = Result<(), Box<dyn std::error::Error>>;
    type ObservedEvent = (GameEvent, Vector3<f64>, Option<i32>);

    struct EventObserver {
        mob: MobEntity,
        events: Mutex<Vec<ObservedEvent>>,
    }

    impl Mob for EventObserver {
        fn get_mob_entity(&self) -> &MobEntity {
            &self.mob
        }

        fn for_each_game_event_listener(&self, visit: &mut dyn FnMut(&dyn GameEventListener)) {
            visit(self);
        }
    }

    impl GameEventListener for EventObserver {
        fn listener_position(&self) -> Vector3<f64> {
            self.get_entity().pos.load()
        }

        fn listener_radius(&self) -> i32 {
            16
        }

        fn handle_game_event(
            &self,
            _world: &World,
            event: GameEvent,
            context: &GameEventContext<'_>,
            position: Vector3<f64>,
        ) -> bool {
            self.events.lock().unwrap().push((
                event,
                position,
                context
                    .source_entity
                    .map(|entity| entity.get_entity().entity_id),
            ));
            true
        }
    }

    #[tokio::test]
    async fn snowball_updates_water_drag_and_is_removed_in_the_void() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
        for x in 0..3 {
            world.set_block_state(
                &BlockPos::new(x, 64, 0),
                Block::WATER.default_state.id,
                BlockFlags::SKIP_BLOCK_ADDED_CALLBACK | BlockFlags::UPDATE_KNOWN_SHAPE,
            );
        }
        let snowball = SnowballEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.5, 0.5),
            &EntityType::SNOWBALL,
        ));
        snowball
            .get_entity()
            .set_velocity(Vector3::new(0.2, 0.0, 0.0));
        snowball.thrown.process_tick(&snowball);
        assert!(snowball.get_entity().is_in_water());
        assert!(
            !snowball
                .get_entity()
                .first_tick
                .load(std::sync::atomic::Ordering::Relaxed)
        );
        snowball.thrown.process_tick(&snowball);
        // Vanilla 26.3: first step uses air drag, second step uses water drag.
        assert!((snowball.get_entity().velocity.load().x - 0.15840000388622288).abs() < 1.0e-12);
        snowball.get_entity().set_pos(Vector3::new(
            0.5,
            f64::from(world.dimension.min_y) - 65.0,
            0.5,
        ));
        snowball.thrown.process_tick(&snowball);
        assert!(snowball.get_entity().is_removed());
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn snowball_near_miss_depends_on_projectile_age() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
        let target = SnowGolemEntity::new(Entity::new(
            world.clone(),
            Vector3::new(3.5, 64.0, 1.0),
            &EntityType::SNOW_GOLEM,
        ));
        world.spawn_entity(target);
        for (age, should_hit) in [(1, false), (8, true)] {
            let snowball = SnowballEntity::new(Entity::new(
                world.clone(),
                Vector3::new(0.5, 65.5, 0.5),
                &EntityType::SNOWBALL,
            ));
            snowball
                .get_entity()
                .tick_count
                .store(age, std::sync::atomic::Ordering::Relaxed);
            snowball
                .get_entity()
                .set_velocity(Vector3::new(6.0, 0.0, 0.0));
            snowball.thrown.process_tick(&snowball);
            assert_eq!(snowball.get_entity().is_removed(), should_hit);
        }
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn snowball_redirects_a_fireball_and_transfers_ownership() -> TestResult {
        use crate::entity::projectile::fireball::FireballEntity;
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
        let owner = SnowGolemEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::SNOW_GOLEM,
        ));
        owner.get_entity().set_rotation(90.0, 0.0);
        world.spawn_entity(owner.clone());
        let fireball = Arc::new(FireballEntity::new(Entity::new(
            world.clone(),
            Vector3::new(4.5, 65.0, 0.5),
            &EntityType::FIREBALL,
        )));
        fireball.set_acceleration_power(0.5);
        world.spawn_entity(fireball.clone());
        let snowball = SnowballEntity::new_shot(
            Entity::new(world.clone(), Vector3::default(), &EntityType::SNOWBALL),
            owner.get_entity(),
        );
        snowball
            .get_entity()
            .set_velocity(Vector3::new(6.0, 0.0, 0.0));
        snowball.thrown.process_tick(&snowball);
        assert!(snowball.get_entity().is_removed());
        assert!(!fireball.get_entity().is_removed());
        assert!(fireball.thrown.owner.matches(owner.get_entity()));
        assert!(fireball.get_entity().velocity.load().x < -0.99);
        assert!(fireball.get_acceleration_power() < 0.5);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn snowball_preserves_launch_event_across_reload_and_stops_at_impact() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
        let observer = Arc::new(EventObserver {
            mob: MobEntity::new(Entity::new(
                world.clone(),
                Vector3::new(0.5, 64.0, 5.5),
                &EntityType::SNOW_GOLEM,
            )),
            events: Mutex::new(Vec::new()),
        });
        world.spawn_entity(observer.clone());
        let owner = SnowGolemEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::SNOW_GOLEM,
        ));
        world.spawn_entity(owner.clone());
        let snowball = SnowballEntity::new_shot(
            Entity::new(world.clone(), Vector3::default(), &EntityType::SNOWBALL),
            owner.get_entity(),
        );
        snowball
            .get_entity()
            .set_velocity(Vector3::new(0.5, 0.0, 0.0));
        snowball.thrown.process_tick(&snowball);
        let launch_position = snowball.get_entity().pos.load();
        let mut saved = NbtCompound::new();
        snowball.write_nbt(&mut saved);
        let restored = SnowballEntity::new(Entity::new(
            world.clone(),
            Vector3::default(),
            &EntityType::SNOWBALL,
        ));
        restored.read_nbt_non_mut(&saved);
        restored.get_entity().set_pos(Vector3::new(0.5, 68.0, 0.5));
        restored.thrown.process_tick(&restored);

        world.set_block_state(
            &BlockPos::new(2, 65, 0),
            Block::STONE.default_state.id,
            BlockFlags::SKIP_BLOCK_ADDED_CALLBACK | BlockFlags::UPDATE_KNOWN_SHAPE,
        );
        snowball
            .get_entity()
            .set_velocity(Vector3::new(3.0, 0.0, 0.0));
        snowball.thrown.process_tick(&snowball);
        assert_eq!(snowball.get_entity().pos.load().x, 2.0);
        assert!(snowball.get_entity().is_removed());
        let landed = {
            let events = observer.events.lock().unwrap();
            let launches: Vec<_> = events
                .iter()
                .filter(|(event, _, _)| *event == GameEvent::ProjectileShoot)
                .collect();
            assert_eq!(launches.len(), 1);
            assert_eq!(launches[0].1, launch_position);
            assert_eq!(launches[0].2, Some(owner.get_entity().entity_id));
            events.iter().any(|(event, position, source)| {
                *event == GameEvent::ProjectileLand
                    && *position == Vector3::new(2.5, 65.5, 0.5)
                    && *source == Some(snowball.get_entity().entity_id)
            })
        };
        assert!(landed);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn snowball_deflects_from_breeze_without_landing_or_changing_owner() -> TestResult {
        use crate::entity::mob::breeze::BreezeEntity;

        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
        let owner = SnowGolemEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::SNOW_GOLEM,
        ));
        world.spawn_entity(owner.clone());
        let breeze = BreezeEntity::new(Entity::new(
            world.clone(),
            Vector3::new(4.5, 64.0, 0.5),
            &EntityType::BREEZE,
        ));
        world.spawn_entity(breeze.clone());
        let observer = Arc::new(EventObserver {
            mob: MobEntity::new(Entity::new(
                world.clone(),
                Vector3::new(0.5, 64.0, 5.5),
                &EntityType::SNOW_GOLEM,
            )),
            events: Mutex::new(Vec::new()),
        });
        world.spawn_entity(observer.clone());
        let snowball = SnowballEntity::new_shot(
            Entity::new(world.clone(), Vector3::default(), &EntityType::SNOWBALL),
            owner.get_entity(),
        );
        let start = snowball.get_entity().pos.load();
        snowball
            .get_entity()
            .set_velocity(Vector3::new(6.0, 0.0, 0.0));
        snowball.thrown.process_tick(&snowball);
        assert!(!snowball.get_entity().is_removed());
        assert!((-3.1..-2.9).contains(&snowball.get_entity().velocity.load().x));
        assert!(snowball.thrown.owner.matches(owner.get_entity()));
        assert!(
            breeze
                .mob_entity
                .living_entity
                .get_last_damage_source()
                .is_none()
        );
        assert!(
            !observer
                .events
                .lock()
                .unwrap()
                .iter()
                .any(|(event, _, _)| *event == GameEvent::ProjectileLand)
        );

        // Re-entering the same deflector must not reverse and halve the velocity again.
        snowball.get_entity().set_pos(start);
        snowball
            .get_entity()
            .set_velocity(Vector3::new(6.0, 0.0, 0.0));
        snowball.thrown.process_tick(&snowball);
        assert!(!snowball.get_entity().is_removed());
        assert!(snowball.get_entity().velocity.load().x > 5.9);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn snowball_damage_records_the_golem_as_attacker() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let owner = SnowGolemEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::SNOW_GOLEM,
        ));
        world.spawn_entity(owner.clone());
        let target = BlazeEntity::new(Entity::new(
            world.clone(),
            Vector3::new(4.5, 64.0, 0.5),
            &EntityType::BLAZE,
        ));
        let snowball = SnowballEntity::new_shot(
            Entity::new(world.clone(), Vector3::default(), &EntityType::SNOWBALL),
            owner.get_entity(),
        );
        snowball.on_hit(ProjectileHit::Entity {
            entity: target.clone(),
            hit_pos: target.get_entity().pos.load(),
            normal: Vector3::new(-1.0, 0.0, 0.0),
        });
        assert_eq!(
            target.entity.living_entity.get_last_damage_source(),
            Some((DamageType::THROWN, owner.get_entity().entity_id))
        );
        world.level.shutdown().await;
        Ok(())
    }
}
