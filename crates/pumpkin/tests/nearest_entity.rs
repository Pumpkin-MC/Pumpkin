mod support;

use pumpkin::entity::{Entity, EntityBase};
use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

#[tokio::test]
async fn closest_entity_respects_distance_type_and_current_position()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let world = support::world(dir.path());
    let origin = Vector3::new(0.0, 64.0, 0.0);
    assert!(world.get_closest_entity(origin, 10.0, None).is_none());
    assert!(world.get_closest_player(origin, 10.0).is_none());
    let item: Arc<dyn EntityBase> = Arc::new(Entity::new(world.clone(), origin, &EntityType::ITEM));
    let cow: Arc<dyn EntityBase> = Arc::new(Entity::new(
        world.clone(),
        Vector3::new(3.0, 68.0, 0.0),
        &EntityType::COW,
    ));
    world
        .entities
        .store(Arc::new(vec![cow.clone(), item.clone()]));
    let selected = |radius, types| {
        world
            .get_closest_entity(origin, radius, types)
            .map(|e| e.get_entity().entity_id)
    };
    assert_eq!(selected(5.0, None), Some(item.get_entity().entity_id));
    assert_eq!(
        selected(5.0, Some([&EntityType::COW].as_slice())),
        Some(cow.get_entity().entity_id)
    );
    assert!(selected(4.99, Some([&EntityType::COW].as_slice())).is_none());
    assert!(selected(10.0, Some([].as_slice())).is_none());
    assert!(selected(f64::NAN, None).is_none());
    assert_eq!(
        selected(-5.0, Some([&EntityType::COW].as_slice())),
        Some(cow.get_entity().entity_id)
    );
    cow.get_entity().pos.store(origin);
    item.get_entity().pos.store(Vector3::new(100.0, 64.0, 0.0));
    assert_eq!(selected(0.0, None), Some(cow.get_entity().entity_id));
    cow.get_entity()
        .pos
        .store(Vector3::new(f64::NAN, 64.0, 0.0));
    assert!(selected(10.0, None).is_none());
    assert_eq!(
        selected(f64::INFINITY, None),
        Some(item.get_entity().entity_id)
    );
    cow.get_entity().pos.store(Vector3::new(100.0, 64.0, 0.0));
    let tied = selected(100.0, None);
    assert!(tied == Some(item.get_entity().entity_id) || tied == Some(cow.get_entity().entity_id));
    world.entities.store(Arc::new(vec![cow.clone()]));
    cow.get_entity()
        .pos
        .store(Vector3::new(f64::INFINITY, 64.0, 0.0));
    assert_eq!(
        selected(f64::INFINITY, None),
        Some(cow.get_entity().entity_id)
    );
    world.entities.store(Arc::new(Vec::new()));
    world.level.shutdown().await;
    Ok(())
}
