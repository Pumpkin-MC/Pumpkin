use crate::entity::mob::Mob;
use crate::world::World;

pub const CONVERSION_TIME: i32 = 300;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PiglinArmPose {
    AttackingWithMeleeWeapon,
    CrossbowHold,
    CrossbowCharge,
    AdmiringItem,
    Dancing,
    Default,
}

pub trait AbstractPiglin: Mob {
    fn is_adult(&self) -> bool;
    fn can_hunt(&self) -> bool;
    fn is_converting(&self, world: &World) -> bool;
    fn is_immune_to_zombification(&self) -> bool;
    fn arm_pose(&self) -> PiglinArmPose;
    fn play_converted_sound(&self);
    fn finish_conversion(&self);
}

/// Vanilla `AbstractPiglin.customServerAiStep`: the timer only runs while converting.
pub fn tick_conversion(
    piglin: &dyn AbstractPiglin,
    world: &World,
    time_in_overworld: &std::sync::atomic::AtomicI32,
) {
    use std::sync::atomic::Ordering::Relaxed;

    if !piglin.is_converting(world) {
        time_in_overworld.store(0, Relaxed);
        return;
    }
    let elapsed = time_in_overworld.fetch_add(1, Relaxed) + 1;
    if elapsed <= CONVERSION_TIME {
        return;
    }
    if world.level_info.load().difficulty != pumpkin_util::Difficulty::Peaceful {
        piglin.play_converted_sound();
    }
    piglin.finish_conversion();
}
