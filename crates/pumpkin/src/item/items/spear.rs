use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::entity::combat;
use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase};
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use crate::world::World;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::{
    AttackRangeImpl, AttributeModifiersImpl, EnchantmentsImpl, EquipmentSlot, KineticWeaponImpl,
    Operation, PiercingWeaponImpl, WeaponImpl,
};
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::SoundCategory;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Enchantment, tag};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::{GameMode, Hand};

pub struct SpearItem;

impl ItemMetadata for SpearItem {
    fn ids() -> Box<[u16]> {
        tag::Item::MINECRAFT_SPEARS.1.into()
    }
}

impl ItemBehaviour for SpearItem {
    fn normal_use(&self, item: &Item, player: &Player) {
        if player
            .living_entity
            .item_in_use
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
        {
            return;
        }

        let inventory = player.inventory();
        let hand = if inventory.held_item().item.id == item.id {
            Hand::Right
        } else {
            Hand::Left
        };
        let stack = inventory.get_stack_in_hand(hand);
        let sound = stack
            .get_data_component::<KineticWeaponImpl>()
            .and_then(|weapon| weapon.sound.clone());
        player
            .living_entity
            .set_active_hand(hand, stack, Self::USE_DURATION);
        if let Some(sound) = sound {
            player.world().play_sound_event_expect(
                player,
                &sound,
                SoundCategory::Players,
                &player.position(),
            );
        }
    }

    fn on_spear_jab(&self, stack: &ItemStack, player: &Player) {
        let Some(piercing) = stack.get_data_component::<PiercingWeaponImpl>() else {
            return;
        };
        let world = player.world();
        let Some(server) = world.server.upgrade() else {
            return;
        };

        let tps = f64::from(server.basic_config.tps);
        let attack_delay = tps / Self::attack_speed(player, stack);
        let elapsed = f64::from(player.last_attacked_ticks.load(Ordering::Acquire));
        if elapsed + 5.0 < attack_delay {
            return;
        }

        let damage = Self::attack_damage(player, stack) as f32;
        let mut hit_something = false;
        for target in Self::targets_in_range(player, &server, stack) {
            hit_something |= Self::stab_attack(
                player,
                &server,
                Hand::Right,
                stack,
                &target,
                damage,
                StabEffects {
                    damage: true,
                    knockback: piercing.deals_knockback,
                    dismount: piercing.dismounts,
                },
            );
        }
        player.last_attacked_ticks.store(0, Ordering::Relaxed);

        let position = player.position();
        if hit_something && let Some(sound) = piercing.hit_sound.as_ref() {
            world.play_sound_event(sound, SoundCategory::Players, &position);
        }
        if let Some(sound) = piercing.sound.as_ref() {
            world.play_sound_event_expect(player, sound, SoundCategory::Players, &position);
        }
        player.swing_hand(Hand::Right, false);
    }

    fn on_use_tick(&self, stack: &ItemStack, player: &Player, remaining_use_ticks: i32) {
        let active_hand = *player
            .living_entity
            .active_hand
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(hand) = active_hand else {
            return;
        };
        let held = player.inventory().get_stack_in_hand(hand);
        if held.item.id != stack.item.id {
            player.living_entity.clear_active_hand();
            return;
        }
        Self::kinetic_attack(&held, player, hand, remaining_use_ticks);
    }

    fn get_use_duration(&self) -> i32 {
        Self::USE_DURATION
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Copy)]
struct StabEffects {
    damage: bool,
    knockback: bool,
    dismount: bool,
}

impl SpearItem {
    const USE_DURATION: i32 = 72_000;

    fn stab_attack(
        player: &Player,
        server: &Server,
        hand: Hand,
        stack: &ItemStack,
        target: &Arc<dyn EntityBase>,
        base_damage: f32,
        effects: StabEffects,
    ) -> bool {
        let target_entity = target.get_entity();
        let mut base_damage = base_damage;
        let mut magic_boost = Self::enchantment_damage(stack, target_entity) as f32;
        if !Self::is_using_hand(player, hand) {
            let tps = f64::from(server.basic_config.tps);
            let charge =
                player.get_attack_cooldown_progress(tps, 0.5, Self::attack_speed(player, stack))
                    as f32;
            magic_boost *= charge;
            base_damage *= charge.mul_add(charge * 0.8, 0.2);
        }

        let total_damage = if effects.damage {
            base_damage + magic_boost
        } else {
            0.0
        };
        let was_hurt = effects.damage
            && target.damage_with_context(
                target.as_ref(),
                total_damage,
                DamageType::SPEAR,
                None,
                Some(player),
                Some(player),
            );

        let config = &server.advanced_config.pvp;
        if effects.knockback && config.knockback && target.get_living_entity().is_some() {
            let attacker = player.get_entity();
            combat::handle_knockback(attacker, target.as_ref(), 0.8);
            let knockback_level = Self::knockback_level(stack);
            if knockback_level > 0 {
                combat::handle_knockback(attacker, target.as_ref(), f64::from(knockback_level));
            }
            target_entity.send_velocity();
        }

        let mut dismounted = false;
        if effects.dismount
            && let Some(vehicle) = target_entity.get_vehicle()
        {
            dismounted = true;
            vehicle
                .get_entity()
                .remove_passenger(target_entity.entity_id);
        }

        if !was_hurt && !effects.knockback && !dismounted {
            return false;
        }

        player
            .living_entity
            .last_attacking_id
            .store(target_entity.entity_id, Ordering::Relaxed);
        player.living_entity.last_attack_time.store(
            player.get_entity().age.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
        if was_hurt {
            Self::apply_post_damage_effects(stack, target_entity);
        }
        if target.get_living_entity().is_some()
            && let Some(weapon) = stack.get_data_component::<WeaponImpl>()
        {
            let slot = if hand == Hand::Right {
                EquipmentSlot::MAIN_HAND
            } else {
                EquipmentSlot::OFF_HAND
            };
            player.damage_item_in_slot(&slot, weapon.item_damage_per_attack as i32);
        }
        player.add_exhaustion(0.1);
        true
    }

    fn kinetic_attack(stack: &ItemStack, player: &Player, hand: Hand, remaining_use_ticks: i32) {
        let Some(weapon) = stack.get_data_component::<KineticWeaponImpl>() else {
            return;
        };
        let ticks_used = Self::USE_DURATION - remaining_use_ticks - weapon.delay_ticks;
        if ticks_used < 0 {
            return;
        }
        let world = player.world();
        let Some(server) = world.server.upgrade() else {
            return;
        };

        let look = Self::look_vector(player);
        let attacker_speed = look.dot(&Self::known_speed(player.get_entity()));
        let base_damage = player
            .living_entity
            .get_attribute_base(&Attributes::ATTACK_DAMAGE) as f32;
        let now = player.get_entity().age.load(Ordering::Relaxed);
        let mut affected = false;

        for target in Self::targets_in_range(player, &server, stack) {
            let target_entity = target.get_entity();
            if player.living_entity.was_recently_stabbed(
                target_entity.entity_id,
                now,
                weapon.contact_cooldown_ticks,
            ) {
                continue;
            }
            player
                .living_entity
                .remember_stabbed_entity(target_entity.entity_id, now);

            let target_speed = look.dot(&Self::known_speed(target_entity));
            let relative_speed = (attacker_speed - target_speed).max(0.0);
            let effects = StabEffects {
                damage: weapon.damage_conditions.as_ref().is_some_and(|condition| {
                    condition.test(ticks_used, attacker_speed, relative_speed)
                }),
                knockback: weapon
                    .knockback_conditions
                    .as_ref()
                    .is_some_and(|condition| {
                        condition.test(ticks_used, attacker_speed, relative_speed)
                    }),
                dismount: weapon
                    .dismount_conditions
                    .as_ref()
                    .is_some_and(|condition| {
                        condition.test(ticks_used, attacker_speed, relative_speed)
                    }),
            };
            if !effects.damage && !effects.knockback && !effects.dismount {
                continue;
            }

            let damage =
                base_damage + (relative_speed * f64::from(weapon.damage_multiplier)).floor() as f32;
            affected |= Self::stab_attack(player, &server, hand, stack, &target, damage, effects);
        }

        if affected {
            world.send_entity_status(player.get_entity(), EntityStatus::KineticHit, None);
        }
    }

    fn look_vector(player: &Player) -> Vector3<f64> {
        let (yaw, pitch) = player.rotation();
        Vector3::rotation_vector(f64::from(pitch), f64::from(yaw))
    }

    fn known_speed(entity: &Entity) -> Vector3<f64> {
        let mut movement = entity.movement.load();
        if entity.entity_type != &EntityType::PLAYER {
            let mut vehicle = entity.get_vehicle();
            while let Some(current) = vehicle {
                movement = current.get_entity().movement.load();
                vehicle = current.get_entity().get_vehicle();
            }
        }
        movement * 20.0
    }

    fn is_using_hand(player: &Player, hand: Hand) -> bool {
        let living = &player.living_entity;
        living
            .item_in_use
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
            && *living
                .active_hand
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                == Some(hand)
    }

    fn attack_damage(player: &Player, stack: &ItemStack) -> f64 {
        let mut damage = Self::attribute_with_item_modifier(
            player,
            stack,
            &Attributes::ATTACK_DAMAGE,
            "minecraft:base_attack_damage",
        );
        if let Some(strength) = player.living_entity.get_effect(&StatusEffect::STRENGTH) {
            damage += 3.0 * (f64::from(strength.amplifier) + 1.0);
        }
        if let Some(weakness) = player.living_entity.get_effect(&StatusEffect::WEAKNESS) {
            damage -= 4.0 * (f64::from(weakness.amplifier) + 1.0);
        }
        damage.max(0.0)
    }

    fn attack_speed(player: &Player, stack: &ItemStack) -> f64 {
        Self::attribute_with_item_modifier(
            player,
            stack,
            &Attributes::ATTACK_SPEED,
            "minecraft:base_attack_speed",
        )
        .max(f64::EPSILON)
    }

    fn attribute_with_item_modifier(
        player: &Player,
        stack: &ItemStack,
        attribute: &Attributes,
        modifier_id: &str,
    ) -> f64 {
        let living = &player.living_entity;
        let value = living.get_attribute_value(attribute);
        let already_applied = living
            .attributes
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&attribute.id)
            .is_some_and(|instance| {
                instance
                    .modifiers
                    .iter()
                    .any(|modifier| modifier.id == modifier_id)
            });
        if already_applied {
            return value;
        }
        let Some(modifiers) = stack.get_data_component::<AttributeModifiersImpl>() else {
            return value;
        };
        modifiers
            .attribute_modifiers
            .iter()
            .filter(|modifier| {
                modifier.id == modifier_id && modifier.operation == Operation::AddValue
            })
            .fold(value, |value, modifier| value + modifier.amount)
    }

    fn enchantment_damage(stack: &ItemStack, target: &Entity) -> f64 {
        let Some(enchantments) = stack.get_data_component::<EnchantmentsImpl>() else {
            return 0.0;
        };
        enchantments
            .enchantment
            .iter()
            .map(|(enchantment, level)| {
                let level = f64::from(*level);
                let smite = **enchantment == Enchantment::SMITE
                    && target
                        .entity_type
                        .has_tag(&tag::EntityType::MINECRAFT_SENSITIVE_TO_SMITE);
                let bane = **enchantment == Enchantment::BANE_OF_ARTHROPODS
                    && target
                        .entity_type
                        .has_tag(&tag::EntityType::MINECRAFT_SENSITIVE_TO_BANE_OF_ARTHROPODS);
                if **enchantment == Enchantment::SHARPNESS {
                    0.5 * level + 0.5
                } else if smite || bane {
                    2.5 * level
                } else {
                    0.0
                }
            })
            .sum()
    }

    fn knockback_level(stack: &ItemStack) -> u32 {
        stack
            .get_data_component::<EnchantmentsImpl>()
            .and_then(|enchantments| {
                enchantments
                    .enchantment
                    .iter()
                    .find(|(enchantment, _)| **enchantment == Enchantment::KNOCKBACK)
                    .map(|(_, level)| u32::try_from(*level).unwrap_or(0))
            })
            .unwrap_or(0)
    }

    fn apply_post_damage_effects(stack: &ItemStack, target: &Entity) {
        let Some(enchantments) = stack.get_data_component::<EnchantmentsImpl>() else {
            return;
        };
        for (enchantment, level) in enchantments.enchantment.iter() {
            if **enchantment == Enchantment::FIRE_ASPECT {
                target.set_on_fire_for_ticks(u32::try_from(*level).unwrap_or(0) * 80);
            }
        }
    }

    fn targets_in_range(
        player: &Player,
        server: &Server,
        stack: &ItemStack,
    ) -> Vec<Arc<dyn EntityBase>> {
        let Some(range) = stack.get_data_component::<AttackRangeImpl>() else {
            return Vec::new();
        };
        let world = player.world();
        let look = Self::look_vector(player);
        let eye = player.eye_position();
        let (min_reach, max_reach) = if player.gamemode.load() == GameMode::Creative {
            (
                f64::from(range.min_creative_reach),
                f64::from(range.max_creative_reach),
            )
        } else {
            (f64::from(range.min_reach), f64::from(range.max_reach))
        };
        let from = eye.add(&(look * min_reach));
        let forward = player.get_entity().movement.load().dot(&look).max(0.0);
        let mut to = eye.add(&(look * (max_reach + forward)));
        if let Some(block_hit) = Self::clip_blocks(&world, eye, to) {
            if eye.squared_distance_to_vec(&block_hit) < eye.squared_distance_to_vec(&from) {
                return Vec::new();
            }
            to = block_hit;
        }

        let margin = f64::from(range.hitbox_margin);
        let half_margin = Vector3::new(margin / 2.0, margin / 2.0, margin / 2.0);
        let delta = to.sub(&from);
        let search_box = BoundingBox::new(from.sub(&half_margin), from.add(&half_margin))
            .expand_towards(delta.x, delta.y, delta.z)
            .expand_all(1.0);

        world
            .get_all_at_box(&search_box)
            .into_iter()
            .filter(|target| {
                Self::can_hit(player, server, target.as_ref())
                    && Self::hits_entity(
                        &world,
                        &target.get_entity().bounding_box.load(),
                        from,
                        to,
                        margin,
                    )
            })
            .collect()
    }

    fn hits_entity(
        world: &World,
        bounding_box: &BoundingBox,
        from: Vector3<f64>,
        to: Vector3<f64>,
        margin: f64,
    ) -> bool {
        if contains(bounding_box, from) || clip(bounding_box, from, to).is_some() {
            return true;
        }
        let Some(outside_hit) = clip(&bounding_box.expand_all(margin), from, to) else {
            return false;
        };
        let mut towards_target = bounding_box.min.add(&bounding_box.max) * 0.5;
        if let Some(block_hit) = Self::clip_blocks(world, outside_hit, towards_target) {
            towards_target = block_hit;
        }
        clip(bounding_box, outside_hit, towards_target).is_some()
    }

    fn clip_blocks(world: &World, from: Vector3<f64>, to: Vector3<f64>) -> Option<Vector3<f64>> {
        if from == to {
            return None;
        }
        let start = from.lerp(&to, -1.0e-7);
        let end = to.lerp(&from, -1.0e-7);
        let mut block = BlockPos::floored(start.x, start.y, start.z);
        if let Some(hit) = Self::clip_block(world, &block, from, to) {
            return Some(hit);
        }

        let difference = end.sub(&start);
        let step = difference.sign();
        let delta = Vector3::new(
            if step.x == 0 {
                f64::MAX
            } else {
                f64::from(step.x) / difference.x
            },
            if step.y == 0 {
                f64::MAX
            } else {
                f64::from(step.y) / difference.y
            },
            if step.z == 0 {
                f64::MAX
            } else {
                f64::from(step.z) / difference.z
            },
        );
        let mut next = Vector3::new(
            delta.x
                * (if step.x > 0 {
                    1.0 - (start.x - start.x.floor())
                } else {
                    start.x - start.x.floor()
                }),
            delta.y
                * (if step.y > 0 {
                    1.0 - (start.y - start.y.floor())
                } else {
                    start.y - start.y.floor()
                }),
            delta.z
                * (if step.z > 0 {
                    1.0 - (start.z - start.z.floor())
                } else {
                    start.z - start.z.floor()
                }),
        );

        while next.x <= 1.0 || next.y <= 1.0 || next.z <= 1.0 {
            if next.x < next.y && next.x < next.z {
                block.0.x += step.x;
                next.x += delta.x;
            } else if next.y < next.z {
                block.0.y += step.y;
                next.y += delta.y;
            } else {
                block.0.z += step.z;
                next.z += delta.z;
            }
            if let Some(hit) = Self::clip_block(world, &block, from, to) {
                return Some(hit);
            }
        }
        None
    }

    fn clip_block(
        world: &World,
        pos: &BlockPos,
        from: Vector3<f64>,
        to: Vector3<f64>,
    ) -> Option<Vector3<f64>> {
        world
            .get_block_state(pos)
            .get_block_collision_shapes_at(pos)
            .filter_map(|shape| clip(&shape.shift(pos.0.to_f64()), from, to))
            .min_by(|a, b| {
                from.squared_distance_to_vec(a)
                    .total_cmp(&from.squared_distance_to_vec(b))
            })
    }

    fn can_hit(player: &Player, server: &Server, target: &dyn EntityBase) -> bool {
        let entity = target.get_entity();
        if entity.entity_id == player.entity_id()
            || entity.is_removed()
            || entity.invulnerable.load(Ordering::Relaxed)
            || target.is_spectator()
        {
            return false;
        }
        let Some(living) = target.get_living_entity() else {
            return false;
        };
        if living.health.load() <= 0.0 {
            return false;
        }
        if let Some(other) = target.get_player() {
            let config = &server.advanced_config.pvp;
            if !config.enabled
                || (config.protect_creative && other.gamemode.load() == GameMode::Creative)
            {
                return false;
            }
        }
        Self::root_vehicle_id(player.get_entity()) != Self::root_vehicle_id(entity)
    }

    fn root_vehicle_id(entity: &Entity) -> i32 {
        let mut id = entity.entity_id;
        let mut vehicle = entity.get_vehicle();
        while let Some(current) = vehicle {
            id = current.get_entity().entity_id;
            vehicle = current.get_entity().get_vehicle();
        }
        id
    }
}

fn contains(bounding_box: &BoundingBox, point: Vector3<f64>) -> bool {
    point.x >= bounding_box.min.x
        && point.x < bounding_box.max.x
        && point.y >= bounding_box.min.y
        && point.y < bounding_box.max.y
        && point.z >= bounding_box.min.z
        && point.z < bounding_box.max.z
}

fn clip(bounding_box: &BoundingBox, from: Vector3<f64>, to: Vector3<f64>) -> Option<Vector3<f64>> {
    let min = [bounding_box.min.x, bounding_box.min.y, bounding_box.min.z];
    let max = [bounding_box.max.x, bounding_box.max.y, bounding_box.max.z];
    let origin = [from.x, from.y, from.z];
    let delta = [to.x - from.x, to.y - from.y, to.z - from.z];
    let mut scale = 1.0;
    let mut hit = false;
    for axis in 0..3 {
        if delta[axis] > 1.0e-7 {
            hit |= clip_point(&mut scale, axis, min[axis], &origin, &delta, &min, &max);
        } else if delta[axis] < -1.0e-7 {
            hit |= clip_point(&mut scale, axis, max[axis], &origin, &delta, &min, &max);
        }
    }
    hit.then(|| from.add(&(to.sub(&from) * scale)))
}

fn clip_point(
    scale: &mut f64,
    axis: usize,
    plane: f64,
    origin: &[f64; 3],
    delta: &[f64; 3],
    min: &[f64; 3],
    max: &[f64; 3],
) -> bool {
    let s = (plane - origin[axis]) / delta[axis];
    if s <= 0.0 || s >= *scale {
        return false;
    }
    let b = (axis + 1) % 3;
    let c = (axis + 2) % 3;
    let pb = origin[b] + s * delta[b];
    let pc = origin[c] + s * delta[c];
    if min[b] - 1.0e-7 < pb && pb < max[b] + 1.0e-7 && min[c] - 1.0e-7 < pc && pc < max[c] + 1.0e-7
    {
        *scale = s;
        return true;
    }
    false
}
