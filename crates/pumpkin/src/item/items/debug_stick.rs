use std::any::Any;
use std::sync::{LazyLock, Mutex};

use rustc_hash::FxHashMap;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{Block, BlockDirection, BlockId, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::world::BlockFlags;

static SELECTED_PROPERTIES: LazyLock<Mutex<FxHashMap<BlockId, &'static str>>> =
    LazyLock::new(|| Mutex::new(FxHashMap::default()));

pub struct DebugStickItem;

impl ItemMetadata for DebugStickItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::DEBUG_STICK.id])
    }
}

impl DebugStickItem {
    #[expect(clippy::too_many_lines)]
    fn handle_interaction(
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

        let prop_names: Vec<&'static str> = prop_list.iter().map(|(k, _)| *k).collect();
        let mut map = SELECTED_PROPERTIES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut selected_prop = map.get(&block.id).copied().unwrap_or(prop_names[0]);
        if !prop_names.contains(&selected_prop) {
            selected_prop = prop_names[0];
        }

        if cycle {
            let mut possible_values: Vec<&'static str> = Vec::new();
            for state in block.states {
                if let Some(state_props) = block.properties(state.id) {
                    for (k, v) in state_props.to_props() {
                        if k == selected_prop && !possible_values.contains(&v) {
                            possible_values.push(v);
                        }
                    }
                }
            }

            if possible_values.is_empty() {
                return false;
            }

            let current_val = prop_list
                .iter()
                .find(|(k, _)| *k == selected_prop)
                .map_or(possible_values[0], |(_, v)| *v);
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

            let mut new_props = prop_list.clone();
            for (k, v) in &mut new_props {
                if *k == selected_prop {
                    *v = new_val;
                }
            }

            let new_state_id = block.from_properties(&new_props).to_state_id(block);
            let world = player.world();
            world.set_block_state(pos, new_state_id, BlockFlags::NOTIFY_ALL);

            player.send_system_message_raw(
                &TextComponent::translate(
                    "item.minecraft.debug_stick.update",
                    [
                        TextComponent::text(selected_prop),
                        TextComponent::text(new_val),
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
            map.insert(block.id, new_selected_prop);

            let current_val = prop_list
                .iter()
                .find(|(k, _)| *k == new_selected_prop)
                .map_or("", |(_, v)| *v);

            player.send_system_message_raw(
                &TextComponent::translate(
                    "item.minecraft.debug_stick.select",
                    [
                        TextComponent::text(new_selected_prop),
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
                Self::handle_interaction(player, &pos, block, state.id, false);
            }
        }
        false
    }

    fn use_on_block(
        &self,
        _item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &Block,
        _server: &Server,
    ) {
        if player.can_use_game_master_blocks() {
            let world = player.world();
            let state_id = world.get_block_state_id(&location);
            Self::handle_interaction(player, &location, block, state_id, true);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
