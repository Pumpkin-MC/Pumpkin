use std::sync::Arc;

use crate::block::entities::bed::BedBlockEntity;
use pumpkin_data::block_properties::BedPart;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::translation;
use pumpkin_data::{Block, BlockState, BlockStateId};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_world::world::BlockFlags;

use crate::block::OnLandedUponArgs;
use crate::block::UpdateEntityMovementAfterFallOnArgs;
use crate::block::bounce_entity_after_fall;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BrokenArgs, CanPlaceAtArgs, NormalUseArgs, OnPlaceArgs, OnStateReplacedArgs,
    PathComputationType, PlacedArgs,
};
use crate::entity::{EntityBase, player::Player};
use crate::world::World;

type BedProperties = pumpkin_data::block_properties::WhiteBedLikeProperties;

const NO_SLEEP_IDS: &[u16] = &[
    pumpkin_data::entity::EntityType::BLAZE.id,
    pumpkin_data::entity::EntityType::BOGGED.id,
    pumpkin_data::entity::EntityType::SKELETON.id,
    pumpkin_data::entity::EntityType::STRAY.id,
    pumpkin_data::entity::EntityType::WITHER_SKELETON.id,
    pumpkin_data::entity::EntityType::BREEZE.id,
    pumpkin_data::entity::EntityType::CREAKING.id,
    pumpkin_data::entity::EntityType::CREEPER.id,
    pumpkin_data::entity::EntityType::DROWNED.id,
    pumpkin_data::entity::EntityType::ENDERMITE.id,
    pumpkin_data::entity::EntityType::EVOKER.id,
    pumpkin_data::entity::EntityType::GIANT.id,
    pumpkin_data::entity::EntityType::GUARDIAN.id,
    pumpkin_data::entity::EntityType::ELDER_GUARDIAN.id,
    pumpkin_data::entity::EntityType::ILLUSIONER.id,
    pumpkin_data::entity::EntityType::OCELOT.id,
    pumpkin_data::entity::EntityType::PIGLIN.id,
    pumpkin_data::entity::EntityType::PIGLIN_BRUTE.id,
    pumpkin_data::entity::EntityType::PILLAGER.id,
    pumpkin_data::entity::EntityType::PHANTOM.id,
    pumpkin_data::entity::EntityType::RAVAGER.id,
    pumpkin_data::entity::EntityType::SILVERFISH.id,
    pumpkin_data::entity::EntityType::SPIDER.id,
    pumpkin_data::entity::EntityType::CAVE_SPIDER.id,
    pumpkin_data::entity::EntityType::VEX.id,
    pumpkin_data::entity::EntityType::VINDICATOR.id,
    pumpkin_data::entity::EntityType::WARDEN.id,
    pumpkin_data::entity::EntityType::WITCH.id,
    pumpkin_data::entity::EntityType::WITHER.id,
    pumpkin_data::entity::EntityType::ZOGLIN.id,
    pumpkin_data::entity::EntityType::ZOMBIE.id,
    pumpkin_data::entity::EntityType::ZOMBIE_VILLAGER.id,
    pumpkin_data::entity::EntityType::HUSK.id,
    pumpkin_data::entity::EntityType::ENDERMAN.id,
    pumpkin_data::entity::EntityType::ZOMBIFIED_PIGLIN.id,
];

#[pumpkin_block("minecraft:straw_bed")]
pub struct StrawBedBlock;

impl BlockBehaviour for StrawBedBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        if let Some(player) = args.player {
            let facing = player.get_entity().get_horizontal_facing();
            return args
                .block_accessor
                .get_block_state(args.position)
                .replaceable()
                && args
                    .block_accessor
                    .get_block_state(&args.position.offset(facing.to_offset()))
                    .replaceable();
        }
        false
    }

    fn on_landed_upon(&self, args: OnLandedUponArgs<'_>) {
        if let Some(living) = args.entity.get_living_entity() {
            living.handle_fall_damage(args.entity, args.fall_distance * 0.5, 1.0);
        }
    }

    fn update_entity_movement_after_fall_on(&self, args: UpdateEntityMovementAfterFallOnArgs<'_>) {
        bounce_entity_after_fall(args.entity, 0.66);
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut bed_props = BedProperties::default(args.block);
        bed_props.facing = args.player.get_entity().get_horizontal_facing();
        bed_props.part = BedPart::Foot;
        bed_props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let bed_entity = BedBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(bed_entity));

        let mut bed_head_props = BedProperties::default(args.block);
        bed_head_props.facing = BedProperties::from_state_id(args.state_id).facing;
        bed_head_props.part = BedPart::Head;

        let bed_head_pos = args.position.offset(bed_head_props.facing.to_offset());
        args.world.set_block_state(
            &bed_head_pos,
            bed_head_props.to_state_id(args.block),
            BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
        );

        let bed_head_entity = BedBlockEntity::new(bed_head_pos);
        args.world.add_block_entity(Arc::new(bed_head_entity));
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        let bed_props = BedProperties::from_state_id(args.state.id);
        let other_half_pos = if bed_props.part == BedPart::Head {
            args.position
                .offset(bed_props.facing.opposite().to_offset())
        } else {
            args.position.offset(bed_props.facing.to_offset())
        };
        let neighbor_state_id = args.world.get_block_state_id(&other_half_pos);
        if neighbor_state_id.to_block_id() != args.block.id {
            args.world.update_neighbors(&other_half_pos, None);
            return;
        }

        let is_creative = args.player.gamemode.load() == GameMode::Creative;
        let flags = if bed_props.part == BedPart::Foot && !is_creative {
            BlockFlags::NOTIFY_ALL
        } else {
            BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_ALL
        };

        args.world
            .break_block(&other_half_pos, Some(args.player), flags);
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        if args.moved {
            return;
        }

        let bed_props = BedProperties::from_state_id(args.old_state_id);
        let other_half_pos = if bed_props.part == BedPart::Head {
            args.position
                .offset(bed_props.facing.opposite().to_offset())
        } else {
            args.position.offset(bed_props.facing.to_offset())
        };

        let (other_block, other_state) = args.world.get_block_and_state(&other_half_pos);
        if other_block == args.block {
            let other_props = BedProperties::from_state_id(other_state.id);
            if other_props.part != bed_props.part {
                args.world.break_block(
                    &other_half_pos,
                    None,
                    BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_ALL,
                );
            }
        }
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        Self::use_bed(args.world, args.player, args.block, args.position)
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

impl StrawBedBlock {
    pub fn destroy_after_use(world: &Arc<World>, bed_head_pos: BlockPos) {
        let (block, state) = world.get_block_and_state_id(&bed_head_pos);
        if block != &Block::STRAW_BED {
            return;
        }
        let bed_props = BedProperties::from_state_id(state);
        let (head_pos, foot_pos) = if bed_props.part == BedPart::Head {
            (
                bed_head_pos,
                bed_head_pos.offset(bed_props.facing.opposite().to_offset()),
            )
        } else {
            (
                bed_head_pos.offset(bed_props.facing.to_offset()),
                bed_head_pos,
            )
        };
        world.play_block_sound(
            Sound::BlockStrawBedBreakLeave,
            SoundCategory::Blocks,
            head_pos,
        );
        world.break_block(
            &head_pos,
            None,
            BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_ALL,
        );
        world.break_block(
            &foot_pos,
            None,
            BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_ALL,
        );
    }

    /// Returns the message to show the player when they cannot sleep here, or
    /// `None` if the bed is usable.
    fn sleep_refusal(
        world: &Arc<World>,
        player: &Arc<Player>,
        occupied: bool,
        bed_head_pos: BlockPos,
        bed_foot_pos: BlockPos,
    ) -> Option<TextComponent> {
        if !world.dimension.bed_rule.can_sleep(world.is_dark_outside()) {
            return Some(pumpkin_macros::translate_cross!(
                translation::java::BLOCK_MINECRAFT_BED_NO_SLEEP,
                translation::bedrock::TILE_BED_NOSLEEP
            ));
        }

        if world.get_block_state(&bed_head_pos.up()).is_solid()
            || world.get_block_state(&bed_foot_pos.up()).is_solid()
        {
            return Some(pumpkin_macros::translate_cross!(
                translation::java::BLOCK_MINECRAFT_BED_OBSTRUCTED,
                translation::bedrock::TILE_BED_OBSTRUCTED
            ));
        }

        if occupied {
            return Some(pumpkin_macros::translate_cross!(
                translation::java::BLOCK_MINECRAFT_BED_OCCUPIED,
                translation::bedrock::TILE_BED_OCCUPIED
            ));
        }

        if !player
            .position()
            .is_within_bounds(bed_head_pos.to_f64(), 3.0, 3.0, 3.0)
            && !player
                .position()
                .is_within_bounds(bed_foot_pos.to_f64(), 3.0, 3.0, 3.0)
        {
            return Some(pumpkin_macros::translate_cross!(
                translation::java::BLOCK_MINECRAFT_BED_TOO_FAR_AWAY,
                translation::bedrock::TILE_BED_TOOFAR
            ));
        }

        for entity in world.entities.load().iter() {
            if !NO_SLEEP_IDS.contains(&entity.get_entity().entity_type.id) {
                continue;
            }
            let pos = entity.get_entity().pos.load();
            if pos.is_within_bounds(bed_head_pos.to_f64(), 8.0, 5.0, 8.0)
                || pos.is_within_bounds(bed_foot_pos.to_f64(), 8.0, 5.0, 8.0)
            {
                return Some(pumpkin_macros::translate_cross!(
                    translation::java::BLOCK_MINECRAFT_BED_NOT_SAFE,
                    translation::bedrock::TILE_BED_NOTSAFE
                ));
            }
        }

        None
    }

    fn use_bed(
        world: &Arc<World>,
        player: &Arc<Player>,
        block: &Block,
        position: &BlockPos,
    ) -> BlockActionResult {
        let state_id = world.get_block_state_id(position);
        let bed_props = BedProperties::from_state_id(state_id);

        let (bed_head_pos, bed_foot_pos) = if bed_props.part == BedPart::Head {
            (
                *position,
                position.offset(bed_props.facing.opposite().to_offset()),
            )
        } else {
            (position.offset(bed_props.facing.to_offset()), *position)
        };

        if world.dimension.bed_rule.explodes {
            world.break_block(&bed_head_pos, None, BlockFlags::SKIP_DROPS);
            world.break_block(&bed_foot_pos, None, BlockFlags::SKIP_DROPS);
            world.explode(
                bed_head_pos.to_centered_f64(),
                5.0,
                crate::world::ExplosionInteraction::Block,
            );
            return BlockActionResult::SuccessServer;
        }

        if let Some(message) = Self::sleep_refusal(
            world,
            player,
            bed_props.occupied,
            bed_head_pos,
            bed_foot_pos,
        ) {
            player.send_system_message_raw(&message, true);
            return BlockActionResult::SuccessServer;
        }

        if let Some(server) = world.server.upgrade() {
            let mut event =
                crate::plugin::api::events::player::player_bed::PlayerBedEnterEvent::new(
                    player.clone(),
                    bed_head_pos,
                );
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                return BlockActionResult::SuccessServer;
            }
        }

        player.sleep(bed_head_pos);
        player.trigger_advancement(
            crate::entity::player::advancement::trigger::AdvancementTrigger::SleptInBed,
        );
        player.increment_stat(
            pumpkin_data::statistic::StatisticCategory::Custom,
            pumpkin_data::statistic::CustomStatistic::SleepInStrawBed as i32,
            1,
        );
        crate::block::blocks::bed::BedBlock::set_occupied(true, world, block, position, state_id);

        BlockActionResult::SuccessServer
    }
}
