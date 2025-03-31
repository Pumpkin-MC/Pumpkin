use crate::entity::item::ItemEntity;
use crate::entity::player::Player;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::server::Server;
use crate::world::BlockFlags;
use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::tag::Tagable;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
use std::sync::Arc;

pub struct HoeItem;

impl ItemMetadata for HoeItem {
    fn ids() -> Box<[u16]> {
        Item::get_tag_values("#minecraft:hoes")
            .expect("This is a valid vanilla tag")
            .iter()
            .map(|key| {
                Item::from_registry_key(key)
                    .expect("We just got this key from the registry")
                    .id
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}

#[async_trait]
impl PumpkinItem for HoeItem {
    async fn use_on_block(
        &self,
        _item: &Item,
        player: &Player,
        location: BlockPos,
        face: &BlockDirection,
        block: &Block,
        _server: &Server,
    ) {
        // Yes, Minecraft does hardcode these
        if block == &Block::GRASS_BLOCK
            || block == &Block::DIRT_PATH
            || block == &Block::DIRT
            || block == &Block::COARSE_DIRT
            || block == &Block::ROOTED_DIRT
        {
            let mut future_block = block.clone();
            let world = player.world().await;

            //For every block except rooted
            if face != &BlockDirection::Down {
                // grass, dirt && dirt path become farmland
                if (block == &Block::GRASS_BLOCK
                    || block == &Block::DIRT_PATH
                    || block == &Block::DIRT)
                    && world.get_block_state(&location.up()).await.unwrap().air
                {
                    future_block = Block::FARMLAND
                }
                //Coarse dirt become dirt
                else if block == &Block::COARSE_DIRT || block == &Block::ROOTED_DIRT {
                    future_block = Block::DIRT
                }
            //Only rooted can be right-clicked on the bottom of the block
            } else {
                if block == &Block::ROOTED_DIRT {
                    future_block = Block::DIRT
                }
            }

            //Rooted dirt become dirt but can be right-clicked even from the bottom of the block
            world
                .set_block_state(
                    &location,
                    future_block.default_state_id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;

            //Also rooted_dirt drop a hanging_root when you use a hoe.
            if block == &Block::ROOTED_DIRT {
                let entity;

                match face {
                    BlockDirection::Up => {
                        entity = world.create_entity(location.up().to_f64(), EntityType::ITEM);
                    }
                    BlockDirection::Down => {
                        entity = world.create_entity(location.down().to_f64(), EntityType::ITEM);
                    }
                    BlockDirection::North => {
                        entity = world.create_entity(
                            location.up().to_f64().add_raw(0.0, -0.4, -1.0),
                            EntityType::ITEM,
                        );
                    }
                    BlockDirection::South => {
                        entity = world.create_entity(
                            location.up().to_f64().add_raw(0.0, -0.4, 1.0),
                            EntityType::ITEM,
                        );
                    }
                    BlockDirection::West => {
                        entity = world.create_entity(
                            location.up().to_f64().add_raw(-1.0, -0.4, 0.0),
                            EntityType::ITEM,
                        );
                    }
                    BlockDirection::East => {
                        entity = world.create_entity(
                            location.up().to_f64().add_raw(1.0, -0.4, 0.0),
                            EntityType::ITEM,
                        );
                    }
                }

                // TODO: Merge stacks together
                let item_entity =
                    Arc::new(ItemEntity::new(entity, Block::HANGING_ROOTS.item_id, 1).await);
                world.spawn_entity(item_entity.clone()).await;
                item_entity.send_meta_packet().await;
            }
        }
    }
}
