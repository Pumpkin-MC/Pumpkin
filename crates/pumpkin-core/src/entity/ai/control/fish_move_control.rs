use crate::entity::ai::control::move_control::MoveControl;
use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::mob::Mob;

/// Vanilla `AbstractFish.FishMoveControl`: holds a fish level under water and tilts it towards the
/// next node. The navigator does the steering.
#[derive(Default)]
pub struct FishMoveControl {
    inner: MoveControl,
}

impl Control for FishMoveControl {}

impl MoveControlTrait for FishMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        let living = &mob.get_mob_entity().living_entity;
        let entity = &living.entity;
        let mut velocity = entity.velocity.load();
        if entity.is_submerged_in_water() {
            velocity.y += 0.005;
        }
        // The navigator points the pitch at the next node, so its sine is vanilla's dy / distance.
        let speed = living.movement_input.load().z;
        if speed != 0.0 {
            velocity.y -= speed * f64::from(entity.pitch.load().to_radians().sin()) * 0.1;
        }
        entity.velocity.store(velocity);

        self.inner.tick(mob);
    }

    fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed_modifier: f64) {
        MoveControlTrait::set_wanted_position(&mut self.inner, x, y, z, speed_modifier);
    }

    fn strafe(&mut self, forward: f32, right: f32) {
        self.inner.strafe(forward, right);
    }

    fn has_wanted(&self) -> bool {
        MoveControlTrait::has_wanted(&self.inner)
    }
}
