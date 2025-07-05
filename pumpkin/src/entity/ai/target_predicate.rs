use std::pin::Pin;
use std::sync::Arc;
use crate::entity::living::LivingEntity;
use crate::world::World;

const MIN_DISTANCE: f64 = 2.0;

pub type PredicateFn =
dyn Fn(Arc<LivingEntity>, Arc<World>) -> Pin<Box<dyn Future<Output = bool> + Send>>
+ Send
+ Sync;

pub struct TargetPredicate {
    pub attackable: bool,
    pub base_max_distance: f64,
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
        let mut instance = Self::default();
        instance.attackable = attackable;
        instance
    }
    
    pub fn attackable() -> Self {
        Self::new(true)
    }
    
    pub fn non_attackable() -> Self {
        Self::new(false)
    }
    
    pub fn copy(&self) -> Self {
        let mut instance = if self.attackable { Self::attackable() } else { Self::non_attackable() };
        instance.base_max_distance = self.base_max_distance;
        instance.respects_visibility = self.respects_visibility;
        instance.use_distance_scaling_factor = self.use_distance_scaling_factor;
        instance.predicate = self.predicate.clone();
        
        instance
    }
    
    pub fn ignore_visibility(&mut self) {
        self.respects_visibility = false;
    }
    
    pub fn ignore_distance_scaling(&mut self) {
        self.use_distance_scaling_factor = false;
    }
    
    pub async fn test(world: Arc<World>, tester: Option<Arc<LivingEntity>>, target: Arc<LivingEntity>) -> bool {
        if tester.is_some() && Arc::ptr_eq(&tester.unwrap(), &target) {
            return false;
        } else if !target.is_part_of_game().await {
            return false;
        }
        //TODO: continue
        true
    }
}