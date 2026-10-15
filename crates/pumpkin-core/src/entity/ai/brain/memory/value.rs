use std::any::Any;
use std::sync::Arc;

use pumpkin_data::damage::DamageType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rustc_hash::FxHashSet;
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::entity::ai::pathfinder::path::Path;
use crate::entity::player::Player;

use super::global_pos::GlobalPos;
use super::nearest_visible::NearestVisibleLivingEntities;
use super::position_tracker::PositionTracker;
use super::walk_target::WalkTarget;

pub trait MemoryValue: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn describe(&self) -> String;
    /// Vanilla clears a slot instead of storing an empty collection in it.
    fn is_empty_collection(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpearStatus {
    Approach,
    Charging,
    Retreat,
}

#[derive(Clone)]
pub struct DamageSourceMemory {
    pub damage_type: &'static DamageType,
    pub attacker: Option<Arc<dyn EntityBase>>,
}

#[must_use]
pub fn describe_entity(entity: &dyn EntityBase) -> String {
    let entity = entity.get_entity();
    format!("{}#{}", entity.entity_type.resource_name, entity.entity_id)
}

macro_rules! debug_memory_values {
    ($($ty:ty),* $(,)?) => {
        $(impl MemoryValue for $ty {
            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }

            fn describe(&self) -> String {
                format!("{self:?}")
            }
        })*
    };
}

macro_rules! collection_memory_values {
    ($($ty:ty),* $(,)?) => {
        $(impl MemoryValue for $ty {
            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }

            fn describe(&self) -> String {
                format!("{self:?}")
            }

            fn is_empty_collection(&self) -> bool {
                self.is_empty()
            }
        })*
    };
}

debug_memory_values!(
    (),
    bool,
    i32,
    i64,
    Uuid,
    BlockPos,
    Vector3<f64>,
    GlobalPos,
    WalkTarget,
    Arc<dyn PositionTracker>,
    Path,
    SpearStatus,
    NearestVisibleLivingEntities,
);

collection_memory_values!(Vec<Uuid>, Vec<GlobalPos>, FxHashSet<GlobalPos>);

fn describe_entity_list<'a>(entities: impl Iterator<Item = &'a dyn EntityBase>) -> String {
    let described: Vec<String> = entities.map(describe_entity).collect();
    format!("[{}]", described.join(", "))
}

impl MemoryValue for Arc<dyn EntityBase> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn describe(&self) -> String {
        describe_entity(self.as_ref())
    }
}

impl MemoryValue for Arc<Player> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn describe(&self) -> String {
        describe_entity(self.as_ref())
    }
}

impl MemoryValue for Vec<Arc<dyn EntityBase>> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn describe(&self) -> String {
        describe_entity_list(self.iter().map(AsRef::as_ref))
    }

    fn is_empty_collection(&self) -> bool {
        self.is_empty()
    }
}

impl MemoryValue for Vec<Arc<Player>> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn describe(&self) -> String {
        describe_entity_list(self.iter().map(|player| player.as_ref() as &dyn EntityBase))
    }

    fn is_empty_collection(&self) -> bool {
        self.is_empty()
    }
}

impl MemoryValue for DamageSourceMemory {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn describe(&self) -> String {
        self.attacker.as_ref().map_or_else(
            || self.damage_type.message_id.to_owned(),
            |attacker| {
                format!(
                    "{} by {}",
                    self.damage_type.message_id,
                    describe_entity(attacker.as_ref())
                )
            },
        )
    }
}
