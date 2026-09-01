use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::entity::combat;
use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase};
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::{
    AttributeModifiersImpl, EnchantmentsImpl, EquipmentSlot, Operation, WeaponImpl,
};
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Enchantment, tag};
use pumpkin_util::math::boundingbox::BoundingBox;
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
        player
            .living_entity
            .set_active_hand(hand, stack, Self::USE_DURATION);
        player.world().play_sound_expect(
            player,
            Self::use_sound(item),
            SoundCategory::Players,
            &player.position(),
        );
    }

    fn on_spear_jab(&self, stack: &ItemStack, player: &Player) {
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
        for target in Self::targets_in_range(player, &server) {
            hit_something |= Self::stab_attack(
                player,
                &server,
                Hand::Right,
                stack,
                &target,
                damage,
                StabEffects {
                    damage: true,
                    knockback: true,
                    dismount: false,
                },
            );
        }
        player.last_attacked_ticks.store(0, Ordering::Relaxed);

        let position = player.position();
        if hit_something {
            world.play_sound(
                Self::hit_sound(stack.item),
                SoundCategory::Players,
                &position,
            );
        }
        world.play_sound_expect(
            player,
            Self::attack_sound(stack.item),
            SoundCategory::Players,
            &position,
        );
        player.swing_hand(Hand::Right, false);
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
    const MIN_RANGE: f64 = 2.0;
    const SURVIVAL_RANGE: f64 = 4.5;
    const CREATIVE_RANGE: f64 = 6.5;
    const HITBOX_MARGIN: f64 = 0.125;

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

    fn targets_in_range(player: &Player, server: &Server) -> Vec<Arc<dyn EntityBase>> {
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
            if !Self::can_hit(player, server, target.as_ref()) {
                continue;
            }
            let entity = target.get_entity();

            let Some(intersection) = ray_intersection(
                &start,
                &ray,
                &entity.bounding_box.load().expand_all(Self::HITBOX_MARGIN),
            ) else {
                continue;
            };
            if intersection * ray.length() < Self::MIN_RANGE - Self::HITBOX_MARGIN {
                continue;
            }

            let hit_position = start.add(&(ray * intersection));
            if world
                .raycast(start, hit_position, |pos, ray_world| {
                    !ray_world.get_block_state(pos).is_air()
                })
                .is_none()
            {
                targets.push((intersection, target));
            }
        }

        targets.sort_by(|a, b| a.0.total_cmp(&b.0));
        targets.into_iter().map(|(_, target)| target).collect()
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
