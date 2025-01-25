pub mod entity_registry;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FacingDirection {
    North,
    South,
    East,
    West,
}
