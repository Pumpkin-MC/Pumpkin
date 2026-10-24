use crate::block::BlockBehaviour;
use pumpkin_data::block_state::BlockStateId;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use rand::random;

#[pumpkin_block("minecraft:netherrack")]
pub struct NetherrackBlock;

impl BlockBehaviour for NetherrackBlock {
    fn is_valid_bonemeal_target(&self, args: crate::block::BonemealArgs<'_>) -> bool {
        let above_block = args.world.get_block_state(&args.position.up());

        if above_block.is_full_cube()
            || above_block.is_liquid()
            || !args.world.is_loaded(&args.position.up())
        {
            return false;
        }

        for block_pos in BlockPos::iterate_outwards_ref(args.position, 1, 1, 1) {
            if args
                .world
                .get_block(&block_pos)
                .has_tag(&tag::Block::MINECRAFT_NYLIUM)
            {
                return true;
            }
        }

        false
    }

    fn perform_bonemeal(&self, args: crate::block::BonemealArgs<'_>) {
        let mut warped = false;
        let mut crimson = false;

        for block_pos in BlockPos::iterate_outwards_ref(args.position, 1, 1, 1) {
            let block = args.world.get_block(&block_pos);

            if block.id == Block::WARPED_NYLIUM.id {
                warped = true;
            }

            if block.id == Block::CRIMSON_NYLIUM.id {
                crimson = true;
            }

            if warped && crimson {
                break;
            }
        }

        if !warped && !crimson {
            return;
        }

        let end_block: BlockStateId = match (warped, crimson) {
            (true, true) => {
                if random::<bool>() {
                    Block::WARPED_NYLIUM.default_state.id
                } else {
                    Block::CRIMSON_NYLIUM.default_state.id
                }
            }
            (true, false) => Block::WARPED_NYLIUM.default_state.id,
            (false, true) => Block::CRIMSON_NYLIUM.default_state.id,
            (false, false) => Block::NETHERRACK.default_state.id,
        };

        args.world
            .set_block_state(args.position, end_block, BlockFlags::NOTIFY_ALL);
    }
}
