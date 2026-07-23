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
                        target_pos.x - attacker_pos.x,
                        target_pos.z - attacker_pos.z,
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
        let start = player.eye_position();
        let (yaw, pitch) = player.rotation();
        let direction = Vector3::rotation_vector(f64::from(pitch), f64::from(yaw));
        let max_range = if player.gamemode.load() == GameMode::Creative {
            Self::CREATIVE_RANGE
        } else {
            Self::SURVIVAL_RANGE
        };
        let ray = direction * (max_range + Self::HITBOX_MARGIN);
        let end = start.add(&ray);
        let search_box = BoundingBox::new(
            Vector3::new(start.x.min(end.x), start.y.min(end.y), start.z.min(end.z)),
            Vector3::new(start.x.max(end.x), start.y.max(end.y), start.z.max(end.z)),
        )
        .expand_all(Self::HITBOX_MARGIN);

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
            let distance = intersection * ray.length();
            if distance < Self::MIN_RANGE - Self::HITBOX_MARGIN {
                continue;
            }

            let hit_position = start.add(&(ray * intersection));
            if world
                .raycast(start, hit_position, async |pos, ray_world| {
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

        assert_eq!(ray_intersection(&start, &ray, &hit), Some(0.4));
        assert_eq!(ray_intersection(&start, &ray, &miss), None);
    }
}
