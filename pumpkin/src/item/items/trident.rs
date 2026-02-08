use std::pin::Pin;
use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::{GameMode, Hand};

use crate::{
    entity::Entity,
    entity::player::Player,
    entity::projectile::trident::ThrownTridentEntity,
    item::{ItemBehaviour, ItemMetadata},
};
use pumpkin_world::item::ItemStack;

pub struct TridentItem;

impl ItemMetadata for TridentItem {
    fn ids() -> Box<[u16]> {
        [Item::TRIDENT.id].into()
    }
}

impl ItemBehaviour for TridentItem {
    fn release_use<'a>(
        &'a self,
        item: &'a mut ItemStack,
        player: &'a Player,
        used_ticks: i32,
        _hand: Hand,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        Box::pin(async move {
            const MIN_THROW_TICKS: i32 = 10;
            const POWER: f32 = 2.5;

            if used_ticks < MIN_THROW_TICKS
                || item
                    .get_max_damage()
                    .is_some_and(|max_damage| item.get_damage() + 1 >= max_damage)
            {
                return false;
            }

            let world = player.world();
            let position = player.position();
            let is_creative = player.gamemode.load() == GameMode::Creative;

            let mut projectile_stack = item.copy_with_count(1);
            if !is_creative {
                projectile_stack.damage_item_with_context(1, false);
                item.decrement(1);
            }

            world
                .play_sound(Sound::ItemTridentThrow, SoundCategory::Players, &position)
                .await;

            let entity = Entity::new(world.clone(), position, &EntityType::TRIDENT);
            let trident = ThrownTridentEntity::new_shot(
                entity,
                &player.living_entity.entity,
                projectile_stack,
                !is_creative,
            );

            let yaw = player.living_entity.entity.yaw.load();
            let pitch = player.living_entity.entity.pitch.load();
            trident.thrown.set_velocity_from(
                &player.living_entity.entity,
                pitch,
                yaw,
                0.0,
                POWER,
                1.0,
            );

            world.spawn_entity(Arc::new(trident)).await;
            true
        })
    }

    fn can_mine(&self, player: &Player) -> bool {
        player.gamemode.load() != GameMode::Creative
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
