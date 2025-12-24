use crate::entity::Entity;
use crate::entity::item::ItemEntity;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::BlockDirection;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::{Block, tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::item::ItemStack;
use pumpkin_world::world::BlockFlags;
use std::pin::Pin;
use std::sync::Arc;
use uuid::Uuid;

pub struct HoeItem;

impl ItemMetadata for HoeItem {
    fn ids() -> Box<[u16]> {
        tag::Item::MINECRAFT_HOES.1.to_vec().into_boxed_slice()
    }
}

impl ItemBehaviour for HoeItem {
    fn use_on_block<'a>(
        &'a self,
        _item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        face: BlockDirection,
        block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            // Yes, Minecraft does hardcode these
            if block == &Block::GRASS_BLOCK
                || block == &Block::DIRT_PATH
                || block == &Block::DIRT
                || block == &Block::COARSE_DIRT
                || block == &Block::ROOTED_DIRT
            {
                let mut future_block = block;
                let world = player.world();

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
                        && world.get_block_state(&location.up()).await.is_air()
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
                        future_block.default_state.id,
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
                    let entity = Entity::new(
                        Uuid::new_v4(),
                        world.clone(),
                        location,
                        &EntityType::ITEM,
                        false,
                    );
                    // TODO: Merge stacks together
                    let item_entity = Arc::new(
                        ItemEntity::new(entity, ItemStack::new(1, &Item::HANGING_ROOTS)).await,
                    );
                    world.spawn_entity(item_entity).await;
                }
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
