use rand::RngExt;
use pumpkin_data::{Block, Enchantment};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_world::BlockStateId;

use crate::block::{BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, blocks::plant::PlantBlockBase, BrokenArgs};

pub struct ShortPlantBlock;

impl BlockMetadata for ShortPlantBlock {
    fn ids() -> Box<[u16]> {
        [Block::SHORT_GRASS.id, Block::FERN.id].into()
    }
}

impl BlockBehaviour for ShortPlantBlock {
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position).await
        })
    }

    fn broken<'a>(&'a self, _args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let held = _args.player.inventory.held_item();
            let held = held.lock().await;
            if held.item == &Item::SHEARS || held.get_enchantment_level(&Enchantment::SILK_TOUCH) > 0 {
                return;
            }

            drop(held);
            if rand::rng().random_bool(0.125) {
                _args.world
                    .drop_stack(_args.position, ItemStack::new(1, &Item::WHEAT_SEEDS))
                    .await;
            }

        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            <Self as PlantBlockBase>::get_state_for_neighbor_update(
                self,
                args.world,
                args.position,
                args.state_id,
            )
                .await
        })
    }
}

impl PlantBlockBase for ShortPlantBlock {}

