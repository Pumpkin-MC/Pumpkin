use std::any::Any;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::codec::var_int::VarInt;
use rand::RngExt;

use crate::entity::Entity;

pub trait ItemSteerable: Send + Sync {
    /// Attempts to boost speed. Returns `true` if boost was successfully activated.
    fn boost(&self) -> bool;

    fn as_any(&self) -> &dyn Any;
}

pub struct ItemBasedSteering {
    boost_time_data: TrackedData,
    pub boosting: AtomicBool,
    pub boost_time: AtomicI32,
    pub boost_time_total: AtomicI32,
}

impl ItemBasedSteering {
    pub const MIN_BOOST_TIME: i32 = 140;
    pub const MAX_BOOST_TIME: i32 = 700;

    #[must_use]
    pub const fn new(boost_time_data: TrackedData) -> Self {
        Self {
            boost_time_data,
            boosting: AtomicBool::new(false),
            boost_time: AtomicI32::new(0),
            boost_time_total: AtomicI32::new(0),
        }
    }

    #[must_use]
    pub fn boost(&self, entity: &Entity, random: &mut impl RngExt) -> bool {
        if self.boosting.swap(true, Ordering::AcqRel) {
            return false;
        }
        self.boost_time.store(0, Ordering::Relaxed);
        // The roll is MIN + MAX + 1 wide, not MAX - MIN + 1.
        let random_time = random.random_range(0..=(Self::MIN_BOOST_TIME + Self::MAX_BOOST_TIME))
            + Self::MIN_BOOST_TIME;
        self.boost_time_total.store(random_time, Ordering::Relaxed);
        entity.set_synced_data(self.boost_time_data, VarInt(random_time));
        true
    }

    pub fn tick_boost(&self) {
        if self.boosting.load(Ordering::Relaxed) {
            let prev = self.boost_time.fetch_add(1, Ordering::Relaxed);
            if prev > self.boost_time_total.load(Ordering::Relaxed) {
                self.boosting.store(false, Ordering::Relaxed);
            }
        }
    }

    #[must_use]
    pub fn boost_factor(&self) -> f32 {
        if self.boosting.load(Ordering::Relaxed) {
            let current = self.boost_time.load(Ordering::Relaxed) as f32;
            let total = self.boost_time_total.load(Ordering::Relaxed).max(1) as f32;
            1.0 + 1.15 * (current / total * std::f32::consts::PI).sin()
        } else {
            1.0
        }
    }
}
