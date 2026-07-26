use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
};

use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_inventory::screen_handler::InventoryPlayer;

use crate::entity::{
    Entity, EntityBaseFuture, NBTStorage,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    player::Player,
};

/// Represents a Parrot, a passive flying mob that can mimic nearby mob sounds.
///
/// Wiki: <https://minecraft.wiki/w/Parrot>
pub struct ParrotEntity {
    pub mob_entity: MobEntity,
    pub cookie_cooldown: Arc<AtomicBool>,
}

impl ParrotEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);

        let parrot = Self {
            mob_entity,
            cookie_cooldown: Arc::new(AtomicBool::new(false)),
        };

        let mob_arc = Arc::new(parrot);

        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

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

impl NBTStorage for ParrotEntity {}

impl Mob for ParrotEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if item_stack.item.id == Item::COOKIE.id {
                // Prevent duplicate interaction packets consuming 2 cookies
                if self.cookie_cooldown.swap(true, Ordering::Relaxed) {
                    return true;
                }

                let cooldown = self.cookie_cooldown.clone();

                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    cooldown.store(false, Ordering::Relaxed);
                });

                // TODO: Parrot should get poison before dying.
                // Bedrock should get fatal poison effect and not die.

                if !player.has_infinite_materials() {
                    item_stack.decrement(1);
                }

                let entity = &self.mob_entity.living_entity.entity;
                let world = entity.world.load();

                if let Some(dyn_self) = world.get_entity_by_id(entity.entity_id) {
                    dyn_self
                        .damage(&*dyn_self, f32::MAX, DamageType::MAGIC)
                        .await;
                }

                return true;
            }

            self.mob_entity.mob_interact(player, item_stack).await
        })
    }
}
