use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::CampfireLikeProperties;
use pumpkin_data::block_transformer::SHOVEL;
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
        let get_block = |dx: i8, dy: i8, dz: i8| {
            let check_pos = BlockPos(location.0 + Vector3::new(dx as i32, dy as i32, dz as i32));
            world.get_block(&check_pos)
        };

        let mut changed = false;
        let mut damage = 1;

        if let Some(result) =
            SHOVEL.transform(block, world.get_block_state_id(&location), face, &get_block)
        {
            if let Some(sound) = result.entry.sound {
                world.play_sound(sound, SoundCategory::Blocks, &location.to_f64());
            }
            if let Some(particle) = result.entry.particle {
                world.sync_world_event(particle, location, 0);
            }

            world.set_block_state(&location, result.new_state_id, BlockFlags::NOTIFY_ALL);
            damage = result.entry.item_damage_per_use;
            changed = true;
        } else if block == &Block::CAMPFIRE || block == &Block::SOUL_CAMPFIRE {
            let mut campfire_props =
                CampfireLikeProperties::from_state_id(world.get_block_state(&location).id);
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
            let _ = item.damage_item(i32::from(damage));
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
