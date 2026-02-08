use std::pin::Pin;
use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::item::ItemEntity;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::block_properties::{
    BeeNestLikeProperties, BlockProperties, HorizontalFacing, Integer0To5,
    WallTorchLikeProperties,
};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::item::ItemStack;
use pumpkin_world::world::BlockFlags;
use rand::{RngExt, rng};

pub struct ShearsItem;

impl ItemMetadata for ShearsItem {
    fn ids() -> Box<[u16]> {
        [Item::SHEARS.id].into()
    }
}

impl ItemBehaviour for ShearsItem {
    fn use_on_block<'a>(
        &'a self,
        item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();

            // Pumpkin → Carved Pumpkin + drop pumpkin seeds
            let carved = block == &Block::PUMPKIN;
            if carved {
                // Carved pumpkin faces toward the player (use the clicked face)
                let facing = match face {
                    BlockDirection::North => HorizontalFacing::North,
                    BlockDirection::South => HorizontalFacing::South,
                    BlockDirection::West => HorizontalFacing::West,
                    BlockDirection::East => HorizontalFacing::East,
                    // If clicked on top/bottom, use player's facing direction
                    _ => {
                        let (yaw, _) = player.rotation();
                        let yaw = yaw.rem_euclid(360.0);
                        if (315.0..360.0).contains(&yaw) || (0.0..45.0).contains(&yaw) {
                            HorizontalFacing::South
                        } else if (45.0..135.0).contains(&yaw) {
                            HorizontalFacing::West
                        } else if (135.0..225.0).contains(&yaw) {
                            HorizontalFacing::North
                        } else {
                            HorizontalFacing::East
                        }
                    }
                };

                // Find the carved pumpkin state with correct facing
                let carved = &Block::CARVED_PUMPKIN;
                let mut props = WallTorchLikeProperties::default(carved);
                props.facing = facing;
                let state_id = props.to_state_id(carved);

                world
                    .set_block_state(&location, state_id, BlockFlags::NOTIFY_ALL)
                    .await;

                // Drop 4 pumpkin seeds at the face location
                let drop_pos = match face {
                    BlockDirection::North => location.to_f64().add_raw(0.5, 0.5, -0.5),
                    BlockDirection::South => location.to_f64().add_raw(0.5, 0.5, 1.5),
                    BlockDirection::West => location.to_f64().add_raw(-0.5, 0.5, 0.5),
                    BlockDirection::East => location.to_f64().add_raw(1.5, 0.5, 0.5),
                    BlockDirection::Up => location.to_f64().add_raw(0.5, 1.0, 0.5),
                    BlockDirection::Down => location.to_f64().add_raw(0.5, 0.0, 0.5),
                };
                let entity = Entity::new(world.clone(), drop_pos, &EntityType::ITEM);
                let item_entity = Arc::new(
                    ItemEntity::new(entity, ItemStack::new(4, &Item::PUMPKIN_SEEDS)).await,
                );
                world.spawn_entity(item_entity).await;

                let seed = rng().random::<f64>();
                player
                    .play_sound(
                        Sound::BlockPumpkinCarve as u16,
                        SoundCategory::Blocks,
                        &location.to_f64(),
                        1.0,
                        1.0,
                        seed,
                    )
                    .await;
            }

            // Beehive / Bee Nest with honey_level 5 → harvest 3 honeycombs
            let harvested = if (block == &Block::BEEHIVE || block == &Block::BEE_NEST)
                && BeeNestLikeProperties::handles_block_id(block.id)
            {
                let state_id = world.get_block_state(&location).await.id;
                let mut props = BeeNestLikeProperties::from_state_id(state_id, block);
                if props.honey_level == Integer0To5::L5 {
                    // Reset honey level to 0
                    props.honey_level = Integer0To5::L0;
                    world
                        .set_block_state(
                            &location,
                            props.to_state_id(block),
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;

                    // Drop 3 honeycombs
                    let drop_pos = location.to_f64().add_raw(0.5, 1.0, 0.5);
                    let entity = Entity::new(world.clone(), drop_pos, &EntityType::ITEM);
                    let item_entity = Arc::new(
                        ItemEntity::new(entity, ItemStack::new(3, &Item::HONEYCOMB)).await,
                    );
                    world.spawn_entity(item_entity).await;

                    true
                } else {
                    false
                }
            } else {
                false
            };

            if (carved || harvested) && player.gamemode.load() != GameMode::Creative {
                item.damage_item_with_context(1, false);
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
