use std::sync::Arc;

use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{StructureBlockLikeProperties, StructureblockMode};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::{GameMode, PermissionLvl};

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::entities::structure_block::StructureBlockBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, NormalUseArgs, OnNeighborUpdateArgs, OnPlaceArgs, PlacedArgs, PlayerPlacedArgs,
};

#[pumpkin_block("minecraft:structure_block")]
pub struct StructureBlock;

impl BlockBehaviour for StructureBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = StructureBlockLikeProperties::default(args.block);
        props.mode = StructureblockMode::Load;
        props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let mode = StructureBlockBlockEntity::mode_from_block_state(args.state_id)
            .unwrap_or(StructureblockMode::Load);
        let entity = StructureBlockBlockEntity::new_with_mode(*args.position, mode);
        args.world.add_block_entity(Arc::new(entity));
    }

    fn player_placed(&self, args: PlayerPlacedArgs<'_>) {
        let Some(entity) = args.world.get_block_entity(args.position) else {
            return;
        };
        let Some(structure_block) = entity.as_any().downcast_ref::<StructureBlockBlockEntity>()
        else {
            return;
        };
        structure_block.set_author(args.player.gameprofile.name.clone());
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if args.player.permission_lvl.load() < PermissionLvl::Two {
            return BlockActionResult::Pass;
        }
        if args.player.gamemode.load() != GameMode::Creative {
            return BlockActionResult::Pass;
        }
        let Some(block_entity) = args.world.get_block_entity(args.position) else {
            return BlockActionResult::Pass;
        };
        if block_entity
            .as_any()
            .downcast_ref::<StructureBlockBlockEntity>()
            .is_none()
        {
            return BlockActionResult::Pass;
        }
        args.world.update_block_entity(&block_entity);

        BlockActionResult::Success
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let Some(entity) = args.world.get_block_entity(args.position) else {
            return;
        };
        let Some(structure_block) = entity.as_any().downcast_ref::<StructureBlockBlockEntity>()
        else {
            return;
        };

        let should_trigger = block_receives_redstone_power(args.world, args.position);
        let is_powered = structure_block.is_powered();
        if should_trigger && !is_powered {
            structure_block.set_powered(true);
            match structure_block.mode() {
                StructureblockMode::Save => {
                    structure_block.save_structure(args.world, false);
                }
                StructureblockMode::Load => {
                    structure_block.place_structure(args.world);
                }
                StructureblockMode::Corner => {
                    structure_block.unload_structure();
                }
                StructureblockMode::Data => {}
            }
        } else if !should_trigger && is_powered {
            structure_block.set_powered(false);
        }
    }
}
