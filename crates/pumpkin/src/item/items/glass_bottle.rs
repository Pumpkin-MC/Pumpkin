use std::any::Any;
use std::sync::Arc;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use crate::world::World;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

pub struct GlassBottleItem;

impl ItemMetadata for GlassBottleItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::GLASS_BOTTLE.id])
    }
}

impl ItemBehaviour for GlassBottleItem {
    fn normal_use(&self, _item: &Item, player: &Player) {
        let world = player.world();
        let (start_pos, end_pos) = self.get_start_and_end_pos(player);
        let checker = |pos: &BlockPos, world_inner: &Arc<World>| {
            let state_id = world_inner.get_block_state_id(pos);
            let block = Block::from_state_id(state_id);
            if state_id == Block::AIR.default_state.id {
                return false;
            }
            block.id == Block::WATER.id || block.is_waterlogged(state_id)
        };

        if let Some((hit_pos, _)) = world.raycast(start_pos, end_pos, checker) {
            world.play_sound(
                Sound::ItemBottleFill,
                SoundCategory::Players,
                &hit_pos.to_f64(),
            );

            let water_bottle = ItemStack::new(1, &Item::POTION);
            let mut held = player.inventory().held_item();
            let mut is_main = true;
            if held.is_empty() || held.item.id != Item::GLASS_BOTTLE.id {
                held = player.inventory().off_hand_item();
                is_main = false;
                if held.is_empty() || held.item.id != Item::GLASS_BOTTLE.id {
                    return;
                }
            }

            if held.item_count == 1 && player.gamemode.load() != pumpkin_util::GameMode::Creative {
                if is_main {
                    player.inventory().set_held_item(water_bottle);
                } else {
                    player
                        .inventory()
                        .set_stack_in_hand(pumpkin_util::Hand::Left, water_bottle);
                }
            } else {
                held.decrement_unless_creative(player.gamemode.load(), 1);
                if is_main {
                    player.inventory().set_held_item(held);
                } else {
                    player
                        .inventory()
                        .set_stack_in_hand(pumpkin_util::Hand::Left, held);
                }
                let mut stack_to_give = water_bottle;
                let was_added = player.inventory().insert_stack_anywhere(&mut stack_to_give);
                if !was_added && !stack_to_give.is_empty() {
                    world.drop_stack(&player.position().to_block_pos(), stack_to_give);
                }
            }
        }
    }

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

        let is_water_target = block.id == Block::WATER.id || block.id == Block::WATER_CAULDRON.id;

        let check_pos = if is_water_target {
            location
        } else {
            location.offset(face.to_offset())
        };

        let (check_block, check_state_id) = world.get_block_and_state_id(&check_pos);

        if check_block.id == Block::WATER.id || check_block.id == Block::WATER_CAULDRON.id {
            let cauldron_action = (check_block.id == Block::WATER_CAULDRON.id)
                .then(|| {
                    check_block.properties(check_state_id).and_then(|props| {
                        let prop_map = props.to_props();
                        prop_map
                            .iter()
                            .find(|(k, _)| *k == "level")
                            .and_then(|(_, level_str)| level_str.parse::<u8>().ok())
                            .map(|level| {
                                if level > 1 {
                                    let new_level = (level - 1).to_string();
                                    let new_props: Vec<(&str, &str)> = prop_map
                                        .iter()
                                        .map(|(k, v)| {
                                            if *k == "level" {
                                                (*k, new_level.as_str())
                                            } else {
                                                (*k, *v)
                                            }
                                        })
                                        .collect();
                                    check_block
                                        .from_properties(&new_props)
                                        .to_state_id(check_block)
                                } else {
                                    Block::CAULDRON.default_state.id
                                }
                            })
                    })
                })
                .flatten();

            if let Some(new_state_id) = cauldron_action {
                world.set_block_state(&check_pos, new_state_id, BlockFlags::NOTIFY_ALL);
                player.increment_stat(
                    pumpkin_data::statistic::StatisticCategory::Custom,
                    pumpkin_data::statistic::CustomStatistic::UseCauldron as i32,
                    1,
                );
            }

            world.play_sound(
                Sound::ItemBottleFill,
                SoundCategory::Players,
                &check_pos.to_f64(),
            );

            let mut water_bottle = ItemStack::new(1, &Item::POTION);
            if item.item_count == 1 && player.gamemode.load() != pumpkin_util::GameMode::Creative {
                *item = water_bottle;
            } else {
                item.decrement_unless_creative(player.gamemode.load(), 1);
                let was_added = player.inventory().insert_stack_anywhere(&mut water_bottle);
                if !was_added && !water_bottle.is_empty() {
                    world.drop_stack(&player.position().to_block_pos(), water_bottle);
                }
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
