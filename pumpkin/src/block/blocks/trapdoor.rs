use crate::block::BlockIsReplacing;
use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::{BlockHalf, BlockProperties};
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{RegistryKey, Tagable, get_tag_values};
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

type TrapDoorProperties = pumpkin_data::block_properties::OakTrapdoorLikeProperties;

async fn toggle_trapdoor(player: &Player, world: &Arc<World>, block_pos: &BlockPos) {
    let (block, block_state) = world.get_block_and_block_state(block_pos).await;
    let mut trapdoor_props = TrapDoorProperties::from_state_id(block_state.id, &block);
    trapdoor_props.open = !trapdoor_props.open;

    world
        .play_block_sound_expect(
            player,
            get_sound(&block, trapdoor_props.open),
            SoundCategory::Blocks,
            *block_pos,
        )
        .await;

    world
        .set_block_state(
            block_pos,
            trapdoor_props.to_state_id(&block),
            BlockFlags::NOTIFY_LISTENERS,
        )
        .await;
}

fn can_open_trapdoor(block: &Block) -> bool {
    if block.id == Block::IRON_TRAPDOOR.id {
        return false;
    }
    true
}

// Todo: The sounds should be from BlockSetType
fn get_sound(block: &Block, open: bool) -> Sound {
    if open {
        if block.is_tagged_with("minecraft:wooden_trapdoors").unwrap() {
            Sound::BlockWoodenTrapdoorOpen
        } else if block.id == Block::IRON_TRAPDOOR.id {
            Sound::BlockIronTrapdoorOpen
        } else {
            Sound::BlockCopperTrapdoorOpen
        }
    } else if block.is_tagged_with("minecraft:wooden_trapdoors").unwrap() {
        Sound::BlockWoodenTrapdoorClose
    } else if block.id == Block::IRON_TRAPDOOR.id {
        Sound::BlockIronTrapdoorClose
    } else {
        Sound::BlockCopperTrapdoorClose
    }
}
pub struct TrapDoorBlock;
impl BlockMetadata for TrapDoorBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:trapdoors").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for TrapDoorBlock {
    async fn normal_use(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        _server: &Server,
        world: &Arc<World>,
    ) {
        if can_open_trapdoor(block) {
            toggle_trapdoor(player, world, &location).await;
        }
    }

    async fn use_with_item(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        _item: &Item,
        _server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        if !can_open_trapdoor(block) {
            return BlockActionResult::Continue;
        }

        toggle_trapdoor(player, world, &location).await;

        BlockActionResult::Consume
    }

    async fn on_place(
        &self,
        _server: &Server,
        world: &World,
        player: &Player,
        block: &Block,
        block_pos: &BlockPos,
        face: BlockDirection,
        replacing: BlockIsReplacing,
        use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        let mut trapdoor_props = TrapDoorProperties::default(block);
        trapdoor_props.waterlogged = replacing.water_source();

        let powered = block_receives_redstone_power(world, block_pos).await;
        let direction = player
            .living_entity
            .entity
            .get_horizontal_facing()
            .opposite();

        trapdoor_props.facing = direction;
        trapdoor_props.half = match face {
            BlockDirection::Up => BlockHalf::Top,
            BlockDirection::Down => BlockHalf::Bottom,
            _ => match use_item_on.cursor_pos.y {
                0.0...0.5 => BlockHalf::Bottom,
                _ => BlockHalf::Top,
            },
        };
        trapdoor_props.powered = powered;
        trapdoor_props.open = powered;

        trapdoor_props.to_state_id(block)
    }

    async fn on_neighbor_update(
        &self,
        world: &Arc<World>,
        block: &Block,
        pos: &BlockPos,
        _source_block: &Block,
        _notify: bool,
    ) {
        let block_state = world.get_block_state(pos).await;
        let mut trapdoor_props = TrapDoorProperties::from_state_id(block_state.id, block);
        let powered = block_receives_redstone_power(world, pos).await;
        if powered != trapdoor_props.powered {
            trapdoor_props.powered = !trapdoor_props.powered;

            if powered != trapdoor_props.open {
                trapdoor_props.open = trapdoor_props.powered;

                world
                    .play_block_sound(get_sound(block, powered), SoundCategory::Blocks, *pos)
                    .await;
            }
        }

        world
            .set_block_state(
                pos,
                trapdoor_props.to_state_id(block),
                BlockFlags::NOTIFY_LISTENERS,
            )
            .await;
    }
}
