use pumpkin_data::packet::clientbound::PLAY_AWARD_STATS;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

/// Sent to the client to update one or more statistics.
/// <https://wiki.vg/Protocol#Award_Statistics>
#[derive(Serialize)]
#[java_packet(PLAY_AWARD_STATS)]
pub struct CAwardStats {
    pub statistics: Vec<Statistic>,
}

#[derive(Serialize)]
pub struct Statistic {
    /// Category ID (`VarInt` enum):
    /// 0 = mined, 1 = crafted, 2 = used, 3 = broken, 4 = `picked_up`, 5 = dropped,
    /// 6 = killed, 7 = `killed_by`, 8 = custom
    pub category_id: VarInt,
    /// Statistic ID within the category (registry ID of block/item/entity/custom stat)
    pub statistic_id: VarInt,
    /// The new value of the statistic
    pub value: VarInt,
}

impl CAwardStats {
    #[must_use]
    pub const fn new(statistics: Vec<Statistic>) -> Self {
        Self { statistics }
    }
}

impl Statistic {
    #[must_use]
    pub const fn new(category_id: VarInt, statistic_id: VarInt, value: VarInt) -> Self {
        Self {
            category_id,
            statistic_id,
            value,
        }
    }
}
