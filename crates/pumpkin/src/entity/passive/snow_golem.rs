use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;

use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, ranged_attack::RangedAttackGoal,
        wander_around::WanderAroundGoal,
    },
    item::ItemEntity,
    mob::{Mob, MobEntity, RangedAttackMob},
    player::Player,
    projectile::snowball::SnowballEntity,
};

pub struct SnowGolemEntity {
    pub mob_entity: MobEntity,
    pub has_pumpkin: AtomicBool,
}

impl SnowGolemEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let snow_golem = Self {
            mob_entity,
            has_pumpkin: AtomicBool::new(true),
        };
        let mob_arc = Arc::new(snow_golem);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        let ranged_weak: Weak<dyn RangedAttackMob> = {
            let ranged_arc: Arc<dyn RangedAttackMob> = mob_arc.clone();
            Arc::downgrade(&ranged_arc)
        };

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(
                1,
                Box::new(RangedAttackGoal::new(ranged_weak, 1.25, 20, 10.0)),
            );
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::ZOMBIE, true),
            );
        };

        mob_arc
    }

    #[must_use]
    pub fn has_pumpkin(&self) -> bool {
        self.has_pumpkin.load(Ordering::Relaxed)
    }

    pub fn set_has_pumpkin(&self, has_pumpkin: bool) {
        self.has_pumpkin.store(has_pumpkin, Ordering::Relaxed);
        let entity = self.get_entity();
        let flags = if has_pumpkin { 16u8 } else { 0u8 };
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::snow_golem::DATA_PUMPKIN_ID,
                flags as i8,
            )],
            None,
        );
    }

    pub fn throw_snowball(&self, target: &Arc<dyn EntityBase>) {
        let entity = self.get_entity();
        let world = entity.world.load_full();

        let snowball_entity = Entity::new(world.clone(), entity.pos.load(), &EntityType::SNOWBALL);
        let snowball = SnowballEntity::new_shot(snowball_entity, entity);

        let mob_pos = entity.pos.load();
        let target_entity = target.get_entity();
        let target_pos = target_entity.pos.load();
        let target_height = f64::from(target_entity.entity_dimension.load().height);

        let dx = target_pos.x - mob_pos.x;
        let dy = (target_pos.y + target_height / 3.0) - snowball.get_entity().pos.load().y;
        let dz = target_pos.z - mob_pos.z;
        let horizontal_distance = dx.hypot(dz);
        let yo = horizontal_distance * 0.2;

        snowball.thrown.set_velocity(dx, dy + yo, dz, 1.6, 12.0);

        if !entity.silent.load(Ordering::Relaxed) {
            world.play_sound(Sound::EntitySnowballThrow, SoundCategory::Neutral, &mob_pos);
        }

        let snowball_arc: Arc<dyn EntityBase> = Arc::new(snowball);
        world.spawn_entity(snowball_arc);
    }
}

impl Mob for SnowGolemEntity {
    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("Pumpkin", self.has_pumpkin());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(pumpkin) = nbt.get_bool("Pumpkin") {
            self.set_has_pumpkin(pumpkin);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let flags = if self.has_pumpkin() { 16u8 } else { 0u8 };
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::snow_golem::DATA_PUMPKIN_ID,
                flags as i8,
            )],
            None,
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if item_stack.get_item() == &Item::SHEARS && self.has_pumpkin() {
            self.set_has_pumpkin(false);
            let entity = self.get_entity();
            let world = entity.world.load();
            let pos = entity.pos.load();
            world.play_sound(Sound::EntitySnowGolemShear, SoundCategory::Players, &pos);
            let item_entity = Arc::new(ItemEntity::new(
                Entity::new(world.clone(), pos, &EntityType::ITEM),
                ItemStack::new(1, &Item::CARVED_PUMPKIN),
            ));
            world.spawn_entity(item_entity);
            player.damage_held_item(1);
            return true;
        }
        false
    }
}

impl RangedAttackMob for SnowGolemEntity {
    fn perform_ranged_attack(&self, target: &Arc<dyn EntityBase>, _power: f32) {
        self.throw_snowball(target);
    }
}
