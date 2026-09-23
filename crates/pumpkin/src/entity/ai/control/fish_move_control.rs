use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::mob::Mob;
use pumpkin_util::math::vector3::Vector3;

/// Vanilla's `AbstractFish.FishMoveControl`, less the steering, which water
/// navigation already does here. What is left is the lift that keeps a fish
/// with its eyes under water from sinking while it idles.
#[derive(Default)]
pub struct FishMoveControl;

impl Control for FishMoveControl {}

impl MoveControlTrait for FishMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        let entity = mob.get_entity();
        if entity.is_submerged_in_water() {
            // A plain store: the tracker syncs motion, as vanilla's setDeltaMovement
            // leaves to it; set_velocity would send a packet every tick.
            let velocity = entity.velocity.load();
            entity
                .velocity
                .store(Vector3::new(velocity.x, velocity.y + 0.005, velocity.z));
        }
    }
}
