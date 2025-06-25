use crate::entity::player::Player;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::{BlockProperties, CampfireLikeProperties};
use pumpkin_data::item::Item;
use pumpkin_data::tag::Tagable;
use pumpkin_data::world::WorldEvent;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

pub struct ShovelItem;

impl ItemMetadata for ShovelItem {
    fn ids() -> Box<[u16]> {
        Item::get_tag_values("#minecraft:shovels")
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
impl PumpkinItem for ShovelItem {
    async fn use_on_block(
        &self,
        _item: &Item,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        block: &Block,
        _server: &Server,
    ) {
        let world = player.world().await;
        // Yes, Minecraft does hardcode these
        if (block == &Block::GRASS_BLOCK
            || block == &Block::DIRT
            || block == &Block::COARSE_DIRT
            || block == &Block::ROOTED_DIRT
            || block == &Block::PODZOL
            || block == &Block::MYCELIUM)
            && face != BlockDirection::Down
            && world.get_block_state(&location.up()).await.is_air()
        {
            world
                .set_block_state(
                    &location,
                    Block::DIRT_PATH.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
        }
        if block == &Block::CAMPFIRE || block == &Block::SOUL_CAMPFIRE {
            let mut campfire_props = CampfireLikeProperties::from_state_id(
                world.get_block_state(&location).await.id,
                block,
            );
            if campfire_props.lit {
                world
                    .sync_world_event(WorldEvent::FireExtinguished, location, 0)
                    .await;

                campfire_props.lit = false;
                world
                    .set_block_state(
                        &location,
                        campfire_props.to_state_id(block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        }
    }
}
