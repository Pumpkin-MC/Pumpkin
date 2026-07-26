//! Ranged bow / crossbow attack (vanilla `RangedBowAttackGoal` +
//! `RangedCrossbowAttackGoal` stand-in).
//!
//! Ground truth from CFR of server-26.2.jar — see `goal/vanilla-26.2-decompile.md`.
//!
//! - Bow (skeleton): startUsingItem → ~20 ticks → shoot.
//! - Crossbow (pillager): startUsingItem + charge 25 ticks (1.25s) → delay 20–40
//!   → shoot; move at full speed when uncharged, half while charging if still pathing.

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
use pumpkin_util::math::subtract_angles;
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

pub struct BowAttackGoal {
    /// Cooldown after a shot before drawing again (goal ticks).
    attack_time: i32,
    see_time: i32,
    strafing_time: i32,
    strafing_clockwise: bool,
    strafing_backwards: bool,
    /// Ticks spent drawing the current shot (-1 = not drawing).
    draw_time: i32,
    /// Crossbow only: ticks left in the vanilla CHARGED state before firing
    /// (-1 = not charged).
    charged_delay: i32,
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
            charged_delay: -1,
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
        let horiz = dx.hypot(dz);
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
        // Vanilla 26.2 (CFR RangedCrossbowAttackGoal + RangedBowAttackGoal):
        // both call LivingEntity.startUsingItem. Crossbow use animation is
        // ItemUseAnimation.CROSSBOW (client shows crossbow charge, not bow pull).
        // Charge length: CrossbowItem.getChargeDuration ≈ floor(1.25 * 20) = 25 ticks.
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

        let arrow_entity = Entity::new(world.clone(), eye, &EntityType::ARROW);
        let arrow = ArrowEntity::new_shot(arrow_entity, shooter, ArrowPickup::Disallowed);

        // Vanilla AbstractSkeleton.performRangedAttack (AbstractSkeleton.java:178-190):
        // aim at 1/3 target height, lift by horizontal distance * 0.2 BLOCKS,
        // divergence 14 - 4 * difficulty id.
        let target_entity = target.get_entity();
        let target_pos = target_entity.pos.load();
        let target_height = f64::from(target_entity.entity_dimension.load().height);
        let arrow_pos = arrow.entity.pos.load();
        let xd = target_pos.x - shooter.pos.load().x;
        let yd = target_pos.y + target_height / 3.0 - arrow_pos.y;
        let zd = target_pos.z - shooter.pos.load().z;
        let horizontal = xd.hypot(zd);
        let divergence = f64::from(
            14 - 4 * match world.level_info.load().difficulty {
                pumpkin_util::Difficulty::Peaceful => 0,
                pumpkin_util::Difficulty::Easy => 1,
                pumpkin_util::Difficulty::Normal => 2,
                pumpkin_util::Difficulty::Hard => 3,
            },
        );
        arrow.set_velocity(xd, horizontal.mul_add(0.2, yd), zd, 1.6, divergence);

        // Vanilla releases the use state before spawning the projectile.
        living.clear_active_hand().await;

        let arrow_arc: Arc<dyn EntityBase> = Arc::new(arrow);
        world.spawn_entity(arrow_arc).await;

        let pos = shooter.pos.load();
        // Vanilla: 1.0 / (random * 0.4 + 0.8) → 0.83..1.25.
        let pitch_sound = 1.0 / rand::rng().random_range(0.0f32..1.0).mul_add(0.4, 0.8);
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
    }

    fn shooter_alive(mob: &dyn Mob) -> bool {
        mob.get_mob_entity().living_entity.is_alive()
    }

    /// Vanilla `Mob.lookAt(target, 30, 30)` — rotates the body (yaw/pitch)
    /// toward the target, clamped per tick; used while strafing.
    fn body_look_at(mob: &dyn Mob, target: &dyn EntityBase, max_yaw: f32, max_pitch: f32) {
        let entity = mob.get_entity();
        let (desired_yaw, desired_pitch) =
            Self::look_angles(entity.get_eye_pos(), target.get_entity().get_eye_pos());
        let yaw = entity.yaw.load();
        let new_yaw = yaw + subtract_angles(yaw, desired_yaw).clamp(-max_yaw, max_yaw);
        entity.yaw.store(new_yaw);
        entity.body_yaw.store(new_yaw);
        entity.head_yaw.store(new_yaw);
        let pitch = entity.pitch.load();
        entity
            .pitch
            .store(pitch + subtract_angles(pitch, desired_pitch).clamp(-max_pitch, max_pitch));
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

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            // RangedBowAttackGoal marks bow users aggressive; the crossbow goal
            // has a separate charging state and does not set it on start.
            if !Self::main_hand_is_crossbow(mob).await {
                mob.get_mob_entity().set_attacking(true);
            }
            self.see_time = 0;
            self.strafing_time = -1;
            self.attack_time = 0;
            self.draw_time = -1;
            self.charged_delay = -1;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            mob.get_mob_entity().set_attacking(false);
            self.see_time = 0;
            self.attack_time = -1;
            self.draw_time = -1;
            self.charged_delay = -1;
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
            // Vanilla RangedBowAttackGoal.tick: seeTime resets when LOS flips.
            if can_see != (self.see_time > 0) {
                self.see_time = 0;
            }
            self.see_time = if can_see {
                self.see_time + 1
            } else {
                self.see_time - 1
            };

            let is_crossbow = Self::main_hand_is_crossbow(mob).await;

            if is_crossbow {
                // From CFR RangedCrossbowAttackGoal (26.2): attackRadius=8 for
                // pillager; canRun() only when UNCHARGED → half speed while
                // pathing during charge.
                {
                    let eye = target.get_entity().get_eye_pos();
                    let mut look = mob.get_mob_entity().look_control.lock().unwrap();
                    look.look_at_with_range(eye.x, eye.y, eye.z, 30.0, 30.0);
                }
                let crossbow_radius_sq = 64.0f64;
                let charging = self.draw_time >= 0;
                {
                    // needsToMove = dist > radius² || seeTime < 5
                    let needs_to_move = dist_sq > crossbow_radius_sq || self.see_time < 5;
                    let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                    if needs_to_move {
                        let speed = if charging {
                            self.speed * 0.5
                        } else {
                            self.speed
                        };
                        navigator.set_progress(NavigatorGoal::new(mob_pos, target_pos, speed));
                    } else {
                        navigator.stop();
                    }
                }

                // Crossbow: startUsingItem → getChargeDuration(25) → CHARGED
                // for attackDelay = 20 + random(20) ticks → shoot with LOS
                // (RangedCrossbowAttackGoal CHARGING→CHARGED→READY_TO_ATTACK).
                self.attack_time -= 1;
                let in_range = dist_sq <= crossbow_radius_sq && self.see_time >= 5;
                if self.charged_delay >= 0 {
                    self.charged_delay -= 1;
                    if self.charged_delay < 0 && can_see {
                        Self::shoot_arrow(mob, target.as_ref()).await;
                        self.attack_time = 0;
                    }
                } else if in_range {
                    if self.draw_time < 0 && self.attack_time <= 0 && self.see_time >= 5 {
                        Self::begin_draw(mob).await;
                        self.draw_time = 0;
                    }

                    if self.draw_time >= 0 {
                        self.draw_time += 1;
                        if self.draw_time >= CROSSBOW_CHARGE_TICKS {
                            self.draw_time = -1;
                            // Vanilla sets the attack delay when the charge
                            // completes, then fires after it elapses.
                            self.charged_delay = 20 + mob.get_random().random_range(0..20);
                        }
                    }
                } else if self.draw_time >= 0 && self.see_time < -10 {
                    self.draw_time = -1;
                    mob.get_mob_entity().living_entity.clear_active_hand().await;
                }
            } else {
                // Vanilla RangedBowAttackGoal.tick (RangedBowAttackGoal.java:92-137).
                if dist_sq > ATTACK_RADIUS_SQ || self.see_time < 20 {
                    let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                    navigator.set_progress(NavigatorGoal::new(mob_pos, target_pos, self.speed));
                    self.strafing_time = -1;
                } else {
                    mob.get_mob_entity().navigator.lock().unwrap().stop();
                    self.strafing_time += 1;
                }

                if self.strafing_time >= 20 {
                    if mob.get_random().random_range(0.0f32..1.0) < 0.3 {
                        self.strafing_clockwise = !self.strafing_clockwise;
                    }
                    if mob.get_random().random_range(0.0f32..1.0) < 0.3 {
                        self.strafing_backwards = !self.strafing_backwards;
                    }
                    self.strafing_time = 0;
                }

                if self.strafing_time > -1 {
                    if dist_sq > ATTACK_RADIUS_SQ * 0.75 {
                        self.strafing_backwards = false;
                    } else if dist_sq < ATTACK_RADIUS_SQ * 0.25 {
                        self.strafing_backwards = true;
                    }
                    // Vanilla MoveControl.strafe: yaw-relative input, no waypoint.
                    // Routing this through the navigator turned the body toward a
                    // rotating side-point every tick — the 360° spin players saw.
                    mob.get_mob_entity().move_control.lock().unwrap().strafe(
                        if self.strafing_backwards { -0.5 } else { 0.5 },
                        if self.strafing_clockwise { 0.5 } else { -0.5 },
                    );
                    Self::body_look_at(mob, target.as_ref(), 30.0, 30.0);
                } else {
                    let eye = target.get_entity().get_eye_pos();
                    let mut look = mob.get_mob_entity().look_control.lock().unwrap();
                    look.look_at_with_range(eye.x, eye.y, eye.z, 30.0, 30.0);
                }

                // Bow item use: startUsingItem → ≥20 ticks → release. The draw
                // survives brief LOS loss; only seeTime < -60 cancels (vanilla).
                if self.draw_time >= 0 {
                    self.draw_time += 1;
                    if !can_see && self.see_time < -60 {
                        self.draw_time = -1;
                        mob.get_mob_entity().living_entity.clear_active_hand().await;
                    } else if can_see && self.draw_time >= BOW_DRAW_TICKS {
                        Self::shoot_arrow(mob, target.as_ref()).await;
                        self.draw_time = -1;
                        self.attack_time = self.attack_interval;
                    }
                } else {
                    self.attack_time -= 1;
                    if self.attack_time <= 0 && self.see_time >= -60 {
                        Self::begin_draw(mob).await;
                        self.draw_time = 0;
                    }
                }
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
