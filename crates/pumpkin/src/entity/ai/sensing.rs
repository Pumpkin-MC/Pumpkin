use rustc_hash::FxHashSet;

use crate::entity::Entity;

/// Per tick cache of the line of sight checks a mob makes, so several goals asking
/// about the same target only pay for one raycast.
#[derive(Default)]
pub struct Sensing {
    seen: FxHashSet<i32>,
    unseen: FxHashSet<i32>,
}

impl Sensing {
    pub fn tick(&mut self) {
        self.seen.clear();
        self.unseen.clear();
    }

    pub fn has_line_of_sight(&mut self, mob: &Entity, target: &Entity) -> bool {
        let target_id = target.entity_id;
        if self.seen.contains(&target_id) {
            return true;
        }
        if self.unseen.contains(&target_id) {
            return false;
        }

        let has_line_of_sight = mob.has_line_of_sight(target);
        if has_line_of_sight {
            self.seen.insert(target_id);
        } else {
            self.unseen.insert(target_id);
        }
        has_line_of_sight
    }
}
