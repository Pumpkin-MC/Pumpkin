use std::sync::{
    Arc, Weak,
    atomic::{AtomicU8, Ordering},
};

use pumpkin_data::dye_color::DyeColor;
use pumpkin_data::{entity::EntityType, item::Item};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::Hand;
use pumpkin_util::math::vector3::Vector3;
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
    item::ItemEntity,
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
    shearable::{Shearable, shear_by_player, shearing_loot},
};

use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};

const TEMPT_ITEMS: &[&Item] = &[&Item::WHEAT];
/// The sheared flag in vanilla's `DATA_WOOL_ID` byte.
const SHEARED_BIT: u8 = 0x10;

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
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.1, TEMPT_ITEMS, false)));
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
        (self.get_packed_byte() & SHEARED_BIT) != 0
    }

    fn set_packed_and_sync(&self, byte: u8) {
        self.color_and_sheared.store(byte, Ordering::Relaxed);
        self.mob_entity
            .living_entity
            .entity
            .set_synced_data(pumpkin_data::tracked_data::sheep::WOOL_ID, byte as i8);
    }

    pub fn set_color(&self, color: u8) {
        let byte = (self.get_packed_byte() & 0xF0) | (color & 0x0F);
        self.set_packed_and_sync(byte);
    }

    pub fn set_sheared(&self, sheared: bool) {
        let byte = if sheared {
            self.get_packed_byte() | SHEARED_BIT
        } else {
            self.get_packed_byte() & !SHEARED_BIT
        };
        self.set_packed_and_sync(byte);
    }
}

impl Shearable for SheepEntity {
    fn shear(&self, sound_category: SoundCategory, tool: &ItemStack) -> bool {
        if self
            .color_and_sheared
            .fetch_or(SHEARED_BIT, Ordering::Relaxed)
            & SHEARED_BIT
            != 0
        {
            return false;
        }

        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        world.play_sound(Sound::EntitySheepShear, sound_category, &pos);

        let color = DyeColor::by_id(self.get_color()).unwrap_or(DyeColor::White);
        let loot_key = format!("minecraft:shearing/sheep/{}", color.name()); // is there a better way to do this
        let drop_pos = Vector3::new(pos.x, pos.y + 1.0, pos.z);
        let mut rng = rand::rng();
        for drop in shearing_loot(entity, &loot_key, tool) {
            for _ in 0..drop.item_count {
                let item_entity = ItemEntity::new(
                    Entity::new(world.clone(), drop_pos, &EntityType::ITEM),
                    drop.copy_with_count(1),
                );
                let item_base = item_entity.get_entity();
                item_base.velocity.store(
                    item_base.velocity.load()
                        + Vector3::new(
                            // magic numbers from vanilla
                            f64::from((rng.random::<f32>() - rng.random::<f32>()) * 0.1),
                            f64::from(rng.random::<f32>() * 0.05),
                            f64::from((rng.random::<f32>() - rng.random::<f32>()) * 0.1),
                        ),
                );
                world.spawn_entity(Arc::new(item_entity));
            }
        }

        // Bit is set, send change to clients
        self.set_sheared(true);
        true
    }

    fn ready_for_shearing(&self) -> bool {
        !self.is_sheared() && !self.is_baby()
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

    fn as_shearable(&self) -> Option<&dyn Shearable> {
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
        let byte = (color & 0x0F) | if sheared { SHEARED_BIT } else { 0 };
        self.color_and_sheared.store(byte, Ordering::Relaxed);
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn on_eating_grass(&self) {
        self.set_sheared(false);
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        use super::animal::{Animal, get_dye_color_from_item};
        let item = item_stack.get_item();

        if item == &Item::SHEARS && self.ready_for_shearing() {
            if !shear_by_player(self, player, item_stack) {
                return false;
            }
            // Vanilla returns `SUCCESS_SERVER` here, so the client waits for the server to swing its arm.
            player.swing_hand(Hand::Right, true);
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
