use std::sync::Arc;

use crate::entity::player::Player;
use async_trait::async_trait;
use pumpkin_data::{
    Block, BlockState,
    fluid::Fluid,
    item::Item,
    sound::{Sound, SoundCategory},
};
use pumpkin_registry::VanillaDimensionType;
use pumpkin_util::{
    GameMode,
    math::{position::BlockPos, vector3::Vector3},
};
use pumpkin_world::{inventory::Inventory, item::ItemStack, world::BlockFlags};

use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::world::World;

pub struct EmptyBucketItem;
pub struct FilledBucketItem;
pub struct MilkBucketItem;

impl ItemMetadata for EmptyBucketItem {
    fn ids() -> Box<[u16]> {
        [Item::BUCKET.id].into()
    }
}

impl ItemMetadata for FilledBucketItem {
    fn ids() -> Box<[u16]> {
        [
            Item::WATER_BUCKET.id,
            Item::LAVA_BUCKET.id,
            Item::POWDER_SNOW_BUCKET.id,
            Item::AXOLOTL_BUCKET.id,
            Item::COD_BUCKET.id,
            Item::SALMON_BUCKET.id,
            Item::TROPICAL_FISH_BUCKET.id,
            Item::PUFFERFISH_BUCKET.id,
            Item::TADPOLE_BUCKET.id,
        ]
        .into()
    }
}

impl ItemMetadata for MilkBucketItem {
    fn ids() -> Box<[u16]> {
        [Item::MILK_BUCKET.id].into()
    }
}

fn get_start_and_end_pos(player: &Player) -> (Vector3<f64>, Vector3<f64>) {
    let start_pos = player.eye_position();
    let (yaw, pitch) = player.rotation();
    let (yaw_rad, pitch_rad) = (f64::from(yaw.to_radians()), f64::from(pitch.to_radians()));
    let block_interaction_range = 4.5; // This is not the same as the block_interaction_range in the
    // player entity.
    let direction = Vector3::new(
        -yaw_rad.sin() * pitch_rad.cos() * block_interaction_range,
        -pitch_rad.sin() * block_interaction_range,
        pitch_rad.cos() * yaw_rad.cos() * block_interaction_range,
    );

    let end_pos = start_pos.add(&direction);
    (start_pos, end_pos)
}

fn waterlogged_check(block: &Block, state: &BlockState) -> Option<bool> {
    block.properties(state.id).and_then(|properties| {
        properties
            .to_props()
            .into_iter()
            .find(|p| p.0 == "waterlogged")
            .map(|(_, value)| value == true.to_string())
    })
}

fn set_waterlogged(block: &Block, state: &BlockState, waterlogged: bool) -> u16 {
    let original_props = &block.properties(state.id).unwrap().to_props();
    let waterlogged = waterlogged.to_string();
    let props = original_props
        .iter()
        .map(|(key, value)| {
            if key == "waterlogged" {
                ("waterlogged", waterlogged.as_str())
            } else {
                (key.as_str(), value.as_str())
            }
        })
        .collect();
    block.from_properties(props).unwrap().to_state_id(block)
}

#[async_trait]
impl PumpkinItem for EmptyBucketItem {
    async fn normal_use(&self, _item: &Item, player: &Player) {
        let world = player.world().await.clone();
        let (start_pos, end_pos) = get_start_and_end_pos(player);

        let checker = async |pos: &BlockPos, world_inner: &Arc<World>| {
            let state_id = world_inner.get_block_state_id(pos).await;

            let block = Block::from_state_id(state_id);

            if state_id == Block::AIR.default_state.id {
                return false;
            }

            (block.id != Block::WATER.id && block.id != Block::LAVA.id)
                || ((block.id == Block::WATER.id && state_id == Block::WATER.default_state.id)
                    || (block.id == Block::LAVA.id && state_id == Block::LAVA.default_state.id))
        };

        let Some((block_pos, direction)) = world.raycast(start_pos, end_pos, checker).await else {
            return;
        };

        let (block, state) = world.get_block_and_block_state(&block_pos).await;

        if block
            .properties(state.id)
            .and_then(|properties| {
                properties
                    .to_props()
                    .into_iter()
                    .find(|p| p.0 == "waterlogged")
                    .map(|(_, value)| value == true.to_string())
            })
            .unwrap_or(false)
        {
            let state_id = set_waterlogged(block, state, false);
            world
                .set_block_state(&block_pos, state_id, BlockFlags::NOTIFY_NEIGHBORS)
                .await;
            world.schedule_fluid_tick(block.id, block_pos, 5).await;
        } else if state.id == Block::LAVA.default_state.id
            || state.id == Block::WATER.default_state.id
        {
            world
                .break_block(&block_pos, None, BlockFlags::NOTIFY_NEIGHBORS)
                .await;
            world
                .set_block_state(
                    &block_pos,
                    Block::AIR.default_state.id,
                    BlockFlags::NOTIFY_NEIGHBORS,
                )
                .await;
        } else {
            let (block, state) = world
                .get_block_and_block_state(&block_pos.offset(direction.to_offset()))
                .await;
            if waterlogged_check(block, state).is_some() {
                let state_id = set_waterlogged(block, state, false);
                world
                    .set_block_state(
                        &block_pos.offset(direction.to_offset()),
                        state_id,
                        BlockFlags::NOTIFY_NEIGHBORS,
                    )
                    .await;
                world
                    .schedule_fluid_tick(block.id, block_pos.offset(direction.to_offset()), 5)
                    .await;
            } else {
                return;
            }
        }

        let item = if state.id == Block::LAVA.default_state.id {
            &Item::LAVA_BUCKET
        } else {
            &Item::WATER_BUCKET
        };

        if player.gamemode.load() == GameMode::Creative {
            //Check if player already has the item in their inventory
            for i in 0..player.inventory.main_inventory.len() {
                if player.inventory.main_inventory[i].lock().await.item.id == item.id {
                    return;
                }
            }
            //If not, add it to the inventory
            let mut item_stack = ItemStack::new(1, item);
            player
                .inventory
                .insert_stack_anywhere(&mut item_stack)
                .await;
        } else {
            let item_stack = ItemStack::new(1, item);
            player
                .inventory
                .set_stack(player.inventory.get_selected_slot().into(), item_stack)
                .await;
        }
    }
}

#[async_trait]
impl PumpkinItem for FilledBucketItem {
    async fn normal_use(&self, item: &Item, player: &Player) {
        let world = player.world().await.clone();
        let (start_pos, end_pos) = get_start_and_end_pos(player);
        let checker = async |pos: &BlockPos, world_inner: &Arc<World>| {
            let state_id = world_inner.get_block_state_id(pos).await;
            if Fluid::from_state_id(state_id).is_some() {
                return false;
            }
            state_id != Block::AIR.id
        };

        let Some((pos, direction)) = world.raycast(start_pos, end_pos, checker).await else {
            return;
        };

        if item.id != Item::LAVA_BUCKET.id
            && world.dimension_type == VanillaDimensionType::TheNether
        {
            world
                .play_sound_raw(
                    Sound::BlockFireExtinguish as u16,
                    SoundCategory::Blocks,
                    &player.position(),
                    0.5,
                    2.6 + (rand::random::<f32>() - rand::random::<f32>()) * 0.8,
                )
                .await;
            return;
        }
        let (block, state) = world.get_block_and_block_state(&pos).await;
        if waterlogged_check(block, state).is_some() && item.id == Item::WATER_BUCKET.id {
            let state_id = set_waterlogged(block, state, true);
            world
                .set_block_state(&pos, state_id, BlockFlags::NOTIFY_NEIGHBORS)
                .await;
            world.schedule_fluid_tick(block.id, pos, 5).await;
        } else {
            let (block, state) = world
                .get_block_and_block_state(&pos.offset(direction.to_offset()))
                .await;

            if waterlogged_check(block, state).is_some() {
                if item.id == Item::LAVA_BUCKET.id {
                    return;
                }
                let state_id = set_waterlogged(block, state, true);

                world
                    .set_block_state(
                        &pos.offset(direction.to_offset()),
                        state_id,
                        BlockFlags::NOTIFY_NEIGHBORS,
                    )
                    .await;
                world
                    .schedule_fluid_tick(block.id, pos.offset(direction.to_offset()), 5)
                    .await;
            } else if state.id == Block::AIR.default_state.id || state.is_liquid() {
                world
                    .set_block_state(
                        &pos.offset(direction.to_offset()),
                        if item.id == Item::LAVA_BUCKET.id {
                            Block::LAVA.default_state.id
                        } else {
                            Block::WATER.default_state.id
                        },
                        BlockFlags::NOTIFY_NEIGHBORS,
                    )
                    .await;
            } else {
                return;
            }
        }

        //TODO: Spawn entity if applicable
        if player.gamemode.load() != GameMode::Creative {
            let item_stack = ItemStack::new(1, &Item::BUCKET);
            player
                .inventory
                .set_stack(player.inventory.get_selected_slot().into(), item_stack)
                .await;
        }
    }
}

//TODO: Implement MilkBucketItem
