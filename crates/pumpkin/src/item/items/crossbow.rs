use std::any::Any;
use std::sync::atomic::Ordering;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::items::projectile_weapon::ProjectileWeaponItem;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::data_component_impl::ChargedProjectilesImpl;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_util::{GameMode, Hand};

pub struct CrossbowItem;

impl ItemMetadata for CrossbowItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::CROSSBOW.id])
    }
}

impl ItemBehaviour for CrossbowItem {
    fn normal_use_in_hand(
        &self,
        _item: &Item,
        player: &Player,
        hand: Hand,
        _yaw: f32,
        _pitch: f32,
    ) {
        let inventory = player.inventory();
        let stack = inventory.get_stack_in_hand(hand);

        // Every crossbow carries a ChargedProjectiles component by default, so its mere
        // presence does not mean the crossbow is loaded. Vanilla checks the list is also
        // non-empty (CrossbowItem.java:68).
        if stack
            .get_data_component::<ChargedProjectilesImpl>()
            .is_some_and(|charged| !charged.projectiles.is_empty())
        {
            Self::fire_projectiles(player, hand);
            return;
        }

        let has_arrows = Self::find_ammo(player).is_some();
        if !has_arrows && player.gamemode.load() != GameMode::Creative {
            return;
        }

        player
            .living_entity
            .set_active_hand(hand, stack, Self::USE_DURATION);
    }

    fn on_stopped_using(&self, stack: &ItemStack, player: &Player) {
        Self::load_projectiles(
            player,
            stack,
            player.living_entity.item_use_time.load(Ordering::Relaxed),
        );
        player.living_entity.clear_active_hand();
    }

    fn on_use_tick(&self, stack: &ItemStack, player: &Player, remaining_use_ticks: i32) {
        Self::load_projectiles(player, stack, remaining_use_ticks);
    }

    fn get_use_duration(&self) -> i32 {
        Self::USE_DURATION
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CrossbowItem {
    pub const USE_DURATION: i32 = 72000;

    pub const ARROW_POWER: f32 = 3.15;

    pub const FIREWORK_POWER: f32 = 1.6;

    fn load_projectiles(player: &Player, active_stack: &ItemStack, remaining_use_ticks: i32) {
        let use_ticks = Self::USE_DURATION - remaining_use_ticks;

        let hand = player
            .living_entity
            .active_hand
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .unwrap_or(Hand::Right);
        let mut stack = player.inventory().get_stack_in_hand(hand);
        if stack
            .get_data_component::<ChargedProjectilesImpl>()
            .is_some_and(|charged| !charged.projectiles.is_empty())
        {
            return;
        }
        if stack.item.id != Item::CROSSBOW.id || !stack.are_items_and_components_equal(active_stack)
        {
            player.living_entity.clear_active_hand();
            return;
        }
        let charge_time =
            crate::enchantment::EnchantmentHelper::modify_crossbow_charge_time(&stack, 25);

        if use_ticks >= charge_time {
            let arrow_slot = Self::find_ammo(player);
            let gamemode = player.gamemode.load();
            let is_creative = gamemode == GameMode::Creative;

            if arrow_slot.is_some() || is_creative {
                let projectile = arrow_slot.map_or_else(
                    || ItemStack::new(1, &Item::ARROW),
                    |slot| {
                        let inventory = player.inventory();
                        inventory.get_slot(slot)
                    },
                );

                let mut drawn = ProjectileWeaponItem::draw(&stack, &projectile, is_creative);
                let ammo_used = if is_creative {
                    0
                } else {
                    crate::enchantment::EnchantmentHelper::process_ammo_use(&stack, &projectile, 1)
                        .max(0)
                };
                let ammo_used = if ammo_used <= i32::from(projectile.item_count) {
                    ammo_used
                } else {
                    0
                };
                if ammo_used > 0
                    && let Some(first) = drawn.first_mut()
                {
                    first.item_count = ammo_used as u8;
                }
                if !drawn.is_empty() {
                    let mut charged_nbts = Vec::new();
                    for item in drawn {
                        let mut arrow_nbt = pumpkin_nbt::compound::NbtCompound::new();
                        item.write_item_stack(&mut arrow_nbt);
                        charged_nbts.push(arrow_nbt);
                    }

                    stack.set_data_component(ChargedProjectilesImpl {
                        projectiles: charged_nbts,
                    });
                    player.inventory().set_stack_in_hand(hand, stack.clone());

                    if let Some(slot) = arrow_slot
                        && !is_creative
                    {
                        let mut remaining = player.inventory().get_slot(slot);
                        remaining.decrement(ammo_used as u8);
                        player.inventory().set_slot(slot, remaining);
                    }

                    player.world().play_sound(
                        Sound::ItemCrossbowLoadingEnd,
                        SoundCategory::Players,
                        &player.position(),
                    );
                }
            }
        }
    }

    fn find_ammo(player: &Player) -> Option<usize> {
        let inventory = player.inventory();
        for slot in [
            PlayerInventory::OFF_HAND_SLOT,
            usize::from(inventory.get_selected_slot()),
        ] {
            let stack = inventory.get_slot(slot);
            if !stack.is_empty() && ProjectileWeaponItem::is_arrow_or_firework(stack.item) {
                return Some(slot);
            }
        }
        player.find_arrow()
    }

    fn fire_projectiles(player: &Player, hand: Hand) {
        let mut held = player.inventory().get_stack_in_hand(hand);
        let charged_opt = held.get_data_component::<ChargedProjectilesImpl>().cloned();

        if let Some(charged) = charged_opt {
            let mut projectiles = Vec::new();
            for projectile_nbt in charged.projectiles {
                if let Some(projectile) = ItemStack::read_item_stack(&projectile_nbt) {
                    projectiles.push(projectile);
                }
            }

            if !projectiles.is_empty() {
                held.set_data_component(ChargedProjectilesImpl {
                    projectiles: Vec::new(),
                });
                player.inventory().set_stack_in_hand(hand, held.clone());
                if projectiles
                    .iter()
                    .any(|item| item.item.id == Item::FIREWORK_ROCKET.id)
                {
                    Self::fire_rockets(player, hand, &held, &projectiles);
                } else {
                    let world = player.world();
                    world.play_sound(
                        Sound::ItemCrossbowShoot,
                        SoundCategory::Players,
                        &player.position(),
                    );
                    ProjectileWeaponItem::shoot_projectiles(
                        &world,
                        player.get_entity(),
                        &held,
                        &projectiles,
                        Self::ARROW_POWER,
                        1.0,
                        false,
                        player.gamemode.load() == GameMode::Creative,
                    );
                    player.damage_item_in_slot(&Self::equipment_slot(hand), 1);
                }
            }
        }
    }

    fn equipment_slot(hand: Hand) -> EquipmentSlot {
        if hand == Hand::Right {
            EquipmentSlot::MAIN_HAND
        } else {
            EquipmentSlot::OFF_HAND
        }
    }

    fn fire_rockets(player: &Player, hand: Hand, weapon: &ItemStack, projectiles: &[ItemStack]) {
        let world = player.world();
        let shooter = player.get_entity();
        let spread = crate::enchantment::EnchantmentHelper::process_projectile_spread(weapon, 0.0);
        let step = if projectiles.len() == 1 {
            0.0
        } else {
            2.0 * spread / (projectiles.len() as f32 - 1.0)
        };
        let offset = ((projectiles.len() - 1) % 2) as f32 * step / 2.0;
        let mut direction = 1.0;
        for (index, projectile) in projectiles.iter().enumerate() {
            if projectile.is_empty() {
                continue;
            }
            let angle = offset + direction * index.div_ceil(2) as f32 * step;
            direction = -direction;
            let rocket_item = projectile.item.id == Item::FIREWORK_ROCKET.id;
            if rocket_item {
                let mut position = shooter.get_eye_pos();
                position.y -= f64::from(0.15f32);
                let rocket =
                    crate::entity::projectile::firework_rocket::FireworkRocketEntity::new_crossbow(
                        crate::entity::Entity::new(
                            world.clone(),
                            position,
                            &pumpkin_data::entity::EntityType::FIREWORK_ROCKET,
                        ),
                        shooter,
                        projectile.clone(),
                    );
                rocket.shoot(
                    shooter.pitch.load(),
                    shooter.yaw.load(),
                    angle,
                    Self::FIREWORK_POWER,
                    1.0,
                );
                rocket.spawn(&world);
            } else {
                let arrow = ProjectileWeaponItem::create_projectile(
                    world.clone(),
                    shooter,
                    weapon,
                    projectile,
                    true,
                    player.gamemode.load() == GameMode::Creative,
                );
                arrow.set_velocity_from_rotation(
                    shooter.pitch.load(),
                    shooter.yaw.load() + angle,
                    0.0,
                    Self::FIREWORK_POWER,
                    1.0,
                );
                world.spawn_entity(std::sync::Arc::new(arrow));
            }
            let pitch = if index == 0 {
                1.0
            } else {
                1.0 / (rand::random::<f32>() * 0.5 + 1.8) + if index % 2 == 1 { 0.63 } else { 0.43 }
            };
            world.play_sound_fine(
                Sound::ItemCrossbowShoot,
                SoundCategory::Players,
                &player.position(),
                1.0,
                pitch,
            );
            player
                .damage_item_in_slot(&Self::equipment_slot(hand), if rocket_item { 3 } else { 1 });
            if player.inventory().get_stack_in_hand(hand).is_empty() {
                break;
            }
        }
    }
}
