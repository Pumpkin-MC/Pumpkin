use std::sync::atomic::Ordering;

use crate::entity::EntityBase;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{
    attributes::Attributes,
    particle::Particle,
    sound::{Sound, SoundCategory},
};
use pumpkin_util::math::vector3::Vector3;

use crate::{
    entity::{Entity, player::Player},
    world::World,
};

use crate::net::ClientPlatform;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackType {
    Knockback,
    Critical,
    Sweeping,
    Strong,
    Weak,
    MaceSmash,
}

impl AttackType {
    pub fn new(player: &Player, attack_cooldown_progress: f32) -> Self {
        let entity = &player.get_entity();

        let sprinting = entity.is_sprinting();
        let on_ground = entity.on_ground.load(Ordering::Relaxed);
        let fall_distance = player.living_entity.fall_distance.load();
        let held_item = player.inventory().held_item();
        let is_mace = held_item.item.id == pumpkin_data::item::Item::MACE.id;

        if is_mace && !on_ground && fall_distance > 1.5 {
            return Self::MaceSmash;
        }

        let sword = held_item.is_sword();
        let is_bedrock = matches!(player.client.as_ref(), ClientPlatform::Bedrock(_));

        let is_strong = attack_cooldown_progress > 0.9;
        if sprinting && is_strong {
            return Self::Knockback;
        }

        if is_strong && !on_ground && fall_distance > 0.0 {
            return Self::Critical;
        }

        if sword && is_strong && !is_bedrock {
            return Self::Sweeping;
        }

        if is_strong { Self::Strong } else { Self::Weak }
    }
}

/// Scales a knockback `strength` by a living entity's knockback resistance,
/// mirroring vanilla `LivingEntity.knockback`: `strength *= 1.0 - resistance`.
/// A resistance of 1.0 (iron golem, warden, ...) cancels the knockback entirely.
pub fn knockback_after_resistance(strength: f64, resistance: f64) -> f64 {
    strength * (1.0 - resistance)
}

pub fn handle_knockback(attacker: &Entity, victim: &dyn EntityBase, strength: f64) {
    let resistance = victim.get_living_entity().map_or(0.0, |living| {
        living.get_attribute_value(&Attributes::KNOCKBACK_RESISTANCE)
    });
    let strength = knockback_after_resistance(strength * 0.5, resistance);

    if strength > 0.0 {
        let yaw = attacker.yaw.load();
        victim.get_entity().knockback(
            strength,
            f64::from((yaw.to_radians()).sin()),
            f64::from(-(yaw.to_radians()).cos()),
        );
    }

    let velocity = attacker.velocity.load();
    attacker.velocity.store(velocity.multiply(0.6, 1.0, 0.6));
}

pub fn spawn_sweep_particle(attacker_entity: &Entity, world: &World, pos: &Vector3<f64>) {
    let yaw = attacker_entity.yaw.load();
    let d = -f64::from((yaw.to_radians()).sin());
    let e = f64::from((yaw.to_radians()).cos());

    let scale = 0.5;
    let body_y = f64::from(attacker_entity.height()).mul_add(scale, pos.y);

    world.spawn_particle(
        Vector3::new(pos.x + d, body_y, pos.z + e),
        Vector3::new(0.0, 0.0, 0.0),
        0.0,
        0,
        Particle::SweepAttack,
    );
}

pub fn player_attack_sound(pos: &Vector3<f64>, world: &World, attack_type: AttackType) {
    match attack_type {
        AttackType::Knockback => {
            world.play_sound(
                Sound::EntityPlayerAttackKnockback,
                SoundCategory::Players,
                pos,
            );
        }
        AttackType::Critical => {
            world.play_sound(Sound::EntityPlayerAttackCrit, SoundCategory::Players, pos);
        }
        AttackType::Sweeping => {
            world.play_sound(Sound::EntityPlayerAttackSweep, SoundCategory::Players, pos);
        }
        AttackType::Strong => {
            world.play_sound(Sound::EntityPlayerAttackStrong, SoundCategory::Players, pos);
        }
        AttackType::Weak => {
            world.play_sound(Sound::EntityPlayerAttackWeak, SoundCategory::Players, pos);
        }
        AttackType::MaceSmash => {
            world.play_sound(Sound::ItemMaceSmashAir, SoundCategory::Players, pos);
        }
    }
}

/// Combat calculation rules mirroring vanilla `net.minecraft.world.damagesource.CombatRules`.
pub struct CombatRules;

impl CombatRules {
    pub const MAX_ARMOR: f32 = 20.0;
    pub const ARMOR_PROTECTION_DIVIDER: f32 = 25.0;
    pub const BASE_ARMOR_TOUGHNESS: f32 = 2.0;
    pub const MIN_ARMOR_RATIO: f32 = 0.2;

    /// Calculates damage after armor reduction, mirroring vanilla `CombatRules.getDamageAfterAbsorb`.
    #[must_use]
    pub fn get_damage_after_absorb(
        damage: f32,
        total_armor: f32,
        armor_toughness: f32,
        breach_level: u32,
    ) -> f32 {
        let toughness = Self::BASE_ARMOR_TOUGHNESS + armor_toughness / 4.0;
        let real_armor = (total_armor - damage / toughness)
            .clamp(total_armor * Self::MIN_ARMOR_RATIO, Self::MAX_ARMOR);
        let mut armor_fraction = real_armor / Self::ARMOR_PROTECTION_DIVIDER;

        if breach_level > 0 {
            let reduction = (breach_level as f32 * 0.15).min(1.0);
            armor_fraction = (armor_fraction * (1.0 - reduction)).clamp(0.0, 1.0);
        }

        let damage_multiplier = 1.0 - armor_fraction;
        damage * damage_multiplier
    }

    /// Calculates damage after magic/enchantment armor reduction, mirroring vanilla `CombatRules.getDamageAfterMagicAbsorb`.
    #[must_use]
    pub fn get_damage_after_magic_absorb(damage: f32, total_magic_armor: f32) -> f32 {
        let real_armor = total_magic_armor.clamp(0.0, Self::MAX_ARMOR);
        damage * (1.0 - real_armor / Self::ARMOR_PROTECTION_DIVIDER)
    }
}

pub const RESET_DAMAGE_STATUS_TIME: i64 = 100;
pub const RESET_COMBAT_STATUS_TIME: i64 = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallLocation {
    Generic,
    Ladder,
    Vines,
    WeepingVines,
    TwistingVines,
    Scaffolding,
    OtherClimbable,
    Water,
}

impl FallLocation {
    #[must_use]
    pub const fn language_key(self) -> &'static str {
        match self {
            Self::Generic => "death.fell.accident.generic",
            Self::Ladder => "death.fell.accident.ladder",
            Self::Vines => "death.fell.accident.vines",
            Self::WeepingVines => "death.fell.accident.weeping_vines",
            Self::TwistingVines => "death.fell.accident.twisting_vines",
            Self::Scaffolding => "death.fell.accident.scaffolding",
            Self::OtherClimbable => "death.fell.accident.other_climbable",
            Self::Water => "death.fell.accident.water",
        }
    }

    pub fn get_current_fall_location(
        living: &crate::entity::living::LivingEntity,
        world: &World,
    ) -> Self {
        let pos = living
            .climbing_pos
            .load()
            .unwrap_or_else(|| living.entity.block_pos.load());
        let block = world.get_block(&pos);
        if block == &pumpkin_data::Block::LADDER
            || block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_TRAPDOORS)
        {
            Self::Ladder
        } else if block == &pumpkin_data::Block::VINE {
            Self::Vines
        } else if block == &pumpkin_data::Block::WEEPING_VINES
            || block == &pumpkin_data::Block::WEEPING_VINES_PLANT
        {
            Self::WeepingVines
        } else if block == &pumpkin_data::Block::TWISTING_VINES
            || block == &pumpkin_data::Block::TWISTING_VINES_PLANT
        {
            Self::TwistingVines
        } else if block == &pumpkin_data::Block::SCAFFOLDING {
            Self::Scaffolding
        } else if living.entity.is_in_water() {
            Self::Water
        } else if block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_CLIMBABLE) {
            Self::OtherClimbable
        } else {
            Self::Generic
        }
    }
}

#[derive(Clone)]
pub struct CombatEntry {
    pub damage_type: pumpkin_data::damage::DamageType,
    pub damage: f32,
    pub fall_location: Option<FallLocation>,
    pub fall_distance: f32,
    pub timestamp: i64,
    pub source_id: Option<i32>,
    pub attacker_id: Option<i32>,
    pub attacker_name: Option<pumpkin_util::text::TextComponent>,
    pub attacker_item_name: Option<pumpkin_util::text::TextComponent>,
    pub attacker_is_living: bool,
    pub attacker_is_player: bool,
}

#[derive(Default)]
pub struct CombatTracker {
    entries: Vec<CombatEntry>,
    last_damage_time: i64,
    combat_start_time: i64,
    combat_end_time: i64,
    in_combat: bool,
    taking_damage: bool,
}

impl CombatTracker {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_damage(
        &mut self,
        current_tick: i64,
        is_alive: bool,
        fall_distance: f32,
        fall_location: FallLocation,
        damage_type: pumpkin_data::damage::DamageType,
        damage: f32,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) {
        self.recheck_status(current_tick, is_alive);

        let attacker = cause.or(source);
        let attacker_id = attacker.map(|e| e.get_entity().entity_id);
        let attacker_name = attacker.map(super::EntityBase::get_display_name);
        let attacker_is_living = attacker.is_some_and(|e| e.get_living_entity().is_some());
        let attacker_is_player = attacker.is_some_and(|e| {
            e.get_entity().entity_type == &pumpkin_data::entity::EntityType::PLAYER
        });

        // Check for custom named item in attacker's main hand
        let attacker_item_name =
            attacker.and_then(|e| {
                if let Some(player) = e.get_player() {
                    let hand_stack = player
                        .inventory()
                        .get_stack_in_hand(pumpkin_util::Hand::Right);
                    if !hand_stack.is_empty()
                    && let Some(custom_name) = hand_stack
                        .get_data_component::<pumpkin_data::data_component_impl::CustomNameImpl>()
                {
                    return Some(custom_name.name.clone());
                }
                } else if let Some(living) = e.get_living_entity() {
                    let equip_guard = living
                        .entity_equipment
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    if let Some(stack) = equip_guard
                    .equipment
                    .get(&pumpkin_data::data_component_impl::EquipmentSlot::MAIN_HAND)
                    && !stack.is_empty()
                    && let Some(custom_name) = stack
                        .get_data_component::<pumpkin_data::data_component_impl::CustomNameImpl>()
                {
                    return Some(custom_name.name.clone());
                }
                }
                None
            });

        let entry = CombatEntry {
            damage_type,
            damage,
            fall_location: Some(fall_location),
            fall_distance,
            timestamp: current_tick,
            source_id: source.map(|e| e.get_entity().entity_id),
            attacker_id,
            attacker_name,
            attacker_item_name,
            attacker_is_living,
            attacker_is_player,
        };

        self.entries.push(entry);
        self.last_damage_time = current_tick;
        self.taking_damage = true;

        if !self.in_combat && is_alive && attacker_is_living {
            self.in_combat = true;
            self.combat_start_time = current_tick;
            self.combat_end_time = self.combat_start_time;
        }
    }

    pub fn recheck_status(&mut self, current_tick: i64, is_alive: bool) {
        let reset = if self.in_combat {
            RESET_COMBAT_STATUS_TIME
        } else {
            RESET_DAMAGE_STATUS_TIME
        };
        if self.taking_damage && (!is_alive || current_tick - self.last_damage_time > reset) {
            self.taking_damage = false;
            self.in_combat = false;
            self.combat_end_time = current_tick;
            self.entries.clear();
        }
    }

    #[must_use]
    pub const fn get_combat_duration(&self, current_tick: i64) -> i64 {
        if self.in_combat {
            current_tick - self.combat_start_time
        } else {
            self.combat_end_time - self.combat_start_time
        }
    }

    #[must_use]
    pub const fn is_in_combat(&self) -> bool {
        self.in_combat
    }

    #[must_use]
    pub fn has_player_attacker(&self) -> bool {
        self.entries.iter().any(|e| e.attacker_is_player)
    }

    #[must_use]
    pub fn get_killer_entry(&self) -> Option<&CombatEntry> {
        let mut best_living: Option<&CombatEntry> = None;
        let mut best_player: Option<&CombatEntry> = None;
        let mut max_living_dmg = 0.0f32;
        let mut max_player_dmg = 0.0f32;

        for entry in &self.entries {
            if entry.attacker_is_player && (best_player.is_none() || entry.damage > max_player_dmg)
            {
                max_player_dmg = entry.damage;
                best_player = Some(entry);
            }
            if entry.attacker_is_living && (best_living.is_none() || entry.damage > max_living_dmg)
            {
                max_living_dmg = entry.damage;
                best_living = Some(entry);
            }
        }

        if best_player.is_some() && max_player_dmg >= max_living_dmg / 3.0 {
            best_player
        } else {
            best_living
        }
    }

    #[must_use]
    pub fn get_death_message(
        &self,
        victim_name: pumpkin_util::text::TextComponent,
    ) -> pumpkin_util::text::TextComponent {
        if self.entries.is_empty() {
            return pumpkin_util::text::TextComponent::translate_cross(
                "death.attack.generic",
                "death.attack.generic",
                [victim_name],
            );
        }

        let killing_blow = &self.entries[self.entries.len() - 1];
        let killing_damage_type = killing_blow.damage_type;
        let knock_off_entry = self.get_most_significant_fall();

        match killing_damage_type.death_message_type {
            pumpkin_data::damage::DeathMessageType::FallVariants => {
                if let Some(knock_off) = knock_off_entry {
                    Self::get_fall_message(victim_name, knock_off, killing_blow)
                } else {
                    let loc = killing_blow.fall_location.unwrap_or(FallLocation::Generic);
                    pumpkin_util::text::TextComponent::translate_cross(
                        loc.language_key(),
                        loc.language_key(),
                        [victim_name],
                    )
                }
            }
            pumpkin_data::damage::DeathMessageType::IntentionalGameDesign => {
                let death_msg = format!("death.attack.{}", killing_damage_type.message_id);
                let link = pumpkin_util::text::TextComponent::text("[")
                    .add_child(pumpkin_util::text::TextComponent::translate_cross(
                        format!("{death_msg}.link"),
                        format!("{death_msg}.link"),
                        [],
                    ))
                    .add_child(pumpkin_util::text::TextComponent::text("]"))
                    .click_event(pumpkin_util::text::click::ClickEvent::OpenUrl {
                        url: "https://bugs.mojang.com/browse/MCPE-28723".into(),
                    })
                    .hover_event(pumpkin_util::text::hover::HoverEvent::show_text(
                        pumpkin_util::text::TextComponent::text("MCPE-28723"),
                    ));

                pumpkin_util::text::TextComponent::translate_cross(
                    format!("{death_msg}.message"),
                    format!("{death_msg}.message"),
                    [victim_name, link],
                )
            }
            pumpkin_data::damage::DeathMessageType::Default => {
                self.get_default_death_message(victim_name, killing_blow)
            }
        }
    }

    fn get_most_significant_fall(&self) -> Option<&CombatEntry> {
        let mut result = None;
        let mut alternative = None;
        let mut alt_damage = 0.0f32;
        let mut best_fall = 0.0f32;

        for i in 0..self.entries.len() {
            let entry = &self.entries[i];
            let previous = (i > 0).then(|| &self.entries[i - 1]);
            let damage_type = entry.damage_type;
            let is_fake_fall = damage_type
                .has_tag(&pumpkin_data::tag::DamageType::MINECRAFT_ALWAYS_MOST_SIGNIFICANT_FALL);
            let fall_distance = if is_fake_fall {
                f32::MAX
            } else {
                entry.fall_distance
            };

            if (damage_type.has_tag(&pumpkin_data::tag::DamageType::MINECRAFT_IS_FALL)
                || is_fake_fall)
                && fall_distance > 0.0
                && (result.is_none() || fall_distance > best_fall)
            {
                if i > 0 {
                    result = previous;
                } else {
                    result = Some(entry);
                }
                best_fall = fall_distance;
            }

            if entry.fall_location.is_some() && (alternative.is_none() || entry.damage > alt_damage)
            {
                alternative = Some(entry);
                alt_damage = entry.damage;
            }
        }

        if best_fall > 5.0 && result.is_some() {
            result
        } else if alt_damage > 5.0 && alternative.is_some() {
            alternative
        } else {
            None
        }
    }

    fn get_fall_message(
        victim_name: pumpkin_util::text::TextComponent,
        knock_off_entry: &CombatEntry,
        killing_blow: &CombatEntry,
    ) -> pumpkin_util::text::TextComponent {
        let knock_off_type = knock_off_entry.damage_type;
        if !knock_off_type.has_tag(&pumpkin_data::tag::DamageType::MINECRAFT_IS_FALL)
            && !knock_off_type
                .has_tag(&pumpkin_data::tag::DamageType::MINECRAFT_ALWAYS_MOST_SIGNIFICANT_FALL)
        {
            let killer_name = killing_blow.attacker_name.as_ref();
            let attacker_name = knock_off_entry.attacker_name.as_ref();
            let attacker_item = knock_off_entry.attacker_item_name.clone();

            if let Some(attacker_name) = attacker_name
                && (killer_name.is_none() || killer_name != Some(attacker_name))
            {
                Self::get_message_for_assisted_fall(
                    victim_name,
                    attacker_name.clone(),
                    attacker_item,
                    "death.fell.assist.item",
                    "death.fell.assist",
                )
            } else if let Some(killer_name) = killer_name {
                Self::get_message_for_assisted_fall(
                    victim_name,
                    killer_name.clone(),
                    killing_blow.attacker_item_name.clone(),
                    "death.fell.finish.item",
                    "death.fell.finish",
                )
            } else {
                pumpkin_util::text::TextComponent::translate_cross(
                    "death.fell.killer",
                    "death.fell.killer",
                    [victim_name],
                )
            }
        } else {
            let loc = knock_off_entry
                .fall_location
                .unwrap_or(FallLocation::Generic);
            pumpkin_util::text::TextComponent::translate_cross(
                loc.language_key(),
                loc.language_key(),
                [victim_name],
            )
        }
    }

    fn get_message_for_assisted_fall(
        victim_name: pumpkin_util::text::TextComponent,
        attacker_name: pumpkin_util::text::TextComponent,
        attacker_item: Option<pumpkin_util::text::TextComponent>,
        message_with_item: &'static str,
        message_without_item: &'static str,
    ) -> pumpkin_util::text::TextComponent {
        if let Some(item_name) = attacker_item {
            pumpkin_util::text::TextComponent::translate_cross(
                message_with_item,
                message_with_item,
                [victim_name, attacker_name, item_name],
            )
        } else {
            pumpkin_util::text::TextComponent::translate_cross(
                message_without_item,
                message_without_item,
                [victim_name, attacker_name],
            )
        }
    }

    fn get_default_death_message(
        &self,
        victim_name: pumpkin_util::text::TextComponent,
        killing_blow: &CombatEntry,
    ) -> pumpkin_util::text::TextComponent {
        let damage_type = killing_blow.damage_type;
        let msg_id = damage_type.message_id;

        if let Some(attacker_name) = &killing_blow.attacker_name {
            if let Some(item_name) = &killing_blow.attacker_item_name {
                pumpkin_util::text::TextComponent::translate_cross(
                    format!("death.attack.{msg_id}.item"),
                    format!("death.attack.{msg_id}.item"),
                    [victim_name, attacker_name.clone(), item_name.clone()],
                )
            } else {
                pumpkin_util::text::TextComponent::translate_cross(
                    format!("death.attack.{msg_id}.player"),
                    format!("death.attack.{msg_id}.player"),
                    [victim_name, attacker_name.clone()],
                )
            }
        } else if let Some(killer) = self.get_killer_entry()
            && let Some(killer_name) = &killer.attacker_name
        {
            pumpkin_util::text::TextComponent::translate_cross(
                format!("death.attack.{msg_id}.player"),
                format!("death.attack.{msg_id}.player"),
                [victim_name, killer_name.clone()],
            )
        } else {
            pumpkin_util::text::TextComponent::translate_cross(
                format!("death.attack.{msg_id}"),
                format!("death.attack.{msg_id}"),
                [victim_name],
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_resistance_keeps_full_strength() {
        assert_eq!(knockback_after_resistance(0.4, 0.0), 0.4);
    }

    #[test]
    fn full_resistance_cancels_knockback() {
        // Iron golem / warden have KNOCKBACK_RESISTANCE == 1.0.
        assert_eq!(knockback_after_resistance(0.4, 1.0), 0.0);
    }

    #[test]
    fn partial_resistance_scales_strength() {
        // Ravager has KNOCKBACK_RESISTANCE == 0.75.
        assert!((knockback_after_resistance(0.4, 0.75) - 0.1).abs() < 1e-9);
    }

    #[test]
    fn over_full_resistance_is_negative_so_callers_skip_it() {
        // Stacked armour modifiers can push resistance above 1.0; the result is
        // negative and callers guard on `strength > 0.0`.
        assert!(knockback_after_resistance(0.4, 1.2) < 0.0);
    }
}
