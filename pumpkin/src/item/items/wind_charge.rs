use std::pin::Pin;
use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::player::Player;
use crate::entity::projectile::ThrownItemEntityCondition::Owned;
use crate::entity::projectile::wind_charge::WindChargeEntity;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::Sound;

pub struct WindChargeItem;

impl ItemMetadata for WindChargeItem {
    fn ids() -> Box<[u16]> {
        [Item::WIND_CHARGE.id].into()
    }
}

const POWER: f32 = 1.5;

impl ItemBehaviour for WindChargeItem {
    fn normal_use<'a>(
        &'a self,
        _block: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();
            let position = player.position();

            // TODO: Implement Cooldown to throw the item

            world
                .play_sound(
                    Sound::EntityWindChargeThrow,
                    pumpkin_data::sound::SoundCategory::Neutral,
                    &position,
                )
                .await;

            let entity = Entity::new(world.clone(), position, &EntityType::WIND_CHARGE);

            let wind_charge_entity =
                WindChargeEntity::new_normal(entity, &Owned(&player.living_entity.entity));

            let yaw = player.living_entity.entity.yaw.load();
            let pitch = player.living_entity.entity.pitch.load();

            wind_charge_entity
                .get_thrown_item_entity()
                .set_velocity_from(&player.living_entity.entity, pitch, yaw, 0.0, POWER, 1.0);

            // TODO: player.incrementStat(Stats.USED)

            // TODO: Implement that the projectile will explode on impact on ground
            world.spawn_entity(Arc::new(wind_charge_entity)).await;
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
