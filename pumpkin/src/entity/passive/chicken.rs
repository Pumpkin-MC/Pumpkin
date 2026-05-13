use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, Ordering, Ordering::Relaxed},
};

use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{entity::EntityType, item::Item};
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    player::Player,
};
use pumpkin_nbt::compound::NbtCompound;

const TEMPT_ITEMS: &[&Item] = &[
    &Item::WHEAT_SEEDS,
    &Item::MELON_SEEDS,
    &Item::PUMPKIN_SEEDS,
    &Item::BEETROOT_SEEDS,
    &Item::TORCHFLOWER_SEEDS,
    &Item::PITCHER_POD,
];

/// Represents a Chicken, a passive mob that lays eggs and is immune to fall damage.
///
/// Wiki: <https://minecraft.wiki/w/Chicken>
pub struct ChickenEntity {
    pub mob_entity: MobEntity,
    egg_lay_time: AtomicI32,
}

impl ChickenEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let egg_lay_time = rand::rng().random_range(6000..12000);
        let chicken = Self {
            mob_entity,
            egg_lay_time: AtomicI32::new(egg_lay_time),
        };
        let mob_arc = Arc::new(chicken);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.4));
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.0, TEMPT_ITEMS)));
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.1)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }
}

impl NBTStorage for ChickenEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            nbt.put_int("EggLayTime", self.egg_lay_time.load(Ordering::Relaxed));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.egg_lay_time
                .store(nbt.get_int("EggLayTime").unwrap_or(6000), Ordering::Relaxed);
        })
    }
}

impl Mob for ChickenEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async {
            if self.mob_entity.living_entity.dead.load(Relaxed) {
                return;
            }
            if self.egg_lay_time.fetch_sub(1, Ordering::Relaxed) <= 1 {
                let next_time = rand::rng().random_range(6000..12000);
                let entity = &self.mob_entity.living_entity.entity;
                let world = entity.world.load_full();
                let pos = entity.block_pos.load();
                world.drop_stack(&pos, ItemStack::new(1, &Item::EGG)).await;
                self.egg_lay_time.store(next_time, Ordering::Relaxed);
            }
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let is_food = TEMPT_ITEMS.iter().any(|i| i.id == item_stack.item.id);
            if is_food && self.is_breeding_ready() && !self.is_in_love() {
                item_stack.decrement_unless_creative(player.gamemode.load(), 1);

                self.mob_entity.set_love_ticks(600);
                let entity = &self.mob_entity.living_entity.entity;
                let world = entity.world.load();
                let pos = entity.pos.load();

                world
                    .spawn_particle(
                        pos + Vector3::new(0.0, f64::from(entity.height()), 0.0),
                        Vector3::new(0.5, 0.5, 0.5),
                        1.0,
                        7,
                        Particle::Heart,
                    )
                    .await;
                world
                    .play_sound(
                        Sound::EntityChickenAmbient,
                        SoundCategory::Neutral,
                        &entity.pos.load(),
                    )
                    .await;
                return true;
            }
            false
        })
    }
}
