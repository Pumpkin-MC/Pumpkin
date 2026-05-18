use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_data::Enchantment;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_protocol::IdOr;
use pumpkin_protocol::java::client::play::CSoundEffect;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::player::Player;
use crate::entity::projectile::trident::{TridentEnchants, TridentEntity, TridentPickup};
use crate::entity::{Entity, EntityBase};
use crate::item::{ItemBehaviour, ItemMetadata};

pub struct TridentItem;

impl TridentItem {
    pub const USE_DURATION: i32 = 72000;
    /// Minimum ticks the player must hold to actually throw / riptide.
    const MIN_USE_TICKS: i32 = 10;
    /// Velocity multiplier applied to the trident direction vector.
    const THROW_SPEED: f32 = 2.5;
}

impl ItemMetadata for TridentItem {
    fn ids() -> Box<[u16]> {
        [Item::TRIDENT.id].into()
    }
}

impl ItemBehaviour for TridentItem {
    fn can_mine(&self, player: &Player) -> bool {
        player.gamemode.load() != GameMode::Creative
    }

    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            // If we are already charging this trident, don't re-arm. The
            // client can send SUseItem more than once while right-click is
            // held, and re-broadcasting the "using item" metadata each time
            // causes the held-trident pose to flicker on every other client.
            if player.living_entity.item_use_time.load(Ordering::Relaxed) > 0 {
                return;
            }

            let inventory = player.inventory();
            let held = inventory.held_item();
            let stack = held.lock().await.clone();

            // Block throw when the trident is about to break next use.
            if let Some(max) = stack.get_max_damage()
                && stack.get_damage() >= max - 1
            {
                return;
            }

            // Riptide can only be used when the player is in water OR exposed
            // to rain. Vanilla silently refuses to start charging otherwise.
            let riptide = stack.get_enchantment_level(&Enchantment::RIPTIDE);
            if riptide > 0 && !can_riptide(player).await {
                return;
            }

            player
                .living_entity
                .set_active_hand(pumpkin_util::Hand::Right, stack, Self::USE_DURATION)
                .await;
        })
    }

    fn on_stopped_using<'a>(
        &'a self,
        _stack: &'a ItemStack,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let remaining = player.living_entity.item_use_time.load(Ordering::Relaxed);
            let use_ticks = Self::USE_DURATION - remaining;
            // Always clear the "using item" living-entity flag — otherwise the
            // player stays stuck in the trident-charging pose on every client.
            player.living_entity.clear_active_hand().await;

            if use_ticks < Self::MIN_USE_TICKS {
                return;
            }

            let held = player.inventory().held_item();
            let mut held_guard = held.lock().await;

            // Re-check break-soon gate (state may have changed).
            if let Some(max) = held_guard.get_max_damage()
                && held_guard.get_damage() >= max - 1
            {
                return;
            }

            let riptide = held_guard.get_enchantment_level(&Enchantment::RIPTIDE);
            let loyalty = held_guard
                .get_enchantment_level(&Enchantment::LOYALTY)
                .max(0) as u8;
            let channeling = held_guard.get_enchantment_level(&Enchantment::CHANNELING) > 0;
            let impaling = held_guard
                .get_enchantment_level(&Enchantment::IMPALING)
                .max(0) as u8;

            if riptide > 0 {
                // Riptide path: don't spawn a projectile; propel the player.
                if !can_riptide(player).await {
                    return;
                }
                // Apply 1 durability of wear; trident stays in hand.
                let dmg = held_guard.get_damage();
                held_guard.patch.push((
                    pumpkin_data::data_component::DataComponent::Damage,
                    Some(Box::new(pumpkin_data::data_component_impl::DamageImpl {
                        damage: dmg + 1,
                    })),
                ));
                drop(held_guard);
                Self::riptide_launch(player, riptide as u8).await;
                return;
            }

            // Normal throw: remove trident from inventory; the projectile
            // carries the (bumped-durability) stack.
            let mut captured = held_guard.clone();
            if player.gamemode.load() != GameMode::Creative {
                let dmg = captured.get_damage();
                captured.patch.push((
                    pumpkin_data::data_component::DataComponent::Damage,
                    Some(Box::new(pumpkin_data::data_component_impl::DamageImpl {
                        damage: dmg + 1,
                    })),
                ));
                *held_guard = ItemStack::EMPTY.clone();
            }
            drop(held_guard);

            let enchants = TridentEnchants {
                loyalty,
                channeling,
                impaling,
            };
            Self.throw_trident(player, captured, enchants).await;
        })
    }

    fn get_use_duration(&self) -> i32 {
        Self::USE_DURATION
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TridentItem {
    async fn throw_trident(&self, player: &Player, stack: ItemStack, enchants: TridentEnchants) {
        let world = player.world();
        let position = player.position();

        let entity = Entity::new(world.clone(), position, &EntityType::TRIDENT);

        let pickup = if player.gamemode.load() == GameMode::Creative {
            TridentPickup::CreativeOnly
        } else {
            TridentPickup::Allowed
        };

        let trident = TridentEntity::new_shot(
            entity,
            &player.living_entity.entity,
            pickup,
            stack,
            enchants,
        );

        let yaw = player.living_entity.entity.yaw.load();
        let pitch = player.living_entity.entity.pitch.load();
        trident.set_velocity_from_rotation(pitch, yaw, 0.0, Self::THROW_SPEED, 1.0);

        let trident_arc: Arc<dyn EntityBase> = Arc::new(trident);
        world.spawn_entity(trident_arc).await;

        let sound_packet = CSoundEffect::new(
            IdOr::Id(Sound::ItemTridentThrow as u16),
            SoundCategory::Players,
            &position,
            1.0,
            1.0,
            0.0,
        );
        world.broadcast_packet_all(&sound_packet);
    }

    /// Riptide: propel the player along their look vector, play the
    /// level-specific sound, and start a 20-tick spin attack that damages
    /// nearby entities for 8 hp each per hit (rate-limited by their hurt
    /// cooldowns). Trident durability damage is applied by the caller.
    async fn riptide_launch(player: &Player, level: u8) {
        let entity = &player.living_entity.entity;
        let yaw = entity.yaw.load();
        let pitch = entity.pitch.load();
        let yaw_rad = yaw.to_radians();
        let pitch_rad = pitch.to_radians();
        let look = Vector3::new(
            f64::from(-yaw_rad.sin() * pitch_rad.cos()),
            f64::from(-pitch_rad.sin()),
            f64::from(yaw_rad.cos() * pitch_rad.cos()),
        );
        // Vanilla formula: 3.0 * (1 + level) / 4
        //   L1 = 1.5, L2 = 2.25, L3 = 3.0
        let power = 3.0 * (1.0 + f64::from(level)) / 4.0;
        let push = look.multiply(power, power, power);
        entity.add_velocity(push);

        // Vanilla also nudges the player up by ~1.2 if they were on the
        // ground, so a Riptide jump actually clears the ground instead of
        // dragging along it.
        if entity.on_ground.load(Ordering::Relaxed) {
            let mut pos = entity.pos.load();
            pos.y += 1.1999999;
            entity.set_pos(pos);
        }

        // Start the spin attack. Damage = 8.0 (vanilla).
        player.living_entity.start_riptide(20, 8.0);
        player.update_player_pose().await;

        let sound = match level {
            1 => Sound::ItemTridentRiptide1,
            2 => Sound::ItemTridentRiptide2,
            _ => Sound::ItemTridentRiptide3,
        };
        let world = player.world();
        let sound_packet = CSoundEffect::new(
            IdOr::Id(sound as u16),
            SoundCategory::Players,
            &player.position(),
            1.0,
            1.0,
            0.0,
        );
        world.broadcast_packet_all(&sound_packet);
    }
}

/// True if the player can currently use Riptide: in water, OR exposed to rain.
async fn can_riptide(player: &Player) -> bool {
    let entity = &player.living_entity.entity;
    if entity.touching_water.load(Ordering::Relaxed) {
        return true;
    }
    let world = player.world();
    if !world.is_raining().await {
        return false;
    }
    // "Raining at" approximation: full sky exposure above the player.
    let pos = player.position();
    let block_pos = BlockPos::new(
        pos.x.floor() as i32,
        pos.y.floor() as i32,
        pos.z.floor() as i32,
    );
    world.get_sky_light_level(&block_pos) >= 15
}
