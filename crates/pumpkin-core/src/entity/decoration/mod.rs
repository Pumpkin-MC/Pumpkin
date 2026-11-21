pub mod armor_stand;
pub mod cushion;
pub mod display;
pub mod end_crystal;
pub mod item_frame;
pub mod leash_knot;
pub mod mannequin;
pub mod painting;

fn attachment_ignores_explosion(
    explosion: &crate::world::Explosion<'_>,
    world: &crate::world::World,
) -> bool {
    explosion
        .source()
        .is_some_and(|source| source.get_entity().is_in_water())
        || !explosion.should_affect_blocklike_entities(world)
}
