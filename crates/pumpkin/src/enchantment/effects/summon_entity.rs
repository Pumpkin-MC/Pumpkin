use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::seq::IndexedRandom;
use uuid::Uuid;

use crate::entity::lightning::LightningBoltEntity;
use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase};
use crate::world::World;

/// Enchantment entity effect that summons an entity (such as lightning) at a given position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SummonEntityEffect {
    pub entity_types: Vec<&'static EntityType>,
    pub join_team: bool,
}

impl SummonEntityEffect {
    #[must_use]
    pub const fn new(entity_types: Vec<&'static EntityType>, join_team: bool) -> Self {
        Self {
            entity_types,
            join_team,
        }
    }

    #[must_use]
    pub fn select_random_entity_type(&self) -> Option<&'static EntityType> {
        let mut rng = rand::rng();
        self.entity_types.choose(&mut rng).copied()
    }

    /// Applies the summon entity effect at the given position in the world.
    pub fn apply(
        &self,
        world: &Arc<World>,
        position: Vector3<f64>,
        owner: Option<&Arc<Player>>,
        _entity: Option<&Entity>,
    ) -> Option<Arc<dyn EntityBase>> {
        let block_pos = BlockPos::floored_v(position);
        if !world.is_in_build_limit(block_pos) {
            return None;
        }

        let entity_type = self.select_random_entity_type()?;
        let spawned =
            crate::entity::r#type::from_type(entity_type, position, world, Uuid::new_v4());

        if let Some(bolt) = spawned.cast_any().downcast_ref::<LightningBoltEntity>()
            && let Some(player) = owner
        {
            bolt.set_cause(Some(player.clone()));
        }

        world.spawn_entity(spawned.clone());
        Some(spawned)
    }
}

impl super::EnchantmentEntityEffectExt for SummonEntityEffect {
    fn apply(
        &self,
        world: &Arc<World>,
        _enchantment_level: i32,
        owner: Option<&Arc<Player>>,
        entity: Option<&Entity>,
        position: Vector3<f64>,
    ) {
        let _ = self.apply(world, position, owner, entity);
    }
}
