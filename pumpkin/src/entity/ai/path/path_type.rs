//! Path type enumeration for pathfinding.
//!
//! This module defines the different types of path nodes that can be encountered
//! during pathfinding, along with their associated traversal costs (malus values).

/// Represents the type of a pathfinding node.
///
/// Each path type has an associated malus (penalty) value that affects
/// the cost calculation during A* pathfinding. Negative malus values
/// indicate impassable terrain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PathType {
    /// Completely blocked, cannot pass through
    #[default]
    Blocked,
    /// Open air, passable
    Open,
    /// Solid walkable ground
    Walkable,
    /// Walkable door (can pass through)
    WalkableDoor,
    /// Trapdoor block
    Trapdoor,
    /// Powder snow block (dangerous)
    PowderSnow,
    /// Near powder snow (caution)
    DangerPowderSnow,
    /// Fence block (impassable for most mobs)
    Fence,
    /// Lava (dangerous/impassable)
    Lava,
    /// Water (swimmable for some mobs)
    Water,
    /// Near water edge
    WaterBorder,
    /// Rail block
    Rail,
    /// Unpassable rail
    UnpassableRail,
    /// Near fire (dangerous)
    DangerFire,
    /// In fire (damaging)
    DamageFire,
    /// Near other danger
    DangerOther,
    /// In damaging block
    DamageOther,
    /// Open door
    DoorOpen,
    /// Closed wooden door
    DoorWoodClosed,
    /// Closed iron door
    DoorIronClosed,
    /// Breach (for dolphins)
    Breach,
    /// Leaves block
    Leaves,
    /// Honey block (sticky)
    StickyHoney,
    /// Cocoa block
    Cocoa,
    /// Cautious damage (wither rose, pointed dripstone)
    DamageCautious,
    /// Trapdoor danger
    DangerTrapdoor,
}

impl PathType {
    /// Returns the default malus (penalty) value for this path type.
    ///
    /// Higher values make the path less desirable. Negative values indicate
    /// impassable terrain that should not be traversed.
    #[must_use]
    pub const fn malus(&self) -> f32 {
        match self {
            Self::Blocked
            | Self::PowderSnow
            | Self::Fence
            | Self::Lava
            | Self::UnpassableRail
            | Self::DamageOther
            | Self::DoorWoodClosed
            | Self::DoorIronClosed
            | Self::Leaves => -1.0,
            Self::Open
            | Self::Walkable
            | Self::WalkableDoor
            | Self::Trapdoor
            | Self::DangerPowderSnow
            | Self::Rail
            | Self::DoorOpen
            | Self::Cocoa
            | Self::DamageCautious
            | Self::DangerTrapdoor => 0.0,
            Self::Water
            | Self::WaterBorder
            | Self::DangerFire
            | Self::DangerOther
            | Self::StickyHoney => 8.0,
            Self::DamageFire => 16.0,
            Self::Breach => 4.0,
        }
    }

    /// Returns whether this path type represents passable terrain.
    #[must_use]
    pub const fn is_passable(&self) -> bool {
        self.malus() >= 0.0
    }

    /// Returns whether this path type represents dangerous terrain.
    #[must_use]
    pub const fn is_dangerous(&self) -> bool {
        matches!(
            self,
            Self::DangerFire
                | Self::DamageFire
                | Self::DangerOther
                | Self::DamageOther
                | Self::DangerPowderSnow
                | Self::DamageCautious
                | Self::DangerTrapdoor
        )
    }

    /// Returns whether this path type has partial collision (fences, doors).
    #[must_use]
    pub const fn has_partial_collision(&self) -> bool {
        matches!(
            self,
            Self::Fence | Self::DoorWoodClosed | Self::DoorIronClosed
        )
    }
}
