use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::entity::player::Player;
use crate::entity::projectile::arrow::{ArrowEntity, ArrowPickup};
use crate::entity::{Entity, EntityBase};
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_protocol::IdOr;
use pumpkin_protocol::java::client::play::CSoundEffect;
use pumpkin_util::GameMode;

pub struct BowItem;

impl ItemMetadata for BowItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::BOW.id])
    }
}

impl ItemBehaviour for BowItem {
    fn normal_use(&self, _item: &Item, player: &Player) {
        // Check if player has arrows (or is in creative mode)
        let has_arrows = Self::has_arrows(player);
        let gamemode = player.gamemode.load();

        if !has_arrows && gamemode != GameMode::Creative {
            return;
        }

        // Get the held item stack
        let inventory = player.inventory();
        let stack = inventory.held_item();

        // Start the bow drawing animation
        player
            .living_entity
            .set_active_hand(pumpkin_util::Hand::Right, stack, Self::USE_DURATION);
    }

    fn on_stopped_using(&self, _stack: &ItemStack, player: &Player) {
        Self::release_bow(player);
    }

    fn get_use_duration(&self) -> i32 {
        Self::USE_DURATION
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl BowItem {
    /// The maximum number of ticks a bow can be drawn for
    pub const USE_DURATION: i32 = 72000;
    const MAX_DRAW_DURATION: f32 = 20.0;
    const ARROW_SPEED_MULTIPLIER: f32 = 3.0;

    /// Called when the player releases the bow
    pub fn release_bow(player: &Player) {
        // Get the used ticks
        let use_ticks = player.living_entity.item_use_time.load(Ordering::Relaxed);
        let use_ticks = Self::USE_DURATION - use_ticks;

        // Check minimum draw time
        if use_ticks < 3 {
            return;
        }

        // Check arrows again
        let arrow_slot = player.find_arrow();
        let gamemode = player.gamemode.load();

        if arrow_slot.is_none() && gamemode != GameMode::Creative {
            return;
        }

        let projectile = arrow_slot.map_or_else(
            || ItemStack::new(1, &Item::ARROW),
            |slot| {
                let stack = player.inventory.get_slot(slot);
                stack.copy_with_count(1)
            },
        );
        let infinite_projectile = projectile.item.id == Item::ARROW.id;

        // Calculate power and fire
        let power = Self::get_power_for_time(use_ticks);

        // Check for Infinity enchantment
        let mut has_infinity = false;
        let held = player.inventory().held_item();
        if let Some(enchantments) =
            held.get_data_component::<pumpkin_data::data_component_impl::EnchantmentsImpl>()
        {
            has_infinity = enchantments
                .enchantment
                .iter()
                .any(|(e, _)| **e == pumpkin_data::Enchantment::INFINITY);
        }

        Self::fire_arrow(player, power, &projectile);

        // Consume arrow (if not creative and no Infinity)
        if let Some(slot) = arrow_slot
            && gamemode != GameMode::Creative
            && !(has_infinity && infinite_projectile)
        {
            player.consume_arrow(slot);
        }

        // Damage bow
        player.damage_held_item(1);
    }

    /// Check if player has arrows in their inventory
    fn has_arrows(player: &Player) -> bool {
        player.find_arrow().is_some()
    }

    /// Calculate the power/charge of the bow based on time held
    #[must_use]
    pub fn get_power_for_time(time_held: i32) -> f32 {
        let mut power = time_held as f32 / Self::MAX_DRAW_DURATION;
        power = (power * power + power * 2.0) / 3.0;
        if power > 1.0 {
            power = 1.0;
        }
        power
    }

    /// Fire an arrow from the bow
    pub fn fire_arrow(player: &Player, power: f32, projectile: &ItemStack) {
        if power < 0.1 {
            return; // Not enough charge
        }

        let world = player.world();
        let position = player.position();

        // Create arrow entity
        let arrow_entity = Entity::new(
            world.clone(),
            position,
            ArrowEntity::entity_type_for_item(projectile.item),
        );

        // Determine pickup mode based on gamemode
        let gamemode = player.gamemode.load();
        let pickup = if gamemode == GameMode::Creative {
            ArrowPickup::CreativeOnly
        } else {
            ArrowPickup::Allowed
        };

        let mut arrow =
            ArrowEntity::new_shot(arrow_entity, player.get_entity(), projectile, pickup);

        // Read enchantments of the held item (bow)
        let stack = player.inventory().held_item();
        if let Some(enchantments) =
            stack.get_data_component::<pumpkin_data::data_component_impl::EnchantmentsImpl>()
        {
            for (enchantment, level) in enchantments.enchantment.iter() {
                if **enchantment == pumpkin_data::Enchantment::POWER {
                    arrow.base_damage *= 1.0 + 0.25 * (f64::from(*level) + 1.0);
                } else if **enchantment == pumpkin_data::Enchantment::PUNCH {
                    arrow.punch_level.store(*level as u8, Ordering::Relaxed);
                } else if **enchantment == pumpkin_data::Enchantment::FLAME {
                    arrow.is_flame.store(true, Ordering::Relaxed);
                }
            }
        }
        drop(stack);

        // Set velocity based on player's look direction and power
        let (yaw, pitch) = player.rotation();
        let speed = power * Self::ARROW_SPEED_MULTIPLIER;
        arrow.set_velocity_from_rotation(pitch, yaw, 0.0, speed, 1.0);

        // Set critical if fully charged
        if power >= 1.0 {
            arrow.set_critical(true);
        }

        // Spawn the arrow entity in the world
        let arrow_arc: Arc<dyn EntityBase> = Arc::new(arrow);
        world.spawn_entity(arrow_arc);

        // Play bow shoot sound
        let sound_pitch = 1.0 / (rand::random::<f32>() * 0.4 + 1.2) + power * 0.5;
        let sound_packet = CSoundEffect::new(
            IdOr::Id(Sound::EntityArrowShoot as u16),
            SoundCategory::Neutral,
            &position,
            1.0,
            sound_pitch,
            0.0,
        );
        let chunk_pos = player.get_entity().chunk_pos.load();
        world.broadcast_to_chunk(chunk_pos, &sound_packet);
    }
}
