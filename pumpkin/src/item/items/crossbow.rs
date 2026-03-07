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

pub struct CrossbowItem;

impl ItemMetadata for CrossbowItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::CROSSBOW.id])
    }
}

impl ItemBehaviour for CrossbowItem {
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

            // Start the crossbow loading animation
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

impl CrossbowItem {
    const CHARGE_DURATION: i32 = 25;
    const ARROW_SPEED: f32 = 3.15;

    /// Called when the player releases the crossbow
    pub async fn release_crossbow(player: &Player) {
        // Get the used ticks
        let use_ticks = player.living_entity.item_use_time.load(Ordering::Relaxed);
        let use_ticks = 72000 - use_ticks;

        // Crossbow needs to be fully charged
        if use_ticks < Self::CHARGE_DURATION {
            return;
        }

        // Check arrows again
        let arrow_slot = player.find_arrow().await;
        let gamemode = player.gamemode.load();

        if arrow_slot.is_none() && gamemode != GameMode::Creative {
            return;
        }

        // Fire the arrow
        Self.fire_arrow(player).await;

        // Consume arrow (if not creative)
        if let Some(slot) = arrow_slot
            && gamemode != GameMode::Creative
        {
            player.consume_arrow(slot).await;
        }

        // Damage crossbow
        player.damage_held_item(1).await;
    }

    /// Check if player has arrows in their inventory
    async fn has_arrows(&self, player: &Player) -> bool {
        player.find_arrow().await.is_some()
    }

    /// Fire an arrow from the crossbow
    pub async fn fire_arrow(&self, player: &Player) {
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

        // Set velocity based on player's look direction
        // Crossbows have consistent power (no charge levels)
        let yaw = player.living_entity.entity.yaw.load();
        let pitch = player.living_entity.entity.pitch.load();
        arrow.set_velocity_from_rotation(pitch, yaw, 0.0, Self::ARROW_SPEED, 1.0);

        // Crossbow arrows are not critical
        arrow.set_critical(false);

        // Spawn the arrow entity in the world
        let arrow_arc: Arc<dyn EntityBase> = Arc::new(arrow);
        world.spawn_entity(arrow_arc).await;

        // Play crossbow shoot sound
        let sound_packet = CSoundEffect::new(
            IdOr::Id(Sound::ItemCrossbowShoot as u16),
            SoundCategory::Neutral,
            &position,
            1.0,
            1.0,
            0.0,
        );
        world.broadcast_packet_all(&sound_packet).await;
    }
}
