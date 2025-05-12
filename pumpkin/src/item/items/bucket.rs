use std::sync::Arc;

use crate::entity::player::Player;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item::Item;
use pumpkin_registry::DimensionType;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::item::ItemStack;
use pumpkin_world::world::BlockFlags;

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

#[async_trait]
impl PumpkinItem for EmptyBucketItem {
    #[allow(clippy::too_many_lines)]
    async fn normal_use(&self, _item: &Item, player: &Player) {
        let world = player.world().await.clone();
        let (start_pos, end_pos) = get_start_and_end_pos(player);

        let checker = async |pos: &BlockPos, world_inner: &Arc<World>| {
            let Ok(state_id) = world_inner.get_block_state_id(pos).await else {
                return false;
            };

            let block = Block::from_state_id(state_id).unwrap();

            if state_id == Block::AIR.default_state_id {
                return false;
            }

            (block.id != Block::WATER.id && block.id != Block::LAVA.id)
                || ((block.id == Block::WATER.id && state_id == Block::WATER.default_state_id)
                    || (block.id == Block::LAVA.id && state_id == Block::LAVA.default_state_id))
        };

        let (block_pos, _) = world.raytrace(start_pos, end_pos, checker).await;

        if let Some(pos) = block_pos {
            let Ok(state_id) = world.get_block_state_id(&pos).await else {
                return;
            };

            let block = Block::from_state_id(state_id).unwrap();

            if block
                .properties(state_id)
                .and_then(|properties| {
                    properties
                        .to_props()
                        .into_iter()
                        .find(|p| p.0 == "waterlogged")
                        .map(|(_, value)| value == true.to_string())
                })
                .unwrap_or(false)
            {
                //get props and set waterlogged to false
                let original_props = &block.properties(state_id).unwrap().to_props();
                let mut props_vec: Vec<(&str, &str)> = Vec::with_capacity(original_props.len());
                for (key, value) in original_props {
                    if key == "waterlogged" {
                        props_vec.push((key.as_str(), "false"));
                    } else {
                        props_vec.push((key.as_str(), value.as_str()));
                    }
                }
                let block_state_id = block
                    .from_properties(props_vec)
                    .unwrap()
                    .to_state_id(&block);
                world
                    .set_block_state(&pos, block_state_id, BlockFlags::NOTIFY_NEIGHBORS)
                    .await;
            } else if state_id == Block::LAVA.default_state_id
                || state_id == Block::WATER.default_state_id
            {
                world
                    .break_block(&pos, None, BlockFlags::NOTIFY_NEIGHBORS)
                    .await;
                world
                    .set_block_state(
                        &pos,
                        Block::AIR.default_state_id,
                        BlockFlags::NOTIFY_NEIGHBORS,
                    )
                    .await;
            }

            let item = if state_id == Block::LAVA.default_state_id {
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
                player
                    .inventory
                    .set_stack(
                        player.inventory.get_selected_slot().into(),
                        ItemStack::new(1, item),
                    )
                    .await;
            }
        }
    }
}

#[async_trait]
impl PumpkinItem for FilledBucketItem {
    async fn normal_use(&self, item: &Item, player: &Player) {
        let world = player.world().await.clone();
        let (start_pos, end_pos) = get_start_and_end_pos(player);
        let checker = async |pos: &BlockPos, world_inner: &Arc<World>| {
            let Ok(state_id) = world_inner.get_block_state_id(pos).await else {
                return false;
            };
            if Fluid::from_state_id(state_id).is_some() {
                return false;
            }
            state_id != Block::AIR.id
        };

        let (block_pos, block_direction) = world.raytrace(start_pos, end_pos, checker).await;

        if let (Some(pos), Some(direction)) = (block_pos, block_direction) {
            if item.id != Item::LAVA_BUCKET.id && world.dimension_type == DimensionType::TheNether {
                return;
            }
            let Ok(block) = world.get_block(&pos).await else {
                return;
            };

            let Ok(state_id) = world.get_block_state_id(&pos).await else {
                return;
            };

            let waterlogged_check = block.properties(state_id).and_then(|properties| {
                properties
                    .to_props()
                    .into_iter()
                    .find(|p| p.0 == "waterlogged")
            });

            if waterlogged_check.is_some() {
                //get props and set waterlogged to true
                let original_props = &block.properties(state_id).unwrap().to_props();
                let mut props_vec: Vec<(&str, &str)> = Vec::with_capacity(original_props.len());
                for (key, value) in original_props {
                    if key == "waterlogged" {
                        props_vec.push((key.as_str(), "true"));
                    } else {
                        props_vec.push((key.as_str(), value.as_str()));
                    }
                }
                let block_state_id = block
                    .from_properties(props_vec)
                    .unwrap()
                    .to_state_id(&block);
                world
                    .set_block_state(&pos, block_state_id, BlockFlags::NOTIFY_NEIGHBORS)
                    .await;
                world
                    .schedule_fluid_tick(Block::from_state_id(block_state_id).unwrap().id, pos, 5)
                    .await;
            } else {
                world
                    .set_block_state(
                        &pos.offset(direction.to_offset()),
                        if item.id == Item::LAVA_BUCKET.id {
                            Block::LAVA.default_state_id
                        } else {
                            Block::WATER.default_state_id
                        },
                        BlockFlags::NOTIFY_NEIGHBORS,
                    )
                    .await;
            }

            //TODO: Spawn entity if applicable
            if player.gamemode.load() != GameMode::Creative {
                player
                    .inventory
                    .set_stack(
                        player.inventory.get_selected_slot().into(),
                        ItemStack::new(1, &Item::BUCKET),
                    )
                    .await;
            }
        }
    }
}

//TODO: Implement MilkBucketItem
