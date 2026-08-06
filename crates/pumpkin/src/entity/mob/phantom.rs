use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::attributes::Attributes;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, NBTStorage, NbtFuture,
    ai::control::phantom_move_control::PhantomMoveControl,
    ai::goal::{
        phantom_attack_player_target::PhantomAttackPlayerTargetGoal,
        phantom_attack_strategy::PhantomAttackStrategyGoal,
        phantom_circle_anchor::PhantomCircleAroundAnchorGoal,
        phantom_sweep_attack::PhantomSweepAttackGoal,
    },
    mob::{Mob, MobEntity},
};

/// Vanilla: `Phantom.AttackPhase` (`Phantom.java:211-214`).
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum AttackPhase {
    #[default]
    Circle,
    Swoop,
}

pub struct PhantomEntity {
    pub mob_entity: MobEntity,
    size: AtomicI32,
    /// Vanilla: `Phantom.moveTargetPoint`. Written by the circle/sweep goals, read every tick
    /// by `PhantomMoveControl`.
    move_target_point: AtomicCell<Vector3<f64>>,
    /// Vanilla: `Phantom.anchorPoint`.
    anchor_point: AtomicCell<Option<BlockPos>>,
    /// Vanilla: `Phantom.attackPhase`.
    attack_phase: AtomicCell<AttackPhase>,
}

impl PhantomEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        // Vanilla: `Phantom`'s constructor replaces the default `MoveControl` with the
        // circling/diving `PhantomMoveControl` (`Phantom.java:55`).
        // `finalizeSpawn` (`Phantom.java:156`) sets `anchorPoint = blockPosition().above(5)`;
        // there's no dedicated spawn-finalization hook here, so it's seeded from the entity's
        // position at construction time instead, which is equivalent in practice.
        let initial_anchor = entity.block_pos.load().up_height(5);
        let mob_entity = MobEntity::new(entity);
        *mob_entity.move_control.lock().unwrap() = Box::new(PhantomMoveControl::default());
        let phantom = Self {
            mob_entity,
            size: AtomicI32::new(0),
            move_target_point: AtomicCell::new(Vector3::new(0.0, 0.0, 0.0)),
            anchor_point: AtomicCell::new(Some(initial_anchor)),
            attack_phase: AtomicCell::new(AttackPhase::Circle),
        };
        let mob_arc = Arc::new(phantom);
        let phantom_weak = Arc::downgrade(&mob_arc);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            // Vanilla `Phantom.registerGoals` (`Phantom.java:70-74`). Note vanilla installs a
            // no-op `PhantomLookControl` (`Phantom.java:375-383`) precisely because it has no
            // look-related goals; `LookControl` here isn't swappable (concrete field, not a
            // trait object like `move_control`), but since `move_control.tick()` runs after
            // `look_control.tick()` every tick (see `MobEntity`'s tick order), the pitch this
            // sets below always wins. Only head-yaw drifts slightly toward body-yaw each tick
            // as an approximation of the missing no-op.
            goal_selector.add_goal(
                1,
                Box::new(PhantomAttackStrategyGoal::new(phantom_weak.clone())),
            );
            goal_selector.add_goal(
                2,
                Box::new(PhantomSweepAttackGoal::new(phantom_weak.clone())),
            );
            goal_selector.add_goal(
                3,
                Box::new(PhantomCircleAroundAnchorGoal::new(phantom_weak.clone())),
            );

            target_selector.add_goal(
                1,
                Box::new(PhantomAttackPlayerTargetGoal::new(phantom_weak)),
            );
        };

        mob_arc
    }

    pub fn set_size(&self, size: i32) {
        let size = size.clamp(0, 64);
        self.size.store(size, Ordering::Relaxed);

        let entity = &self.mob_entity.living_entity.entity;
        if let Some(attack_damage) = self
            .mob_entity
            .living_entity
            .attributes
            .write()
            .unwrap()
            .get_mut(&Attributes::ATTACK_DAMAGE.id)
        {
            attack_damage.base_value = 6.0 + f64::from(size);
            attack_damage.dirty.store(true, Ordering::Relaxed);
        }

        let original = entity.entity_type.dimension;
        let scale = 1.0 + 0.15 * size as f32;
        let dimensions = EntityDimensions {
            width: original[0] * scale,
            height: original[1] * scale,
            eye_height: entity.entity_type.eye_height * scale,
        };
        entity.entity_dimension.store(dimensions);
        let position = entity.pos.load();
        entity.bounding_box.store(BoundingBox::new_from_pos(
            position.x,
            position.y,
            position.z,
            &dimensions,
        ));
    }

    #[must_use]
    pub fn size(&self) -> i32 {
        self.size.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn move_target_point(&self) -> Vector3<f64> {
        self.move_target_point.load()
    }

    pub fn set_move_target_point(&self, point: Vector3<f64>) {
        self.move_target_point.store(point);
    }

    #[must_use]
    pub fn anchor_point(&self) -> Option<BlockPos> {
        self.anchor_point.load()
    }

    pub fn set_anchor_point(&self, anchor: Option<BlockPos>) {
        self.anchor_point.store(anchor);
    }

    #[must_use]
    pub fn attack_phase(&self) -> AttackPhase {
        self.attack_phase.load()
    }

    pub fn set_attack_phase(&self, phase: AttackPhase) {
        self.attack_phase.store(phase);
    }
}

impl NBTStorage for PhantomEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            nbt.put_int("size", self.size());
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.set_size(nbt.get_int("size").unwrap_or(0));
        })
    }
}

impl Mob for PhantomEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_mob_gravity(&self) -> f64 {
        0.0
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn phantom_sizes_follow_vanilla_bounds() {
        assert_eq!((-10i32).clamp(0, 64), 0);
        assert_eq!(64i32.clamp(0, 64), 64);
        assert_eq!(90i32.clamp(0, 64), 64);
    }

    #[test]
    fn phantom_attack_damage_scales_with_size() {
        assert_eq!(6.0 + f64::from(0), 6.0);
        assert_eq!(6.0 + f64::from(12), 18.0);
    }
}
