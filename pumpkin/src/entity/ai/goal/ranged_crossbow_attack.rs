use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_protocol::IdOr;
use pumpkin_protocol::java::client::play::CSoundEffect;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::goal::{Controls, Goal, GoalFuture};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::projectile::arrow::{ArrowEntity, ArrowPickup};
use crate::entity::{Entity, EntityBase};

pub struct RangedCrossbowAttackGoal {
    attack_cooldown: i32,
    range: f64,
}

impl RangedCrossbowAttackGoal {
    #[must_use]
    pub const fn new(range: f64) -> Self {
        Self {
            attack_cooldown: 0,
            range,
        }
    }

    async fn has_crossbow(mob: &dyn Mob) -> bool {
        let stack = mob.get_mob_entity().living_entity.held_item(mob).await;
        stack.lock().await.item.id == Item::CROSSBOW.id
    }

    async fn has_line_of_sight(mob: &dyn Mob, target: &dyn EntityBase) -> bool {
        let entity = mob.get_entity();
        entity
            .world
            .load_full()
            .raycast(
                entity.get_eye_pos(),
                target.get_entity().get_eye_pos(),
                async |block_pos, world| world.get_block_state(block_pos).is_solid(),
            )
            .await
            .is_none()
    }

    async fn shoot(mob: &dyn Mob, target: &dyn EntityBase) {
        let shooter = mob.get_entity();
        let world = shooter.world.load_full();
        let arrow_entity = Entity::new(world.clone(), shooter.pos.load(), &EntityType::ARROW);
        let arrow_item = pumpkin_data::item_stack::ItemStack::new(1, &Item::ARROW);
        let arrow =
            ArrowEntity::new_shot(arrow_entity, shooter, &arrow_item, ArrowPickup::Disallowed);
        let shooter_pos = shooter.get_eye_pos();
        let target_pos = target.get_entity().pos.load();
        let dx = target_pos.x - shooter_pos.x;
        let dz = target_pos.z - shooter_pos.z;
        let horizontal = dx.hypot(dz);
        let direction = Vector3::new(
            dx,
            target_pos.y + target.get_entity().get_eye_height() / 3.0 - shooter_pos.y
                + horizontal * 0.2,
            dz,
        );
        arrow.set_velocity(direction.x, direction.y, direction.z, 1.6, 10.0);
        world.spawn_entity(Arc::new(arrow)).await;

        let sound = CSoundEffect::new(
            IdOr::Id(Sound::ItemCrossbowShoot as u16),
            SoundCategory::Hostile,
            &shooter.pos.load(),
            1.0,
            1.0,
            0.0,
        );
        world.broadcast_to_chunk(shooter.chunk_pos.load(), &sound);
    }
}

impl Goal for RangedCrossbowAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            Self::has_crossbow(mob).await
                && mob
                    .get_mob_entity()
                    .target
                    .lock()
                    .await
                    .as_ref()
                    .is_some_and(|target| target.get_entity().is_alive())
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            Self::has_crossbow(mob).await
                && mob
                    .get_mob_entity()
                    .target
                    .lock()
                    .await
                    .as_ref()
                    .is_some_and(|target| target.get_entity().is_alive())
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.attack_cooldown = 20;
            mob.get_mob_entity().set_attacking(true);
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.get_mob_entity().navigator.lock().unwrap().stop();
            mob.get_mob_entity().set_attacking(false);
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(target) = mob.get_mob_entity().target.lock().await.clone() else {
                return;
            };
            let entity = mob.get_entity();
            let target_pos = target.get_entity().pos.load();
            let distance_squared = entity.pos.load().squared_distance_to_vec(&target_pos);
            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap()
                .look_at_entity_with_range(&target, 30.0, 30.0);
            self.attack_cooldown = (self.attack_cooldown - 1).max(0);

            if distance_squared > self.range * self.range {
                mob.get_mob_entity()
                    .navigator
                    .lock()
                    .unwrap()
                    .set_progress(NavigatorGoal {
                        current_progress: entity.pos.load(),
                        destination: target_pos,
                        speed: 1.0,
                    });
            } else {
                mob.get_mob_entity().navigator.lock().unwrap().stop();
                if self.attack_cooldown == 0 && Self::has_line_of_sight(mob, target.as_ref()).await
                {
                    Self::shoot(mob, target.as_ref()).await;
                    self.attack_cooldown = 40;
                }
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
