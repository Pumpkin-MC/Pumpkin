use std::sync::Arc;

use crate::entity::ageable::AgeableMob;
use crate::world::World;
use pumpkin_data::sound::Sound;
use pumpkin_nbt::compound::NbtCompound;

use crate::entity::{
    Entity, EntityBase,
    custom_sound::CustomSound,
    mob::{Mob, MobEntity, slime::SlimeEntity},
};

pub struct MagmaCubeEntity {
    pub slime: Arc<SlimeEntity>,
}

impl MagmaCubeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let slime = SlimeEntity::new(entity);
        Arc::new(Self { slime })
    }
}

impl CustomSound for MagmaCubeEntity {
    fn sound_volume(&self) -> f32 {
        self.slime.get_sound_volume()
    }

    fn sound_category(&self) -> pumpkin_data::sound::SoundCategory {
        pumpkin_data::sound::SoundCategory::Hostile
    }

    fn death_sound(&self) -> Option<Sound> {
        let size = self.slime.get_size();
        Some(if size == 1 {
            Sound::EntityMagmaCubeDeathSmall
        } else {
            Sound::EntityMagmaCubeDeath
        })
    }
    fn hurt_sound(&self) -> Option<Sound> {
        let size = self.slime.get_size();
        Some(if size == 1 {
            Sound::EntityMagmaCubeHurtSmall
        } else {
            Sound::EntityMagmaCubeHurt
        })
    }
}

impl Mob for MagmaCubeEntity {
    fn as_cube_mob(&self) -> Option<&SlimeEntity> {
        Some(&self.slime)
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(&*self.slime)
    }

    fn spawn_as_baby(&self) -> bool {
        false
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn get_max_look_pitch_change(&self) -> f32 {
        0.0
    }

    fn get_base_experience_reward(&self) -> u32 {
        self.slime.get_size() as u32
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        group_data: Option<super::spawn::SpawnGroupData>,
    ) -> Option<super::spawn::SpawnGroupData> {
        self.slime.finalize_spawn(world, group_data)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.slime.mob_write_nbt(nbt);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.slime.mob_read_nbt(nbt);
    }

    fn on_removed(&self) {
        self.slime.on_removed();
    }

    fn get_mob_entity(&self) -> &MobEntity {
        self.slime.get_mob_entity()
    }

    fn mob_metadata(&self) -> Option<crate::entity::mob::MobMetadata> {
        self.slime.mob_metadata()
    }

    fn mob_tick(&self, caller: &dyn EntityBase) {
        self.slime.mob_tick(caller);
    }

    fn post_tick(&self) {
        self.slime.post_tick();
    }

    fn mob_player_collision(&self, player: &Arc<crate::entity::player::Player>) {
        self.slime.deal_damage(self, &**player);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;
    use pumpkin_data::{attributes::Attributes, entity::EntityType};
    use pumpkin_util::math::vector3::Vector3;
    use std::sync::atomic::Ordering;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn cube(world: Arc<World>) -> Arc<MagmaCubeEntity> {
        MagmaCubeEntity::new(Entity::new(
            world,
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::MAGMA_CUBE,
        ))
    }

    #[tokio::test]
    async fn size_nbt_restores_health_dimensions_armor_and_experience() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let cube = cube(world.clone());
        let mut nbt = NbtCompound::new();
        nbt.put_int("Size", 3);
        nbt.put_bool("wasOnGround", true);
        nbt.put_int("Age", -24000);
        cube.read_nbt_non_mut(&nbt);
        let living = &cube.get_mob_entity().living_entity;
        assert_eq!(cube.slime.get_size(), 4);
        assert_eq!(living.health.load(), 16.0);
        assert_eq!(living.get_attribute_value(&Attributes::ARMOR), 12.0);
        assert_eq!(living.get_attribute_value(&Attributes::ATTACK_DAMAGE), 4.0);
        assert!((living.get_attribute_value(&Attributes::MOVEMENT_SPEED) - 0.6).abs() < 1.0e-6);
        assert!((cube.get_entity().height() - 2.08).abs() < 1.0e-6);
        assert_eq!(cube.get_experience_reward(None), 4);
        assert!(!cube.spawn_as_baby());
        assert!(!cube.slime.is_baby());
        living.health.store(7.0);
        let mut saved = NbtCompound::new();
        cube.write_nbt(&mut saved);
        assert_eq!(saved.get_int("Size"), Some(3));
        assert_eq!(saved.get_bool("wasOnGround"), Some(true));
        assert_eq!(saved.get_int("Age"), None);
        cube.slime.set_size(1, true);
        cube.read_nbt_non_mut(&saved);
        assert_eq!(cube.slime.get_size(), 4);
        assert_eq!(living.health.load(), 7.0);
        nbt.put_int("Size", i32::MAX);
        cube.read_nbt_non_mut(&nbt);
        assert_eq!(cube.slime.get_size(), 1);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn size_armor_absorbs_damage_and_combines_with_equipment() -> TestResult {
        use pumpkin_data::{
            damage::DamageType, data_component_impl::EquipmentSlot, item::Item,
            item_stack::ItemStack,
        };
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let cube = cube(world.clone());
        let living = &cube.get_mob_entity().living_entity;
        cube.slime.set_size(4, true);
        assert!(
            (living.get_damage_after_armor_absorb(10.0, &DamageType::MOB_ATTACK, None) - 7.2).abs()
                < 1.0e-6
        );
        living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .put(
                &EquipmentSlot::CHEST,
                ItemStack::new(1, &Item::DIAMOND_CHESTPLATE),
            );
        assert!(
            (living.get_damage_after_armor_absorb(10.0, &DamageType::MOB_ATTACK, None) - 3.6).abs()
                < 1.0e-6
        );
        cube.slime.set_size(127, true);
        assert_eq!(living.health.load(), 1024.0);
        assert!(
            (living.get_damage_after_armor_absorb(10.0, &DamageType::MOB_ATTACK, None) - 2.0).abs()
                < 1.0e-6
        );
        assert_eq!(
            living.get_damage_after_armor_absorb(10.0, &DamageType::GENERIC, None),
            10.0
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn contact_damage_uses_modified_attribute_plus_two_and_respects_no_ai_and_range()
    -> TestResult {
        struct Target {
            entity: Entity,
            damage: crossbeam::atomic::AtomicCell<f32>,
        }
        impl EntityBase for Target {
            fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
                None
            }
            fn get_entity(&self) -> &Entity {
                &self.entity
            }
            fn cast_any(&self) -> &dyn std::any::Any {
                self
            }
            fn damage_with_context(
                &self,
                _caller: &dyn EntityBase,
                amount: f32,
                _damage_type: pumpkin_data::damage::DamageType,
                _position: Option<Vector3<f64>>,
                _source: Option<&dyn EntityBase>,
                _cause: Option<&dyn EntityBase>,
            ) -> bool {
                self.damage.store(amount);
                true
            }
        }
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        world.level.loaded_chunks.insert(
            pumpkin_util::math::vector2::Vector2::new(0, 0),
            pumpkin_world::chunk::ChunkData::empty_sync(0, 0),
        );
        let cube = cube(world.clone());
        let target = Target {
            entity: Entity::new(
                world.clone(),
                Vector3::new(0.25, 64.0, 0.0),
                &EntityType::IRON_GOLEM,
            ),
            damage: crossbeam::atomic::AtomicCell::new(0.0),
        };
        cube.get_mob_entity()
            .living_entity
            .set_attribute_base(&Attributes::ATTACK_DAMAGE, 10.0);
        cube.get_mob_entity().set_no_ai(true);
        cube.slime.deal_damage(&*cube, &target);
        assert_eq!(target.damage.load(), 0.0);
        cube.get_mob_entity().set_no_ai(false);
        cube.slime.deal_damage(&*cube, &target);
        assert_eq!(target.damage.swap(0.0), 12.0);
        target.entity.set_pos(Vector3::new(20.0, 64.0, 0.0));
        cube.slime.deal_damage(&*cube, &target);
        assert_eq!(target.damage.load(), 0.0);
        target.entity.set_pos(Vector3::new(0.25, 64.0, 0.0));
        cube.push(&target);
        assert_eq!(target.damage.swap(0.0), 12.0);
        let slime = SlimeEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::SLIME,
        ));
        slime.deal_damage(&*slime, &target);
        assert_eq!(target.damage.load(), 0.0);
        slime.set_size(2, true);
        slime.deal_damage(&*slime, &target);
        assert_eq!(target.damage.load(), 2.0);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn dying_cubes_split_on_removal_into_the_same_type_with_inherited_state() -> TestResult {
        for entity_type in [&EntityType::MAGMA_CUBE, &EntityType::SLIME] {
            let folder = tempfile::tempdir()?;
            let world = world(&folder);
            let cube = crate::entity::r#type::from_type(
                entity_type,
                Vector3::new(0.0, 64.0, 0.0),
                &world,
                uuid::Uuid::new_v4(),
            );
            let Some(parent) = cube.get_mob() else {
                return Err("missing parent mob".into());
            };
            let Some(shared) = parent.as_cube_mob() else {
                return Err("missing cube state".into());
            };
            shared.set_size(4, true);
            parent.get_mob_entity().set_no_ai(true);
            parent
                .get_mob_entity()
                .persistence_required
                .store(true, Ordering::Relaxed);
            cube.get_entity().set_silent(true);
            cube.get_entity().set_has_no_gravity(true);
            cube.get_entity().add_scoreboard_tag("split-parent");
            cube.get_entity().set_on_fire(true);
            world.spawn_entity(cube.clone());
            parent.get_mob_entity().living_entity.health.store(0.0);
            parent.post_tick();
            assert_eq!(world.entities.load().len(), 1);
            cube.get_entity().remove();
            let children = world.entities.load_full();
            assert!((2..=4).contains(&children.len()));
            for child in children.iter() {
                assert_eq!(
                    child.cast_any().is::<MagmaCubeEntity>(),
                    entity_type == &EntityType::MAGMA_CUBE
                );
                assert_eq!(
                    child.cast_any().is::<SlimeEntity>(),
                    entity_type == &EntityType::SLIME
                );
                let Some(mob) = child.get_mob() else {
                    return Err("missing child mob".into());
                };
                assert_eq!(child.get_entity().data.load(Ordering::Relaxed), 2);
                assert_eq!(mob.get_mob_entity().living_entity.health.load(), 4.0);
                assert_eq!(
                    mob.get_mob_entity()
                        .living_entity
                        .get_attribute_value(&Attributes::ARMOR),
                    if entity_type == &EntityType::MAGMA_CUBE {
                        6.0
                    } else {
                        0.0
                    }
                );
                assert!(mob.get_mob_entity().is_no_ai());
                assert!(
                    mob.get_mob_entity()
                        .persistence_required
                        .load(Ordering::Relaxed)
                );
                assert!(child.get_entity().is_silent());
                assert!(child.get_entity().has_no_gravity());
                assert_eq!(
                    child.get_entity().is_on_fire(),
                    entity_type == &EntityType::SLIME
                );
                assert!(
                    child
                        .get_entity()
                        .scoreboard_tags
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .contains("split-parent")
                );
                assert!((child.get_entity().pos.load().x.abs() - 0.52).abs() < 1.0e-6);
            }
            cube.get_entity().remove();
            assert_eq!(world.entities.load().len(), children.len());
            world.level.shutdown().await;
        }
        Ok(())
    }
}
