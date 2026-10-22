use std::any::Any;

use crate::block::registry::BlockActionResult;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::data_component_impl::DebugStickStateImpl;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{Block, BlockDirection, BlockStateId, PropertyKey};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::world::BlockFlags;

pub struct DebugStickItem;

impl ItemMetadata for DebugStickItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::DEBUG_STICK.id])
    }
}

impl DebugStickItem {
    #[expect(clippy::too_many_lines)]
    fn handle_interaction(
        item: &mut ItemStack,
        player: &Player,
        pos: &BlockPos,
        block: &Block,
        state_id: BlockStateId,
        cycle: bool,
    ) -> bool {
        let Some(props) = block.properties(state_id) else {
            player.send_system_message_raw(
                &TextComponent::translate(
                    "item.minecraft.debug_stick.empty",
                    [TextComponent::text(block.name)],
                ),
                true,
            );
            return false;
        };

        let prop_list = props.to_props();
        if prop_list.is_empty() {
            player.send_system_message_raw(
                &TextComponent::translate(
                    "item.minecraft.debug_stick.empty",
                    [TextComponent::text(block.name)],
                ),
                true,
            );
            return false;
        }

        let prop_names: Vec<PropertyKey> = prop_list.iter().map(|p| p.key()).collect();
        let block_key = format!("minecraft:{}", block.name);
        let mut stick_state = item
            .get_data_component::<DebugStickStateImpl>()
            .cloned()
            .unwrap_or_default();
        let selected_prop = stick_state
            .properties
            .iter()
            .find(|(k, _)| k == &block_key || k == block.name)
            .and_then(|(_, v)| prop_names.iter().find(|p| p.to_str() == v).copied())
            .unwrap_or(prop_names[0]);

        if cycle {
            let mut possible_values: Vec<pumpkin_data::Property> = Vec::new();
            for state in block.states {
                if let Some(state_props) = block.properties(state.id) {
                    for p in state_props.to_props() {
                        if p.key() == selected_prop && !possible_values.contains(&p) {
                            possible_values.push(p);
                        }
                    }
                }
            }

            if possible_values.is_empty() {
                return false;
            }

            let current_val = prop_list
                .iter()
                .find(|p| p.key() == selected_prop)
                .copied()
                .unwrap_or(possible_values[0]);
            let cur_idx = possible_values
                .iter()
                .position(|v| *v == current_val)
                .unwrap_or(0);
            let is_backward = player.get_entity().is_sneaking();
            let new_idx = if is_backward {
                (cur_idx + possible_values.len() - 1) % possible_values.len()
            } else {
                (cur_idx + 1) % possible_values.len()
            };
            let new_val = possible_values[new_idx];

            let mut new_props = prop_list;
            for p in &mut new_props {
                if p.key() == selected_prop {
                    *p = new_val;
                }
            }

            let new_state_id = block.from_properties(&new_props).to_state_id(block);
            let world = player.world();
            world.set_block_state(pos, new_state_id, BlockFlags::NOTIFY_ALL);

            player.send_system_message_raw(
                &TextComponent::translate(
                    "item.minecraft.debug_stick.update",
                    [
                        TextComponent::text(selected_prop.to_str()),
                        TextComponent::text(new_val.value_str()),
                    ],
                ),
                true,
            );
        } else {
            let cur_idx = prop_names
                .iter()
                .position(|name| *name == selected_prop)
                .unwrap_or(0);
            let is_backward = player.get_entity().is_sneaking();
            let new_idx = if is_backward {
                (cur_idx + prop_names.len() - 1) % prop_names.len()
            } else {
                (cur_idx + 1) % prop_names.len()
            };
            let new_selected_prop = prop_names[new_idx];
            if let Some((_, val)) = stick_state
                .properties
                .iter_mut()
                .find(|(k, _)| k == &block_key || k == block.name)
            {
                *val = new_selected_prop.to_str().to_string();
            } else {
                stick_state
                    .properties
                    .push((block_key, new_selected_prop.to_str().to_string()));
            }
            item.set_data_component(stick_state);

            let current_val = prop_list
                .iter()
                .find(|p| p.key() == new_selected_prop)
                .map_or("", |p| p.value_str());

            player.send_system_message_raw(
                &TextComponent::translate(
                    "item.minecraft.debug_stick.select",
                    [
                        TextComponent::text(new_selected_prop.to_str()),
                        TextComponent::text(current_val),
                    ],
                ),
                true,
            );
        }

        true
    }
}

impl ItemBehaviour for DebugStickItem {
    fn can_mine(&self, player: &Player) -> bool {
        if player.can_use_game_master_blocks() {
            let (start, end) = self.get_start_and_end_pos(player);
            let world = player.world();
            if let Some((pos, _)) =
                world.raycast(start, end, |pos, world| world.get_block(pos) != &Block::AIR)
            {
                let (block, state) = world.get_block_and_state(&pos);
                let mut held = player.inventory().held_item();
                Self::handle_interaction(&mut held, player, &pos, block, state.id, false);
                player.inventory().set_held_item(held);
            }
        }
        false
    }

    fn use_on_block(
        &self,
        item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &Block,
        _server: &Server,
    ) -> BlockActionResult {
        if player.can_use_game_master_blocks() {
            let world = player.world();
            let state_id = world.get_block_state_id(&location);
            if Self::handle_interaction(item, player, &location, block, state_id, true) {
                return BlockActionResult::Success;
            }
        }
        BlockActionResult::Fail
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
