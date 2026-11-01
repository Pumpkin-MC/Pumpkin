use std::sync::Arc;

use pumpkin_data::{BlockState, entity::EntityType};
use pumpkin_util::math::{
    boundingbox::{BoundingBox, EntityDimensions},
    position::BlockPos,
    vector3::Vector3,
};
use rand::RngExt;
use uuid::Uuid;

use crate::entity::{Entity, EntityBase, mob::Mob};
use crate::world::World;

#[derive(Clone, Copy)]
pub enum SpawnStrategy {
    OnTopOfCollider,
}

impl SpawnStrategy {
    fn can_spawn_on(self, state: &BlockState, above_state: &BlockState) -> bool {
        match self {
            Self::OnTopOfCollider => {
                above_state.get_block_collision_shapes().next().is_none() && is_face_full_up(state)
            }
        }
    }
}

fn is_face_full_up(state: &BlockState) -> bool {
    state.is_full_cube()
        || state.get_block_collision_shapes().any(|shape| {
            shape.max.y >= 1.0
                && shape.min.x <= 0.0
                && shape.min.z <= 0.0
                && shape.max.x >= 1.0
                && shape.max.z >= 1.0
        })
}

#[expect(clippy::too_many_arguments)]
pub fn try_spawn_mob<T: Mob + 'static>(
    entity_type: &'static EntityType,
    create: fn(Entity) -> Arc<T>,
    world: &Arc<World>,
    start: &BlockPos,
    spawn_attempts: i32,
    spawn_range_xz: i32,
    spawn_range_y: i32,
    strategy: SpawnStrategy,
    check_collisions: bool,
) -> Option<Arc<T>> {
    for _ in 0..spawn_attempts {
        let (dx, dz) = {
            let mut random = rand::rng();
            (
                random.random_range(-spawn_range_xz..=spawn_range_xz),
                random.random_range(-spawn_range_xz..=spawn_range_xz),
            )
        };
        let search_pos = start.add(dx, spawn_range_y, dz);
        let in_border = world
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains_block(search_pos.0.x, search_pos.0.z);
        if !in_border {
            continue;
        }
        let Some(spawn_pos) =
            move_to_possible_spawn_position(world, spawn_range_y, search_pos, strategy)
        else {
            continue;
        };

        let position = Vector3::new(
            f64::from(spawn_pos.0.x) + 0.5,
            f64::from(spawn_pos.0.y),
            f64::from(spawn_pos.0.z) + 0.5,
        );
        if check_collisions {
            let dimensions = EntityDimensions::new(
                entity_type.dimension[0],
                entity_type.dimension[1],
                entity_type.eye_height,
            );
            let bounding_box =
                BoundingBox::new_from_pos(position.x, position.y, position.z, &dimensions);
            if !world.is_space_empty(bounding_box) {
                continue;
            }
        }

        let mob = create(Entity::from_uuid(
            Uuid::new_v4(),
            world.clone(),
            position,
            entity_type,
        ));
        if !mob.check_spawn_obstruction(world) {
            continue;
        }
        world.spawn_entity(mob.clone() as Arc<dyn EntityBase>);
        return Some(mob);
    }

    None
}

fn move_to_possible_spawn_position(
    world: &World,
    spawn_range_y: i32,
    mut search_pos: BlockPos,
    strategy: SpawnStrategy,
) -> Option<BlockPos> {
    let mut above_state = world.get_block_state(&search_pos);

    for _ in -spawn_range_y..=spawn_range_y {
        search_pos = search_pos.down();
        let current_state = world.get_block_state(&search_pos);
        if strategy.can_spawn_on(current_state, above_state) {
            return Some(search_pos.up());
        }
        above_state = current_state;
    }

    None
}
