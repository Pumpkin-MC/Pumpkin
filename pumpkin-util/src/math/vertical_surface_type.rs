use serde::Deserialize;

/// Represents the different types of vertical surfaces used for collision detection and placement logic.
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerticalSurfaceType {
    /// Upward-facing surface (top of a block/space) used for ceiling collisions and hanging placement.
    Ceiling,
    /// Downward-facing surface (bottom of a block/space) used for ground collisions and standing placement.
    Floor,
}
