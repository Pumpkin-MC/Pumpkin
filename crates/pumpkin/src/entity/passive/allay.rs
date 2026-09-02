use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        escape_danger::EscapeDangerGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    player::Player,
};

pub struct AllayEntity {
    pub mob_entity: MobEntity,
    pub dancing: AtomicBool,
    pub can_duplicate: AtomicBool,
    pub duplication_cooldown: AtomicI32,
    pub owner: AtomicCell<Option<Uuid>>,
}

impl AllayEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let allay = Self {
            mob_entity,
            dancing: AtomicBool::new(false),
            can_duplicate: AtomicBool::new(true),
            duplication_cooldown: AtomicI32::new(0),
            owner: AtomicCell::new(None),
        };
        let mob_arc = Arc::new(allay);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.25));
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    #[must_use]
    pub fn is_dancing(&self) -> bool {
        self.dancing.load(Ordering::Relaxed)
    }

    pub fn set_dancing(&self, dancing: bool) {
        self.dancing.store(dancing, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::allay::DATA_DANCING,
                dancing,
            )],
            None,
        );
    }

    #[must_use]
    pub fn can_duplicate(&self) -> bool {
        self.can_duplicate.load(Ordering::Relaxed)
    }

    pub fn set_can_duplicate(&self, can_duplicate: bool) {
        self.can_duplicate.store(can_duplicate, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::allay::DATA_CAN_DUPLICATE,
                can_duplicate,
            )],
            None,
        );
    }
}

impl Mob for AllayEntity {
    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("CanDuplicate", self.can_duplicate());
        nbt.put_int(
            "DuplicationCooldown",
            self.duplication_cooldown.load(Ordering::Relaxed),
        );
        if let Some(owner) = self.owner.load() {
            nbt.put_uuid("Owner", owner);
        }
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(can) = nbt.get_bool("CanDuplicate") {
            self.set_can_duplicate(can);
        }
        if let Some(cd) = nbt.get_int("DuplicationCooldown") {
            self.duplication_cooldown.store(cd, Ordering::Relaxed);
        }
        if let Some(owner) = nbt.get_uuid("Owner") {
            self.owner.store(Some(owner));
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let cd = self.duplication_cooldown.load(Ordering::Relaxed);
        if cd > 0 {
            let next = cd - 1;
            self.duplication_cooldown.store(next, Ordering::Relaxed);
            if next == 0 {
                self.set_can_duplicate(true);
            }
        }
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::allay::DATA_DANCING,
                self.is_dancing(),
            )],
            None,
        );
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::allay::DATA_CAN_DUPLICATE,
                self.can_duplicate(),
            )],
            None,
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let item = item_stack.get_item();

        if self.is_dancing() && self.can_duplicate() && item == &Item::AMETHYST_SHARD {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            self.set_can_duplicate(false);
            self.duplication_cooldown.store(6000, Ordering::Relaxed);

            let entity = self.get_entity();
            let world = entity.world.load();
            let pos = entity.pos.load();
            world.spawn_particle(
                pos + Vector3::new(0.0, f64::from(entity.height()), 0.0),
                Vector3::new(0.5, 0.5, 0.5),
                1.0,
                7,
                Particle::Heart,
            );
            world.play_sound(
                Sound::EntityAllayAmbientWithoutItem,
                SoundCategory::Neutral,
                &pos,
            );

            let new_allay = Self::new(Entity::new(world.clone(), pos, &EntityType::ALLAY));
            world.spawn_entity(new_allay);
            return true;
        }

        if self.owner.load().is_none() {
            self.owner.store(Some(player.gameprofile.id));
        }

        false
    }
}
