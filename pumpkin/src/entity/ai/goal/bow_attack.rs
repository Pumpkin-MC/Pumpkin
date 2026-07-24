//! Ranged bow attack goal (vanilla `RangedBowAttackGoal`).
//!
//! Skeleton draws the bow (using-item pose) for ~20 ticks, then releases an arrow.

use super::{Controls, Goal, GoalFuture};
use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::projectile::arrow::{ArrowEntity, ArrowPickup};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_protocol::IdOr;
use pumpkin_protocol::java::client::play::CSoundEffect;
use pumpkin_util::Hand;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use std::sync::Arc;

const ATTACK_RADIUS: f64 = 15.0;
const ATTACK_RADIUS_SQ: f64 = ATTACK_RADIUS * ATTACK_RADIUS;
const MIN_ATTACK_INTERVAL: i32 = 20;
/// Vanilla full bow draw ticks before release.
const BOW_DRAW_TICKS: i32 = 20;
/// Vanilla crossbow charge (~1.25s). Pillagers use RangedCrossbowAttackGoal —
/// they do **not** use USING_ITEM bow-draw (which slows and raises the bow pose).
const CROSSBOW_CHARGE_TICKS: i32 = 25;
const STRAFE_CHANCE: i32 = 20;

pub struct BowAttackGoal {
    /// Cooldown after a shot before drawing again (goal ticks).
    attack_time: i32,
    see_time: i32,
    strafing_time: i32,
    strafing_clockwise: bool,
    strafing_backwards: bool,
    /// Ticks spent drawing the current shot (-1 = not drawing).
    draw_time: i32,
    speed: f64,
    attack_interval: i32,
}

impl BowAttackGoal {
    #[must_use]
    pub fn new(speed: f64, attack_interval_ticks: i32) -> Box<Self> {
        Box::new(Self {
            attack_time: 0,
            see_time: 0,
            strafing_time: -1,
            strafing_clockwise: false,
            strafing_backwards: false,
            draw_time: -1,
            speed: speed.max(0.25),
            attack_interval: attack_interval_ticks.max(MIN_ATTACK_INTERVAL),
        })
    }

    fn target_alive(target: &dyn EntityBase) -> bool {
        if let Some(living) = target.get_living_entity() {
            living.is_alive()
        } else {
            target.get_entity().is_alive()
        }
    }

    async fn has_line_of_sight(mob: &dyn Mob, target: &dyn EntityBase) -> bool {
        let from = mob.get_entity().get_eye_pos();
        let to = target.get_entity().get_eye_pos();
        let world = mob.get_entity().world.load();
        world
            .raycast(from, to, async |block_pos, w| {
                let state = w.get_block_state(block_pos);
                state.is_solid()
            })
            .await
            .is_none()
    }

    fn look_angles(from: Vector3<f64>, to: Vector3<f64>) -> (f32, f32) {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let dz = to.z - from.z;
        let horiz = (dx * dx + dz * dz).sqrt();
        let yaw = (dz.atan2(dx).to_degrees() as f32) - 90.0;
        let pitch = -(dy.atan2(horiz).to_degrees() as f32);
        (yaw, pitch)
    }

    async fn ensure_bow_equipped(mob: &dyn Mob) -> ItemStack {
        use pumpkin_data::data_component_impl::EquipmentSlot;
        let living = &mob.get_mob_entity().living_entity;
        let mut eq = living.entity_equipment.lock().await;
        let hand = eq.get(&EquipmentSlot::MAIN_HAND);
        let current = hand.lock().await.clone();
        // Keep an already-equipped ranged weapon. Pillagers use this goal as a
        // stand-in for vanilla RangedCrossbowAttackGoal and must keep their
        // crossbow — do not replace it with a bow.
        if !current.is_empty()
            && (current.item.id == Item::BOW.id || current.item.id == Item::CROSSBOW.id)
        {
            return current;
        }
        // Summon / reload path sometimes skips equip; force a bow so clients render it
        // and the draw pose looks correct (vanilla AbstractSkeleton always has a bow).
        let bow = ItemStack::new(1, &Item::BOW);
        drop(hand);
        eq.put(&EquipmentSlot::MAIN_HAND, bow.clone()).await;
        drop(eq);
        living.send_equipment_changes(&[(EquipmentSlot::MAIN_HAND, bow.clone())]);
        bow
    }

    async fn main_hand_is_crossbow(mob: &dyn Mob) -> bool {
        use pumpkin_data::data_component_impl::EquipmentSlot;
        let living = &mob.get_mob_entity().living_entity;
        let eq = living.entity_equipment.lock().await;
        let hand = eq.get(&EquipmentSlot::MAIN_HAND);
        let current = hand.lock().await;
        !current.is_empty() && current.item.id == Item::CROSSBOW.id
    }

    async fn begin_draw(mob: &dyn Mob) {
        let living = &mob.get_mob_entity().living_entity;
        let stack = Self::ensure_bow_equipped(mob).await;
        // Crossbow (pillager): vanilla charges via entity data flag, NOT startUsingItem.
        // USING_ITEM with a crossbow makes clients pull the string every shot and apply
        // use-item movement slow — wrong for pillagers.
        if stack.item.id == Item::CROSSBOW.id {
            return;
        }
        // Bow (skeleton): USING_ITEM living flag → client raises the bow.
        living.set_active_hand(Hand::Right, stack, 72000).await;
    }

    async fn shoot_arrow(mob: &dyn Mob, target: &dyn EntityBase) {
        let living = &mob.get_mob_entity().living_entity;
        // Never fire after death (corpse "ghost arrow" bug).
        if !living.is_alive() {
            living.clear_active_hand().await;
            return;
        }
        if !Self::target_alive(target) {
            living.clear_active_hand().await;
            return;
        }

        let is_crossbow = Self::main_hand_is_crossbow(mob).await;
        let shooter = mob.get_entity();
        let world = shooter.world.load();
        let eye = shooter.get_eye_pos();
        let target_eye = target.get_entity().get_eye_pos();

        let arrow_entity = Entity::new(world.clone(), eye, &EntityType::ARROW);
        let arrow = ArrowEntity::new_shot(arrow_entity, shooter, ArrowPickup::Disallowed);

        let (yaw, pitch) = Self::look_angles(eye, target_eye);
        let dist = eye.squared_distance_to_vec(&target_eye).sqrt();
        let pitch_adjust = (dist * 0.2).min(1.0) as f32;
        // Vanilla AbstractSkeleton / CrossbowAttackMob: speed ~1.6, divergence ~12.
        arrow.set_velocity_from_rotation(pitch - pitch_adjust, yaw, 0.0, 1.6, 12.0);

        let arrow_arc: Arc<dyn EntityBase> = Arc::new(arrow);
        world.spawn_entity(arrow_arc).await;

        let pos = shooter.pos.load();
        let pitch_sound = 1.0 / (rand::rng().random_range(0.8f32..1.2)) + 0.5;
        let sound = if is_crossbow {
            Sound::ItemCrossbowShoot
        } else {
            Sound::EntitySkeletonShoot
        };
        let sound_packet = CSoundEffect::new(
            IdOr::Id(sound as u16),
            SoundCategory::Hostile,
            &pos,
            1.0,
            pitch_sound,
            0,
        );
        world.broadcast_to_chunk(shooter.chunk_pos.load(), &sound_packet);

        living.clear_active_hand().await;
    }

    fn shooter_alive(mob: &dyn Mob) -> bool {
        mob.get_mob_entity().living_entity.is_alive()
    }
}

impl Goal for BowAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            if !Self::shooter_alive(mob) {
                return false;
            }
            let target = mob.get_mob_entity().target.lock().await;
            let Some(target) = target.as_ref() else {
                return false;
            };
            Self::target_alive(target.as_ref())
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            if !Self::shooter_alive(mob) {
                return false;
            }
            let target = mob.get_mob_entity().target.lock().await;
            let Some(target) = target.as_ref() else {
                return false;
            };
            if !Self::target_alive(target.as_ref()) {
                return false;
            }
            !target
                .get_player()
                .is_some_and(|p| p.is_spectator() || p.is_creative())
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.see_time = 0;
            self.strafing_time = -1;
            self.attack_time = 0;
            self.draw_time = -1;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.see_time = 0;
            self.attack_time = -1;
            self.draw_time = -1;
            mob.get_mob_entity().living_entity.clear_active_hand().await;
            mob.get_mob_entity().navigator.lock().unwrap().stop();
            // Clear dead targets so ActiveTarget can pick the next enemy.
            let clear = {
                let t = mob.get_mob_entity().target.lock().await;
                t.as_ref().is_none_or(|e| !Self::target_alive(e.as_ref()))
            };
            if clear {
                mob.set_mob_target(None).await;
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            if !Self::shooter_alive(mob) {
                self.draw_time = -1;
                mob.get_mob_entity().living_entity.clear_active_hand().await;
                return;
            }
            let target = mob.get_mob_entity().target.lock().await.clone();
            let Some(target) = target else {
                return;
            };
            if !Self::target_alive(target.as_ref()) {
                mob.set_mob_target(None).await;
                mob.get_mob_entity().living_entity.clear_active_hand().await;
                self.draw_time = -1;
                return;
            }

            let mob_pos = mob.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            let dist_sq = mob_pos.squared_distance_to_vec(&target_pos);

            let can_see = Self::has_line_of_sight(mob, target.as_ref()).await;
            if can_see {
                self.see_time += 1;
            } else {
                self.see_time -= 1;
                // Lost LOS while drawing — cancel draw.
                if self.draw_time >= 0 {
                    self.draw_time = -1;
                    mob.get_mob_entity().living_entity.clear_active_hand().await;
                }
            }

            {
                let eye = target.get_entity().get_eye_pos();
                let mut look = mob.get_mob_entity().look_control.lock().unwrap();
                look.look_at_with_range(eye.x, eye.y, eye.z, 30.0, 30.0);
            }

            let is_crossbow = Self::main_hand_is_crossbow(mob).await;

            if dist_sq <= ATTACK_RADIUS_SQ && self.see_time >= 5 {
                // Bow: stop and strafe slowly while drawing (vanilla RangedBowAttackGoal).
                // Crossbow / pillager: keep walking at full speed while charging
                // (vanilla RangedCrossbowAttackGoal does not apply use-item slow).
                if is_crossbow {
                    let dest = target_pos;
                    let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                    // Hold distance ~8 blocks like vanilla crossbow range.
                    if dist_sq > 64.0 {
                        navigator.set_progress(NavigatorGoal::new(mob_pos, dest, self.speed));
                    } else if dist_sq < 25.0 {
                        // Too close — back off slightly without stopping entirely.
                        let dx = mob_pos.x - target_pos.x;
                        let dz = mob_pos.z - target_pos.z;
                        let len = (dx * dx + dz * dz).sqrt().max(0.001);
                        let back = Vector3::new(
                            mob_pos.x + dx / len * 2.0,
                            mob_pos.y,
                            mob_pos.z + dz / len * 2.0,
                        );
                        navigator.set_progress(NavigatorGoal::new(mob_pos, back, self.speed));
                    } else {
                        // In good range: light strafe at full speed.
                        self.strafing_time += 1;
                        if self.strafing_time >= 20 {
                            if mob.get_random().random_range(0..STRAFE_CHANCE) == 0 {
                                self.strafing_clockwise = !self.strafing_clockwise;
                            }
                            self.strafing_time = 0;
                        }
                        let dx = target_pos.x - mob_pos.x;
                        let dz = target_pos.z - mob_pos.z;
                        let len = (dx * dx + dz * dz).sqrt().max(0.001);
                        let (sx, sz) = if self.strafing_clockwise {
                            (-dz / len, dx / len)
                        } else {
                            (dz / len, -dx / len)
                        };
                        let side =
                            Vector3::new(mob_pos.x + sx * 1.5, mob_pos.y, mob_pos.z + sz * 1.5);
                        navigator.set_progress(NavigatorGoal::new(mob_pos, side, self.speed));
                    }
                } else {
                    mob.get_mob_entity().navigator.lock().unwrap().stop();

                    self.strafing_time += 1;
                    if self.strafing_time >= 20 {
                        if mob.get_random().random_range(0..STRAFE_CHANCE) == 0 {
                            self.strafing_clockwise = !self.strafing_clockwise;
                        }
                        if mob.get_random().random_range(0..STRAFE_CHANCE) == 0 {
                            self.strafing_backwards = !self.strafing_backwards;
                        }
                        self.strafing_time = 0;
                    }

                    let too_close = dist_sq < 16.0;
                    let back = self.strafing_backwards || too_close;
                    let dir = {
                        let dx = target_pos.x - mob_pos.x;
                        let dz = target_pos.z - mob_pos.z;
                        let len = (dx * dx + dz * dz).sqrt().max(0.001);
                        let fx = dx / len;
                        let fz = dz / len;
                        let (sx, sz) = if self.strafing_clockwise {
                            (-fz, fx)
                        } else {
                            (fz, -fx)
                        };
                        let scale = if back { -0.8 } else { 0.4 };
                        Vector3::new(
                            mob_pos.x + sx * 0.6 + fx * scale,
                            mob_pos.y,
                            mob_pos.z + sz * 0.6 + fz * scale,
                        )
                    };
                    let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                    navigator.set_progress(NavigatorGoal::new(mob_pos, dir, self.speed * 0.7));
                }
            } else {
                let dest = {
                    let b = target_pos.to_block_pos();
                    Vector3::new(
                        f64::from(b.0.x) + 0.5,
                        f64::from(b.0.y),
                        f64::from(b.0.z) + 0.5,
                    )
                };
                let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                navigator.set_progress(NavigatorGoal::new(mob_pos, dest, self.speed));
                self.strafing_time = -1;
            }

            // Bow: startUsingItem → 20 ticks → shoot.
            // Crossbow: silent charge timer (no USING_ITEM) → shoot.
            self.attack_time -= 1;
            let charge_ticks = if is_crossbow {
                CROSSBOW_CHARGE_TICKS
            } else {
                BOW_DRAW_TICKS
            };

            if can_see && dist_sq <= ATTACK_RADIUS_SQ {
                if self.draw_time < 0 && self.attack_time <= 0 && self.see_time >= 5 {
                    Self::begin_draw(mob).await;
                    self.draw_time = 0;
                }

                if self.draw_time >= 0 {
                    self.draw_time += 1;
                    if self.draw_time >= charge_ticks {
                        Self::shoot_arrow(mob, target.as_ref()).await;
                        self.draw_time = -1;
                        self.attack_time = self.attack_interval;
                    }
                }
            } else if self.draw_time >= 0 && self.see_time < -10 {
                self.draw_time = -1;
                mob.get_mob_entity().living_entity.clear_active_hand().await;
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
