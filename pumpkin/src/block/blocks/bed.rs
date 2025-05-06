use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockState;
use pumpkin_data::block_properties::BedPart;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_registry::DimensionType;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::BlockDirection;
use pumpkin_world::block::entities::bed::BedBlockEntity;

use crate::block::BlockIsReplacing;
use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::BlockFlags;
use crate::world::World;

type BedProperties = pumpkin_data::block_properties::WhiteBedLikeProperties;

pub struct BedBlock;
impl BlockMetadata for BedBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:beds").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for BedBlock {
    async fn can_place_at(
        &self,
        _server: &Server,
        world: &World,
        player: &Player,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: &SUseItemOn,
    ) -> bool {
        let facing = player.living_entity.entity.get_horizontal_facing();

        world
            .get_block_state(block_pos)
            .await
            .is_ok_and(|state| state.replaceable())
            && world
                .get_block_state(&block_pos.offset(facing.to_offset()))
                .await
                .is_ok_and(|state| state.replaceable())
    }

    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        player: &Player,
        block: &Block,
        _block_pos: &BlockPos,
        _face: BlockDirection,
        _replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        let mut bed_props = BedProperties::default(block);

        bed_props.facing = player.living_entity.entity.get_horizontal_facing();
        bed_props.part = BedPart::Foot;

        bed_props.to_state_id(block)
    }

    async fn placed(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: BlockStateId,
        block_pos: &BlockPos,
        _old_state_id: BlockStateId,
        _notify: bool,
    ) {
        let bed_entity = BedBlockEntity::new(*block_pos);
        world.add_block_entity(Arc::new(bed_entity)).await;

        let mut bed_head_props = BedProperties::default(block);
        bed_head_props.facing = BedProperties::from_state_id(state_id, block).facing;
        bed_head_props.part = BedPart::Head;

        let bed_head_pos = block_pos.offset(bed_head_props.facing.to_offset());
        world
            .set_block_state(
                &bed_head_pos,
                bed_head_props.to_state_id(block),
                BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
            )
            .await;

        let bed_head_entity = BedBlockEntity::new(bed_head_pos);
        world.add_block_entity(Arc::new(bed_head_entity)).await;
    }

    async fn broken(
        &self,
        block: &Block,
        player: &Arc<Player>,
        block_pos: BlockPos,
        _server: &Server,
        world: Arc<World>,
        state: BlockState,
    ) {
        let bed_props = BedProperties::from_state_id(state.id, block);
        let other_half_pos = if bed_props.part == BedPart::Head {
            block_pos.offset(bed_props.facing.opposite().to_offset())
        } else {
            block_pos.offset(bed_props.facing.to_offset())
        };

        world
            .break_block(
                &other_half_pos,
                Some(player.clone()),
                if player.gamemode.load() == GameMode::Creative {
                    BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_NEIGHBORS
                } else {
                    BlockFlags::NOTIFY_NEIGHBORS
                },
            )
            .await;
    }

    async fn normal_use(
        &self,
        block: &Block,
        player: &Player,
        block_pos: BlockPos,
        _server: &Server,
        world: &Arc<World>,
    ) {
        if world.dimension_type == DimensionType::Overworld {
            let state_id = world.get_block_state_id(&block_pos).await.unwrap();
            let bed_props = BedProperties::from_state_id(state_id, block);

            let bed_head_pos = if bed_props.part == BedPart::Head {
                block_pos
            } else {
                block_pos.offset(bed_props.facing.to_offset())
            };

            player.set_respawn_point(bed_head_pos, player.get_entity().yaw.load());

            if bed_props.occupied {
                // Wake up villager
            } else if can_sleep(world).await {
                player.sleep(bed_head_pos).await;
                set_bed_occupied(true, world, block, &block_pos, bed_props).await;
            } else {
                player
                    .send_system_message(&TextComponent::translate("block.minecraft.set_spawn", []))
                    .await;
            }
        }
    }
}

async fn set_bed_occupied(
    occupied: bool,
    world: &Arc<World>,
    block: &Block,
    block_pos: &BlockPos,
    mut bed_props: BedProperties,
) {
    bed_props.occupied = occupied;
    world
        .set_block_state(
            block_pos,
            bed_props.to_state_id(block),
            BlockFlags::NOTIFY_LISTENERS,
        )
        .await;

    let other_half_pos = if bed_props.part == BedPart::Head {
        block_pos.offset(bed_props.facing.opposite().to_offset())
    } else {
        block_pos.offset(bed_props.facing.to_offset())
    };
    bed_props.part = if bed_props.part == BedPart::Head {
        BedPart::Foot
    } else {
        BedPart::Head
    };
    world
        .set_block_state(
            &other_half_pos,
            bed_props.to_state_id(block),
            BlockFlags::NOTIFY_LISTENERS,
        )
        .await;
}

async fn can_sleep(world: &Arc<World>) -> bool {
    let time = world.level_time.lock().await;
    let weather = world.weather.lock().await;

    if weather.thundering {
        true
    } else if weather.raining {
        time.time_of_day > 12010 && time.time_of_day < 23991
    } else {
        time.time_of_day > 12542 && time.time_of_day < 23459
    }
}
