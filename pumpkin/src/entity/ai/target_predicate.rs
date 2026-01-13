use crate::entity::living::LivingEntity;
use crate::world::World;
use std::pin::Pin;
use std::sync::Arc;

#[expect(dead_code)]
const MIN_DISTANCE: f32 = 2.0;

pub type PredicateFn = dyn Fn(Arc<LivingEntity>, Arc<World>) -> Pin<Box<dyn Future<Output = bool> + Send>>
    + Send
    + Sync;

pub struct TargetPredicate {
    pub attackable: bool,
    pub base_max_distance: f32,
    pub respects_visibility: bool,
    pub use_distance_scaling_factor: bool,
    pub predicate: Option<Arc<PredicateFn>>,
}

impl Default for TargetPredicate {
    fn default() -> Self {
        Self {
            attackable: true,
            base_max_distance: -1.0,
            respects_visibility: true,
            use_distance_scaling_factor: true,
            predicate: None,
        }
    }
}

impl TargetPredicate {
    fn new(attackable: bool) -> Self {
        Self {
            attackable,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn attackable() -> Self {
        Self::new(true)
    }

    #[must_use]
    pub fn non_attackable() -> Self {
        Self::new(false)
    }

    #[must_use]
    pub fn copy(&self) -> Self {
        let mut instance = if self.attackable {
            Self::attackable()
        } else {
            Self::non_attackable()
        };
        instance.base_max_distance = self.base_max_distance;
        instance.respects_visibility = self.respects_visibility;
        instance.use_distance_scaling_factor = self.use_distance_scaling_factor;
        instance.predicate.clone_from(&self.predicate);

        instance
    }

    pub const fn ignore_visibility(&mut self) {
        self.respects_visibility = false;
    }

    pub const fn ignore_distance_scaling(&mut self) {
        self.use_distance_scaling_factor = false;
    }

    pub fn test(
        &self,
        _world: Arc<World>,
        tester: Option<&LivingEntity>,
        target: &LivingEntity,
    ) -> bool {
        if (tester.is_some() && std::ptr::eq(tester.unwrap(), target)) || !target.is_part_of_game()
        {
            return false;
        }
        //TODO: continue
        true
    }

    pub fn set_predicate<F, Fut>(&mut self, predicate: F)
    where
        F: Fn(Arc<LivingEntity>, Arc<World>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = bool> + Send + 'static,
    {
        self.predicate = Some(Arc::new(
            move |living_entity: Arc<LivingEntity>, world: Arc<World>| {
                Box::pin(predicate(living_entity, world))
            },
        ));
    }
}
