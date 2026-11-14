use pumpkin_data::poi_type::PoiType;
use pumpkin_util::math::position::BlockPos;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoiRecord {
    pub pos: BlockPos,
    pub poi_type: PoiType,
    pub free_tickets: i32,
}

impl PoiRecord {
    #[must_use]
    pub const fn has_space(&self) -> bool {
        self.free_tickets > 0
    }

    #[must_use]
    pub const fn is_occupied(&self) -> bool {
        self.free_tickets != self.poi_type.max_tickets()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Occupancy {
    HasSpace,
    IsOccupied,
    Any,
}

impl Occupancy {
    #[must_use]
    pub const fn test(self, record: &PoiRecord) -> bool {
        match self {
            Self::HasSpace => record.has_space(),
            Self::IsOccupied => record.is_occupied(),
            Self::Any => true,
        }
    }
}
