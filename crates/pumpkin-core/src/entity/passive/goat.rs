use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    ai::pathfinder::node::PathType,
    custom_sound::CustomSound,
    mob::{Mob, MobEntity, spawn::SpawnGroupData},
    passive::animal::Animal,
    player::Player,
};
use crate::item::items::bucket::fill_bucket_stack;
use crate::world::World;

const TEMPT_ITEMS: &[&Item] = &[&Item::WHEAT];
const ADULT_ATTACK_DAMAGE: f64 = 2.0;
const BABY_ATTACK_DAMAGE: f64 = 1.0;
const GOAT_FALL_DAMAGE_REDUCTION: f32 = 10.0;
const GOAT_SCREAMING_CHANCE: f64 = 0.02;
const UNIHORN_CHANCE: f32 = 0.1;

pub struct GoatEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub is_screaming: AtomicBool,
    pub has_left_horn: AtomicBool,
    pub has_right_horn: AtomicBool,
}

impl GoatEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        {
            let navigator = mob_entity
                .navigator
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            navigator.set_can_float(true);
            navigator.set_pathfinding_malus(PathType::PowderSnow, -1.0);
            // Vanilla's `PathType.ON_TOP_OF_POWDER_SNOW`.
            navigator.set_pathfinding_malus(PathType::DangerPowderSnow, -1.0);
        }
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
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(2.0));
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.25, TEMPT_ITEMS, false)));
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
        entity.set_synced_data(
            pumpkin_data::tracked_data::goat::DATA_IS_SCREAMING_GOAT,
            screaming,
        );
    }

    #[must_use]
    pub fn has_left_horn(&self) -> bool {
        self.has_left_horn.load(Ordering::Relaxed)
    }

    pub fn set_has_left_horn(&self, has_horn: bool) {
        self.has_left_horn.store(has_horn, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::goat::DATA_HAS_LEFT_HORN,
            has_horn,
        );
    }

    #[must_use]
    pub fn has_right_horn(&self) -> bool {
        self.has_right_horn.load(Ordering::Relaxed)
    }

    fn screaming_sound(&self, regular: Sound, screaming: Sound) -> Sound {
        if self.is_screaming() {
            screaming
        } else {
            regular
        }
    }

    pub fn set_has_right_horn(&self, has_horn: bool) {
        self.has_right_horn.store(has_horn, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::goat::DATA_HAS_RIGHT_HORN,
            has_horn,
        );
    }
}

impl CustomSound for GoatEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(self.screaming_sound(Sound::EntityGoatDeath, Sound::EntityGoatScreamingDeath))
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(self.screaming_sound(Sound::EntityGoatHurt, Sound::EntityGoatScreamingHurt))
    }
}

impl AgeableMob for GoatEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }

    fn age_boundary_reached(&self) {
        let damage = if self.is_baby() {
            BABY_ATTACK_DAMAGE
        } else {
            ADULT_ATTACK_DAMAGE
        };
        self.mob_entity
            .living_entity
            .set_attribute_base(&Attributes::ATTACK_DAMAGE, damage);
    }
}

impl Animal for GoatEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_GOAT_FOOD)
    }

    fn get_breed_offspring(&self, partner: &dyn EntityBase) -> Arc<dyn EntityBase> {
        let entity = self.get_entity();
        let baby = Self::new(Entity::new(
            entity.world.load_full(),
            entity.pos.load(),
            &EntityType::GOAT,
        ));
        let mut random = self.get_random();
        let selected_parent_screaming = if random.random::<bool>() {
            self.is_screaming()
        } else {
            partner
                .cast_any()
                .downcast_ref::<Self>()
                .is_some_and(Self::is_screaming)
        };
        baby.set_screaming(
            selected_parent_screaming || random.random::<f64>() < GOAT_SCREAMING_CHANCE,
        );
        baby
    }

    /// Vanilla `Goat.playEatingSound`.
    fn play_eating_sound(&self, sound: Sound) {
        let entity = self.get_entity();
        entity.world.load().play_sound_fine(
            sound,
            SoundCategory::Neutral,
            &entity.pos.load(),
            1.0,
            self.get_random().random_range(0.8..1.2),
        );
    }
}

impl Mob for GoatEntity {
    fn finalize_spawn(
        &self,
        _world: &Arc<World>,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        let mut random = self.get_random();
        self.set_screaming(random.random::<f64>() < GOAT_SCREAMING_CHANCE);
        self.age_boundary_reached();
        if !self.is_baby() && random.random::<f32>() < UNIHORN_CHANCE {
            if random.random::<bool>() {
                self.set_has_left_horn(false);
            } else {
                self.set_has_right_horn(false);
            }
        }
        self.mob_entity.finalize_spawn_base();
        group_data
    }

    fn calculate_fall_damage(&self, damage: f32) -> f32 {
        damage - GOAT_FALL_DAMAGE_REDUCTION
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
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

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::goat::DATA_BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::goat::DATA_IS_SCREAMING_GOAT,
            self.is_screaming(),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::goat::DATA_HAS_LEFT_HORN,
            self.has_left_horn(),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::goat::DATA_HAS_RIGHT_HORN,
            self.has_right_horn(),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if item_stack.get_item() == &Item::BUCKET && !self.is_baby() {
            self.get_entity().world.load().play_sound_expect(
                player,
                self.screaming_sound(Sound::EntityGoatMilk, Sound::EntityGoatScreamingMilk),
                SoundCategory::Players,
                &player.position(),
            );
            fill_bucket_stack(item_stack, player, &Item::MILK_BUCKET);
            return true;
        }

        let eating_sound =
            self.screaming_sound(Sound::EntityGoatEat, Sound::EntityGoatScreamingEat);
        let consumed = self.animal_interact(player, item_stack, eating_sound);
        // Vanilla `Goat.mobInteract` plays the eating sound again on top of `Animal`'s, unless
        // the last food item was used up.
        if consumed && self.is_food(item_stack) {
            self.play_eating_sound(eating_sound);
        }
        consumed
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::ai::test_support::world;

    #[tokio::test]
    async fn attack_damage_follows_the_age_boundary() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let goat = GoatEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::GOAT,
        ));
        let attack_damage =
            || goat.mob_entity.living_entity.get_attribute_base(&Attributes::ATTACK_DAMAGE);
        goat.set_baby(true);
        assert_eq!(attack_damage(), BABY_ATTACK_DAMAGE);
        goat.set_baby(false);
        assert_eq!(attack_damage(), ADULT_ATTACK_DAMAGE);
        world.level.shutdown().await;
        Ok(())
    }
}
