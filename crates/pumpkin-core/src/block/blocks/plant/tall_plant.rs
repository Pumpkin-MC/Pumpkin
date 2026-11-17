use crate::block::{BonemealArgs, BrokenArgs, PlacedArgs};
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockId;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{DoubleBlockHalf, TallSeagrassLikeProperties};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_world::world::BlockFlags;

use crate::block::{
    BlockBehaviour, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    blocks::plant::PlantBlockBase,
};

pub struct TallPlantBlock;

impl BlockMetadata for TallPlantBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::TALL_GRASS,
            BlockId::LARGE_FERN,
            BlockId::PITCHER_PLANT,
            // TallFlowerBlocks
            BlockId::SUNFLOWER,
            BlockId::LILAC,
            BlockId::PEONY,
            BlockId::ROSE_BUSH,
        ]
        .into()
    }
}

impl BlockBehaviour for TallPlantBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let props = TallSeagrassLikeProperties::from_state_id(args.state.id);
        if props.half == DoubleBlockHalf::Upper {
            let (below_block, below_state) = args
                .block_accessor
                .get_block_and_state(&args.position.down());
            return below_block == args.block
                && TallSeagrassLikeProperties::from_state_id(below_state.id).half
                    == DoubleBlockHalf::Lower;
        }

        if args.player.is_some() {
            let up_pos = args.position.up();
            if let Some(world) = args.world
                && up_pos.0.y > world.get_top_y()
            {
                return false;
            }
            let upper_state = args.block_accessor.get_block_state(&up_pos);
            if !upper_state.replaceable() {
                return false;
            }
        }

        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let tall_plant_props = TallSeagrassLikeProperties::from_state_id(args.state_id);
        let (support_block_pos, other_block_pos) = match tall_plant_props.half {
            DoubleBlockHalf::Upper => (args.position.down_height(2), args.position.down()),
            DoubleBlockHalf::Lower => (args.position.down(), args.position.up()),
        };
        if !<Self as PlantBlockBase>::can_place_at(self, args.world, &support_block_pos.up()) {
            return Block::AIR.default_state.id;
        }

        let (other_block, other_state_id) = args.world.get_block_and_state_id(&other_block_pos);
        if other_block == args.block {
            let other_props = TallSeagrassLikeProperties::from_state_id(other_state_id);
            if other_props.half != tall_plant_props.half {
                return args.state_id;
            }
        }
        Block::AIR.default_state.id
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let mut tall_plant_props = TallSeagrassLikeProperties::from_state_id(args.state_id);
        if tall_plant_props.half != DoubleBlockHalf::Lower {
            return;
        }
        tall_plant_props.half = DoubleBlockHalf::Upper;
        args.world.set_block_state(
            &args.position.offset(BlockDirection::Up.to_offset()),
            tall_plant_props.to_state_id(args.block),
            BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
        );
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        // When one half of a tall plant is broken, break the other half too
        let tall_plant_props = TallSeagrassLikeProperties::from_state_id(args.state.id);
        let other_block_pos = match tall_plant_props.half {
            DoubleBlockHalf::Upper => args.position.down(),
            DoubleBlockHalf::Lower => args.position.up(),
        };
        let (other_block, other_state_id) = args.world.get_block_and_state_id(&other_block_pos);
        if other_block == args.block {
            let other_props = TallSeagrassLikeProperties::from_state_id(other_state_id);
            if other_props.half != tall_plant_props.half {
                // Break the other half, using SKIP_DROPS to prevent double drops
                args.world.break_block(
                    &other_block_pos,
                    None,
                    BlockFlags::SKIP_DROPS
                        | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK
                        | BlockFlags::NOTIFY_ALL,
                );
            }
        }
    }

    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        matches!(
            args.block.id,
            BlockId::SUNFLOWER | BlockId::LILAC | BlockId::PEONY | BlockId::ROSE_BUSH
        )
    }

    fn is_bonemeal_success(&self, args: BonemealArgs<'_>) -> bool {
        matches!(
            args.block.id,
            BlockId::SUNFLOWER | BlockId::LILAC | BlockId::PEONY | BlockId::ROSE_BUSH
        )
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        if matches!(
            args.block.id,
            BlockId::SUNFLOWER | BlockId::LILAC | BlockId::PEONY | BlockId::ROSE_BUSH
        ) && let Some(item) = Item::from_id(args.block.item_id)
        {
            args.world
                .drop_stack(args.position, ItemStack::new(1, item));
        }
    }
}

impl PlantBlockBase for TallPlantBlock {}
