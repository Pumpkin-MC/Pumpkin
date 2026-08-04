use pumpkin_data::item::Item;
use pumpkin_data::{
    block_properties::{BlockProperties, RespawnAnchorLikeProperties},
    dimension::Dimension,
    sound::{Sound, SoundCategory},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::GameMode;
use pumpkin_world::world::BlockFlags;

use crate::block::{
    BlockBehaviour, BlockFuture, GetComparatorOutputArgs, NormalUseArgs, UseWithItemArgs,
    registry::BlockActionResult,
};

/// `net.minecraft.world.level.block.RespawnAnchorBlock`: charges with glowstone, sets the
/// player's respawn point on an empty-hand use, and explodes instead outside the Nether.
#[pumpkin_block("minecraft:respawn_anchor")]
pub struct RespawnAnchorBlock;

impl RespawnAnchorBlock {
    const MAX_CHARGES: u8 = 4;

    /// Mirrors `RespawnAnchorBlock.canSetSpawn`, which reads the
    /// `minecraft:gameplay/respawn_anchor_works` dimension attribute (true only in the Nether).
    fn works_here(world: &crate::world::World) -> bool {
        world.dimension == Dimension::THE_NETHER
    }

    /// Mirrors `RespawnAnchorBlock.getAnalogOutputSignal`: `charge * 15 / MAX_CHARGES`.
    const fn charges_to_comparator_output(charges: u8) -> u8 {
        charges * 15 / Self::MAX_CHARGES
    }
}

impl BlockBehaviour for RespawnAnchorBlock {
    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let state_id = args.world.get_block_state_id(args.position);
            let mut props = RespawnAnchorLikeProperties::from_state_id(state_id, args.block);

            let item = args.item_stack.lock().await.item;
            if item != &Item::GLOWSTONE || props.charges >= Self::MAX_CHARGES {
                return BlockActionResult::PassToDefaultBlockAction;
            }

            if args.player.gamemode.load() != GameMode::Creative {
                args.item_stack.lock().await.decrement(1);
            }

            props.charges += 1;
            args.world
                .set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
            args.world.play_sound(
                Sound::BlockRespawnAnchorCharge,
                SoundCategory::Blocks,
                &args.position.to_centered_f64(),
            );

            BlockActionResult::Success
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let state_id = args.world.get_block_state_id(args.position);
            let props = RespawnAnchorLikeProperties::from_state_id(state_id, args.block);
            if props.charges == 0 {
                return BlockActionResult::Pass;
            }

            if !Self::works_here(args.world) {
                args.world
                    .break_block(args.position, None, BlockFlags::SKIP_DROPS)
                    .await;
                args.world
                    .explode(args.position.to_centered_f64(), 5.0)
                    .await;
                return BlockActionResult::SuccessServer;
            }

            let changed = args
                .player
                .set_respawn_point(
                    args.world.dimension.clone(),
                    *args.position,
                    0.0,
                    0.0,
                    false,
                )
                .await;
            if changed {
                args.world.play_sound(
                    Sound::BlockRespawnAnchorSetSpawn,
                    SoundCategory::Blocks,
                    &args.position.to_centered_f64(),
                );
                return BlockActionResult::SuccessServer;
            }
            BlockActionResult::Consume
        })
    }

    fn get_comparator_output<'a>(
        &'a self,
        args: GetComparatorOutputArgs<'a>,
    ) -> BlockFuture<'a, Option<u8>> {
        Box::pin(async move {
            let props = RespawnAnchorLikeProperties::from_state_id(args.state.id, args.block);
            Some(Self::charges_to_comparator_output(props.charges))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparator_output_matches_vanilla_scaling() {
        assert_eq!(RespawnAnchorBlock::charges_to_comparator_output(0), 0);
        assert_eq!(RespawnAnchorBlock::charges_to_comparator_output(1), 3);
        assert_eq!(RespawnAnchorBlock::charges_to_comparator_output(2), 7);
        assert_eq!(RespawnAnchorBlock::charges_to_comparator_output(3), 11);
        assert_eq!(RespawnAnchorBlock::charges_to_comparator_output(4), 15);
    }
}
