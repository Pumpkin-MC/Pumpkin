use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::data_component_impl::CustomNameImpl;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::GameMode;

pub struct NameTagItem;

impl NameTagItem {
    /// Mirrors `NameTagItem.interactLivingEntity`. Returns whether the tag named the target.
    pub fn interact_living_entity(
        item: &mut ItemStack,
        gamemode: GameMode,
        target: &dyn EntityBase,
    ) -> bool {
        let Some(living) = target.get_living_entity() else {
            return false;
        };
        let Some(name) = item.get_data_component::<CustomNameImpl>() else {
            return false;
        };
        let entity = target.get_entity();
        if !entity.entity_type.saveable || !living.entity.is_alive() {
            return false;
        }

        entity.set_custom_name(name.name.clone());
        if let Some(mob) = target.get_mob() {
            mob.get_mob_entity()
                .persistence_required
                .store(true, Ordering::Relaxed);
        }
        item.decrement_unless_creative(gamemode, 1);
        true
    }
}

impl ItemMetadata for NameTagItem {
    fn ids() -> Box<[u16]> {
        [Item::NAME_TAG.id].into()
    }
}

impl ItemBehaviour for NameTagItem {
    fn use_on_entity(&self, item: &mut ItemStack, player: &Player, entity: Arc<dyn EntityBase>) {
        Self::interact_living_entity(item, player.gamemode.load(), entity.as_ref());
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use pumpkin_data::data_component_impl::CustomNameImpl;
    use pumpkin_data::entity::EntityType;
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;
    use pumpkin_util::GameMode;
    use pumpkin_util::math::vector3::Vector3;
    use pumpkin_util::text::TextComponent;

    use super::NameTagItem;
    use crate::entity::{
        Entity, EntityBase, ai::test_support::world, mob::Mob, passive::cow::CowEntity,
    };

    fn named_tag(count: u8, name: &str) -> ItemStack {
        let mut tag = ItemStack::new(count, &Item::NAME_TAG);
        tag.set_data_component(CustomNameImpl {
            name: TextComponent::text(name.to_string()),
        });
        tag
    }

    #[tokio::test]
    async fn names_the_target_and_keeps_it_loaded() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let cow = CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::COW,
        ));
        let mut tag = named_tag(2, "Bessie");

        assert!(NameTagItem::interact_living_entity(
            &mut tag,
            GameMode::Survival,
            cow.as_ref()
        ));
        assert_eq!(
            **cow.get_entity().custom_name.load(),
            Some(TextComponent::text("Bessie".to_string()))
        );
        assert!(
            cow.get_mob_entity()
                .persistence_required
                .load(Ordering::Relaxed)
        );
        assert_eq!(tag.item_count, 1);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn creative_keeps_the_tag() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let cow = CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::COW,
        ));
        let mut tag = named_tag(1, "Bessie");

        assert!(NameTagItem::interact_living_entity(
            &mut tag,
            GameMode::Creative,
            cow.as_ref()
        ));
        assert_eq!(tag.item_count, 1);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn ignores_unnamed_tags() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let cow = CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::COW,
        ));

        let mut unnamed = ItemStack::new(1, &Item::NAME_TAG);
        assert!(!NameTagItem::interact_living_entity(
            &mut unnamed,
            GameMode::Survival,
            cow.as_ref()
        ));
        assert!(cow.get_entity().custom_name.load().is_none());

        world.level.shutdown().await;
        Ok(())
    }
}
