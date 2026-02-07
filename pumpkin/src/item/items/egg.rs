use std::pin::Pin;
use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::player::Player;
use crate::entity::projectile::egg::EggEntity;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::Sound;
use pumpkin_world::item::ItemStack;

pub struct EggItem;

impl ItemMetadata for EggItem {
    fn ids() -> Box<[u16]> {
        [Item::EGG.id, Item::BLUE_EGG.id, Item::BROWN_EGG.id].into()
    }
}

const POWER: f32 = 1.5;

impl ItemBehaviour for EggItem {
    fn normal_use<'a>(
        &'a self,
        _block: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let position = player.position();
            let world = player.world();
            world
                .play_sound(
                    Sound::EntityEggThrow,
                    pumpkin_data::sound::SoundCategory::Players,
                    &position,
                )
                .await;

            // Capture the held item stack and pass it to the thrown egg entity
            let item_stack: ItemStack = player.inventory.held_item().lock().await.clone();

            let entity = Entity::new(world.clone(), position, &EntityType::EGG);
            let egg = EggEntity::new_shot(entity, &player.living_entity.entity).await;

            // Propagate the item stack so clients show correct variant
            egg.set_item_stack(item_stack.clone()).await;

            let yaw = player.living_entity.entity.yaw.load();
            let pitch = player.living_entity.entity.pitch.load();
            egg.thrown
                .set_velocity_from(&player.living_entity.entity, pitch, yaw, 0.0, POWER, 1.0);
            world.spawn_entity(Arc::new(egg)).await;
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
