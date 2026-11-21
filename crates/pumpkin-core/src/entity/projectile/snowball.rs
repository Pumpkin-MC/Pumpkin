use crate::entity::projectile::owner::ProjectileOwner;

use crate::entity::projectile::ProjectileHit;
use crate::{
    entity::{Entity, EntityBase, projectile::ThrownItemEntity},
    server::Server,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::{EntityStatus, EntityType};
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

    fn tick(&self, caller: &dyn EntityBase, server: &Server) {
        self.thrown.process_tick(caller, server);
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

        if let ProjectileHit::Entity {
            ref entity,
            hit_pos,
            ..
        } = hit
        {
            let is_blaze = entity.get_entity().entity_type.id == EntityType::BLAZE.id;
            let damage = if is_blaze { 3.0 } else { 0.0 };
            let owner = self.thrown.owner.get(&world);
            entity.damage_with_context(
                entity.as_ref(),
                damage,
                DamageType::THROWN,
                Some(hit_pos),
                Some(self),
                owner.as_deref(),
            );
        }
        world.send_entity_status(self.get_entity(), EntityStatus::Death, ActorEventID::Death);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{
        ai::test_support::world, mob::blaze::BlazeEntity, passive::snow_golem::SnowGolemEntity,
    };
    use pumpkin_util::math::vector3::Vector3;
    type TestResult = Result<(), Box<dyn std::error::Error>>;

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
