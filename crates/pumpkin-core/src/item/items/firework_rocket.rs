use crate::block::registry::BlockActionResult;
use std::sync::Arc;

use crate::entity::player::Player;
use crate::entity::projectile::firework_rocket::FireworkRocketEntity;
use crate::entity::{Entity, EntityBase};
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use super::vehicle_placement::{
    block_use_cooldown, commit_placement_item, finish_consumed_inventory,
    publish_placement_cooldown,
};

pub struct FireworkRocketItem;

impl ItemMetadata for FireworkRocketItem {
    fn ids() -> Box<[u16]> {
        [Item::FIREWORK_ROCKET.id].into()
    }
}

const ROCKET_PLACEMENT_OFFSET: f64 = 0.15;

impl FireworkRocketItem {
    fn rocket_at_block(
        stack: &ItemStack,
        world: &Arc<crate::world::World>,
        location: BlockPos,
        face: BlockDirection,
        cursor_pos: Vector3<f32>,
    ) -> Arc<dyn EntityBase> {
        let step = face.to_offset();
        let entity = Entity::new(
            world.clone(),
            Vector3::new(
                f64::from(location.0.x)
                    + f64::from(cursor_pos.x)
                    + f64::from(step.x) * ROCKET_PLACEMENT_OFFSET,
                f64::from(location.0.y)
                    + f64::from(cursor_pos.y)
                    + f64::from(step.y) * ROCKET_PLACEMENT_OFFSET,
                f64::from(location.0.z)
                    + f64::from(cursor_pos.z)
                    + f64::from(step.z) * ROCKET_PLACEMENT_OFFSET,
            ),
            &EntityType::FIREWORK_ROCKET,
        );
        Arc::new(FireworkRocketEntity::new_with_stack(
            entity,
            stack.copy_with_count(1),
        ))
    }

    /// Commits a block launch together with its physical inventory debit.
    pub(crate) fn use_from_slot(
        stack: &ItemStack,
        slot: usize,
        player: &Player,
        world: &Arc<crate::world::World>,
        location: BlockPos,
        hit: &crate::block::BlockHitResult<'_>,
        can_commit: impl FnOnce() -> bool,
    ) -> BlockActionResult {
        use pumpkin_util::GameMode;

        if stack.is_empty() {
            return BlockActionResult::Fail;
        }
        if player.get_entity().is_fall_flying() {
            return BlockActionResult::Pass;
        }
        let (group, duration) = block_use_cooldown(stack);
        if player.is_on_cooldown(group) {
            return BlockActionResult::Pass;
        }
        let inventory = player.inventory();
        let rocket = Self::rocket_at_block(stack, world, location, *hit.face, *hit.cursor_pos);
        let Ok((consumed, cooldown, mode)) = world.spawn_placed_vehicle(&rocket, || {
            let mode = player.gamemode.load();
            if mode == GameMode::Spectator || player.get_entity().is_fall_flying() || !can_commit()
            {
                return None;
            }
            let (consumed, cooldown) =
                commit_placement_item(player, slot, stack, mode, group, duration)?;
            Some((consumed, cooldown, mode))
        }) else {
            return BlockActionResult::Fail;
        };
        player.increment_stat(StatisticCategory::Used, i32::from(stack.get_item().id), 1);
        if !finish_consumed_inventory(player, slot, stack, &consumed, mode) {
            player.sync_hand_slot(slot, inventory.get_slot(slot));
        }
        if let Some(cooldown) = cooldown {
            publish_placement_cooldown(player, group, cooldown);
        }
        BlockActionResult::Success
    }
}

impl ItemBehaviour for FireworkRocketItem {
    fn use_on_block(
        &self,
        item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        cursor_pos: Vector3<f32>,
        _block: &Block,
        _server: &Server,
    ) -> BlockActionResult {
        if player.get_entity().is_fall_flying() {
            return BlockActionResult::Pass;
        }
        let world = player.world();
        world.spawn_entity(Self::rocket_at_block(
            item, &world, location, face, cursor_pos,
        ));
        item.decrement_unless_creative(player.gamemode.load(), 1);
        BlockActionResult::Success
    }

    fn normal_use(&self, item: &Item, player: &Player) {
        let (yaw, pitch) = player.rotation();
        self.normal_use_with_hand(item, player, yaw, pitch, pumpkin_util::Hand::Right);
    }

    fn normal_use_with_hand(
        &self,
        _item: &Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: pumpkin_util::Hand,
    ) {
        if player.get_entity().is_fall_flying() {
            let mut held = match hand {
                pumpkin_util::Hand::Right => player.inventory().held_item(),
                pumpkin_util::Hand::Left => player.inventory().off_hand_item(),
            };
            if held.is_empty() || held.item.id != Item::FIREWORK_ROCKET.id {
                return;
            }

            let world = player.world();
            if player.get_entity().drop_all_leash_connections() {
                world.play_sound_fine(
                    Sound::ItemLeadBreak,
                    SoundCategory::Neutral,
                    &player.position(),
                    1.0,
                    1.0,
                );
            }

            let entity = Entity::new(
                world.clone(),
                player.get_entity().pos.load(),
                &EntityType::FIREWORK_ROCKET,
            );
            let entity = FireworkRocketEntity::new_shot_with_stack(
                entity,
                player.get_entity(),
                held.copy_with_count(1),
            );
            world.spawn_entity(Arc::new(entity));

            let item_id = held.item.id;
            held.decrement_unless_creative(player.gamemode.load(), 1);
            match hand {
                pumpkin_util::Hand::Right => player.inventory().set_held_item(held),
                pumpkin_util::Hand::Left => player
                    .inventory()
                    .set_stack_in_hand(pumpkin_util::Hand::Left, held),
            }
            player.increment_stat(StatisticCategory::Used, i32::from(item_id), 1);
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
