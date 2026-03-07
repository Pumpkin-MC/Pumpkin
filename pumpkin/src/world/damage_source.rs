use crate::entity::EntityBase;
use crate::world::World;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
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
        Self::new(
            if direct_entity.is_some() && causing_entity.is_some() {
                DamageType::PLAYER_EXPLOSION
            } else {
                DamageType::EXPLOSION
            },
            direct_entity,
            causing_entity,
            None,
        )
    }

    #[must_use]
    pub fn from_explosion_direct(
        world: &World,
        direct_entity: Option<Arc<dyn EntityBase>>,
    ) -> Self {
        Self::from_explosion(
            direct_entity.clone(),
            Self::indirect_source_entity(direct_entity, world),
        )
    }

    /// Gets the indirect source entity from a direct one.
    fn indirect_source_entity(
        source_entity: Option<Arc<dyn EntityBase>>,
        world: &World,
    ) -> Option<Arc<dyn EntityBase>> {
        source_entity.and_then(|e| {
            if e.get_entity().entity_type == &EntityType::TNT {
                return None; // TODO: get owner of TNT
            }
            if e.get_living_entity().is_some() {
                return Some(e);
            }
            if let Some(thrown_entity) = e.get_thrown_item_entity()
                && let Some(i) = thrown_entity.owner_id.load()
            {
                return world.get_player_by_id(i).map(|a| a as Arc<dyn EntityBase>);
            }
            None
        })
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
