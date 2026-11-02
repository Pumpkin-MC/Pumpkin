use std::fmt;
use std::sync::{Arc, Mutex, PoisonError};

use rustc_hash::FxHashMap;

use crate::entity::EntityBase;
use crate::entity::ai::brain::VisibilityContext;
use crate::entity::ai::brain::sensing::is_entity_targetable;

use super::value::describe_entity;

#[derive(Default)]
pub struct NearestVisibleLivingEntities {
    nearby: Vec<Arc<dyn EntityBase>>,
    targetable: Mutex<FxHashMap<i32, bool>>,
}

impl NearestVisibleLivingEntities {
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn new(nearby: Vec<Arc<dyn EntityBase>>) -> Self {
        Self {
            nearby,
            targetable: Mutex::new(FxHashMap::default()),
        }
    }

    #[must_use]
    pub fn nearby(&self) -> &[Arc<dyn EntityBase>] {
        &self.nearby
    }

    fn is_targetable(&self, entity: &Arc<dyn EntityBase>, ctx: &VisibilityContext<'_>) -> bool {
        let id = entity.get_entity().entity_id;
        let cached = self
            .targetable
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&id)
            .copied();
        if let Some(cached) = cached {
            return cached;
        }
        let result = is_entity_targetable(ctx, entity.as_ref());
        self.targetable
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(id, result);
        result
    }

    pub fn find_closest(
        &self,
        ctx: &VisibilityContext<'_>,
        filter: impl Fn(&Arc<dyn EntityBase>) -> bool,
    ) -> Option<Arc<dyn EntityBase>> {
        self.nearby
            .iter()
            .find(|entity| filter(entity) && self.is_targetable(entity, ctx))
            .cloned()
    }

    pub fn find_all<'s>(
        &'s self,
        ctx: &'s VisibilityContext<'_>,
        filter: impl Fn(&Arc<dyn EntityBase>) -> bool + 's,
    ) -> impl Iterator<Item = &'s Arc<dyn EntityBase>> + 's {
        self.nearby
            .iter()
            .filter(move |entity| filter(entity) && self.is_targetable(entity, ctx))
    }

    #[must_use]
    pub fn contains(&self, target: &dyn EntityBase, ctx: &VisibilityContext<'_>) -> bool {
        let target_id = target.get_entity().entity_id;
        self.nearby.iter().any(|entity| {
            entity.get_entity().entity_id == target_id && self.is_targetable(entity, ctx)
        })
    }

    pub fn contains_any(
        &self,
        ctx: &VisibilityContext<'_>,
        filter: impl Fn(&Arc<dyn EntityBase>) -> bool,
    ) -> bool {
        self.nearby
            .iter()
            .any(|entity| filter(entity) && self.is_targetable(entity, ctx))
    }
}

impl fmt::Debug for NearestVisibleLivingEntities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let described: Vec<String> = self
            .nearby
            .iter()
            .map(|entity| describe_entity(entity.as_ref()))
            .collect();
        write!(f, "[{}]", described.join(", "))
    }
}
