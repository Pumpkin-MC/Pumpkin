use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_data::{BlockState, block_properties::BedPart};
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_registry::VanillaDimensionType;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::bed::BedBlockEntity;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::BlockFlags;

use crate::block::BlockIsReplacing;
use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase};
use crate::server::Server;
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
        _server: Option<&Server>,
        world: Option<&World>,
        _block_accessor: &dyn BlockAccessor,
        player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        if let Some(player) = player {
            let facing = player.living_entity.entity.get_horizontal_facing();
            let world = world.unwrap();
            return world.get_block_state(block_pos).await.replaceable()
                && world
                    .get_block_state(&block_pos.offset(facing.to_offset()))
                    .await
                    .replaceable();
        }
        false
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
        server: &Server,
        world: &Arc<World>,
    ) {
        let state_id = world.get_block_state_id(&block_pos).await;
        let bed_props = BedProperties::from_state_id(state_id, block);

        let (bed_head_pos, bed_foot_pos) = if bed_props.part == BedPart::Head {
            (
                block_pos,
                block_pos.offset(bed_props.facing.opposite().to_offset()),
            )
        } else {
            (block_pos.offset(bed_props.facing.to_offset()), block_pos)
        };

        // Explode if not in the overworld
        if world.dimension_type != VanillaDimensionType::Overworld {
            world
                .break_block(&bed_head_pos, None, BlockFlags::SKIP_DROPS)
                .await;
            world
                .break_block(&bed_foot_pos, None, BlockFlags::SKIP_DROPS)
                .await;

            world
                .explode(server, bed_head_pos.to_centered_f64(), 5.0)
                .await;

            return;
        }

        // Make sure the bed is not obstructed
        if world.get_block_state(&bed_head_pos.up()).await.is_solid()
            || world.get_block_state(&bed_head_pos.up()).await.is_solid()
        {
            player
                .send_system_message_raw(
                    &TextComponent::translate("block.minecraft.bed.obstructed", []),
                    true,
                )
                .await;
            return;
        }

        // Make sure the bed is not occupied
        if bed_props.occupied {
            // TODO: Wake up villager

            player
                .send_system_message_raw(
                    &TextComponent::translate("block.minecraft.bed.occupied", []),
                    true,
                )
                .await;
            return;
        }

        // Make sure player is close enough
        if !player
            .position()
            .is_within_bounds(bed_head_pos.to_f64(), 3.0, 3.0, 3.0)
            && !player
                .position()
                .is_within_bounds(bed_foot_pos.to_f64(), 3.0, 3.0, 3.0)
        {
            player
                .send_system_message_raw(
                    &TextComponent::translate("block.minecraft.bed.too_far_away", []),
                    true,
                )
                .await;
            return;
        }

        // Set respawn point
        if player
            .set_respawn_point(
                world.dimension_type,
                bed_head_pos,
                player.get_entity().yaw.load(),
            )
            .await
        {
            player
                .send_system_message(&TextComponent::translate("block.minecraft.set_spawn", []))
                .await;
        }

        // Make sure the time and weather allows sleep
        if !can_sleep(world).await {
            player
                .send_system_message_raw(
                    &TextComponent::translate("block.minecraft.bed.no_sleep", []),
                    true,
                )
                .await;
            return;
        }

        // Make sure there are no monsters nearby
        for entity in world.entities.read().await.values() {
            if !entity_prevents_sleep(entity.get_entity()) {
                continue;
            }

            let pos = entity.get_entity().pos.load();
            if pos.is_within_bounds(bed_head_pos.to_f64(), 8.0, 5.0, 8.0)
                || pos.is_within_bounds(bed_foot_pos.to_f64(), 8.0, 5.0, 8.0)
            {
                player
                    .send_system_message_raw(
                        &TextComponent::translate("block.minecraft.bed.not_safe", []),
                        true,
                    )
                    .await;
                return;
            }
        }

        player.sleep(bed_head_pos).await;
        Self::set_occupied(true, world, block, &block_pos, state_id).await;
    }
}

impl BedBlock {
    pub async fn set_occupied(
        occupied: bool,
        world: &Arc<World>,
        block: &Block,
        block_pos: &BlockPos,
        state_id: u16,
    ) {
        let mut bed_props = BedProperties::from_state_id(state_id, block);
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

fn entity_prevents_sleep(entity: &Entity) -> bool {
    match entity.entity_type {
        EntityType::BLAZE
        | EntityType::BOGGED
        | EntityType::SKELETON
        | EntityType::STRAY
        | EntityType::WITHER_SKELETON
        | EntityType::BREEZE
        | EntityType::CREAKING
        | EntityType::CREEPER
        | EntityType::DROWNED
        | EntityType::ENDERMITE
        | EntityType::EVOKER
        | EntityType::GIANT
        | EntityType::GUARDIAN
        | EntityType::ELDER_GUARDIAN
        | EntityType::ILLUSIONER
        | EntityType::OCELOT
        | EntityType::PIGLIN
        | EntityType::PIGLIN_BRUTE
        | EntityType::PILLAGER
        | EntityType::PHANTOM
        | EntityType::RAVAGER
        | EntityType::SILVERFISH
        | EntityType::SPIDER
        | EntityType::CAVE_SPIDER
        | EntityType::VEX
        | EntityType::VINDICATOR
        | EntityType::WARDEN
        | EntityType::WITCH
        | EntityType::WITHER
        | EntityType::ZOGLIN
        | EntityType::ZOMBIE
        | EntityType::ZOMBIE_VILLAGER
        | EntityType::HUSK => true,
        EntityType::ENDERMAN | EntityType::ZOMBIFIED_PIGLIN => {
            // TODO: Only when hostile
            #[allow(clippy::match_same_arms)]
            true
        }
        _ => false,
    }
}
