use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
};

use pumpkin_data::BlockState;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity, equipment::equip_on_target},
    passive::abstract_horse::{
        AbstractHorse, AbstractHorseData, BEDROCK_SADDLE_INVENTORY_SIZE, spawn_age_up_particle,
    },
    passive::animal::Animal,
    player::Player,
};

const TEMPT_ITEMS: &[&Item] = &[&Item::CACTUS];
const CAMEL_AGE_UP_SECONDS: i32 = 10;
const MAX_PASSENGERS: usize = 2;

pub struct CamelEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    horse_data: AbstractHorseData,
    pub dashing: AtomicBool,
}

impl CamelEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let camel = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            horse_data: AbstractHorseData::default(),
            dashing: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(camel);
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
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.0)));
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(0.7)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    #[must_use]
    pub fn is_dashing(&self) -> bool {
        self.dashing.load(Ordering::Relaxed)
    }

    pub fn set_dashing(&self, dashing: bool) {
        self.dashing.store(dashing, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(pumpkin_data::tracked_data::camel::DASH, dashing);
    }
}

crate::impl_abstract_horse!(CamelEntity, {
    fn get_saddle_equip_sound(&self) -> Sound {
        Sound::EntityCamelSaddle
    }

    fn can_perform_rearing(&self) -> bool {
        false
    }

    // TODO: start the dash once camels tick their dash cooldown.
    fn handle_start_jump(&self, _jump_scale: i32) {}

    fn get_eating_sound(&self) -> Option<Sound> {
        Some(Sound::EntityCamelEat)
    }

    fn bedrock_inventory_size(&self) -> i32 {
        BEDROCK_SADDLE_INVENTORY_SIZE
    }

    fn handle_eating(&self, player: &Arc<Player>, stack: &ItemStack) -> bool {
        if !self.is_food(stack) {
            return false;
        }
        let living = &self.mob_entity.living_entity;
        let could_heal = living.health.load() < living.get_max_health();
        if could_heal {
            living.heal(2.0);
        }
        let could_set_in_love =
            self.is_tamed() && self.get_age() == 0 && !self.mob_entity.is_in_love();
        if could_set_in_love {
            self.set_in_love(player);
        }
        let could_age_up = self.can_age_up();
        if could_age_up {
            spawn_age_up_particle(self.get_entity(), &mut self.get_random());
            self.age_up(CAMEL_AGE_UP_SECONDS, false);
        }
        if !could_heal && !could_set_in_love && !could_age_up {
            return false;
        }
        if let Some(sound) = self.get_eating_sound() {
            self.play_varied_sound(sound);
        }
        let entity = self.get_entity();
        entity
            .world
            .load()
            .emit_game_event(GameEvent::Eat.name(), entity.pos.load());
        true
    }
});

impl Animal for CamelEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_CAMEL_FOOD)
    }

    fn can_mate(&self, partner: &dyn EntityBase) -> bool {
        self.can_mate_with(partner, &[&EntityType::CAMEL])
    }
}

impl Mob for CamelEntity {
    crate::abstract_horse_mob_hooks!(always_tamed);

    fn can_use_slot(&self, slot: &EquipmentSlot) -> bool {
        crate::entity::passive::abstract_horse::can_use_slot(self, slot)
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, state: &BlockState) {
        let sound = if state
            .id
            .to_block()
            .has_tag(&tag::Block::MINECRAFT_CAMEL_SAND_STEP_SOUND_BLOCKS)
        {
            Sound::EntityCamelStepSand
        } else {
            Sound::EntityCamelStep
        };
        self.play_entity_sound(sound, 1.0, 1.0);
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_horse_nbt(nbt);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.read_horse_nbt(nbt);
        self.mob_entity
            .read_legacy_saddle(nbt.get_bool("Saddle") == Some(true));
    }

    fn mob_init_data_tracker(&self) {
        self.init_horse_data_tracker();
        self.get_entity()
            .set_synced_data(pumpkin_data::tracked_data::camel::DASH, self.is_dashing());
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if let Some(result) = self.handle_important_interactions(player, item_stack) {
            return result;
        }
        if player.get_entity().is_sneaking() && !self.is_baby() {
            self.open_custom_inventory_screen(player);
            return true;
        }
        if equip_on_target(self, player, item_stack) {
            return true;
        }
        if self.is_food(item_stack) {
            return self.fed_food(player, item_stack);
        }
        let entity = self.get_entity();
        let passengers = entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len();
        if passengers < MAX_PASSENGERS && !self.is_baby() {
            self.do_player_ride(player);
            return true;
        }
        self.is_baby()
            && item_stack.item == &Item::GOLDEN_DANDELION
            && self.animal_interact(player, item_stack, Sound::EntityCamelAmbient)
    }
}
