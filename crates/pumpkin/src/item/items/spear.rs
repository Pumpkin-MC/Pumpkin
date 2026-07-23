use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::{AttributeModifiersImpl, Operation};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag;
use pumpkin_util::GameMode;
use pumpkin_util::Hand;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::vector3::Vector3;

pub struct SpearItem;

impl ItemMetadata for SpearItem {
    fn ids() -> Box<[u16]> {
        tag::Item::MINECRAFT_SPEARS.1.into()
    }
}

impl ItemBehaviour for SpearItem {
    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let held = player.inventory().held_item();
            let stack = held.lock().await.clone();

            player
                .living_entity
                .set_active_hand(Hand::Right, stack.clone(), Self::USE_DURATION)
                .await;
            player.world().play_sound(
                Self::use_sound(stack.item),
                SoundCategory::Players,
                &player.position(),
            );
        })
    }

    fn on_spear_jab<'a>(
        &'a self,
        stack: &'a ItemStack,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if player.gamemode.load() == GameMode::Spectator {
                return;
            }

            let (damage, attack_speed) = Self::attack_attributes(player, stack);
            let server = player.world().server.upgrade().unwrap();
            let attack_delay = f64::from(server.basic_config.tps) / attack_speed;
            let elapsed = f64::from(player.last_attacked_ticks.load(Ordering::Acquire));
            if elapsed + 5.0 < attack_delay {
                return;
            }

            let charge = player.get_attack_cooldown_progress(
                f64::from(server.basic_config.tps),
                0.5,
                attack_speed,
            );
            let scaled_damage = damage * charge.mul_add(charge * 0.8, 0.2);
            player.last_attacked_ticks.store(0, Ordering::Relaxed);

            let mut hit_something = false;
            for target in Self::targets_in_range(player).await {
                let target_entity = target.get_entity();
                if target
                    .damage_with_context(
                        target.as_ref(),
                        scaled_damage as f32,
                        DamageType::SPEAR,
                        None,
                        Some(player),
                        Some(player),
                    )
                    .await
                {
                    let attacker_pos = player.position();
                    let target_pos = target_entity.pos.load();
                    target_entity.knockback(
                        0.4,
                        attacker_pos.x - target_pos.x,
                        attacker_pos.z - target_pos.z,
                    );
                    player.damage_held_item(1).await;
                    hit_something = true;
                }
            }

            let world = player.world();
            if hit_something {
                world.play_sound(
                    Self::hit_sound(stack.item),
                    SoundCategory::Players,
                    &player.position(),
                );
            }
            world.play_sound(
                Self::attack_sound(stack.item),
                SoundCategory::Players,
                &player.position(),
            );
            player.swing_hand(Hand::Right, false).await;
        })
    }

    fn on_use_tick<'a>(
        &'a self,
        stack: &'a ItemStack,
        player: &'a Player,
        remaining_use_ticks: i32,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            Self::kinetic_attack(stack, player, remaining_use_ticks).await;
        })
    }

    fn get_use_duration(&self) -> i32 {
        Self::USE_DURATION
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl SpearItem {
    const USE_DURATION: i32 = 72_000;
    const MIN_RANGE: f64 = 2.0;
    const SURVIVAL_RANGE: f64 = 4.5;
    const CREATIVE_RANGE: f64 = 6.5;
    const HITBOX_MARGIN: f64 = 0.125;
    const CONTACT_COOLDOWN: i32 = 10;

    fn attack_attributes(player: &Player, stack: &ItemStack) -> (f64, f64) {
        let mut damage = player
            .living_entity
            .get_attribute_value(&Attributes::ATTACK_DAMAGE);
        let mut attack_speed = 4.0;

        if let Some(modifiers) = stack.get_data_component::<AttributeModifiersImpl>() {
            for modifier in modifiers.attribute_modifiers.iter() {
                if modifier.operation != Operation::AddValue {
                    continue;
                }
                if modifier.id == "minecraft:base_attack_damage" {
                    damage += modifier.amount;
                } else if modifier.id == "minecraft:base_attack_speed" {
                    attack_speed += modifier.amount;
                }
            }
        }

        (damage.max(0.0), attack_speed.max(f64::EPSILON))
    }

    async fn targets_in_range(player: &Player) -> Vec<Arc<dyn EntityBase>> {
        let eye_position = player.eye_position();
        let (yaw, pitch) = player.rotation();
        let direction = Vector3::rotation_vector(f64::from(pitch), f64::from(yaw));
        let max_range = if player.gamemode.load() == GameMode::Creative {
            Self::CREATIVE_RANGE
        } else {
            Self::SURVIVAL_RANGE
        };
        let forward_movement = direction
            .dot(&player.get_entity().movement.load())
            .max(0.0);
        let start = eye_position.add(&(direction * Self::MIN_RANGE));
        let end = eye_position.add(&(direction * (max_range + forward_movement)));
        let ray = end - start;
        let search_box = BoundingBox::new(
            Vector3::new(start.x.min(end.x), start.y.min(end.y), start.z.min(end.z)),
            Vector3::new(start.x.max(end.x), start.y.max(end.y), start.z.max(end.z)),
        )
        .expand_all(1.0 + Self::HITBOX_MARGIN);

        let world = player.world();
        let mut targets = Vec::new();
        for target in world.get_all_at_box(&search_box) {
            let entity = target.get_entity();
            if entity.entity_id == player.entity_id()
                || entity.is_removed()
                || entity.invulnerable.load(Ordering::Relaxed)
                || target.is_spectator()
                || target
                    .get_living_entity()
                    .is_none_or(|living| living.health.load() <= 0.0)
            {
                continue;
            }

            let Some(intersection) = ray_intersection(
                &start,
                &ray,
                &entity.bounding_box.load().expand_all(Self::HITBOX_MARGIN),
            ) else {
                continue;
            };
            let hit_position = start.add(&(ray * intersection));
            if world
                .raycast(eye_position, hit_position, async |pos, ray_world| {
                    let block = ray_world.get_block(pos);
                    block != &pumpkin_data::Block::AIR
                        && block != &pumpkin_data::Block::WATER
                        && block != &pumpkin_data::Block::LAVA
                })
                .await
                .is_none()
            {
                targets.push((intersection, target));
            }
        }

        targets.sort_by(|a, b| a.0.total_cmp(&b.0));
        targets.into_iter().map(|(_, target)| target).collect()
    }

    async fn kinetic_attack(stack: &ItemStack, player: &Player, remaining_use_ticks: i32) {
        let Some(properties) = KineticProperties::from_item(stack.item) else {
            return;
        };
        let ticks_used = Self::USE_DURATION - remaining_use_ticks - properties.delay_ticks;
        if ticks_used < 0 {
            return;
        }

        let (yaw, pitch) = player.rotation();
        let look = Vector3::rotation_vector(f64::from(pitch), f64::from(yaw));
        let attacker_speed = look.dot(&(player.get_entity().movement.load() * 20.0));
        let current_tick = player.get_entity().age.load(Ordering::Relaxed);
        let base_damage = player
            .living_entity
            .get_attribute_base(&Attributes::ATTACK_DAMAGE);
        let mut affected = false;

        for target in Self::targets_in_range(player).await {
            let target_entity = target.get_entity();
            {
                let mut recent = player.living_entity.recent_kinetic_enemies.lock().await;
                recent.retain(|_, hit_tick| current_tick - *hit_tick < Self::CONTACT_COOLDOWN);
                if recent.contains_key(&target_entity.entity_id) {
                    continue;
                }
                recent.insert(target_entity.entity_id, current_tick);
            }

            let target_speed = look.dot(&(target_entity.movement.load() * 20.0));
            let relative_speed = (attacker_speed - target_speed).max(0.0);
            let deals_damage = properties
                .damage
                .test(ticks_used, attacker_speed, relative_speed);
            let deals_knockback =
                properties
                    .knockback
                    .test(ticks_used, attacker_speed, relative_speed);
            let dismounts = properties
                .dismount
                .test(ticks_used, attacker_speed, relative_speed);

            if !deals_damage && !deals_knockback && !dismounts {
                continue;
            }

            if deals_damage {
                let damage = base_damage + (relative_speed * properties.damage_multiplier).floor();
                if target
                    .damage_with_context(
                        target.as_ref(),
                        damage as f32,
                        DamageType::SPEAR,
                        None,
                        Some(player),
                        Some(player),
                    )
                    .await
                {
                    player.damage_held_item(1).await;
                    affected = true;
                }
            }

            if deals_knockback {
                let attacker_pos = player.position();
                let target_pos = target_entity.pos.load();
                target_entity.knockback(
                    0.4,
                    attacker_pos.x - target_pos.x,
                    attacker_pos.z - target_pos.z,
                );
                affected = true;
            }

            if dismounts && let Some(vehicle) = target_entity.vehicle.lock().await.clone() {
                vehicle
                    .get_entity()
                    .remove_passenger(target_entity.entity_id)
                    .await;
                affected = true;
            }
        }

        if affected {
            player.world().play_sound(
                Self::hit_sound(stack.item),
                SoundCategory::Players,
                &player.position(),
            );
        }
    }

    const fn use_sound(item: &Item) -> Sound {
        if item.id == Item::WOODEN_SPEAR.id {
            Sound::ItemSpearWoodUse
        } else {
            Sound::ItemSpearUse
        }
    }

    const fn hit_sound(item: &Item) -> Sound {
        if item.id == Item::WOODEN_SPEAR.id {
            Sound::ItemSpearWoodHit
        } else {
            Sound::ItemSpearHit
        }
    }

    const fn attack_sound(item: &Item) -> Sound {
        if item.id == Item::WOODEN_SPEAR.id {
            Sound::ItemSpearWoodAttack
        } else {
            Sound::ItemSpearAttack
        }
    }
}

#[derive(Clone, Copy)]
struct KineticCondition {
    max_duration_ticks: i32,
    min_speed: f64,
    min_relative_speed: f64,
}

impl KineticCondition {
    const fn attacker_speed(max_duration_ticks: i32, min_speed: f64) -> Self {
        Self {
            max_duration_ticks,
            min_speed,
            min_relative_speed: 0.0,
        }
    }

    const fn relative_speed(max_duration_ticks: i32, min_relative_speed: f64) -> Self {
        Self {
            max_duration_ticks,
            min_speed: 0.0,
            min_relative_speed,
        }
    }

    fn test(self, ticks_used: i32, attacker_speed: f64, relative_speed: f64) -> bool {
        ticks_used <= self.max_duration_ticks
            && attacker_speed >= self.min_speed
            && relative_speed >= self.min_relative_speed
    }
}

#[derive(Clone, Copy)]
struct KineticProperties {
    delay_ticks: i32,
    damage_multiplier: f64,
    dismount: KineticCondition,
    knockback: KineticCondition,
    damage: KineticCondition,
}

impl KineticProperties {
    const fn from_item(item: &Item) -> Option<Self> {
        match item.id {
            // A = delay_ticks, B = damage_multiplier, C = dismount_ticks,
            // D = dismount_speed, E = knockback_ticks, F = damage_ticks
            //                                                   A    B    C    D     E    F
            id if id == Item::WOODEN_SPEAR.id => Some(Self::new(15, 0.7, 100, 14.0, 200, 300)),
            id if id == Item::STONE_SPEAR.id => Some(Self::new(14, 0.82, 90, 13.0, 180, 275)),
            id if id == Item::COPPER_SPEAR.id => Some(Self::new(13, 0.82, 80, 12.0, 165, 250)),
            id if id == Item::IRON_SPEAR.id => Some(Self::new(12, 0.95, 50, 11.0, 135, 225)),
            id if id == Item::GOLDEN_SPEAR.id => Some(Self::new(14, 0.7, 70, 13.0, 170, 275)),
            id if id == Item::DIAMOND_SPEAR.id => Some(Self::new(10, 1.075, 60, 10.0, 130, 200)),
            id if id == Item::NETHERITE_SPEAR.id => Some(Self::new(8, 1.2, 50, 9.0, 110, 175)),
            _ => None,
        }
    }

    const fn new(
        delay_ticks: i32,
        damage_multiplier: f64,
        dismount_ticks: i32,
        dismount_speed: f64,
        knockback_ticks: i32,
        damage_ticks: i32,
    ) -> Self {
        Self {
            delay_ticks,
            damage_multiplier,
            dismount: KineticCondition::attacker_speed(dismount_ticks, dismount_speed),
            knockback: KineticCondition::attacker_speed(knockback_ticks, 5.1),
            damage: KineticCondition::relative_speed(damage_ticks, 4.6),
        }
    }
}

fn ray_intersection(
    start: &Vector3<f64>,
    ray: &Vector3<f64>,
    bounding_box: &BoundingBox,
) -> Option<f64> {
    let mut minimum = 0.0f64;
    let mut maximum = 1.0f64;
    let box_min = [bounding_box.min.x, bounding_box.min.y, bounding_box.min.z];
    let box_max = [bounding_box.max.x, bounding_box.max.y, bounding_box.max.z];
    let start = [start.x, start.y, start.z];
    let ray = [ray.x, ray.y, ray.z];

    for axis in 0..3 {
        if ray[axis].abs() < f64::EPSILON {
            if start[axis] < box_min[axis] || start[axis] > box_max[axis] {
                return None;
            }
        } else {
            let first = (box_min[axis] - start[axis]) / ray[axis];
            let second = (box_max[axis] - start[axis]) / ray[axis];
            minimum = minimum.max(first.min(second));
            maximum = maximum.min(first.max(second));
            if maximum < minimum {
                return None;
            }
        }
    }

    (0.0..=1.0).contains(&minimum).then_some(minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_every_spear() {
        assert_eq!(SpearItem::ids().as_ref(), tag::Item::MINECRAFT_SPEARS.1);
    }

    #[test]
    fn ray_intersection_rejects_misses_and_orders_hits() {
        let start = Vector3::new(0.0, 0.0, 0.0);
        let ray = Vector3::new(0.0, 0.0, 5.0);
        let hit = BoundingBox::new(Vector3::new(-0.5, -0.5, 2.0), Vector3::new(0.5, 0.5, 3.0));
        let miss = BoundingBox::new(Vector3::new(1.0, -0.5, 2.0), Vector3::new(2.0, 0.5, 3.0));
        let containing_start =
            BoundingBox::new(Vector3::new(-0.5, -0.5, -0.5), Vector3::new(0.5, 0.5, 0.5));

        assert_eq!(ray_intersection(&start, &ray, &hit), Some(0.4));
        assert_eq!(ray_intersection(&start, &ray, &miss), None);
        assert_eq!(
            ray_intersection(&start, &ray, &containing_start),
            Some(0.0)
        );
    }

    #[test]
    fn kinetic_properties_match_vanilla_material_values() {
        let cases = [
            // A = delay_ticks, B = damage_multiplier, C = dismount_ticks,
            // D = dismount_speed, E = knockback_ticks, F = damage_ticks
            //                    A    B    C    D     E    F
            (&Item::WOODEN_SPEAR, 15, 0.7, 100, 14.0, 200, 300),
            (&Item::STONE_SPEAR, 14, 0.82, 90, 13.0, 180, 275),
            (&Item::COPPER_SPEAR, 13, 0.82, 80, 12.0, 165, 250),
            (&Item::IRON_SPEAR, 12, 0.95, 50, 11.0, 135, 225),
            (&Item::GOLDEN_SPEAR, 14, 0.7, 70, 13.0, 170, 275),
            (&Item::DIAMOND_SPEAR, 10, 1.075, 60, 10.0, 130, 200),
            (&Item::NETHERITE_SPEAR, 8, 1.2, 50, 9.0, 110, 175),
        ];

        for (
            item,
            delay,
            multiplier,
            dismount_ticks,
            dismount_speed,
            knockback_ticks,
            damage_ticks,
        ) in cases
        {
            let properties = KineticProperties::from_item(item).unwrap();
            assert_eq!(properties.delay_ticks, delay);
            assert_eq!(properties.damage_multiplier, multiplier);
            assert_eq!(properties.dismount.max_duration_ticks, dismount_ticks);
            assert_eq!(properties.dismount.min_speed, dismount_speed);
            assert_eq!(properties.knockback.max_duration_ticks, knockback_ticks);
            assert_eq!(properties.knockback.min_speed, 5.1);
            assert_eq!(properties.damage.max_duration_ticks, damage_ticks);
            assert_eq!(properties.damage.min_relative_speed, 4.6);
        }
    }

    #[test]
    fn kinetic_conditions_include_their_duration_and_speed_boundaries() {
        let condition = KineticCondition::relative_speed(20, 4.6);

        assert!(condition.test(20, 0.0, 4.6));
        assert!(!condition.test(21, 0.0, 4.6));
        assert!(!condition.test(20, 0.0, 4.59));
    }
}
