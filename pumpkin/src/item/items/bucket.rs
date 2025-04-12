use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::fluid::{Falling, Fluid, FluidProperties, Level};
use pumpkin_data::item::Item;
use pumpkin_inventory::player::PlayerInventory;
use pumpkin_protocol::client::play::{CSetContainerSlot, CSetHeldItem};
use pumpkin_protocol::codec::slot::Slot;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::block::BlockDirection;
use pumpkin_world::item::ItemStack;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;

use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::world::{BlockFlags, World};

type FlowingFluidProperties = pumpkin_data::fluid::FlowingWaterLikeFluidProperties;

pub struct BucketItem;

impl ItemMetadata for BucketItem {
    fn ids() -> Box<[u16]> {
        [
            Item::BUCKET.id,
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

#[async_trait]
impl PumpkinItem for BucketItem {
    async fn normal_use(&self, item: &Item, player: &Player) {
        log::info!("Using item");
        if item.id == Item::BUCKET.id {
            let start_pos = player.eye_position();
            let (yaw, pitch) = player.rotation();
            let (yaw_rad, pitch_rad) = (yaw.to_radians() as f64, pitch.to_radians() as f64);
            let block_interaction_range = player.block_interaction_range() as f64;
            let world = player.world().await.clone();

            let direction = Vector3::new(
                -yaw_rad.sin() * pitch_rad.cos() * block_interaction_range,
                -pitch_rad.sin() * block_interaction_range,
                pitch_rad.cos() * yaw_rad.cos() * block_interaction_range,
            );

            let end_pos = start_pos.add(&direction);

            let checker = async | pos: &BlockPos, world_inner: &Arc<World> | {
                let Ok(state_id) = world_inner.get_block_state_id(pos).await else {
                    return false;
                };
                let Some(fluid) = Fluid::from_state_id(state_id) else {
                    return false;
                };

                if fluid.name != Fluid::EMPTY.name {
                    let props = FlowingFluidProperties::from_state_id(state_id, &fluid);
                    if props.level == Level::L8 && props.falling == Falling::False {
                        return true;
                    }
                }

                false
            };



            let block_pos = world.traverse_blocks(start_pos, end_pos, checker).await;

            if let Some(pos) = block_pos {
                let Ok (state_id) = world.get_block_state_id(&pos).await else {
                    return;
                };

                let Some(fluid) = Fluid::from_state_id(state_id) else {
                    return;
                };

                if fluid.name == Fluid::FLOWING_WATER.name || fluid.name == Fluid::FLOWING_LAVA.name{
                    world.set_block_state(&pos, Block::AIR.id, BlockFlags::NOTIFY_NEIGHBORS).await;
                    let mut inventory = player.inventory().lock().await;
                    let selected = inventory.get_selected_slot();
                    let item = ItemStack::new(1, if fluid.name == Fluid::FLOWING_WATER.name {Item::WATER_BUCKET} else {Item::LAVA_BUCKET});
                    let slot_data = Slot::from(&item);
                    if let Err(err) = inventory.set_slot(selected, Some(item), false) {
                        log::error!("Failed to set slot: {}", err);
                    }else{
                        let dest_packet = CSetContainerSlot::new(PlayerInventory::CONTAINER_ID, inventory.state_id as i32, selected as i16, &slot_data);
                        player.client.enqueue_packet(&dest_packet).await;
                    }
                }
            }
        }


        //TODO implement Milk bucket
    }

    async fn use_on_block(
        &self,
        item: &Item,
        player: &Player,
        location: BlockPos,
        face: &BlockDirection,
        _block: &Block,
        _server: &crate::server::Server,
    ) {
        log::info!("Using item on block");
        if item.id == Item::WATER_BUCKET.id {
            player.world().await.set_block_state(
                &location.offset(face.to_offset()),
                Block::WATER.default_state_id,
                BlockFlags::NOTIFY_NEIGHBORS,
            ).await;
            let mut inventory = player.inventory().lock().await;
            let selected = inventory.get_selected_slot();
            let item = ItemStack::new(1, Item::BUCKET);
            let slot_data = Slot::from(&item);
            if let Err(err) = inventory.set_slot(selected, Some(item), false) {
                log::error!("Failed to set slot: {}", err);
                return;
            }else{
                let dest_packet = CSetContainerSlot::new(PlayerInventory::CONTAINER_ID, inventory.state_id as i32, selected as i16, &slot_data);
                player.client.enqueue_packet(&dest_packet).await;
            }
        } else if item.id == Item::LAVA_BUCKET.id {
            player.world().await.set_block_state(
                &location.offset(face.to_offset()),
                Block::LAVA.default_state_id,
                BlockFlags::NOTIFY_NEIGHBORS,
            ).await;
            let mut inventory = player.inventory().lock().await;
            let selected = inventory.get_selected_slot();
            let item = ItemStack::new(1, Item::BUCKET);
            let slot_data = Slot::from(&item);
            if let Err(err) = inventory.set_slot(selected, Some(item), false) {
                log::error!("Failed to set slot: {}", err);
                return;
            }else{
                let dest_packet = CSetContainerSlot::new(PlayerInventory::CONTAINER_ID, inventory.state_id as i32, selected as i16, &slot_data);
                player.client.enqueue_packet(&dest_packet).await;
            }
        }
    }
}
