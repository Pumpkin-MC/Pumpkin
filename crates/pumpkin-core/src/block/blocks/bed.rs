use std::sync::Arc;

use crate::block::entities::bed::BedBlockEntity;
use pumpkin_data::block_properties::BedPart;
use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::{Block as BlockTag, Taggable};
use pumpkin_data::translation;
use pumpkin_data::{Block, BlockDirection, BlockState, BlockStateId};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, NormalUseArgs,
    OnPlaceArgs, OnStateReplacedArgs, PathComputationType, PlacedArgs,
};
use crate::entity::dismount_helper;
use crate::entity::{Entity, EntityBase, player::Player};
use crate::world::World;
use pumpkin_data::HorizontalFacingExt;

type BedProperties = pumpkin_data::block_properties::WhiteBedLikeProperties;

const NO_SLEEP_IDS: &[u16] = &[
    EntityType::BLAZE.id,
    EntityType::BOGGED.id,
    EntityType::SKELETON.id,
    EntityType::STRAY.id,
    EntityType::WITHER_SKELETON.id,
    EntityType::BREEZE.id,
    EntityType::CREAKING.id,
    EntityType::CREEPER.id,
    EntityType::DROWNED.id,
    EntityType::ENDERMITE.id,
    EntityType::EVOKER.id,
    EntityType::GIANT.id,
    EntityType::GUARDIAN.id,
    EntityType::ELDER_GUARDIAN.id,
    EntityType::ILLUSIONER.id,
    EntityType::OCELOT.id,
    EntityType::PIGLIN.id,
    EntityType::PIGLIN_BRUTE.id,
    EntityType::PILLAGER.id,
    EntityType::PHANTOM.id,
    EntityType::RAVAGER.id,
    EntityType::SILVERFISH.id,
    EntityType::SPIDER.id,
    EntityType::CAVE_SPIDER.id,
    EntityType::VEX.id,
    EntityType::VINDICATOR.id,
    EntityType::WARDEN.id,
    EntityType::WITCH.id,
    EntityType::WITHER.id,
    EntityType::ZOGLIN.id,
    EntityType::ZOMBIE.id,
    EntityType::ZOMBIE_VILLAGER.id,
    EntityType::HUSK.id,
    EntityType::ENDERMAN.id,
    EntityType::ZOMBIFIED_PIGLIN.id,
];

#[pumpkin_block_from_tag("minecraft:beds")]
pub struct BedBlock;

impl BlockBehaviour for BedBlock {
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
        true
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut bed_props = BedProperties::default(args.block);

        bed_props.facing = args.player.get_entity().get_horizontal_facing();
        bed_props.part = BedPart::Foot;

        bed_props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        {
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
            // Breaking foot in survival -> allow head to drop
            BlockFlags::NOTIFY_ALL
        } else {
            // Breaking head OR creative mode -> skip drops
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

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let mut bed_props = BedProperties::from_state_id(args.state_id);
        let neighbor_dir = if bed_props.part == BedPart::Foot {
            bed_props.facing.to_block_direction()
        } else {
            bed_props.facing.opposite().to_block_direction()
        };
        if args.direction == neighbor_dir {
            let neighbor_block = args.neighbor_state_id.to_block();
            if neighbor_block == args.block {
                let neighbor_props = BedProperties::from_state_id(args.neighbor_state_id);
                if neighbor_props.part != bed_props.part {
                    bed_props.occupied = neighbor_props.occupied;
                    return bed_props.to_state_id(args.block);
                }
            }
            return Block::AIR.default_state.id;
        }
        args.state_id
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        Self::use_bed(args.world, args.player, args.block, args.position)
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

impl BedBlock {
    #[expect(clippy::too_many_lines)]
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

        // Explode if bed rule explodes (EnvironmentAttributes.BED_RULE)
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

        let is_dark = world.is_dark_outside();
        let can_sleep = world.dimension.bed_rule.can_sleep(is_dark);
        let can_set_spawn = world.dimension.bed_rule.can_set_spawn(is_dark);

        if !can_set_spawn && !can_sleep {
            player.send_system_message_raw(
                &pumpkin_macros::translate_cross!(
                    translation::java::BLOCK_MINECRAFT_BED_NO_SLEEP,
                    translation::bedrock::TILE_BED_NOSLEEP
                ),
                true,
            );
            return BlockActionResult::SuccessServer;
        }

        // Make sure the bed is not obstructed
        if world.get_block_state(&bed_head_pos.up()).is_solid()
            || world.get_block_state(&bed_foot_pos.up()).is_solid()
        {
            player.send_system_message_raw(
                &pumpkin_macros::translate_cross!(
                    translation::java::BLOCK_MINECRAFT_BED_OBSTRUCTED,
                    translation::bedrock::TILE_BED_OBSTRUCTED
                ),
                true,
            );
            return BlockActionResult::SuccessServer;
        }

        // Make sure the bed is not occupied
        if bed_props.occupied {
            // TODO: Wake up villager

            player.send_system_message_raw(
                &pumpkin_macros::translate_cross!(
                    translation::java::BLOCK_MINECRAFT_BED_OCCUPIED,
                    translation::bedrock::TILE_BED_OCCUPIED
                ),
                true,
            );
            return BlockActionResult::SuccessServer;
        }

        // Make sure player is close enough
        if !player
            .position()
            .is_within_bounds(bed_head_pos.to_f64(), 3.0, 3.0, 3.0)
            && !player
                .position()
                .is_within_bounds(bed_foot_pos.to_f64(), 3.0, 3.0, 3.0)
        {
            player.send_system_message_raw(
                &pumpkin_macros::translate_cross!(
                    translation::java::BLOCK_MINECRAFT_BED_TOO_FAR_AWAY,
                    translation::bedrock::TILE_BED_TOOFAR
                ),
                true,
            );
            return BlockActionResult::SuccessServer;
        }

        // Set respawn point
        if can_set_spawn
            && player.set_respawn_point(
                world.dimension.clone(),
                bed_head_pos,
                player.get_entity().yaw.load(),
                player.get_entity().pitch.load(),
                false,
            )
        {
            player.send_system_message(&pumpkin_macros::translate_cross!(
                translation::java::BLOCK_MINECRAFT_SET_SPAWN,
                translation::bedrock::TILE_BED_RESPAWNSET
            ));
        }

        // Make sure the time and weather allows sleep
        if !can_sleep {
            player.send_system_message_raw(
                &pumpkin_macros::translate_cross!(
                    translation::java::BLOCK_MINECRAFT_BED_NO_SLEEP,
                    translation::bedrock::TILE_BED_NOSLEEP
                ),
                true,
            );
            return BlockActionResult::SuccessServer;
        }

        // Make sure there are no monsters nearby
        for entity in world.entities.load().iter() {
            if !entity_prevents_sleep(entity.get_entity())
                || entity
                    .get_mob()
                    .is_some_and(|mob| !mob.is_preventing_player_rest(player.as_ref()))
            {
                continue;
            }

            let pos = entity.get_entity().pos.load();
            if pos.is_within_bounds(bed_head_pos.to_f64(), 8.0, 5.0, 8.0)
                || pos.is_within_bounds(bed_foot_pos.to_f64(), 8.0, 5.0, 8.0)
            {
                player.send_system_message_raw(
                    &pumpkin_macros::translate_cross!(
                        translation::java::BLOCK_MINECRAFT_BED_NOT_SAFE,
                        translation::bedrock::TILE_BED_NOTSAFE
                    ),
                    true,
                );
                return BlockActionResult::SuccessServer;
            }
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
            pumpkin_data::statistic::CustomStatistic::SleepInBed as i32,
            1,
        );
        Self::set_occupied(true, world, block, position, state_id);

        BlockActionResult::SuccessServer
    }
}

impl BedBlock {
    pub fn find_stand_up_position(
        entity_type: &'static EntityType,
        world: &World,
        pos: &BlockPos,
        forward: BlockDirection,
        yaw: f32,
    ) -> Option<Vector3<f64>> {
        let right = forward.rotate_clockwise();
        let side = if is_facing_angle(right, yaw) {
            right.opposite()
        } else {
            right
        };

        if is_bunk_bed(world, pos) {
            find_bunk_bed_stand_up_position(entity_type, world, pos, forward, side)
        } else {
            let offsets = bed_stand_up_offsets(forward, side);
            find_stand_up_position_at_offset(entity_type, world, pos, &offsets, true).or_else(
                || find_stand_up_position_at_offset(entity_type, world, pos, &offsets, false),
            )
        }
    }

    pub fn set_occupied(
        occupied: bool,
        world: &Arc<World>,
        block: &Block,
        block_pos: &BlockPos,
        state_id: BlockStateId,
    ) {
        let mut bed_props = BedProperties::from_state_id(state_id);
        bed_props.occupied = occupied;
        world.set_block_state(
            block_pos,
            bed_props.to_state_id(block),
            BlockFlags::NOTIFY_LISTENERS,
        );

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
        world.set_block_state(
            &other_half_pos,
            bed_props.to_state_id(block),
            BlockFlags::NOTIFY_LISTENERS,
        );
    }
}

fn find_bunk_bed_stand_up_position(
    entity_type: &'static EntityType,
    world: &World,
    pos: &BlockPos,
    forward: BlockDirection,
    side: BlockDirection,
) -> Option<Vector3<f64>> {
    let offsets = bed_surround_stand_up_offsets(forward, side);
    if let Some(position) =
        find_stand_up_position_at_offset(entity_type, world, pos, &offsets, true)
    {
        return Some(position);
    }

    let below = pos.down();
    if let Some(position) =
        find_stand_up_position_at_offset(entity_type, world, &below, &offsets, true)
    {
        return Some(position);
    }

    let above_offsets = bed_above_stand_up_offsets(forward);
    if let Some(position) =
        find_stand_up_position_at_offset(entity_type, world, pos, &above_offsets, true)
    {
        return Some(position);
    }

    if let Some(position) =
        find_stand_up_position_at_offset(entity_type, world, pos, &offsets, false)
    {
        return Some(position);
    }

    find_stand_up_position_at_offset(entity_type, world, &below, &offsets, false).or_else(|| {
        find_stand_up_position_at_offset(entity_type, world, pos, &above_offsets, false)
    })
}

fn find_stand_up_position_at_offset(
    entity_type: &'static EntityType,
    world: &World,
    pos: &BlockPos,
    offsets: &[(i32, i32)],
    check_dangerous: bool,
) -> Option<Vector3<f64>> {
    offsets.iter().find_map(|(dx, dz)| {
        let check_pos = BlockPos(Vector3::new(pos.0.x + dx, pos.0.y, pos.0.z + dz));
        dismount_helper::find_safe_dismount_location(
            entity_type,
            world,
            &check_pos,
            check_dangerous,
        )
    })
}

fn is_bunk_bed(world: &World, pos: &BlockPos) -> bool {
    world
        .get_block(&pos.down())
        .has_tag(&BlockTag::MINECRAFT_BEDS)
}

fn is_facing_angle(direction: BlockDirection, yaw: f32) -> bool {
    let radians = yaw.to_radians();
    let dx = -radians.sin();
    let dz = radians.cos();
    let offset = direction.to_offset();
    offset.x as f32 * dx + offset.z as f32 * dz > 0.0
}

fn bed_stand_up_offsets(forward: BlockDirection, side: BlockDirection) -> Vec<(i32, i32)> {
    let mut offsets = bed_surround_stand_up_offsets(forward, side);
    offsets.extend(bed_above_stand_up_offsets(forward));
    offsets
}

fn bed_surround_stand_up_offsets(forward: BlockDirection, side: BlockDirection) -> Vec<(i32, i32)> {
    let (fx, fz) = step(forward);
    let (sx, sz) = step(side);
    vec![
        (sx, sz),
        (sx - fx, sz - fz),
        (sx - fx * 2, sz - fz * 2),
        (-fx * 2, -fz * 2),
        (-sx - fx * 2, -sz - fz * 2),
        (-sx - fx, -sz - fz),
        (-sx, -sz),
        (-sx + fx, -sz + fz),
        (fx, fz),
        (sx + fx, sz + fz),
    ]
}

fn bed_above_stand_up_offsets(forward: BlockDirection) -> Vec<(i32, i32)> {
    let (fx, fz) = step(forward);
    vec![(0, 0), (-fx, -fz)]
}

fn step(direction: BlockDirection) -> (i32, i32) {
    let offset = direction.to_offset();
    (offset.x, offset.z)
}

fn entity_prevents_sleep(entity: &Entity) -> bool {
    NO_SLEEP_IDS.contains(&entity.entity_type.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stand_up_offset_order_matches_vanilla() {
        assert_eq!(
            bed_stand_up_offsets(BlockDirection::North, BlockDirection::East),
            vec![
                (1, 0),
                (1, 1),
                (1, 2),
                (0, 2),
                (-1, 2),
                (-1, 1),
                (-1, 0),
                (-1, -1),
                (0, -1),
                (1, -1),
                (0, 0),
                (0, 1),
            ]
        );
    }
}
