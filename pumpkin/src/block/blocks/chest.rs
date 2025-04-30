use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block_properties::{
    BlockProperties, ChestLikeProperties, ChestType, HorizontalFacing,
};
use pumpkin_data::entity::EntityPose;
use pumpkin_data::item::Item;
use pumpkin_data::{Block, BlockState, block_properties::get_block};
use pumpkin_data::{
    screen::WindowType,
    sound::{Sound, SoundCategory},
};
use pumpkin_inventory::{ChestContainer, OpenContainer};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_protocol::{client::play::CBlockAction, codec::var_int::VarInt};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::chest::ChestBlockEntity;
use pumpkin_world::block::{BlockDirection, HorizontalFacingExt};

use crate::block::BlockIsReplacing;
use crate::entity::EntityBase;
use crate::world::{BlockFlags, World};
use crate::{
    block::{pumpkin_block::PumpkinBlock, registry::BlockActionResult},
    entity::player::Player,
    server::Server,
};

#[derive(PartialEq)]
pub enum ChestState {
    IsOpened,
    IsClosed,
}

#[pumpkin_block("minecraft:chest")]
pub struct ChestBlock;

#[async_trait]
impl PumpkinBlock for ChestBlock {
    async fn on_place(
        &self,
        _server: &Server,
        world: &World,
        block: &Block,
        face: &BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        player: &Player,
        replacing: BlockIsReplacing,
    ) -> BlockStateId {
        let mut chest_props = ChestLikeProperties::default(block);
        let player_facing = player.get_entity().get_horizontal_facing();

        chest_props.waterlogged = replacing.water_source();
        chest_props.facing = player_facing.opposite();

        chest_props.r#type = if player.get_entity().pose.load() as i8 == EntityPose::Crouching as i8
        {
            if let Some(face) = face.to_horizontal_facing() {
                let (clicked_block, clicked_block_state) = world
                    .get_block_and_block_state(&block_pos.offset(face.to_offset()))
                    .await
                    .unwrap();

                if clicked_block != *block {
                    ChestType::Single
                } else {
                    let clicked_props =
                        ChestLikeProperties::from_state_id(clicked_block_state.id, &clicked_block);

                    if clicked_props.facing.rotate_clockwise() == face {
                        chest_props.facing = clicked_props.facing;
                        ChestType::Left
                    } else if clicked_props.facing.rotate_counter_clockwise() == face {
                        chest_props.facing = clicked_props.facing;
                        ChestType::Right
                    } else {
                        ChestType::Single
                    }
                }
            } else {
                ChestType::Single
            }
        } else if get_chest_properties_if_can_connect(
            world,
            block,
            block_pos,
            chest_props.facing,
            chest_props.facing.rotate_clockwise(),
            ChestType::Single,
        )
        .await
        .is_some()
        {
            ChestType::Left
        } else if get_chest_properties_if_can_connect(
            world,
            block,
            block_pos,
            chest_props.facing,
            chest_props.facing.rotate_counter_clockwise(),
            ChestType::Single,
        )
        .await
        .is_some()
        {
            ChestType::Right
        } else {
            ChestType::Single
        };

        chest_props.to_state_id(block)
    }

    async fn placed(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: u16,
        block_pos: &BlockPos,
        _old_state_id: u16,
        _notify: bool,
    ) {
        let chest = ChestBlockEntity::new(block_pos.clone());
        world.add_block_entity(Arc::new(chest)).await;

        let chest_props = ChestLikeProperties::from_state_id(state_id, block);
        let connected_towards = match chest_props.r#type {
            ChestType::Single => return,
            ChestType::Left => chest_props.facing.rotate_clockwise(),
            ChestType::Right => chest_props.facing.rotate_counter_clockwise(),
        };

        if let Some(mut neighbor_props) = get_chest_properties_if_can_connect(
            &world,
            block,
            block_pos,
            chest_props.facing,
            connected_towards,
            ChestType::Single,
        )
        .await
        {
            neighbor_props.r#type = chest_props.r#type.opposite();

            world
                .set_block_state(
                    &block_pos.offset(connected_towards.to_offset()),
                    neighbor_props.to_state_id(block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
        }
    }

    async fn on_state_replaced(
        &self,
        world: &Arc<World>,
        _block: &Block,
        location: BlockPos,
        _old_state_id: u16,
        _moved: bool,
    ) {
        world.remove_block_entity(&location).await;
    }

    async fn use_with_item(
        &self,
        block: &Block,
        player: &Player,
        _location: BlockPos,
        _item: &Item,
        server: &Server,
        _world: &Arc<World>,
    ) -> BlockActionResult {
        self.open_chest_block(block, player, _location, server)
            .await;
        BlockActionResult::Consume
    }

    async fn broken(
        &self,
        block: &Block,
        player: &Player,
        block_pos: BlockPos,
        server: &Server,
        world: Arc<World>,
        state: BlockState,
    ) {
        super::standard_on_broken_with_container(block, player, block_pos, server).await;

        let chest_props = ChestLikeProperties::from_state_id(state.id, block);
        let connected_towards = match chest_props.r#type {
            ChestType::Single => return,
            ChestType::Left => chest_props.facing.rotate_clockwise(),
            ChestType::Right => chest_props.facing.rotate_counter_clockwise(),
        };

        if let Some(mut neighbor_props) = get_chest_properties_if_can_connect(
            &world,
            block,
            &block_pos,
            chest_props.facing,
            connected_towards,
            chest_props.r#type.opposite(),
        )
        .await
        {
            neighbor_props.r#type = ChestType::Single;

            world
                .set_block_state(
                    &block_pos.offset(connected_towards.to_offset()),
                    neighbor_props.to_state_id(block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
        }
    }

    async fn normal_use(
        &self,
        block: &Block,
        player: &Player,
        _location: BlockPos,
        server: &Server,
        _world: &Arc<World>,
    ) {
        self.open_chest_block(block, player, _location, server)
            .await;
    }

    async fn close(
        &self,
        _block: &Block,
        player: &Player,
        location: BlockPos,
        server: &Server,
        container: &mut OpenContainer,
    ) {
        container.remove_player(player.entity_id());

        self.play_chest_action(container, player, location, server, ChestState::IsClosed)
            .await;
    }
}

impl ChestBlock {
    pub async fn open_chest_block(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        server: &Server,
    ) {
        // TODO: shouldn't Chest and window type be constrained together to avoid errors?
        super::standard_open_container::<ChestContainer>(
            block,
            player,
            location,
            server,
            WindowType::Generic9x3,
        )
        .await;

        if let Some(container_id) = server.get_container_id(location, block.clone()).await {
            let open_containers = server.open_containers.read().await;
            if let Some(container) = open_containers.get(&u64::from(container_id)) {
                self.play_chest_action(container, player, location, server, ChestState::IsOpened)
                    .await;
            }
        }
    }

    pub async fn play_chest_action(
        &self,
        container: &OpenContainer,
        player: &Player,
        location: BlockPos,
        server: &Server,
        state: ChestState,
    ) {
        let num_players = container.get_number_of_players() as u8;
        if state == ChestState::IsClosed && num_players == 0 {
            player
                .world()
                .await
                .play_block_sound(Sound::BlockChestClose, SoundCategory::Blocks, location)
                .await;
        } else if state == ChestState::IsOpened && num_players == 1 {
            player
                .world()
                .await
                .play_block_sound(Sound::BlockChestOpen, SoundCategory::Blocks, location)
                .await;
        }

        if let Some(e) = get_block("minecraft:chest") {
            server
                .broadcast_packet_all(&CBlockAction::new(
                    location,
                    1,
                    num_players,
                    VarInt(e.id.into()),
                ))
                .await;
        }
    }
}

async fn get_chest_properties_if_can_connect(
    world: &World,
    block: &Block,
    block_pos: &BlockPos,
    facing: HorizontalFacing,
    direction: HorizontalFacing,
    wanted_type: ChestType,
) -> Option<ChestLikeProperties> {
    let (neighbor_block, neighbor_block_state) = world
        .get_block_and_block_state(&block_pos.offset(direction.to_offset()))
        .await
        .unwrap();

    if neighbor_block != *block {
        return None;
    }

    let neighbor_props =
        ChestLikeProperties::from_state_id(neighbor_block_state.id, &neighbor_block);
    if neighbor_props.facing == facing && neighbor_props.r#type == wanted_type {
        return Some(neighbor_props);
    }

    None
}

trait ChestTypeExt {
    fn opposite(&self) -> ChestType;
}

impl ChestTypeExt for ChestType {
    fn opposite(&self) -> ChestType {
        match self {
            ChestType::Single => ChestType::Single,
            ChestType::Left => ChestType::Right,
            ChestType::Right => ChestType::Left,
        }
    }
}
