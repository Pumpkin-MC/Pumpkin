mod placement;
mod template;

pub use placement::{
    GameTestPosition, TestStructureInstance, clear_placed_test, clear_structure_area,
    clear_success_entities, encase_structure, expand_structure, place_existing_structure,
    place_structure, place_structure_block_entity, place_structure_with_controller_rotation,
    place_structure_with_controller_rotation_and_entities, remove_barriers,
};
pub(crate) use placement::{place_prepared_structure, resolve_structure_placement};
pub use template::{
    GameTestStructureBlock, GameTestStructureEntity, GameTestStructureTemplate, TestBlockMode,
};
