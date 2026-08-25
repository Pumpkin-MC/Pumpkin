use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

use crate::entity::item_steerable::{ItemBasedSteering, ItemSteerable};

pub struct StriderEntity {
    pub mob_entity: MobEntity,
    pub steering: ItemBasedSteering,
    pub saddled: std::sync::atomic::AtomicBool,
}

impl StriderEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let strider = Self {
            mob_entity,
            steering: ItemBasedSteering::default(),
            saddled: std::sync::atomic::AtomicBool::new(false),
        };
        let mob_arc = Arc::new(strider);
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
            goal_selector.add_goal(1, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                2,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(3, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }
}

impl Mob for StriderEntity {
    fn mob_write_nbt<'a>(
        &'a self,
        nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            nbt.put_bool("Saddle", self.is_saddled());
        })
    }

    fn mob_read_nbt<'a>(
        &'a self,
        nbt: &'a pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            if let Some(saddle) = nbt.get_byte("Saddle") {
                self.set_saddled(saddle == 1);
            }
        })
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_item_steerable(&self) -> Option<&dyn ItemSteerable> {
        Some(self)
    }

    fn is_saddled(&self) -> bool {
        self.saddled.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn can_be_saddled(&self) -> bool {
        self.mob_entity.living_entity.entity.is_alive()
    }

    fn set_saddled(&self, saddled: bool) {
        self.saddled
            .store(saddled, std::sync::atomic::Ordering::Relaxed);
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<crate::entity::player::Player>,
        _item_stack: &'a mut pumpkin_data::item_stack::ItemStack,
    ) -> crate::entity::EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if self.is_saddled() {
                let world = player.world();
                let ent = &self.mob_entity.living_entity.entity;
                if let Some(vehicle) = world.get_entity_by_id(ent.entity_id)
                    && let Some(passenger) = world.get_player_by_id(player.entity_id())
                {
                    ent.add_passenger(vehicle, passenger as Arc<dyn EntityBase>)
                        .await;
                    return true;
                }
            }
            false
        })
    }
}

impl ItemSteerable for StriderEntity {
    fn boost(&self) -> bool {
        self.steering.boost()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
