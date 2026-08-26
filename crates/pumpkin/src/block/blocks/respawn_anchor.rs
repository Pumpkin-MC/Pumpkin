use pumpkin_data::block_properties::{BlockProperties, RespawnAnchorLikeProperties};
use pumpkin_data::dimension::Dimension;
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::translation;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::BlockFlags;

use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, NormalUseArgs, UseWithItemArgs};
use crate::entity::EntityBase;

#[pumpkin_block("minecraft:respawn_anchor")]
pub struct RespawnAnchorBlock;

impl BlockBehaviour for RespawnAnchorBlock {
    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        if args.item_stack.item.id != Item::GLOWSTONE.id {
            return BlockActionResult::Pass;
        }

        let state_id = args.world.get_block_state_id(args.position);
        let mut props = RespawnAnchorLikeProperties::from_state_id(state_id, args.block);

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

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position);
        let props = RespawnAnchorLikeProperties::from_state_id(state_id, args.block);

        if args.world.dimension != Dimension::THE_NETHER {
            args.world
                .break_block(args.position, None, BlockFlags::SKIP_DROPS);
            let center_pos = args.position.to_centered_f64();
            args.world
                .explode(center_pos, 5.0, crate::world::ExplosionInteraction::Block);
            return BlockActionResult::SuccessServer;
        }

        if props.charges == 0 {
            args.player
                .send_system_message(&pumpkin_macros::translate_cross!(
                    translation::java::BLOCK_MINECRAFT_BED_NO_SLEEP,
                    translation::bedrock::TILE_BED_NOSLEEP
                ));
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
}
