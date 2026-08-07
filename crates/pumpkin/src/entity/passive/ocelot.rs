use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
};

use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::client::actor_event::ActorEventType;
use pumpkin_protocol::java::client::play::Metadata;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, avoid_entity::AvoidEntityGoal, breed::BreedGoal,
        escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};

const TEMPT_ITEMS: &[&Item] = &[&Item::COD, &Item::SALMON];

/// Represents an Ocelot, a shy passive mob found in jungles.
///
/// Wiki: <https://minecraft.wiki/w/Ocelot>
pub struct OcelotEntity {
    pub mob_entity: MobEntity,
    pub is_trusting: AtomicBool,
}

impl OcelotEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let ocelot = Self {
            mob_entity,
            is_trusting: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(ocelot);
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

            // Goal 1: FloatGoal (SwimGoal)
            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            // Goal 1: PanicGoal (EscapeDangerGoal)
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.5));
            // Goal 3: OcelotTemptGoal
            goal_selector.add_goal(3, Box::new(TemptGoal::new(0.6, TEMPT_ITEMS)));
            // Goal 4: OcelotAvoidEntityGoal (when not trusting)
            goal_selector.add_goal(
                4,
                Box::new(AvoidEntityGoal::new(&EntityType::PLAYER, 16.0, 0.8, 1.33)),
            );
            // Goal 9: BreedGoal
            goal_selector.add_goal(9, BreedGoal::new(0.8));
            // Goal 9: FollowParentGoal
            goal_selector.add_goal(9, Box::new(FollowParentGoal::new(0.8)));
            // Goal 10: WaterAvoidingRandomStrollGoal (WanderAroundGoal)
            goal_selector.add_goal(10, Box::new(WanderAroundGoal::new(0.8)));
            // Goal 11: LookAtPlayerGoal
            goal_selector.add_goal(
                11,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 10.0),
            );
            // Goal 11: RandomLookAroundGoal
            goal_selector.add_goal(11, Box::new(RandomLookAroundGoal::default()));
        };

        {
            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            // Target Goal 1: NearestAttackableTargetGoal for Chicken and Turtle
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::CHICKEN, false),
            );
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::TURTLE, false),
            );
        };

        mob_arc
    }

    pub fn is_trusting(&self) -> bool {
        self.is_trusting.load(Ordering::Relaxed)
    }

    pub fn set_trusting(&self, trusting: bool) {
        self.is_trusting.store(trusting, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                TrackedData::TRUSTING,
                MetaDataType::BOOLEAN,
                trusting,
            )],
            None,
        );
    }
}

impl NBTStorage for OcelotEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            self.write_animal_nbt(nbt);
            nbt.put_bool("Trusting", self.is_trusting.load(Ordering::Relaxed));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.read_animal_nbt(nbt);
            if let Some(trusting) = nbt.get_bool("Trusting") {
                self.is_trusting.store(trusting, Ordering::Relaxed);
            }
        })
    }
}

impl Animal for OcelotEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        let item = item_stack.get_item();
        item.has_tag(&tag::Item::MINECRAFT_OCELOT_FOOD)
            || item == &Item::COD
            || item == &Item::SALMON
    }
}

impl Mob for OcelotEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let is_baby = entity.age.load(Ordering::Relaxed) < 0;
            if is_baby {
                entity.send_meta_data(
                    &[Metadata::new(
                        TrackedData::BABY_ID,
                        MetaDataType::BOOLEAN,
                        true,
                    )],
                    None,
                );
            }
            entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::TRUSTING,
                    MetaDataType::BOOLEAN,
                    self.is_trusting.load(Ordering::Relaxed),
                )],
                None,
            );
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let is_food = self.is_food(item_stack);
            let dist_sqr = self
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&player.get_entity().pos.load());

            if !self.is_trusting() && is_food && dist_sqr < 9.0 {
                item_stack.decrement_unless_creative(player.gamemode.load(), 1);

                let mut rng = rand::rng();
                if rng.random_range(0..3) == 0 {
                    self.set_trusting(true);
                    self.get_entity().world.load().send_entity_status(
                        self.get_entity(),
                        EntityStatus::TrustingSucceeded,
                        Some(ActorEventType::TrustingSucceeded),
                    );
                } else {
                    self.get_entity().world.load().send_entity_status(
                        self.get_entity(),
                        EntityStatus::TrustingFailed,
                        Some(ActorEventType::TrustingFailed),
                    );
                }

                return true;
            }

            self.animal_interact(
                player,
                item_stack,
                pumpkin_data::sound::Sound::EntityOcelotAmbient,
            )
            .await
        })
    }
}
