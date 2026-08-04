use std::sync::Arc;

use pumpkin_macros::pumpkin_block;
use pumpkin_util::PermissionLvl;

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::entities::structure_block::StructureBlockBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockFuture, NormalUseArgs, OnNeighborUpdateArgs, PlacedArgs, PlayerPlacedArgs,
};

/// `net.minecraft.world.level.block.StructureBlock`.
///
/// An operator-only block whose interesting behavior lives on `StructureBlockBlockEntity`. Only
/// the LOAD half is implemented here (see `StructureBlockBlockEntity::place_structure`); SAVE
/// needs filesystem-backed template storage that doesn't exist in this codebase yet and is left
/// as a documented follow-up.
#[pumpkin_block("minecraft:structure_block")]
pub struct StructureBlockBlock;

impl BlockBehaviour for StructureBlockBlock {
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            args.world
                .add_block_entity(Arc::new(StructureBlockBlockEntity::new(*args.position)));
        })
    }

    /// `StructureBlockEntity.setPlacedBy`: records the placing player's name as the author.
    fn player_placed<'a>(&'a self, args: PlayerPlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let Some(block_entity) = args.world.get_block_entity(args.position) else {
                return;
            };
            let Some(structure_block) = block_entity
                .as_any()
                .downcast_ref::<StructureBlockBlockEntity>()
            else {
                return;
            };
            *structure_block.author.lock().await = args.player.gameprofile.name.clone();
        })
    }

    /// `StructureBlock.useWithoutItem`/`StructureBlockEntity.usedBy`: opens the editor GUI for
    /// operators. `StructureBlock` is a `GameMasterBlock` gated on permission level 2, following
    /// the same shape as `jigsaw.rs`'s `normal_use` (the jigsaw port's additional creative-mode
    /// check has no vanilla citation and is deliberately not copied here, per the design doc).
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if args.player.permission_lvl.load() < PermissionLvl::Two {
                return BlockActionResult::Pass;
            }
            let Some(block_entity) = args.world.get_block_entity(args.position) else {
                return BlockActionResult::Pass;
            };
            args.world.update_block_entity(&block_entity);
            BlockActionResult::SuccessServer
        })
    }

    /// `StructureBlock.neighborChanged` (`StructureBlock.java:67-97`): on a redstone signal,
    /// LOAD mode always places the named template (no size-match gating - that's
    /// `placeStructureIfSameSize`, only reachable from the GUI's load button, which requires the
    /// C2S structure-block packet this codebase doesn't have yet). SAVE/CORNER/DATA modes are
    /// no-ops here: SAVE has no filesystem-backed template writer to save to, and CORNER/DATA
    /// don't affect world state.
    ///
    /// Like `TNTBlock::on_neighbor_update` in this codebase, this checks the current redstone
    /// state rather than tracking a false-to-true edge, so a template can be re-placed on any
    /// neighbor update while already powered, not only on the rising edge.
    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !block_receives_redstone_power(args.world, args.position).await {
                return;
            }
            let Some(block_entity) = args.world.get_block_entity(args.position) else {
                return;
            };
            let Some(structure_block) = block_entity
                .as_any()
                .downcast_ref::<StructureBlockBlockEntity>()
            else {
                return;
            };
            match structure_block.mode.lock().await.as_str() {
                "LOAD" => {
                    structure_block.place_structure(args.world).await;
                }
                "SAVE" => {
                    structure_block.save_structure(args.world, false).await;
                }
                _ => {}
            }
        })
    }
}
