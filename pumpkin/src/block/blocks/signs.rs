use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::sign::SignBlockEntity;

use crate::block::pumpkin_block::OnStateReplacedArgs;
use crate::block::pumpkin_block::PlacedArgs;
use crate::block::pumpkin_block::PlayerPlacedArgs;
use crate::block::pumpkin_block::{BlockMetadata, OnPlaceArgs, PumpkinBlock};

type SignProperties = pumpkin_data::block_properties::OakSignLikeProperties;

pub struct SignBlock;

impl BlockMetadata for SignBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:signs").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for SignBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut sign_props = SignProperties::default(args.block);
        sign_props.waterlogged = args.replacing.water_source();

        sign_props.to_state_id(args.block)
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        args.world
            .add_block_entity(Arc::new(SignBlockEntity::empty(*args.location)))
            .await;
    }

    async fn player_placed(&self, args: PlayerPlacedArgs<'_>) {
        args.player.send_sign_packet(*args.location).await;
    }

    async fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        args.world.remove_block_entity(args.location).await;
    }
}
