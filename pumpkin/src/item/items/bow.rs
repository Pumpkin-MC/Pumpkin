use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::entity::player::Player;
use crate::entity::projectile::arrow::{ArrowEntity, ArrowPickup};
use crate::entity::{Entity, EntityBase};
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
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
    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            // Check if player has arrows (or is in creative mode)
            let has_arrows = self.has_arrows(player).await;
            let gamemode = player.gamemode.load();

            if !has_arrows && gamemode != GameMode::Creative {
                return;
            }

            // Get the held item stack
            let inventory = player.inventory();
            let held = inventory.held_item();
            let stack = held.lock().await.clone();

            // Start the bow drawing animation
            player
                .living_entity
                .set_active_hand(pumpkin_util::Hand::Right, stack)
                .await;
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl BowItem {
    const MAX_DRAW_DURATION: f32 = 20.0;
    const ARROW_SPEED_MULTIPLIER: f32 = 3.0;

    /// Called when the player releases the bow
    pub async fn release_bow(player: &Player) {
        // Get the used ticks
        let use_ticks = player.living_entity.item_use_time.load(Ordering::Relaxed);
        let use_ticks = 72000 - use_ticks;

        // Check minimum draw time
        if use_ticks < 3 {
            return;
        }

        // Check arrows again
        let arrow_slot = player.find_arrow().await;
        let gamemode = player.gamemode.load();

        if arrow_slot.is_none() && gamemode != GameMode::Creative {
            return;
        }

        // Calculate power and fire
        let power = Self::get_power_for_time(use_ticks);
        Self.fire_arrow(player, power).await;

        // Consume arrow (if not creative)
        if let Some(slot) = arrow_slot
            && gamemode != GameMode::Creative
        {
            player.consume_arrow(slot).await;
        }

        // Damage bow
        player.damage_held_item(1).await;
    }

    /// Check if player has arrows in their inventory
    async fn has_arrows(&self, player: &Player) -> bool {
        player.find_arrow().await.is_some()
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
    pub async fn fire_arrow(&self, player: &Player, power: f32) {
        if power < 0.1 {
            return; // Not enough charge
        }

        let world = player.world();
        let position = player.position();

        // Create arrow entity
        let arrow_entity = Entity::new(world.clone(), position, &EntityType::ARROW);

        // Determine pickup mode based on gamemode
        let gamemode = player.gamemode.load();
        let pickup = if gamemode == GameMode::Creative {
            ArrowPickup::CreativeOnly
        } else {
            ArrowPickup::Allowed
        };

        let arrow = ArrowEntity::new_shot(arrow_entity, &player.living_entity.entity, pickup).await;

        // Set velocity based on player's look direction and power
        let yaw = player.living_entity.entity.yaw.load();
        let pitch = player.living_entity.entity.pitch.load();
        let speed = power * Self::ARROW_SPEED_MULTIPLIER;
        arrow.set_velocity_from_rotation(pitch, yaw, 0.0, speed, 1.0);

        // Set critical if fully charged
        if power >= 1.0 {
            arrow.set_critical(true);
        }

        // Spawn the arrow entity in the world
        let arrow_arc: Arc<dyn EntityBase> = Arc::new(arrow);
        world.spawn_entity(arrow_arc).await;

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
        world.broadcast_packet_all(&sound_packet).await;
    }
}
