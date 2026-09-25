use pumpkin_data::block_properties::RespawnAnchorLikeProperties;
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{BlockState, translation};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::BlockFlags;

use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, GetComparatorOutputArgs, NormalUseArgs, PathComputationType, UseWithItemArgs,
};
use crate::entity::EntityBase;

/// Vanilla `RespawnAnchorBlock.MAX_CHARGES`.
const MAX_CHARGES: u8 = 4;

/// Returns whether a respawn anchor should explode for the current charge and dimension state.
const fn should_explode(charges: u8, respawn_anchor_works: bool) -> bool {
    charges > 0 && !respawn_anchor_works
}

#[pumpkin_block("minecraft:respawn_anchor")]
pub struct RespawnAnchorBlock;

impl BlockBehaviour for RespawnAnchorBlock {
    /// Charges the respawn anchor when the player uses glowstone and capacity remains.
    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        if args.item_stack.item.id != Item::GLOWSTONE.id {
            return BlockActionResult::Pass;
        }

        let state_id = args.world.get_block_state_id(args.position);
        let mut props = RespawnAnchorLikeProperties::from_state_id(state_id);

        if props.charges >= 4 {
            return BlockActionResult::Pass;
        }

        props.charges += 1;
        args.world.set_block_state(
            args.position,
            props.to_state_id(args.block),
            BlockFlags::NOTIFY_ALL,
        );

        args.item_stack
            .decrement_unless_creative(args.player.gamemode.load(), 1);

        args.world.play_sound(
            Sound::BlockRespawnAnchorCharge,
            SoundCategory::Blocks,
            &args.position.to_f64(),
        );

        BlockActionResult::Success
    }

    /// Uses a charged respawn anchor for respawning or explosion behavior in the current dimension.
    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position);
        let props = RespawnAnchorLikeProperties::from_state_id(state_id);

        if props.charges == 0 {
            return BlockActionResult::Pass;
        }

        if should_explode(props.charges, args.world.dimension.respawn_anchor_works) {
            args.world
                .break_block(args.position, None, BlockFlags::SKIP_DROPS);
            let center_pos = args.position.to_centered_f64();
            args.world
                .explode(center_pos, 5.0, crate::world::ExplosionInteraction::Block);
            return BlockActionResult::SuccessServer;
        }

        let player = args.player;
        let world = args.world;
        let pos = *args.position;
        if player.set_respawn_point(
            world.dimension.clone(),
            pos,
            player.get_entity().yaw.load(),
            player.get_entity().pitch.load(),
            false,
        ) {
            world.play_sound(
                Sound::BlockRespawnAnchorSetSpawn,
                SoundCategory::Blocks,
                &pos.to_f64(),
            );

            player.send_system_message(&pumpkin_macros::translate_cross!(
                translation::java::BLOCK_MINECRAFT_SET_SPAWN,
                translation::bedrock::TILE_BED_RESPAWNSET
            ));
        }

        BlockActionResult::SuccessServer
    }

    /// Charges scale over the full signal range, so each charge is worth 15 / 4.
    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        let props = RespawnAnchorLikeProperties::from_state_id(args.state.id);
        Some(props.charges * 15 / MAX_CHARGES)
    }

    /// Respawn anchors are never considered passable for pathfinding.
    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::should_explode;

    #[test]
    fn uncharged_anchor_never_explodes() {
        assert!(!should_explode(0, false));
        assert!(!should_explode(0, true));
    }

    #[test]
    fn charged_anchor_explodes_only_where_unsupported() {
        assert!(should_explode(1, false));
        assert!(should_explode(4, false));
        assert!(!should_explode(1, true));
        assert!(!should_explode(4, true));
    }
}
