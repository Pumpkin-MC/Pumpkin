use crate::block::registry::BlockActionResult;
use crate::entity::EntityBase;
use crate::entity::mob::equipment::equip_on_target;
use crate::entity::player::Player;
use crate::server::Server;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::data_component_impl::{BlocksAttacksImpl, ConsumableImpl};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::Hand;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rustc_hash::FxHashMap;
use std::sync::Arc;

use super::{ItemBehaviour, ItemMetadata};

pub(crate) const fn should_try_block_placement(result: &BlockActionResult) -> bool {
    matches!(result, BlockActionResult::Pass)
}

#[derive(Default)]
pub struct ItemRegistry {
    items: FxHashMap<u16, (Arc<dyn ItemBehaviour>, bool)>,
}

impl ItemRegistry {
    pub fn register<T: ItemBehaviour + ItemMetadata + 'static>(&mut self, item: T) {
        self.register_item(item, |_| false);
    }

    /// Registers an item whose Java class overrides `Item.use`.
    pub fn register_air_use<T: ItemBehaviour + ItemMetadata + 'static>(&mut self, item: T) {
        self.register_item(item, |_| true);
    }

    pub(crate) fn register_air_use_except<T: ItemBehaviour + ItemMetadata + 'static>(
        &mut self,
        item: T,
        inherited_use: u16,
    ) {
        self.register_item(item, |id| id != inherited_use);
    }

    fn register_item<T: ItemBehaviour + ItemMetadata + 'static>(
        &mut self,
        item: T,
        overrides_air_use: impl Fn(u16) -> bool,
    ) {
        let behaviour = Arc::new(item);
        let ids = T::ids();
        self.items.reserve(ids.len());
        for id in ids {
            self.items
                .insert(id, (behaviour.clone(), overrides_air_use(id)));
        }
    }

    pub(crate) fn overrides_air_use(&self, id: u16) -> bool {
        self.items.get(&id).is_some_and(|(_, overrides)| *overrides)
    }

    pub(crate) fn boat_item(&self, id: u16) -> Option<&crate::item::items::boat::BoatItem> {
        self.items.get(&id)?.0.as_any().downcast_ref()
    }

    pub(crate) fn minecart_item(
        &self,
        id: u16,
    ) -> Option<&crate::item::items::minecart::MinecartItem> {
        self.get_pumpkin_item(id)?.as_any().downcast_ref()
    }

    pub(crate) fn uses_boat_placement(&self, id: u16) -> bool {
        self.boat_item(id).is_some()
    }

    pub fn on_use(&self, stack: &ItemStack, player: &Player, hand: Hand) {
        let (yaw, pitch) = player.rotation();
        self.on_use_with_rotation(stack, player, yaw, pitch, hand);
    }

    pub fn on_use_with_rotation(
        &self,
        stack: &ItemStack,
        player: &Player,
        yaw: f32,
        pitch: f32,
        hand: Hand,
    ) {
        let item = stack.item;
        let registration = self.items.get(&item.id);
        if let Some(boat) = self.boat_item(item.id) {
            boat.place_from_hand(stack, player, yaw, pitch, hand);
            return;
        }
        if self.minecart_item(item.id).is_some() {
            if let Some(slot) = player
                .inventory()
                .equipment_slot_index(&crate::entity::hand_equipment_slot(hand))
            {
                crate::item::items::cart_air_use::use_cart_in_air(
                    player,
                    hand,
                    slot,
                    stack,
                    &player.world(),
                );
            }
            return;
        }
        let cooldown = stack.get_use_cooldown();
        let cooldown_group = cooldown
            .and_then(|c| c.cooldown_group.clone())
            .unwrap_or_else(|| item.registry_key.to_string());

        if player.is_on_cooldown(&cooldown_group) {
            return;
        }

        // Native use overrides take precedence over base item use.
        if !registration.is_some_and(|(_, overrides_air_use)| *overrides_air_use)
            && stack.get_data_component::<BlocksAttacksImpl>().is_some()
            && stack.get_data_component::<ConsumableImpl>().is_none()
        {
            let held = player.inventory().get_stack_in_hand(hand);
            if held.uid == stack.uid && held.are_equal(stack) {
                let duration = held.get_max_use_time();
                player
                    .living_entity
                    .start_blocking_if_idle(hand, held, duration);
            }
            return;
        }

        if let Some((pumpkin_item, _)) = registration {
            pumpkin_item.normal_use_with_hand(item, player, yaw, pitch, hand);
        } else if stack
            .get_data_component::<pumpkin_data::data_component_impl::KineticWeaponImpl>()
            .is_some()
        {
            crate::item::items::spear::SpearItem
                .normal_use_with_hand(item, player, yaw, pitch, hand);
        } else if stack
            .get_data_component::<pumpkin_data::data_component_impl::InstrumentImpl>()
            .is_some()
        {
            crate::item::items::goat_horn::GoatHornItem
                .normal_use_with_hand(item, player, yaw, pitch, hand);
        } else if stack
            .get_data_component::<pumpkin_data::data_component_impl::RecipesImpl>()
            .is_some()
        {
            crate::item::items::knowledge_book::KnowledgeBookItem
                .normal_use_with_hand(item, player, yaw, pitch, hand);
        }

        if let Some(cooldown) = cooldown
            && self.get_use_duration(item.id).is_none()
            && stack.get_max_use_time() == 0
        {
            player.start_cooldown(cooldown_group, (cooldown.seconds * 20.0) as i32);
        }
    }

    pub fn on_stopped_using(&self, stack: &ItemStack, player: &Player) {
        if let Some(behaviour) = self.get_pumpkin_item(stack.item.id) {
            behaviour.on_stopped_using(stack, player);
        }
    }

    pub(crate) fn release_item(&self, player: &Player) {
        player.living_entity.release_active_hand(
            |hand| player.inventory().get_stack_in_hand(hand),
            |held| self.on_stopped_using(held, player),
        );
    }

    pub fn on_spear_jab(&self, stack: &ItemStack, player: &Player) {
        if let Some(behaviour) = self.get_pumpkin_item(stack.item.id) {
            behaviour.on_spear_jab(stack, player);
        } else if stack
            .get_data_component::<pumpkin_data::data_component_impl::PiercingWeaponImpl>()
            .is_some()
        {
            crate::item::items::spear::SpearItem.on_spear_jab(stack, player);
        }
    }

    pub fn on_use_tick(
        &self,
        stack: &ItemStack,
        user: &dyn crate::entity::EntityBase,
        remaining_use_ticks: i32,
    ) {
        // Guarded cart ticks are driven by the living entity that owns the source slot.
        if self.minecart_item(stack.item.id).is_some() {
            return;
        }
        if let Some(behaviour) = self.get_pumpkin_item(stack.item.id) {
            behaviour.on_use_tick(stack, user, remaining_use_ticks);
        } else if stack
            .get_data_component::<pumpkin_data::data_component_impl::KineticWeaponImpl>()
            .is_some()
        {
            crate::item::items::spear::SpearItem.on_use_tick(stack, user, remaining_use_ticks);
        }
    }

    /// Returns the item's use duration in ticks, as defined by its registered behaviour.
    /// Returns `None` if the item has no registered behaviour or its duration is 0.
    #[must_use]
    pub fn get_use_duration(&self, item_id: u16) -> Option<i32> {
        self.get_pumpkin_item(item_id)
            .map(|b| b.get_use_duration())
            .filter(|&d| d > 0)
    }

    #[expect(clippy::too_many_arguments)]
    pub fn use_on_block(
        &self,
        stack: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        cursor_pos: Vector3<f32>,
        block: &Block,
        server: &Server,
    ) -> BlockActionResult {
        if let Some(cart) = self.minecart_item(stack.item.id) {
            return cart.use_on_block(stack, player, location, face, cursor_pos, block, server);
        }

        let cooldown = stack.get_use_cooldown().cloned();
        let cooldown_group = cooldown
            .as_ref()
            .and_then(|c| c.cooldown_group.clone())
            .unwrap_or_else(|| stack.item.registry_key.to_string());

        if player.is_on_cooldown(&cooldown_group) {
            return BlockActionResult::Pass;
        }

        let pumpkin_item = self.get_pumpkin_item(stack.item.id);
        let result = if let Some(pumpkin_item) = pumpkin_item {
            pumpkin_item.use_on_block(stack, player, location, face, cursor_pos, block, server)
        } else if let Some(bt) = stack
            .get_data_component::<pumpkin_data::data_component_impl::BlockTransformerImpl>()
            .map(|b| b.transformer.clone())
        {
            match bt.strip_prefix("minecraft:").unwrap_or(&bt) {
                "axe" => crate::item::items::axe::AxeItem
                    .use_on_block(stack, player, location, face, cursor_pos, block, server),
                "hoe" => crate::item::items::hoe::HoeItem
                    .use_on_block(stack, player, location, face, cursor_pos, block, server),
                "shovel" => crate::item::items::shovel::ShovelItem
                    .use_on_block(stack, player, location, face, cursor_pos, block, server),
                _ => BlockActionResult::Pass,
            }
        } else {
            BlockActionResult::Pass
        };

        if matches!(
            result,
            BlockActionResult::Success | BlockActionResult::SuccessServer
        ) && let Some(cooldown) = cooldown
        {
            player.start_cooldown(cooldown_group, (cooldown.seconds * 20.0) as i32);
        }

        result
    }

    pub fn use_on_entity(
        &self,
        stack: &mut ItemStack,
        player: &Player,
        entity: Arc<dyn EntityBase>,
    ) {
        if let Some(mob) = entity.get_mob()
            && equip_on_target(mob, player, stack)
        {
            return;
        }

        let pumpkin_item = self.get_pumpkin_item(stack.item.id);
        if let Some(pumpkin_item) = pumpkin_item {
            pumpkin_item.use_on_entity(stack, player, entity);
        }
    }

    pub fn can_mine(&self, item: &Item, player: &Player) -> bool {
        let pumpkin_block = self.get_pumpkin_item(item.id);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block.can_mine(player);
        }
        true
    }

    #[must_use]
    pub fn get_pumpkin_item(&self, item: u16) -> Option<&Arc<dyn ItemBehaviour>> {
        self.items.get(&item).map(|(behaviour, _)| behaviour)
    }
}

#[cfg(test)]
mod tests {
    use super::should_try_block_placement;
    use crate::block::registry::BlockActionResult;

    #[test]
    fn block_placement_only_follows_pass() {
        assert!(should_try_block_placement(&BlockActionResult::Pass));

        for result in [
            BlockActionResult::Success,
            BlockActionResult::SuccessServer,
            BlockActionResult::Consume,
            BlockActionResult::Fail,
            BlockActionResult::PassToDefaultBlockAction,
        ] {
            assert!(!should_try_block_placement(&result));
        }
    }
}
