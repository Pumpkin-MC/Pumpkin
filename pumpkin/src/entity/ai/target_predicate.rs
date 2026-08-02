use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::{entity::EntityType, item::Item};
use pumpkin_util::Difficulty;

use crate::entity::living::LivingEntity;
use crate::world::World;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

const MIN_DISTANCE: f64 = 2.0;

pub type PredicateFn = dyn Fn(Arc<LivingEntity>, Arc<World>) -> Pin<Box<dyn Future<Output = bool> + Send>>
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
        Self {
            attackable,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn create_attackable() -> Self {
        Self::new(true)
    }

    #[must_use]
    pub fn create_non_attackable() -> Self {
        Self::new(false)
    }

    #[must_use]
    pub fn copy(&self) -> Self {
        Self {
            attackable: self.attackable,
            base_max_distance: self.base_max_distance,
            respects_visibility: self.respects_visibility,
            use_distance_scaling_factor: self.use_distance_scaling_factor,
            predicate: self.predicate.clone(),
        }
    }

    #[must_use]
    pub const fn set_base_max_distance(mut self, base_max_distance: f64) -> Self {
        self.base_max_distance = base_max_distance;
        self
    }

    #[must_use]
    pub const fn ignore_visibility(mut self) -> Self {
        self.respects_visibility = false;
        self
    }

    #[must_use]
    pub const fn ignore_distance_scaling_factor(mut self) -> Self {
        self.use_distance_scaling_factor = false;
        self
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

    pub async fn test(
        &self,
        world: &World,
        tester: Option<&LivingEntity>,
        target: &LivingEntity,
    ) -> bool {
        if tester.is_some_and(|t| std::ptr::eq(t, target)) {
            return false;
        }

        if !target.is_part_of_game() {
            return false;
        }

        if self.attackable
            && (!target.can_take_damage()
                || world.level_info.load().difficulty == Difficulty::Peaceful)
        {
            return false;
        }

        if let Some(tester_ent) = tester
            && self.base_max_distance > 0.0
        {
            let visibility_modifier = if self.use_distance_scaling_factor {
                targeter_visibility_modifier(tester_ent, target).await
            } else {
                1.0
            };
            let max_dist = (self.base_max_distance * visibility_modifier).max(MIN_DISTANCE);
            let dist_sq = tester_ent
                .entity
                .pos
                .load()
                .squared_distance_to_vec(&target.entity.pos.load());

            if dist_sq > max_dist * max_dist {
                return false;
            }
        }

        if self.respects_visibility
            && let Some(tester_ent) = tester
            && tester_ent
                .entity
                .world
                .load_full()
                .raycast(
                    tester_ent.entity.get_eye_pos(),
                    target.entity.get_eye_pos(),
                    async |block_pos, world| world.get_block_state(block_pos).is_solid(),
                )
                .await
                .is_some()
        {
            return false;
        }

        true
    }
}

async fn targeter_visibility_modifier(targeter: &LivingEntity, target: &LivingEntity) -> f64 {
    let mut modifier = 1.0;

    if target.entity.invisible.load(Relaxed) {
        let equipment = target.entity_equipment.lock().await;
        let armor_slots = [
            EquipmentSlot::HEAD,
            EquipmentSlot::CHEST,
            EquipmentSlot::LEGS,
            EquipmentSlot::FEET,
        ];
        let mut covered = 0;
        for slot in &armor_slots {
            if !equipment.get(slot).lock().await.is_empty() {
                covered += 1;
            }
        }
        modifier *= 0.7 * (f64::from(covered as u8) / 4.0).max(0.1);
    }

    let head_item = target
        .entity_equipment
        .lock()
        .await
        .get(&EquipmentSlot::HEAD)
        .lock()
        .await
        .item
        .id;
    let target_type = targeter.entity.entity_type;
    let head_reduces_visibility = (target_type == &EntityType::SKELETON
        && head_item == Item::SKELETON_SKULL.id)
        || (target_type == &EntityType::ZOMBIE && head_item == Item::ZOMBIE_HEAD.id)
        || ((target_type == &EntityType::PIGLIN || target_type == &EntityType::PIGLIN_BRUTE)
            && head_item == Item::PIGLIN_HEAD.id)
        || (target_type == &EntityType::CREEPER && head_item == Item::CREEPER_HEAD.id);
    if head_reduces_visibility {
        modifier *= 0.5;
    }

    modifier
}

#[cfg(test)]
mod tests {
    #[test]
    fn invisible_targets_without_armor_keep_minimum_visibility() {
        let modifier: f64 = 0.7 * 0.1;
        assert!((modifier - 0.07).abs() < f64::EPSILON);
    }
}
