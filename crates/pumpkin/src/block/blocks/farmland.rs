use std::sync::Arc;

use crate::block::BlockBehaviour;
use crate::block::CanPlaceAtArgs;
use crate::block::GetStateForNeighborUpdateArgs;
use crate::block::OnLandedUponArgs;
use crate::block::OnPlaceArgs;
use crate::block::OnScheduledTickArgs;
use crate::block::RandomTickArgs;
use crate::world::World;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::FarmlandLikeProperties;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

type FarmlandProperties = FarmlandLikeProperties;

#[pumpkin_block("minecraft:farmland")]
pub struct FarmlandBlock;

impl BlockBehaviour for FarmlandBlock {
    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        // TODO: push up entities
        args.world.set_block_state(
            args.position,
            Block::DIRT.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if !can_place_at(args.world, args.position) {
            return Block::DIRT.default_state.id;
        }
        args.block.default_state.id
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.direction == BlockDirection::Up && !can_place_at(args.world, args.position) {
            args.world
                .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
        }
        args.state_id
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.block_accessor, args.position)
    }

    fn on_landed_upon(&self, args: OnLandedUponArgs<'_>) {
        let entity = args.entity.get_entity();
        let dimensions = entity.entity_dimension.load();
        // Small entities (width * width * height <= 0.512) never trample
        if rand::rng().random::<f32>() < args.fall_distance - 0.5
            && args.entity.get_living_entity().is_some()
            && (args.entity.get_player().is_some()
                || args.world.level_info.load().game_rules.mob_griefing)
            && dimensions.width * dimensions.width * dimensions.height > 0.512
        {
            let position = entity.get_pos_with_y_offset(0.2).0;
            args.world.set_block_state(
                &position,
                Block::DIRT.default_state.id,
                BlockFlags::NOTIFY_ALL,
            );

            // Vanilla `turnToDirt` pushes entities up: dirt is a full block
            // while farmland is one pixel lower, so without this the
            // trampler ends up inside the new block and falls through it.
            let top = f64::from(position.0.y) + 1.0;
            let entity_pos = entity.pos.load();
            if entity_pos.y < top {
                if let Some(player) = args.entity.get_player() {
                    player.request_relative_teleport(Vector3::new(0.0, top - entity_pos.y, 0.0));
                } else {
                    entity.set_pos(Vector3::new(entity_pos.x, top, entity_pos.z));
                }
            }
        }

        if let Some(living) = args.entity.get_living_entity() {
            living.handle_fall_damage(args.entity, args.fall_distance, 1.0);
        }
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        // TODO: add rain check. Remember to check which one is most optimized.
        if is_water_nearby(args.world, args.position) {
            let mut props = FarmlandProperties::default(args.block);
            props.moisture = 7;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_NEIGHBORS,
            );
        } else {
            let state_id = args.world.get_block_state_id(args.position);
            let mut props = FarmlandProperties::from_state_id(state_id, args.block);
            if props.moisture == 0 {
                if !args
                    .world
                    .get_block(&args.position.up())
                    .has_tag(&tag::Block::MINECRAFT_MAINTAINS_FARMLAND)
                {
                    //TODO push entities up
                    args.world.set_block_state(
                        args.position,
                        Block::DIRT.default_state.id,
                        BlockFlags::NOTIFY_NEIGHBORS,
                    );
                }
            } else {
                props.moisture = (props.moisture as i32 - 1).clamp(0, 7) as u8;
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_NEIGHBORS,
                );
            }
        }
    }
}

fn can_place_at(world: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
    let state = world.get_block_state(&block_pos.up());
    !state.is_solid() // TODO: add fence gate block
}

fn is_water_nearby(world: &Arc<World>, block_pos: &BlockPos) -> bool {
    for dx in -4..=4 {
        for dy in 0..=1 {
            for dz in -4..=4 {
                let check_pos = block_pos.offset(Vector3 {
                    x: dx,
                    y: dy,
                    z: dz,
                });
                //TODO this should use tag water. It does not seem to work rn.
                if world.get_block(&check_pos) == &Block::WATER {
                    return true;
                }
            }
        }
    }
    false
}
