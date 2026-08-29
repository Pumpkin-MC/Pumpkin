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
    fn use_on_block(
        &self,
        item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &Block,
        _server: &Server,
    ) {
        let world = player.world();
        // Yes, Minecraft does hardcode these
        let mut changed = if (block == &Block::GRASS_BLOCK
            || block == &Block::DIRT
            || block == &Block::COARSE_DIRT
            || block == &Block::ROOTED_DIRT
            || block == &Block::PODZOL
            || block == &Block::MYCELIUM)
            && face != BlockDirection::Down
            && world.get_block_state(&location.up()).is_air()
        {
            world.set_block_state(
                &location,
                Block::DIRT_PATH.default_state.id,
                BlockFlags::NOTIFY_ALL,
            );
            true
        } else {
            false
        };
        if block == &Block::CAMPFIRE || block == &Block::SOUL_CAMPFIRE {
            let mut campfire_props =
                CampfireLikeProperties::from_state_id(world.get_block_state(&location).id, block);
            if campfire_props.lit {
                world.sync_world_event(WorldEvent::SoundExtinguishFire, location, 0);

                campfire_props.lit = false;
                world.set_block_state(
                    &location,
                    campfire_props.to_state_id(block),
                    BlockFlags::NOTIFY_ALL,
                );
                world.play_sound_fine(
                    Sound::BlockFireExtinguish,
                    SoundCategory::Ambient,
                    &location.to_f64(),
                    0.5,
                    2.0,
                );
                changed = true;
            }
        }

        if changed && player.gamemode.load() != GameMode::Creative {
            // TODO: Handle DamageResult::Broken to broadcast item break and update player slot.
            let _ = item.damage_item(1);
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
