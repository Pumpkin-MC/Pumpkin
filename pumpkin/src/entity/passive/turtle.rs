use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::{Arc, Weak};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::position::BlockPos;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ageable::AgeableMob,
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, look_at_entity::LookAtEntityGoal,
        tempt::TemptGoal, turtle_go_home::TurtleGoHomeGoal,
        turtle_go_to_water::TurtleGoToWaterGoal, turtle_lay_egg::TurtleLayEggGoal,
        turtle_random_stroll::TurtleRandomStrollGoal, turtle_travel::TurtleTravelGoal,
    },
    mob::{Mob, MobEntity},
};

/// `ItemTags.TURTLE_FOOD` currently only contains seagrass (confirmed against
/// `pumpkin-data/src/generated/tag.rs`'s `MINECRAFT_TURTLE_FOOD`), so this list already
/// matches the tag exactly rather than being a subset of it.
const TEMPT_ITEMS: &[&Item] = &[&Item::SEAGRASS];

/// `BlockPos.CODEC` serializes as an `[x, y, z]` int array (matches the Bee pattern).
fn block_pos_to_nbt(pos: BlockPos) -> NbtTag {
    NbtTag::IntArray(vec![pos.0.x, pos.0.y, pos.0.z])
}

fn block_pos_from_nbt(nbt: &NbtCompound, name: &str) -> Option<BlockPos> {
    let &[x, y, z] = nbt.get_int_array(name)? else {
        return None;
    };
    Some(BlockPos::new(x, y, z))
}

/// Represents a Turtle, a passive aquatic mob that lays eggs on sand near its home beach.
///
/// Wiki: <https://minecraft.wiki/w/Turtle>
pub struct TurtleEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: crate::entity::ageable::AgeableData,
    has_egg: AtomicBool,
    /// `Turtle.homePos`. `None` means "not yet finalized"; set the first time the data
    /// tracker initializes (see `mob_init_data_tracker`) unless NBT already restored it.
    home_pos: AtomicCell<Option<BlockPos>>,
    going_home: AtomicBool,
    laying_egg: AtomicBool,
}

impl TurtleEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let turtle = Self {
            mob_entity,
            ageable_data: crate::entity::ageable::AgeableData::default(),
            has_egg: AtomicBool::new(false),
            home_pos: AtomicCell::new(None),
            going_home: AtomicBool::new(false),
            laying_egg: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(turtle);
        let turtle_weak = Arc::downgrade(&mob_arc);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            // Priorities follow `Turtle.registerGoals` (`Turtle.java:151-160`). No float/swim
            // goal: vanilla doesn't register one for Turtle either.
            goal_selector.add_goal(0, EscapeDangerGoal::new(1.2));
            goal_selector.add_goal(1, BreedGoal::new(1.0));
            goal_selector.add_goal(1, TurtleLayEggGoal::new(turtle_weak.clone(), 1.0));
            goal_selector.add_goal(2, Box::new(TemptGoal::new(1.1, TEMPT_ITEMS, false)));
            goal_selector.add_goal(3, TurtleGoToWaterGoal::new(turtle_weak.clone(), 1.0));
            goal_selector.add_goal(4, TurtleGoHomeGoal::new(turtle_weak.clone(), 1.0));
            goal_selector.add_goal(7, TurtleTravelGoal::new(turtle_weak.clone(), 1.0));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(9, TurtleRandomStrollGoal::new(turtle_weak, 1.0));
        };

        mob_arc
    }

    #[must_use]
    pub fn has_egg(&self) -> bool {
        self.has_egg.load(Relaxed)
    }

    pub fn set_has_egg(&self, has_egg: bool) {
        self.has_egg.store(has_egg, Relaxed);
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::HAS_EGG,
                MetaDataType::BOOLEAN,
                has_egg,
            )],
            None,
        );
    }

    #[must_use]
    pub fn is_going_home(&self) -> bool {
        self.going_home.load(Relaxed)
    }

    pub fn set_going_home(&self, going_home: bool) {
        self.going_home.store(going_home, Relaxed);
    }

    #[must_use]
    pub fn is_laying_egg(&self) -> bool {
        self.laying_egg.load(Relaxed)
    }

    pub fn set_laying_egg(&self, laying_egg: bool) {
        self.laying_egg.store(laying_egg, Relaxed);
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::LAYING_EGG,
                MetaDataType::BOOLEAN,
                laying_egg,
            )],
            None,
        );
    }
}

impl AgeableMob for TurtleEntity {
    fn get_ageable_data(&self) -> &crate::entity::ageable::AgeableData {
        &self.ageable_data
    }
}

impl NBTStorage for TurtleEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            self.write_ageable_nbt(nbt);
            nbt.put_bool("has_egg", self.has_egg());
            if let Some(home_pos) = self.home_pos.load() {
                nbt.put("home_pos", block_pos_to_nbt(home_pos));
            }
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.read_ageable_nbt(nbt);
            self.has_egg
                .store(nbt.get_bool("has_egg").unwrap_or(false), Relaxed);
            self.home_pos.store(block_pos_from_nbt(nbt, "home_pos"));
        })
    }
}

impl Mob for TurtleEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// Vanilla `Turtle.canBeLeashed` -> `false`.
    fn can_be_leashed(&self) -> bool {
        false
    }

    /// Vanilla `Turtle.isPushedByFluid` -> `false`: turtles aren't swept by currents.
    fn mob_is_pushed_by_fluids(&self) -> bool {
        false
    }

    fn get_home(&self) -> Option<BlockPos> {
        self.home_pos.load()
    }

    /// Vanilla `Turtle.TurtleBreedGoal.breed` (`Turtle.java:300-326`): a successful breed
    /// always leaves the turtle carrying an egg, on top of the generic `BreedGoal` effects.
    fn on_bred(&self, _mate: &dyn EntityBase) {
        self.set_has_egg(true);
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            if self.is_baby() {
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
                    TrackedData::HAS_EGG,
                    MetaDataType::BOOLEAN,
                    self.has_egg(),
                )],
                None,
            );

            // `Turtle.finalizeSpawn`: sets homePos to the spawn position. Reused here as the
            // "not yet set" case, since NBT restore (which runs before this on load) already
            // populated home_pos for existing turtles.
            if self.home_pos.load().is_none() {
                self.home_pos.store(Some(entity.block_pos.load()));
            }
        })
    }
}
