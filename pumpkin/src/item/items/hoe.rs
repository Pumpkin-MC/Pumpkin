use crate::entity::item::ItemEntity;
use crate::entity::player::Player;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::server::Server;
use crate::world::BlockFlags;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::tag::Tagable;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
use pumpkin_world::item::ItemStack;
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
        face: BlockDirection,
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
            let mut future_block = block;
            let world = player.world().await;

            //Only rooted can be right-clicked on the bottom of the block
            if face == BlockDirection::Down {
                if block == &Block::ROOTED_DIRT {
                    future_block = &Block::DIRT;
                }
            } else {
                // grass, dirt && dirt path become farmland
                if (block == &Block::GRASS_BLOCK
                    || block == &Block::DIRT_PATH
                    || block == &Block::DIRT)
                    && world
                        .get_block_state(&location.up())
                        .await
                        .unwrap()
                        .is_air()
                {
                    future_block = &Block::FARMLAND;
                }
                //Coarse dirt and rooted dirt become dirt
                else if block == &Block::COARSE_DIRT || block == &Block::ROOTED_DIRT {
                    future_block = &Block::DIRT;
                }
            }

            world
                .set_block_state(
                    &location,
                    future_block.default_state_id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;

            //Also rooted_dirt drop a hanging_root
            if block == &Block::ROOTED_DIRT {
                let location = match face {
                    BlockDirection::Up => location.up().to_f64(),
                    BlockDirection::Down => location.down().to_f64(),
                    BlockDirection::North => location.up().to_f64().add_raw(0.0, -0.4, -1.0),
                    BlockDirection::South => location.up().to_f64().add_raw(0.0, -0.4, 1.0),
                    BlockDirection::West => location.up().to_f64().add_raw(-1.0, -0.4, 0.0),
                    BlockDirection::East => location.up().to_f64().add_raw(1.0, -0.4, 0.0),
                };
                let entity = world.create_entity(location, EntityType::ITEM);
                // TODO: Merge stacks together
                let item_entity = Arc::new(
                    ItemEntity::new(entity, ItemStack::new(1, &Item::HANGING_ROOTS)).await,
                );
                world.spawn_entity(item_entity.clone()).await;
                item_entity.send_meta_packet().await;
            }
        }
    }
}
