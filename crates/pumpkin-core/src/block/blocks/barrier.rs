use crate::block::{BlockBehaviour, CanPlaceLiquidArgs, OnPlaceArgs, PickupFluidArgs};
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::MangroveRootsLikeProperties as BarrierLikeProperties;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item::Item;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::GameMode;

#[pumpkin_block("minecraft:barrier")]
pub struct BarrierBlock;

impl BlockBehaviour for BarrierBlock {
    fn can_place_liquid(&self, args: CanPlaceLiquidArgs<'_>) -> bool {
        args.fluid == &Fluid::WATER
            && args
                .player
                .is_some_and(|player| player.gamemode.load() == GameMode::Creative)
    }

    fn pickup_fluid(&self, args: PickupFluidArgs<'_>) -> Option<&'static Item> {
        args.player
            .filter(|player| player.gamemode.load() == GameMode::Creative)?;
        crate::block::pickup_fluid(args)
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = BarrierLikeProperties::default(args.block);
        props.waterlogged = args.replacing.water_source();
        props.to_state_id(args.block)
    }
}
