mod placement;
mod template;

pub use placement::{
    GameTestPosition, TestStructureInstance, clear_structure_area, clear_success_entities,
    encase_structure, place_structure, place_structure_with_controller_rotation, remove_barriers,
};
pub use template::{GameTestStructureBlock, GameTestStructureTemplate, TestBlockMode};
