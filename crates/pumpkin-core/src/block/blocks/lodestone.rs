use pumpkin_data::poi_type::PoiType;
use pumpkin_macros::pumpkin_block;

use crate::block::{BlockBehaviour, OnStateReplacedArgs, PlacedArgs};

#[pumpkin_block("minecraft:lodestone")]
pub struct LodestoneBlock;

impl BlockBehaviour for LodestoneBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        args.world
            .portal_poi
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .add_poi(*args.position, PoiType::Lodestone);
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        args.world
            .portal_poi
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(args.position);
    }
}
