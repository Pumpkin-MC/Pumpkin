use std::sync::{
    Arc, Weak,
    atomic::{AtomicU8, Ordering},
};

use pumpkin_data::{entity::EntityType, item::Item};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase,
    ageable::AgeableMob,
    ai::goal::{
        breed::BreedGoal, eat_grass::EatGrassGoal, escape_danger::EscapeDangerGoal,
        follow_parent::FollowParentGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};

use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;

const TEMPT_ITEMS: &[&Item] = &[&Item::WHEAT];

pub struct SheepEntity {
    pub mob_entity: MobEntity,
    color_and_sheared: AtomicU8,
    pub ageable_data: crate::entity::ageable::AgeableData,
}

impl SheepEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let sheep = Self {
            mob_entity,
            color_and_sheared: AtomicU8::new(0),
            ageable_data: crate::entity::ageable::AgeableData::default(),
        };
        let mob_arc = Arc::new(sheep);
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
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.1, TEMPT_ITEMS)));
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.1)));
            goal_selector.add_goal(5, Box::new(EatGrassGoal::default()));
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    fn get_packed_byte(&self) -> u8 {
        self.color_and_sheared.load(Ordering::Relaxed)
    }

    pub fn get_color(&self) -> u8 {
        self.get_packed_byte() & 0x0F
    }

    pub fn is_sheared(&self) -> bool {
        (self.get_packed_byte() & 0x10) != 0
    }

    fn set_packed_and_sync(&self, byte: u8) {
        self.color_and_sheared.store(byte, Ordering::Relaxed);
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::sheep::WOOL_ID,
                byte as i8,
            )],
            None,
        );
    }

    pub fn set_color(&self, color: u8) {
        let byte = (self.get_packed_byte() & 0xF0) | (color & 0x0F);
        self.set_packed_and_sync(byte);
    }

    pub fn set_sheared(&self, sheared: bool) {
        let byte = if sheared {
            self.get_packed_byte() | 0x10
        } else {
            self.get_packed_byte() & !0x10
        };
        self.set_packed_and_sync(byte);
    }
}

impl AgeableMob for SheepEntity {
    fn get_ageable_data(&self) -> &crate::entity::ageable::AgeableData {
        &self.ageable_data
    }
}

impl Animal for SheepEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        use pumpkin_data::tag::Taggable;
        item_stack
            .item
            .has_tag(&pumpkin_data::tag::Item::MINECRAFT_SHEEP_FOOD)
            || TEMPT_ITEMS.iter().any(|i| i.id == item_stack.item.id)
    }
}

impl Mob for SheepEntity {
    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("Sheared", self.is_sheared());
        nbt.put_byte("Color", self.get_color() as i8);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        let sheared = nbt
            .get_bool("Sheared")
            .or_else(|| nbt.get_byte("Sheared").map(|b| b == 1))
            .unwrap_or(false);
        let color = nbt.get_byte("Color").unwrap_or(0) as u8;
        let byte = (color & 0x0F) | if sheared { 0x10 } else { 0 };
        self.color_and_sheared.store(byte, Ordering::Relaxed);
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn on_eating_grass(&self) {
        self.set_sheared(false);
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        use super::animal::{Animal, get_dye_color_from_item, get_wool_item_for_color};
        let item = item_stack.get_item();

        if item == &Item::SHEARS && !self.is_sheared() && !self.is_baby() {
            self.set_sheared(true);
            let entity = self.get_entity();
            let world = entity.world.load();
            let pos = entity.pos.load();
            world.play_sound(
                Sound::EntitySheepShear,
                pumpkin_data::sound::SoundCategory::Players,
                &pos,
            );

            let wool_item = get_wool_item_for_color(self.get_color());
            let mut rng = rand::rng();
            let count = rng.random_range(1..=3);
            let item_entity = Arc::new(crate::entity::item::ItemEntity::new(
                Entity::new(world.clone(), pos, &EntityType::ITEM),
                ItemStack::new(count, wool_item),
            ));
            world.spawn_entity(item_entity);
            player.damage_held_item(1);
            return true;
        }

        if let Some(color) = get_dye_color_from_item(item)
            && !self.is_sheared()
            && color != self.get_color()
        {
            self.set_color(color);
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            return true;
        }

        self.animal_interact(player, item_stack, Sound::EntitySheepAmbient)
    }
}
