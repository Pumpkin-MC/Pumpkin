use std::sync::{
    Arc, Weak,
    atomic::{AtomicU8, Ordering},
};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        beg::BegGoal, breed::BreedGoal, escape_danger::EscapeDangerGoal,
        follow_parent::FollowParentGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct WolfEntity {
    pub mob_entity: MobEntity,
    pub variant: AtomicU8,
}

impl WolfEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let wolf = Self {
            mob_entity,
            variant: AtomicU8::new(3), // Default to pale
        };
        let mob_arc = Arc::new(wolf);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            // goal_selector.add_goal(2, SitGoal::new(mob_arc.clone()));
            goal_selector.add_goal(4, EscapeDangerGoal::new(1.5));
            goal_selector.add_goal(5, BreedGoal::new(1.0));
            // goal_selector.add_goal(6, FollowOwnerGoal::new(1.0, 10.0, 2.0, false));
            goal_selector.add_goal(8, Box::new(FollowParentGoal::new(1.1)));
            goal_selector.add_goal(9, BegGoal::new(8.0, &[&Item::BONE]));
            goal_selector.add_goal(
                10,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(10, Box::new(RandomLookAroundGoal::default()));
            goal_selector.add_goal(12, Box::new(WanderAroundGoal::new(1.0)));
        };

        mob_arc
    }
}

impl NBTStorage for WolfEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            let variant_str = match self.variant.load(Ordering::Relaxed) {
                0 => "minecraft:ashen",
                1 => "minecraft:black",
                2 => "minecraft:chestnut",
                4 => "minecraft:rusty",
                5 => "minecraft:snowy",
                6 => "minecraft:spotted",
                7 => "minecraft:striped",
                8 => "minecraft:woods",
                _ => "minecraft:pale",
            };
            nbt.put_string("variant", variant_str.to_string());
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            if let Some(variant_str) = nbt.get_string("variant") {
                let variant = match variant_str
                    .strip_prefix("minecraft:")
                    .unwrap_or(variant_str)
                {
                    "ashen" => 0,
                    "black" => 1,
                    "chestnut" => 2,
                    "rusty" => 4,
                    "snowy" => 5,
                    "spotted" => 6,
                    "striped" => 7,
                    "woods" => 8,
                    _ => 3,
                };
                self.variant.store(variant, Ordering::Relaxed);
            }
        })
    }
}

impl Mob for WolfEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_set_variant_name(&self, name: &str) {
        let variant = match name.strip_prefix("minecraft:").unwrap_or(name) {
            "ashen" => 0,
            "black" => 1,
            "chestnut" => 2,
            "rusty" => 4,
            "snowy" => 5,
            "spotted" => 6,
            "striped" => 7,
            "woods" => 8,
            _ => 3,
        };
        self.variant.store(variant, Ordering::Relaxed);
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
                    TrackedData::VARIANT,
                    MetaDataType::WOLF_VARIANT,
                    VarInt(self.variant.load(Ordering::Relaxed) as i32),
                )],
                None,
            );
        })
    }
}
