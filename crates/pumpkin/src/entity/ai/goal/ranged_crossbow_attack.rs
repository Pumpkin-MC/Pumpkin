use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_protocol::IdOr;
use pumpkin_protocol::java::client::play::CSoundEffect;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::ai::goal::{Controls, Goal, GoalFuture};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::projectile::arrow::{ArrowEntity, ArrowPickup};
use crate::entity::{Entity, EntityBase};

/// Vanilla: `RangedCrossbowAttackGoal.CrossbowState`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CrossbowState {
    Uncharged,
    Charging,
    Charged,
    ReadyToAttack,
}

/// Vanilla: `CrossbowItem.getChargeDuration` -- `Mth.floor(1.25F * 20.0F)` with no Quick Charge
/// enchant applied (mobs never have it).
const CHARGE_DURATION_TICKS: i32 = 25;

pub struct RangedCrossbowAttackGoal {
    state: CrossbowState,
    speed_modifier: f64,
    attack_radius_sqr: f64,
    see_time: i32,
    attack_delay: i32,
    update_path_delay: i32,
}

impl RangedCrossbowAttackGoal {
    #[must_use]
    pub const fn new(range: f64) -> Self {
        Self {
            state: CrossbowState::Uncharged,
            speed_modifier: 1.0,
            attack_radius_sqr: range * range,
            see_time: 0,
            attack_delay: 0,
            update_path_delay: 0,
        }
    }

    async fn has_crossbow(mob: &dyn Mob) -> bool {
        let stack = mob.get_mob_entity().living_entity.held_item(mob).await;
        stack.lock().await.item.id == Item::CROSSBOW.id
    }

    async fn has_line_of_sight(mob: &dyn Mob, target: &dyn EntityBase) -> bool {
        let entity = mob.get_entity();
        let target_entity = target.get_entity();
        let from = entity.get_eye_pos();
        let to = target_entity.get_eye_pos();
        if from.squared_distance_to_vec(&to) > 128.0 * 128.0 {
            return false;
        }

        let world = entity.world.load_full();
        if !Arc::ptr_eq(&world, &target_entity.world.load_full()) {
            return false;
        }

        world
            .raycast_collision(from, to, async |block_pos, world| {
                !world.get_block_state(block_pos).collision_shapes.is_empty()
            })
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

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
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
            self.state = CrossbowState::Uncharged;
            self.see_time = 0;
            self.attack_delay = 0;
            self.update_path_delay = 0;
            mob.get_mob_entity().set_attacking(true);
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.get_mob_entity().set_target(None).await;
            mob.get_mob_entity().set_attacking(false);
            self.see_time = 0;
            self.state = CrossbowState::Uncharged;
            mob.set_charging_crossbow(false);
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

            let has_line_of_sight = Self::has_line_of_sight(mob, target.as_ref()).await;
            let had_line_of_sight = self.see_time > 0;
            if has_line_of_sight != had_line_of_sight {
                self.see_time = 0;
            }
            if has_line_of_sight {
                self.see_time += 1;
            } else {
                self.see_time -= 1;
            }

            let needs_to_move = (distance_squared > self.attack_radius_sqr || self.see_time < 5)
                && self.attack_delay == 0;

            if needs_to_move {
                self.update_path_delay -= 1;
                if self.update_path_delay <= 0 {
                    let speed = if self.state == CrossbowState::Uncharged {
                        self.speed_modifier
                    } else {
                        self.speed_modifier * 0.5
                    };
                    mob.get_mob_entity()
                        .navigator
                        .lock()
                        .unwrap()
                        .set_progress(NavigatorGoal {
                            current_progress: entity.pos.load(),
                            destination: target_pos,
                            speed,
                        });
                    self.update_path_delay = rand::rng().random_range(20..=40);
                }
            } else {
                self.update_path_delay = 0;
                mob.get_mob_entity().navigator.lock().unwrap().stop();
            }

            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap()
                .look_at_entity_with_range(&target, 30.0, 30.0);

            match self.state {
                CrossbowState::Uncharged => {
                    if !needs_to_move {
                        self.state = CrossbowState::Charging;
                        mob.set_charging_crossbow(true);
                    }
                }
                CrossbowState::Charging => {
                    // Pumpkin has no generic mob item-use-ticks counter to drive this the way
                    // vanilla's `getTicksUsingItem()` does; the goal tracks its own elapsed ticks
                    // instead, which is equivalent since nothing else can interrupt the "use".
                    self.attack_delay += 1;
                    if self.attack_delay >= CHARGE_DURATION_TICKS {
                        self.state = CrossbowState::Charged;
                        self.attack_delay = 20 + rand::rng().random_range(0..20);
                        mob.set_charging_crossbow(false);
                    }
                }
                CrossbowState::Charged => {
                    self.attack_delay -= 1;
                    if self.attack_delay <= 0 {
                        self.state = CrossbowState::ReadyToAttack;
                    }
                }
                CrossbowState::ReadyToAttack => {
                    if has_line_of_sight {
                        Self::shoot(mob, target.as_ref()).await;
                        self.state = CrossbowState::Uncharged;
                        self.attack_delay = 0;
                    }
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
