use serde::Deserialize;

/// Represents a vertical offset definition used to resolve a Y coordinate.
///
/// This value can be expressed in three different ways:
/// - As an absolute world Y value.
/// - As a number of blocks above the world's minimum Y.
/// - As a number of blocks below the world's top boundary.
///
/// The enum is deserialized in an untagged form, meaning the variant
/// is selected based on the provided field.
#[derive(Deserialize)]
#[serde(untagged)]
pub enum YOffset {
    /// Absolute world Y coordinate.
    Absolute(Absolute),
    /// Offset measured upward from the world's minimum Y.
    AboveBottom(AboveBottom),
    /// Offset measured downward from the world's maximum Y.
    BelowTop(BelowTop),
}

impl YOffset {
    /// Resolves the effective world Y coordinate.
    ///
    /// # Arguments
    /// * `min_y` - The minimum Y level of the world.
    /// * `height` - The total world height.
    ///
    /// # Returns
    /// The computed absolute Y coordinate as `i32`.
    ///
    /// # Behaviour
    /// * `Absolute` - Returns the provided Y coordinate directly.
    /// * `AboveBottom` - Returns `min_y + offset`.
    /// * `BelowTop` - Returns `(min_y + height - 1) - offset`.
    #[must_use]
    pub const fn get_y(&self, min_y: i16, height: u16) -> i32 {
        match self {
            Self::AboveBottom(above_bottom) => min_y as i32 + above_bottom.above_bottom as i32,
            Self::BelowTop(below_top) => {
                height as i32 - 1 + min_y as i32 - below_top.below_top as i32
            }
            Self::Absolute(absolute) => absolute.absolute as i32,
        }
    }
}

/// Absolute vertical position in world coordinates.
#[derive(Deserialize)]
pub struct Absolute {
    /// The fixed world Y level.
    pub absolute: i16,
}

/// Offset measured upward from the world's minimum Y level.
#[derive(Deserialize)]
pub struct AboveBottom {
    /// Number of blocks above the minimum Y.
    pub above_bottom: i8,
}

/// Offset measured downward from the world's top boundary.
#[derive(Deserialize)]
pub struct BelowTop {
    /// Number of blocks below the maximum Y.
    pub below_top: i8,
}
