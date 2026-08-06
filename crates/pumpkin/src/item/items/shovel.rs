use std::pin::Pin;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::{BlockProperties, CampfireLikeProperties};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, tag};
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

pub struct ShovelItem;

impl ItemMetadata for ShovelItem {
    fn ids() -> Box<[u16]> {
        tag::Item::MINECRAFT_SHOVELS.1.into()
    }
}

impl ItemBehaviour for ShovelItem {
    fn use_on_block<'a>(
        &'a self,
        _item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            // Vanilla ShovelItem#useOn: `if (context.getClickedFace() == Direction.DOWN) return
            // InteractionResult.PASS;` guards the entire method, both flattening and campfire
            // dowsing -- shoveling the bottom face of a block never does anything.
            if face == BlockDirection::Down {
                return;
            }

            let world = player.world();
            // Yes, Minecraft does hardcode these
            let mut changed = if (block == &Block::GRASS_BLOCK
                || block == &Block::DIRT
                || block == &Block::COARSE_DIRT
                || block == &Block::ROOTED_DIRT
                || block == &Block::PODZOL
                || block == &Block::MYCELIUM)
                && world.get_block_state(&location.up()).is_air()
            {
                world
                    .set_block_state(
                        &location,
                        Block::DIRT_PATH.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
                // Vanilla `ShovelItem#useOn` uses `level.playSound(player, pos, ...)`, which plays
                // the sound at the block centre for every nearby player except the one acting.
                world.play_block_sound_expect(
                    player,
                    Sound::ItemShovelFlatten,
                    SoundCategory::Blocks,
                    location,
                );
                true
            } else {
                false
            };
            if block == &Block::CAMPFIRE || block == &Block::SOUL_CAMPFIRE {
                let mut campfire_props = CampfireLikeProperties::from_state_id(
                    world.get_block_state(&location).id,
                    block,
                );
                if campfire_props.lit {
                    world.sync_world_event(WorldEvent::SoundExtinguishFire, location, 0);

                    campfire_props.lit = false;
                    world
                        .set_block_state(
                            &location,
                            campfire_props.to_state_id(block),
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                    changed = true;
                }
            }

            if changed && player.gamemode.load() != GameMode::Creative {
                player.damage_held_item(1).await;
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
