use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::data_component_impl::CustomNameImpl;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;

pub struct NameTagItem;

impl ItemMetadata for NameTagItem {
    fn ids() -> Box<[u16]> {
        [Item::NAME_TAG.id].into()
    }
}

impl ItemBehaviour for NameTagItem {
    fn use_on_entity(&self, item: &mut ItemStack, player: &Player, target: Arc<dyn EntityBase>) {
        let entity = target.get_entity();
        // Vanilla NameTagItem.java: customName != null && target.getType().canSerialize() && target.isAlive() && !(target instanceof Player)
        if entity.entity_type.saveable
            && entity.is_alive()
            && target.get_player().is_none()
            && let Some(name) = item.get_data_component::<CustomNameImpl>()
        {
            entity.set_custom_name(name.name.clone());
            entity.set_custom_name_visible(true);

            if let Some(mob) = target.get_mob() {
                mob.get_mob_entity()
                    .persistence_required
                    .store(true, Ordering::Relaxed);
            }

            item.decrement_unless_creative(player.gamemode.load(), 1);
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
