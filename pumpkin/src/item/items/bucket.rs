use std::sync::Arc;

use crate::entity::player::Player;
use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item::Item;
use pumpkin_inventory::player::PlayerInventory;
use pumpkin_protocol::client::play::CSetContainerSlot;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::item::ItemStack;

use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::world::{BlockFlags, World};

pub struct EmptyBucketItem;
pub struct FilledBucketItem;

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
            // TODO drink milk
            // Item::MILK_BUCKET.id,
            // TODO implement these buckets, and getting the item from the world
            // Item::POWDER_SNOW_BUCKET.id,
            // Item::AXOLOTL_BUCKET.id,
            // Item::COD_BUCKET.id,
            // Item::SALMON_BUCKET.id,
            // Item::TROPICAL_FISH_BUCKET.id,
            // Item::PUFFERFISH_BUCKET.id,
            // Item::TADPOLE_BUCKET.id,
        ]
        .into()
    }
}

fn get_start_and_end_pos(player: &Player) -> (Vector3<f64>, Vector3<f64>) {
    let start_pos = player.eye_position();
    let (yaw, pitch) = player.rotation();
    let (yaw_rad, pitch_rad) = (f64::from(yaw.to_radians()), f64::from(pitch.to_radians()));
    let block_interaction_range = player.block_interaction_range();
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

            state_id == Block::WATER.default_state_id || state_id == Block::LAVA.default_state_id
        };

        let (block_pos, _) = world.traverse_blocks(start_pos, end_pos, checker).await;

        if let Some(pos) = block_pos {
            let Ok(state_id) = world.get_block_state_id(&pos).await else {
                return;
            };

            world
                .set_block_state(&pos, Block::AIR.id, BlockFlags::NOTIFY_NEIGHBORS)
                .await;
            let mut inventory = player.inventory().lock().await;
            let selected = inventory.get_selected_slot();
            let item_type = if state_id == Block::WATER.default_state_id {
                Item::WATER_BUCKET
            } else {
                Item::LAVA_BUCKET
            };
            let item_stack = Some(ItemStack::new(1, item_type.clone()));
            let slot_data = ItemStackSerializer::from(item_stack.clone());
            let game_mode = player.gamemode.load();
            if game_mode == GameMode::Survival {
                if let Err(err) = inventory.set_slot(selected, item_stack.clone(), false) {
                    log::error!("Failed to set slot: {err}");
                } else {
                    let dest_packet = CSetContainerSlot::new(
                        PlayerInventory::CONTAINER_ID,
                        inventory.state_id as i32,
                        selected as i16,
                        &slot_data,
                    );
                    player.client.enqueue_packet(&dest_packet).await;
                }
            } else {
                let slot = inventory.get_pickup_item_slot(item_type.id);
                if let Some(slot) = slot {
                    if let Err(err) = inventory.set_slot(slot, item_stack, false) {
                        log::error!("Failed to set slot: {err}");
                    } else {
                        let dest_packet = CSetContainerSlot::new(
                            PlayerInventory::CONTAINER_ID,
                            inventory.state_id as i32,
                            slot as i16,
                            &slot_data,
                        );
                        player.client.enqueue_packet(&dest_packet).await;
                    }
                }
            }
        }
    }
}

#[async_trait]
impl PumpkinItem for FilledBucketItem {
    async fn normal_use(&self, item: &Item, player: &Player) {
        if item.id == Item::MILK_BUCKET.id {
            // TODO implement milk bucket
            return;
        }

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

        let (block_pos, block_direction) = world.traverse_blocks(start_pos, end_pos, checker).await;

        if let (Some(pos), Some(direction)) = (block_pos, block_direction) {
            world
                .set_block_state(
                    &pos.offset(direction.to_offset()),
                    Block::WATER.default_state_id,
                    BlockFlags::NOTIFY_NEIGHBORS,
                )
                .await;
            if player.gamemode.load() == GameMode::Survival {
                let mut inventory = player.inventory().lock().await;
                let selected = inventory.get_selected_slot();
                let item = Some(ItemStack::new(1, Item::BUCKET));
                let slot_data = ItemStackSerializer::from(item.clone());
                if let Err(err) = inventory.set_slot(selected, item, false) {
                    log::error!("Failed to set slot: {err}");
                } else {
                    let dest_packet = CSetContainerSlot::new(
                        PlayerInventory::CONTAINER_ID,
                        inventory.state_id as i32,
                        selected as i16,
                        &slot_data,
                    );
                    player.client.enqueue_packet(&dest_packet).await;
                }
            }
        }
    }
}
