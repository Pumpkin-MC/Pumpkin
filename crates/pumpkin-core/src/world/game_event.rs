use pumpkin_data::game_event::GameEvent;
use pumpkin_util::math::{
    boundingbox::BoundingBox,
    position::{BlockPos, chunk_section_from_pos},
    vector3::Vector3,
};
use pumpkin_world::generation::section_coords::section_to_block;

use super::{World, entity_islands};
use crate::entity::EntityBase;

pub struct GameEventContext<'a> {
    pub source_entity: Option<&'a dyn EntityBase>,
}

pub trait GameEventListener {
    fn listener_position(&self) -> Vector3<f64>;

    fn listener_radius(&self) -> i32;

    fn handle_game_event(
        &self,
        world: &World,
        event: GameEvent,
        context: &GameEventContext<'_>,
        source_position: Vector3<f64>,
    ) -> bool;
}

impl World {
    pub(super) fn post_game_event(
        &self,
        event: GameEvent,
        position: Vector3<f64>,
        context: &GameEventContext<'_>,
    ) {
        let radius = event.notification_radius();
        let center = BlockPos::floored_v(position);
        let section_min =
            chunk_section_from_pos(&center.offset(Vector3::new(-radius, -radius, -radius)));
        let section_max =
            chunk_section_from_pos(&center.offset(Vector3::new(radius, radius, radius)));
        let block_min = |section: i32| f64::from(section_to_block(section));
        let area = BoundingBox::new(
            Vector3::new(
                block_min(section_min.x),
                block_min(section_min.y),
                block_min(section_min.z),
            ),
            Vector3::new(
                block_min(section_max.x + 1),
                block_min(section_max.y + 1),
                block_min(section_max.z + 1),
            ),
        );
        for entity in self.entity_index.query(self.entities.load_full(), area) {
            let Some(mob) = entity.get_mob() else {
                continue;
            };
            if !self
                .level
                .is_chunk_loaded(&entity.get_entity().chunk_pos.load())
            {
                continue;
            }
            mob.for_each_game_event_listener(&mut |listener| {
                let listener_block = BlockPos::floored_v(listener.listener_position());
                let section = chunk_section_from_pos(&listener_block);
                let listener_radius = listener.listener_radius();
                if section.x < section_min.x
                    || section.y < section_min.y
                    || section.z < section_min.z
                    || section.x > section_max.x
                    || section.y > section_max.y
                    || section.z > section_max.z
                    || listener_block.squared_distance(&center) > listener_radius * listener_radius
                {
                    return;
                }
                entity_islands::check_reached(entity.get_entity().entity_id);
                listener.handle_game_event(self, event, context, position);
            });
        }
    }
}
