use crate::block::{BlockBehaviour, BlockFuture, BlockMetadata, OnPlaceArgs};
use pumpkin_data::block_properties::{BlockProperties, WallTorchLikeProperties};
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_world::BlockStateId;

pub struct GlazedTerracottaBlock;
impl BlockMetadata for GlazedTerracottaBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "c:glazed_terracottas").unwrap()
    }
}

impl BlockBehaviour for GlazedTerracottaBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut prop = WallTorchLikeProperties::default(args.block);
            prop.facing = args
                .player
                .living_entity
                .entity
                .get_horizontal_facing()
                .opposite();
            prop.to_state_id(args.block)
        })
    }
}
