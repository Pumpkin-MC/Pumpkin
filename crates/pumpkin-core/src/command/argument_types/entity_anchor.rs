use crate::entity::Entity;
pub use pumpkin_command::argument_types::entity_anchor::*;
use pumpkin_util::math::vector3::Vector3;

pub trait EntityAnchorExt {
    fn transform_position(self, position: Vector3<f64>, entity: &Entity) -> Vector3<f64>;
    fn position_at_entity(self, entity: &Entity) -> Vector3<f64>;
}

impl EntityAnchorExt for EntityAnchor {
    fn transform_position(self, position: Vector3<f64>, entity: &Entity) -> Vector3<f64> {
        match self {
            Self::Feet => position,
            Self::Eyes => position.add(&Vector3::new(0.0, entity.get_eye_height(), 0.0)),
        }
    }

    fn position_at_entity(self, entity: &Entity) -> Vector3<f64> {
        self.transform_position(entity.pos.load(), entity)
    }
}
