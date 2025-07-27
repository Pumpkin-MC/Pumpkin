use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::block::entities::sign::SignBlockEntity;

use crate::block::pumpkin_block::OnPlaceArgs;
use crate::block::pumpkin_block::PlacedArgs;
use crate::block::pumpkin_block::PlayerPlacedArgs;
use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::EntityBase;

type SignProperties = pumpkin_data::block_properties::OakSignLikeProperties;

#[pumpkin_block_from_tag("minecraft:signs")]
pub struct SignBlock;

#[async_trait]
impl PumpkinBlock for SignBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> u16 {
        let mut sign_props = SignProperties::default(args.block);
        sign_props.waterlogged = args.replacing.water_source();
        sign_props.rotation = args.player.get_entity().get_flipped_rotation_16();
        sign_props.to_state_id(args.block)
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        args.world
            .add_block_entity(Arc::new(SignBlockEntity::empty(*args.position)))
            .await;
    }

    async fn player_placed(&self, args: PlayerPlacedArgs<'_>) {
        match &args.player.client {
            crate::net::ClientPlatform::Java(java) => java.send_sign_packet(*args.position).await,
            crate::net::ClientPlatform::Bedrock(_bedrock) => todo!(),
        }
    }

    fn has_block_entity(&self) -> bool {
        true
    }
}
