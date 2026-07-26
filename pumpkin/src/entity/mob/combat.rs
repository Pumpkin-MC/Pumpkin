use super::MobEntity;
use crate::entity::EntityBase;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use std::sync::atomic::Ordering::Relaxed;

impl MobEntity {
    pub async fn is_in_attack_range(&self, target: &dyn EntityBase) -> bool {
        const DEFAULT_ATTACK_RANGE: f64 = 0.828_427_12; // sqrt(2.04) - 0.6

        // TODO: Implement DataComponent lookup for ATTACK_RANGE when components are ready
        let max_range = DEFAULT_ATTACK_RANGE;
        let min_range = 0.0;

        let target_hitbox = target.get_entity().bounding_box.load();

        if !self
            .get_attack_box(max_range)
            .await
            .intersects(&target_hitbox)
        {
            return false;
        }

        min_range <= 0.0
            || !self
                .get_attack_box(min_range)
                .await
                .intersects(&target_hitbox)
    }

    /// Melee attack entry — mirrors vanilla `Mob.doHurtTarget` /
    /// `IronGolem.doHurtTarget` (Paper/Leaves leave this NMS path unpatched aside
    /// from Bukkit target-reason hooks).
    pub async fn try_attack(&self, caller: &dyn EntityBase, target: &dyn EntityBase) {
        if self.living_entity.dead.load(Relaxed) {
            return;
        }

        let entity_type = self.living_entity.entity.entity_type;
        let is_golem = entity_type.id == pumpkin_data::entity::EntityType::IRON_GOLEM.id;
        let world = self.living_entity.entity.world.load();

        // --- Vanilla IronGolem.doHurtTarget (Mojmap / Paper sources) ---
        // attackAnimationTick = 10;
        // level.broadcastEntityEvent(this, (byte)4);  // START_ATTACKING / both arms
        // float f = attackDamage attribute;
        // float g = (int)f > 0 ? f/2 + random.nextInt((int)f) : f;
        // boolean bl = target.hurt(mobAttack(this), g);
        // if (bl) {
        //   double e = max(0, 1 - target.knockbackResistance);
        //   target.setDeltaMovement(target.getDeltaMovement().add(0, 0.4F * e, 0));
        //   doEnchantDamageEffects(...);
        // }
        // playSound(IRON_GOLEM_ATTACK);  // always
        // return bl;
        //
        // Paper/Leaves: no change to this knockback formula (only collision
        // target-reason + ironGolemsCanSpawnInAir spawn option).
        if is_golem {
            world.send_entity_status(
                &self.living_entity.entity,
                pumpkin_data::entity::EntityStatus::StartAttacking,
            );
        }

        let base = self
            .living_entity
            .get_attribute_value(&Attributes::ATTACK_DAMAGE) as f32;
        // Vanilla golem: ~7.5–21.5 when base attack is 15.
        let attack_damage = if is_golem {
            let whole = base as i32;
            if whole > 0 {
                base / 2.0 + rand::rng().random_range(0..whole) as f32
            } else {
                base
            }
        } else {
            base
        };

        let damaged = target
            .damage_with_context(
                target,
                attack_damage,
                DamageType::MOB_ATTACK,
                None,
                Some(caller),
                Some(caller),
            )
            .await;

        if damaged {
            if is_golem {
                // Pure vertical fling only (IronGolem overrides Mob.doHurtTarget —
                // no horizontal ATTACK_KNOCKBACK path). LivingEntity.damage skips
                // generic horizontal KB when attacker is an iron golem.
                // res 0 (player) → +0.4 y; res 1 (warden) → +0.
                if let Some(target_living) = target.get_living_entity() {
                    let kb_res = target_living
                        .get_attribute_value(&Attributes::KNOCKBACK_RESISTANCE)
                        .clamp(0.0, 1.0);
                    let lift = 0.4 * (1.0 - kb_res);
                    if lift > 0.0 {
                        let ent = target.get_entity();
                        let mut vel = ent.velocity.load();
                        vel.y += lift;
                        ent.velocity.store(vel);
                        // ClientboundSetEntityMotionPacket
                        ent.send_velocity();
                    }
                }
            }

            // Vanilla on-hit status effects (cave spider poison, husk hunger, bee sting).
            if let Some(target_living) = target.get_living_entity() {
                use pumpkin_data::effect::StatusEffect;
                use pumpkin_data::potion::Effect;
                let id = entity_type.id;
                if id == pumpkin_data::entity::EntityType::CAVE_SPIDER.id {
                    // Easy: 7s poison; hard: 15s (difficulty-scaled TODO).
                    target_living
                        .add_effect(Effect {
                            effect_type: &StatusEffect::POISON,
                            duration: 7 * 20,
                            amplifier: 0,
                            ambient: false,
                            show_particles: true,
                            show_icon: true,
                            blend: false,
                        })
                        .await;
                } else if id == pumpkin_data::entity::EntityType::PUFFERFISH.id {
                    // Inflate sting stand-in (vanilla contact poison).
                    target_living
                        .add_effect(Effect {
                            effect_type: &StatusEffect::POISON,
                            duration: 5 * 20,
                            amplifier: 1,
                            ambient: false,
                            show_particles: true,
                            show_icon: true,
                            blend: false,
                        })
                        .await;
                } else if id == pumpkin_data::entity::EntityType::HUSK.id {
                    target_living
                        .add_effect(Effect {
                            effect_type: &StatusEffect::HUNGER,
                            duration: 7 * 20,
                            amplifier: 0,
                            ambient: false,
                            show_particles: true,
                            show_icon: true,
                            blend: false,
                        })
                        .await;
                } else if id == pumpkin_data::entity::EntityType::BEE.id {
                    // Bee sting poison (short).
                    target_living
                        .add_effect(Effect {
                            effect_type: &StatusEffect::POISON,
                            duration: 18 * 20 / 2, // ~9s stand-in
                            amplifier: 0,
                            ambient: false,
                            show_particles: true,
                            show_icon: true,
                            blend: false,
                        })
                        .await;
                } else if id == pumpkin_data::entity::EntityType::WITHER_SKELETON.id {
                    target_living
                        .add_effect(Effect {
                            effect_type: &StatusEffect::WITHER,
                            duration: 10 * 20,
                            amplifier: 0,
                            ambient: false,
                            show_particles: true,
                            show_icon: true,
                            blend: false,
                        })
                        .await;
                }
            }

            self.living_entity
                .last_attacking_id
                .store(target.get_entity().entity_id, Relaxed);
            self.living_entity
                .last_attack_time
                .store(self.living_entity.entity.age.load(Relaxed), Relaxed);
        }

        // Vanilla always plays golem swing sound (even on a missed/blocked hit).
        if is_golem {
            world.play_sound(
                pumpkin_data::sound::Sound::EntityIronGolemAttack,
                pumpkin_data::sound::SoundCategory::Neutral,
                &self.living_entity.entity.pos.load(),
            );
        }
    }

    async fn get_attack_box(&self, attack_range: f64) -> BoundingBox {
        let vehicle_lock = self.living_entity.entity.vehicle.lock().await;

        let base_box = vehicle_lock.as_ref().map_or_else(
            || self.living_entity.entity.bounding_box.load(),
            |vehicle| {
                let vehicle_box = vehicle.get_entity().bounding_box.load();
                let my_box = self.living_entity.entity.bounding_box.load();

                BoundingBox {
                    min: Vector3::new(
                        my_box.min.x.min(vehicle_box.min.x),
                        my_box.min.y,
                        my_box.min.z.min(vehicle_box.min.z),
                    ),
                    max: Vector3::new(
                        my_box.max.x.max(vehicle_box.max.x),
                        my_box.max.y,
                        my_box.max.z.max(vehicle_box.max.z),
                    ),
                }
            },
        );

        base_box.expand(attack_range, 0.0, attack_range)
    }
}
