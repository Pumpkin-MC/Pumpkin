use crate::entity::projectile::owner::ProjectileOwner;
use std::sync::atomic::Ordering;

use pumpkin_data::damage::DamageType;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

use crate::{
    block::blocks::fire::{FireBlockBase, fire::FireBlock},
    entity::{
        Entity, EntityBase,
        projectile::{HurtingProjectile, ProjectileHit, ThrownItemEntity},
    },
    server::Server,
};

pub struct SmallFireballEntity {
    pub thrown: HurtingProjectile,
}

impl SmallFireballEntity {
    #[must_use]
    pub const fn new(entity: Entity) -> Self {
        let thrown = ThrownItemEntity {
            entity,
            owner: ProjectileOwner::none(),
            gravity: 0.0,
        };
        Self {
            thrown: HurtingProjectile::new(thrown, HurtingProjectile::DEFAULT_ACCELERATION_POWER),
        }
    }

    #[must_use]
    pub fn new_shot(entity: Entity, shooter: &Entity, direction: Vector3<f64>) -> Self {
        let thrown = HurtingProjectile::new(
            ThrownItemEntity::new(entity, shooter, 0.0),
            HurtingProjectile::DEFAULT_ACCELERATION_POWER,
        );
        thrown.assign_directional_movement(direction);
        Self { thrown }
    }
}

impl EntityBase for SmallFireballEntity {
    fn get_projectile_owner(&self) -> Option<&ProjectileOwner> {
        Some(&self.thrown.owner)
    }

    fn on_deflection(&self, by_attack: bool) {
        self.thrown.on_deflection(by_attack);
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        self.thrown.write_nbt(nbt);
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        self.thrown.read_nbt(nbt);
    }

    fn tick(&self, caller: &dyn EntityBase, server: &Server) {
        self.thrown.process_tick(
            caller,
            server,
            HurtingProjectile::INERTIA,
            HurtingProjectile::LIQUID_INERTIA,
            true,
            |_| true,
        );
    }

    fn get_entity(&self) -> &Entity {
        self.thrown.get_entity()
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }
    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, hit: ProjectileHit) {
        let world = self.get_entity().world.load();
        let owner = self.thrown.owner.get(&world);
        match hit {
            ProjectileHit::Entity {
                ref entity,
                hit_pos,
                ..
            } => {
                let target = entity.get_entity();
                let fire_ticks = target.fire_ticks.load(Ordering::Relaxed);
                target.set_on_fire_for(5.0);
                let damage_type = if owner.is_some() {
                    DamageType::FIREBALL
                } else {
                    DamageType::UNATTRIBUTED_FIREBALL
                };
                if !entity.damage_with_context(
                    entity.as_ref(),
                    5.0,
                    damage_type,
                    Some(hit_pos),
                    Some(self),
                    owner.as_deref(),
                ) {
                    target.fire_ticks.store(fire_ticks, Ordering::Relaxed);
                }
            }
            ProjectileHit::Block { pos, face, .. } => {
                let griefing = world.level_info.load().game_rules.mob_griefing;
                if owner.as_ref().is_none_or(|owner| owner.get_mob().is_none()) || griefing {
                    let fire_pos = pos.offset(face.to_offset());
                    if world.get_block_state(&fire_pos).is_air() {
                        let fire = FireBlockBase::get_fire_type(&world, &fire_pos);
                        let state = FireBlock.get_state_for_position(&world, &fire, &fire_pos);
                        world.set_block_state(&fire_pos, state, BlockFlags::NOTIFY_ALL);
                    }
                }
            }
        }
    }
}
