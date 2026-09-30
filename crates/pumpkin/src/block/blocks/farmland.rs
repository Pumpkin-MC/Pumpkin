use std::sync::Arc;

use crate::block::{
    BlockBehaviour, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnLandedUponArgs, OnPlaceArgs,
    OnScheduledTickArgs, PathComputationType, RandomTickArgs,
};
use crate::world::World;
use pumpkin_data::block_properties::FarmlandLikeProperties;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, BlockState, BlockStateId};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::BlockFlags;

type FarmlandProperties = FarmlandLikeProperties;

#[pumpkin_block("minecraft:farmland")]
pub struct FarmlandBlock;

impl BlockBehaviour for FarmlandBlock {
    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        push_entities_up(args.world, args.position);
        args.world.set_block_state(
            args.position,
            Block::DIRT.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );
    }

    fn on_landed_upon(&self, args: OnLandedUponArgs<'_>) {
        if let Some(living) = args.entity.get_living_entity() {
            living.handle_fall_damage(args.entity, args.fall_distance, 1.0);
        }

        let is_player = args.entity.get_player().is_some();
        let mob_griefing = args.world.level_info.load().game_rules.mob_griefing;
        if is_player || mob_griefing {
            let dims = args.entity.get_entity().entity_dimension.load();
            let volume = dims.width * dims.width * dims.height;
            if volume > 0.512 && args.fall_distance > 0.5 {
                let rand_val: f32 = rand::random();
                if rand_val < (args.fall_distance - 0.5) {
                    push_entities_up(args.world, args.position);
                    let current_pos = args.entity.get_entity().pos.load();
                    let top_y = f64::from(args.position.0.y) + 1.0;
                    if current_pos.y < top_y {
                        let new_pos = Vector3::new(current_pos.x, top_y, current_pos.z);
                        args.entity
                            .teleport(new_pos, None, None, args.world.clone());
                    }
                    args.world.set_block_state(
                        args.position,
                        Block::DIRT.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    );
                }
            }
        }
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

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        let is_hydrated = is_water_nearby(args.world, args.position)
            || args.world.is_raining_at(&args.position.up());

        let state_id = args.world.get_block_state_id(args.position);
        let mut props = FarmlandProperties::from_state_id(state_id);

        if is_hydrated {
            if props.moisture < 7 {
                let mut new_moisture = 7;
                if let Some(server) = args.world.server.upgrade() {
                    let mut event =
                        crate::plugin::api::events::block::moisture_change::MoistureChangeEvent::new(
                            *args.position,
                            args.world.clone(),
                            new_moisture,
                        );
                    server.plugin_manager.fire_blocking(&server, &mut event);
                    if event.cancelled {
                        return;
                    }
                    new_moisture = event.new_moisture;
                }
                props.moisture = new_moisture.clamp(0, 7) as u8;
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                );
            }
        } else if props.moisture > 0 {
            let mut new_moisture = (props.moisture as i32 - 1).clamp(0, 7);
            if let Some(server) = args.world.server.upgrade() {
                let mut event =
                    crate::plugin::api::events::block::moisture_change::MoistureChangeEvent::new(
                        *args.position,
                        args.world.clone(),
                        new_moisture,
                    );
                server.plugin_manager.fire_blocking(&server, &mut event);
                if event.cancelled {
                    return;
                }
                new_moisture = event.new_moisture;
            }
            props.moisture = new_moisture.clamp(0, 7) as u8;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );
        } else if !args
            .world
            .get_block(&args.position.up())
            .has_tag(&tag::Block::MINECRAFT_MAINTAINS_FARMLAND)
        {
            push_entities_up(args.world, args.position);
            args.world.set_block_state(
                args.position,
                Block::DIRT.default_state.id,
                BlockFlags::NOTIFY_ALL,
            );
        }
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

fn push_entities_up(world: &Arc<World>, pos: &BlockPos) {
    let top_y = f64::from(pos.0.y) + 1.0;
    let check_box = BoundingBox {
        min: Vector3::new(
            f64::from(pos.0.x),
            f64::from(pos.0.y) + 0.8,
            f64::from(pos.0.z),
        ),
        max: Vector3::new(f64::from(pos.0.x) + 1.0, top_y, f64::from(pos.0.z) + 1.0),
    };
    let entities = world.get_all_at_box(&check_box);
    for entity in entities {
        let current_pos = entity.get_entity().pos.load();
        if current_pos.y < top_y {
            let new_pos = Vector3::new(current_pos.x, top_y, current_pos.z);
            entity.teleport(new_pos, None, None, world.clone());
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
                if world
                    .get_fluid(&check_pos)
                    .has_tag(&tag::Fluid::MINECRAFT_WATER)
                {
                    return true;
                }
            }
        }
    }
    false
}
