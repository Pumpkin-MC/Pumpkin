use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};

const TEMPT_ITEMS: &[&Item] = &[&Item::WHEAT];

pub struct GoatEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub is_screaming: AtomicBool,
    pub has_left_horn: AtomicBool,
    pub has_right_horn: AtomicBool,
}

impl GoatEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let goat = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            is_screaming: AtomicBool::new(false),
            has_left_horn: AtomicBool::new(true),
            has_right_horn: AtomicBool::new(true),
        };
        let mob_arc = Arc::new(goat);
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
            goal_selector.add_goal(1, EscapeDangerGoal::new(2.0));
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.25, TEMPT_ITEMS)));
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.25)));
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
    pub fn is_screaming(&self) -> bool {
        self.is_screaming.load(Ordering::Relaxed)
    }

    pub fn set_screaming(&self, screaming: bool) {
        self.is_screaming.store(screaming, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::goat::DATA_IS_SCREAMING_GOAT,
                screaming,
            )],
            None,
        );
    }

    #[must_use]
    pub fn has_left_horn(&self) -> bool {
        self.has_left_horn.load(Ordering::Relaxed)
    }

    pub fn set_has_left_horn(&self, has_horn: bool) {
        self.has_left_horn.store(has_horn, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::goat::DATA_HAS_LEFT_HORN,
                has_horn,
            )],
            None,
        );
    }

    #[must_use]
    pub fn has_right_horn(&self) -> bool {
        self.has_right_horn.load(Ordering::Relaxed)
    }

    pub fn set_has_right_horn(&self, has_horn: bool) {
        self.has_right_horn.store(has_horn, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::goat::DATA_HAS_RIGHT_HORN,
                has_horn,
            )],
            None,
        );
    }
}

impl AgeableMob for GoatEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Animal for GoatEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_GOAT_FOOD)
            || TEMPT_ITEMS.iter().any(|i| i.id == item_stack.item.id)
    }
}

impl Mob for GoatEntity {
    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("IsScreamingGoat", self.is_screaming());
        nbt.put_bool("HasLeftHorn", self.has_left_horn());
        nbt.put_bool("HasRightHorn", self.has_right_horn());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(screaming) = nbt.get_bool("IsScreamingGoat") {
            self.set_screaming(screaming);
        }
        if let Some(left) = nbt.get_bool("HasLeftHorn") {
            self.set_has_left_horn(left);
        }
        if let Some(right) = nbt.get_bool("HasRightHorn") {
            self.set_has_right_horn(right);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        self.ageable_ai_step();
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::goat::DATA_BABY_ID,
                    true,
                )],
                None,
            );
        }
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::goat::DATA_IS_SCREAMING_GOAT,
                self.is_screaming(),
            )],
            None,
        );
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::goat::DATA_HAS_LEFT_HORN,
                self.has_left_horn(),
            )],
            None,
        );
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::goat::DATA_HAS_RIGHT_HORN,
                self.has_right_horn(),
            )],
            None,
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let item = item_stack.get_item();
        if item == &Item::BUCKET && !self.is_baby() {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            let entity = self.get_entity();
            let world = entity.world.load();
            let sound = if self.is_screaming() {
                Sound::EntityGoatScreamingMilk
            } else {
                Sound::EntityGoatMilk
            };
            world.play_sound(sound, SoundCategory::Neutral, &entity.pos.load());
            return true;
        }

        let ambient_sound = if self.is_screaming() {
            Sound::EntityGoatScreamingAmbient
        } else {
            Sound::EntityGoatAmbient
        };
        self.animal_interact(player, item_stack, ambient_sound)
    }
}
