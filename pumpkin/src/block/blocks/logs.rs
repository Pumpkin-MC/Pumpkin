use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::BlockProperties;
use pumpkin_data::block::CardinalDirection;
use pumpkin_data::block::LogBlockProps;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::block::registry::BlockRegistry;
use crate::server::Server;
use crate::world::World;

macro_rules! define_log_block {
    ($block_name:ident, $block:expr) => {
        pub struct $block_name;
        impl BlockMetadata for $block_name {
            const NAMESPACE: &'static str = "minecraft";
            const ID: &'static str = $block.name;
        }

        #[async_trait]
        impl PumpkinBlock for $block_name {
            async fn on_place(
                &self,
                _server: &Server,
                _world: &World,
                block: &Block,
                face: &BlockDirection,
                _block_pos: &BlockPos,
                _use_item_on: &SUseItemOn,
                _player_direction: &CardinalDirection,
                _other: bool,
            ) -> u16 {
                let mut log_props = LogBlockProps::default(block);
                log_props.axis = face.to_axis();

                log_props.to_state_id(block)
            }
        }
    };
}

/*
[
      "dark_oak_log",
      "dark_oak_wood",
      "stripped_dark_oak_log",
      "stripped_dark_oak_wood",
      "pale_oak_log",
      "pale_oak_wood",
      "stripped_pale_oak_log",
      "stripped_pale_oak_wood",
      "oak_log",
      "oak_wood",
      "stripped_oak_log",
      "stripped_oak_wood",
      "acacia_log",
      "acacia_wood",
      "stripped_acacia_log",
      "stripped_acacia_wood",
      "birch_log",
      "birch_wood",
      "stripped_birch_log",
      "stripped_birch_wood",
      "jungle_log",
      "jungle_wood",
      "stripped_jungle_log",
      "stripped_jungle_wood",
      "spruce_log",
      "spruce_wood",
      "stripped_spruce_log",
      "stripped_spruce_wood",
      "mangrove_log",
      "mangrove_wood",
      "stripped_mangrove_log",
      "stripped_mangrove_wood",
      "cherry_log",
      "cherry_wood",
      "stripped_cherry_log",
      "stripped_cherry_wood",
      "crimson_stem",
      "stripped_crimson_stem",
      "crimson_hyphae",
      "stripped_crimson_hyphae",
      "warped_stem",
      "stripped_warped_stem",
      "warped_hyphae",
      "stripped_warped_hyphae"
    ]
*/

define_log_block!(OakLogBlock, Block::OAK_LOG);
define_log_block!(DarkOakLogBlock, Block::DARK_OAK_LOG);
define_log_block!(PaleOakLogBlock, Block::PALE_OAK_LOG);
define_log_block!(StrippedPaleOakLogBlock, Block::STRIPPED_PALE_OAK_LOG);
define_log_block!(StrippedPaleOakWoodBlock, Block::STRIPPED_PALE_OAK_WOOD);
define_log_block!(BirchLogBlock, Block::BIRCH_LOG);
define_log_block!(JungleLogBlock, Block::JUNGLE_LOG);
define_log_block!(SpruceLogBlock, Block::SPRUCE_LOG);
define_log_block!(MangroveLogBlock, Block::MANGROVE_LOG);
define_log_block!(CherryLogBlock, Block::CHERRY_LOG);
define_log_block!(CrimsonStemBlock, Block::CRIMSON_STEM);
define_log_block!(WarpedStemBlock, Block::WARPED_STEM);

define_log_block!(DarkOakWoodBlock, Block::DARK_OAK_WOOD);
define_log_block!(BirchWoodBlock, Block::BIRCH_WOOD);
define_log_block!(JungleWoodBlock, Block::JUNGLE_WOOD);
define_log_block!(SpruceWoodBlock, Block::SPRUCE_WOOD);
define_log_block!(MangroveWoodBlock, Block::MANGROVE_WOOD);
define_log_block!(CherryWoodBlock, Block::CHERRY_WOOD);

define_log_block!(StrippedDarkOakLogBlock, Block::STRIPPED_DARK_OAK_LOG);
define_log_block!(StrippedDarkOakWoodBlock, Block::STRIPPED_DARK_OAK_WOOD);
define_log_block!(StrippedOakLogBlock, Block::STRIPPED_OAK_LOG);
define_log_block!(StrippedOakWoodBlock, Block::STRIPPED_OAK_WOOD);
define_log_block!(StrippedAcaciaLogBlock, Block::STRIPPED_ACACIA_LOG);
define_log_block!(StrippedAcaciaWoodBlock, Block::STRIPPED_ACACIA_WOOD);
define_log_block!(StrippedBirchLogBlock, Block::STRIPPED_BIRCH_LOG);
define_log_block!(StrippedBirchWoodBlock, Block::STRIPPED_BIRCH_WOOD);
define_log_block!(StrippedJungleLogBlock, Block::STRIPPED_JUNGLE_LOG);
define_log_block!(StrippedJungleWoodBlock, Block::STRIPPED_JUNGLE_WOOD);
define_log_block!(StrippedSpruceLogBlock, Block::STRIPPED_SPRUCE_LOG);
define_log_block!(StrippedSpruceWoodBlock, Block::STRIPPED_SPRUCE_WOOD);
define_log_block!(StrippedMangroveLogBlock, Block::STRIPPED_MANGROVE_LOG);
define_log_block!(StrippedMangroveWoodBlock, Block::STRIPPED_MANGROVE_WOOD);
define_log_block!(StrippedCherryLogBlock, Block::STRIPPED_CHERRY_LOG);
define_log_block!(StrippedCherryWoodBlock, Block::STRIPPED_CHERRY_WOOD);
define_log_block!(StrippedCrimsonStemBlock, Block::STRIPPED_CRIMSON_STEM);
define_log_block!(StrippedCrimsonHyphaeBlock, Block::STRIPPED_CRIMSON_HYPHAE);
define_log_block!(StrippedWarpedStemBlock, Block::STRIPPED_WARPED_STEM);
define_log_block!(StrippedWarpedHyphaeBlock, Block::STRIPPED_WARPED_HYPHAE);

pub fn register_log_blocks(manager: &mut BlockRegistry) {
    manager.register(OakLogBlock);
    manager.register(DarkOakLogBlock);
    manager.register(BirchLogBlock);
    manager.register(JungleLogBlock);
    manager.register(SpruceLogBlock);
    manager.register(MangroveLogBlock);
    manager.register(CherryLogBlock);
    manager.register(CrimsonStemBlock);
    manager.register(WarpedStemBlock);

    manager.register(DarkOakWoodBlock);
    manager.register(BirchWoodBlock);
    manager.register(JungleWoodBlock);
    manager.register(SpruceWoodBlock);
    manager.register(MangroveWoodBlock);
    manager.register(CherryWoodBlock);

    manager.register(StrippedDarkOakLogBlock);
    manager.register(StrippedDarkOakWoodBlock);
    manager.register(StrippedOakLogBlock);
    manager.register(StrippedOakWoodBlock);
    manager.register(StrippedAcaciaLogBlock);
    manager.register(StrippedAcaciaWoodBlock);
    manager.register(StrippedBirchLogBlock);
    manager.register(StrippedBirchWoodBlock);
    manager.register(StrippedJungleLogBlock);
    manager.register(StrippedJungleWoodBlock);
    manager.register(StrippedSpruceLogBlock);
    manager.register(StrippedSpruceWoodBlock);
    manager.register(StrippedMangroveLogBlock);
    manager.register(StrippedMangroveWoodBlock);
    manager.register(StrippedCherryLogBlock);
    manager.register(StrippedCherryWoodBlock);
    manager.register(StrippedCrimsonStemBlock);
    manager.register(StrippedCrimsonHyphaeBlock);
    manager.register(StrippedWarpedStemBlock);
    manager.register(StrippedWarpedHyphaeBlock);
}
