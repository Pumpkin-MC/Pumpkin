use crate::entity::EntityBase;
use pumpkin_data::damage::DamageType;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

/// A source of damage.
#[derive(Clone)]
pub struct DamageSource {
    pub damage_type: DamageType,
    /// The entity that caused the damage in the first place.
    ///
    /// For example, if a skeleton shoots an arrow, this would be the *skeleton*.
    pub causing_entity: Option<Arc<dyn EntityBase>>,
    /// The entity that directly caused the damage.
    ///
    /// For example, if a skeleton shoots an arrow, this would be the *arrow*.
    pub direct_entity: Option<Arc<dyn EntityBase>>,
    pub damage_source_pos: Option<Vector3<f64>>,
}

impl DamageSource {
    #[must_use]
    pub fn new(
        damage_type: DamageType,
        causing_entity: Option<Arc<dyn EntityBase>>,
        direct_entity: Option<Arc<dyn EntityBase>>,
        damage_source_pos: Option<Vector3<f64>>,
    ) -> Self {
        Self {
            damage_type,
            causing_entity,
            direct_entity,
            damage_source_pos,
        }
    }

    #[must_use]
    pub fn from_explosion(
        causing_entity: Option<Arc<dyn EntityBase>>,
        direct_entity: Option<Arc<dyn EntityBase>>,
    ) -> Self {
        Self::new(DamageType::EXPLOSION, causing_entity, direct_entity, None)
    }

    /// Tries to get a source position using this [`DamageSource`]'s properties.
    /// The position falls back to the entities of this source if no position
    /// is found.
    #[must_use]
    pub fn source_position(&self) -> Option<Vector3<f64>> {
        self.damage_source_pos.map_or_else(
            || {
                self.direct_entity
                    .as_ref()
                    .map(|e| e.get_entity().pos.load())
            },
            Some,
        )
    }
}
