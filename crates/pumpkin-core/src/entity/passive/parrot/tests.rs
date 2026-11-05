use super::*;
use crate::entity::{
    ageable::AgeableMob, ai::test_support::world, passive::tamable::TamableAnimal,
};
use pumpkin_data::item::Item;
use pumpkin_util::math::vector3::Vector3;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn loading_a_perched_parrot_preserves_owner_variant_and_sit_order() -> TestResult {
    let folder = tempfile::tempdir()?;
    let world = world(&folder);
    let parrot = ParrotEntity::new(Entity::new(
        world.clone(),
        Vector3::new(0.0, 64.0, 0.0),
        &EntityType::PARROT,
    ));
    let owner = uuid::Uuid::new_v4();
    let mut saved = NbtCompound::new();
    saved.put_string("id", "minecraft:parrot".to_string());
    saved.put_int("Variant", 3);
    saved.put_uuid("Owner", owner);
    saved.put_bool("Sitting", true);
    EntityBase::read_nbt_non_mut(parrot.as_ref(), &saved);
    assert!(parrot.is_tame());
    assert!(parrot.is_ordered_to_sit());
    assert!(parrot.is_in_sitting_pose());
    assert_eq!(parrot.get_owner(), Some(owner));
    let mut written = NbtCompound::new();
    EntityBase::write_nbt(parrot.as_ref(), &mut written);
    assert_eq!(written.get_int("Variant"), Some(3));
    assert_eq!(written.get_uuid("Owner"), Some(owner));
    assert_eq!(written.get_bool("Sitting"), Some(true));
    // Vanilla's legacy integer codec clamps out-of-range IDs and defaults a missing ID.
    for (id, expected) in [(-1, 0), (5, 4), (i32::MAX, 4)] {
        saved.put_int("Variant", id);
        parrot.mob_read_nbt(&saved);
        parrot.mob_write_nbt(&mut written);
        assert_eq!(written.get_int("Variant"), Some(expected));
    }
    parrot.mob_read_nbt(&NbtCompound::new());
    parrot.mob_write_nbt(&mut written);
    assert_eq!(written.get_int("Variant"), Some(0));
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test]
async fn negative_age_does_not_turn_a_parrot_into_a_baby_or_disable_its_loot() -> TestResult {
    let folder = tempfile::tempdir()?;
    let world = world(&folder);
    let parrot = ParrotEntity::new(Entity::new(
        world.clone(),
        Vector3::new(0.0, 64.0, 0.0),
        &EntityType::PARROT,
    ));
    let mut nbt = NbtCompound::new();
    nbt.put_int("Age", -24000);
    EntityBase::read_nbt_non_mut(parrot.as_ref(), &nbt);
    assert!(!parrot.is_baby());
    assert!(!parrot.mob_entity.living_entity.is_baby());
    assert!(parrot.mob_entity.living_entity.should_drop_loot());
    assert!(!parrot.should_check_fall_damage());
    assert!(!super::super::animal::Animal::is_food(
        parrot.as_ref(),
        &ItemStack::new(1, &Item::WHEAT_SEEDS)
    ));
    assert!(!super::super::animal::Animal::can_mate(
        parrot.as_ref(),
        parrot.as_ref()
    ));
    world.level.shutdown().await;
    Ok(())
}
