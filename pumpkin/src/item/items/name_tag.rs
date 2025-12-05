use std::pin::Pin;
use std::sync::Arc;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::data_component_impl::CustomNameImpl;
use pumpkin_data::item::Item;
use pumpkin_util::text::TextComponent;
use pumpkin_world::item::ItemStack;

pub struct NameTagItem;

impl ItemMetadata for NameTagItem {
    fn ids() -> Box<[u16]> {
        [Item::NAME_TAG.id].into()
    }
}

impl ItemBehaviour for NameTagItem {
    fn use_on_entity<'a>(
        &'a self,
        item: &'a mut ItemStack,
        player: &'a Player,
        entity: Arc<dyn EntityBase>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let entity = entity.get_entity();
            if entity.entity_type.saveable
                && let Some(name) = item.get_data_component::<CustomNameImpl>()
            {
                // TODO
                entity.set_custom_name(TextComponent::text(name.name)).await;
                item.decrement_unless_creative(player.gamemode.load(), 1);
            }
        })
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
