use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::block_properties::{DoubleBlockHalf, OakDoorLikeProperties};
use pumpkin_data::block_transformer::AXE;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::SoundCategory;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, tag};
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

pub struct AxeItem;

impl ItemMetadata for AxeItem {
    fn ids() -> Box<[u16]> {
        tag::Item::MINECRAFT_AXES.1.into()
    }
}

impl ItemBehaviour for AxeItem {
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

        let current_state_id = world.get_block_state_id(&location);

        if let Some(result) = AXE.transform(block, current_state_id, face, &get_block) {
            if let Some(sound) = result.entry.sound {
                world.play_sound(sound, SoundCategory::Blocks, &location.to_f64());
            }
            if let Some(particle) = result.entry.particle {
                world.sync_world_event(particle, location, 0);
            }

            if block.has_tag(&tag::Block::MINECRAFT_DOORS) {
                let door_props = OakDoorLikeProperties::from_state_id(current_state_id);
                let other_half_pos = match door_props.half {
                    DoubleBlockHalf::Lower => location.up(),
                    DoubleBlockHalf::Upper => location.down(),
                };
                let (other_block, other_state_id) = world.get_block_and_state_id(&other_half_pos);
                if other_block == block {
                    let other_new_state_id = if result.target_block.states.len() <= 1 {
                        result.target_block.default_state.id
                    } else if let Some(other_props) = other_block.properties(other_state_id) {
                        let props = other_props.to_props();
                        result
                            .target_block
                            .from_properties(&props)
                            .to_state_id(result.target_block)
                    } else {
                        result.target_block.default_state.id
                    };
                    world.set_block_state(
                        &other_half_pos,
                        other_new_state_id,
                        BlockFlags::NOTIFY_ALL,
                    );
                }
            }

            world.set_block_state(&location, result.new_state_id, BlockFlags::NOTIFY_ALL);

            if player.gamemode.load() != GameMode::Creative {
                // TODO: Handle DamageResult::Broken to broadcast item break and update player slot.
                let _ = item.damage_item(i32::from(result.entry.item_damage_per_use));
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
