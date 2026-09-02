use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, AtomicU8, Ordering},
};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use rand::RngExt;

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

const TEMPT_ITEMS: &[&Item] = &[&Item::BAMBOO];

pub const FLAG_SNEEZE: u8 = 2;
pub const FLAG_ROLL: u8 = 4;
pub const FLAG_SIT: u8 = 8;
pub const FLAG_ON_BACK: u8 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum PandaGene {
    #[default]
    Normal = 0,
    Lazy = 1,
    Worried = 2,
    Playful = 3,
    Brown = 4,
    Weak = 5,
    Aggressive = 6,
}

impl PandaGene {
    #[must_use]
    pub const fn from_id(id: u8) -> Self {
        match id {
            1 => Self::Lazy,
            2 => Self::Worried,
            3 => Self::Playful,
            4 => Self::Brown,
            5 => Self::Weak,
            6 => Self::Aggressive,
            _ => Self::Normal,
        }
    }

    #[must_use]
    pub const fn id(self) -> u8 {
        self as u8
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Lazy => "lazy",
            Self::Worried => "worried",
            Self::Playful => "playful",
            Self::Brown => "brown",
            Self::Weak => "weak",
            Self::Aggressive => "aggressive",
        }
    }

    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "lazy" => Self::Lazy,
            "worried" => Self::Worried,
            "playful" => Self::Playful,
            "brown" => Self::Brown,
            "weak" => Self::Weak,
            "aggressive" => Self::Aggressive,
            _ => Self::Normal,
        }
    }

    #[must_use]
    pub fn random_gene() -> Self {
        let mut rng = rand::rng();
        if rng.random_range(0..16) == 0 {
            Self::from_id(rng.random_range(4..=6))
        } else if rng.random_range(0..8) == 0 {
            Self::from_id(rng.random_range(1..=3))
        } else {
            Self::Normal
        }
    }
}

pub struct PandaEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub main_gene: AtomicU8,
    pub hidden_gene: AtomicU8,
    pub flags: AtomicU8,
    pub eat_counter: AtomicI32,
    pub sneeze_counter: AtomicI32,
    pub unhappy_counter: AtomicI32,
}

impl PandaEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let gene = PandaGene::random_gene();
        let hidden = PandaGene::random_gene();
        let panda = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            main_gene: AtomicU8::new(gene.id()),
            hidden_gene: AtomicU8::new(hidden.id()),
            flags: AtomicU8::new(0),
            eat_counter: AtomicI32::new(0),
            sneeze_counter: AtomicI32::new(0),
            unhappy_counter: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(panda);
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
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.2, TEMPT_ITEMS)));
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
    pub fn get_main_gene(&self) -> PandaGene {
        PandaGene::from_id(self.main_gene.load(Ordering::Relaxed))
    }

    pub fn set_main_gene(&self, gene: PandaGene) {
        self.main_gene.store(gene.id(), Ordering::Relaxed);
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::panda::MAIN_GENE_ID,
                gene.id() as i8,
            )],
            None,
        );
    }

    #[must_use]
    pub fn get_hidden_gene(&self) -> PandaGene {
        PandaGene::from_id(self.hidden_gene.load(Ordering::Relaxed))
    }

    pub fn set_hidden_gene(&self, gene: PandaGene) {
        self.hidden_gene.store(gene.id(), Ordering::Relaxed);
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::panda::HIDDEN_GENE_ID,
                gene.id() as i8,
            )],
            None,
        );
    }

    #[must_use]
    pub fn has_flag(&self, flag: u8) -> bool {
        (self.flags.load(Ordering::Relaxed) & flag) != 0
    }

    pub fn set_flag(&self, flag: u8, val: bool) {
        let current = self.flags.load(Ordering::Relaxed);
        let new_flags = if val { current | flag } else { current & !flag };
        self.flags.store(new_flags, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::panda::DATA_ID_FLAGS,
                new_flags as i8,
            )],
            None,
        );
    }

    #[must_use]
    pub fn is_sneezing(&self) -> bool {
        self.has_flag(FLAG_SNEEZE)
    }

    pub fn set_sneezing(&self, val: bool) {
        self.set_flag(FLAG_SNEEZE, val);
    }

    #[must_use]
    pub fn is_sitting(&self) -> bool {
        self.has_flag(FLAG_SIT)
    }

    pub fn set_sitting(&self, val: bool) {
        self.set_flag(FLAG_SIT, val);
    }

    #[must_use]
    pub fn is_on_back(&self) -> bool {
        self.has_flag(FLAG_ON_BACK)
    }

    pub fn set_on_back(&self, val: bool) {
        self.set_flag(FLAG_ON_BACK, val);
    }

    #[must_use]
    pub fn is_rolling(&self) -> bool {
        self.has_flag(FLAG_ROLL)
    }

    pub fn set_rolling(&self, val: bool) {
        self.set_flag(FLAG_ROLL, val);
    }
}

impl AgeableMob for PandaEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Animal for PandaEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_PANDA_FOOD)
            || item_stack.item == &Item::BAMBOO
            || item_stack.item == &Item::CAKE
    }
}

impl Mob for PandaEntity {
    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_ageable_nbt(nbt);
        nbt.put_string("MainGene", self.get_main_gene().as_str().to_string());
        nbt.put_string("HiddenGene", self.get_hidden_gene().as_str().to_string());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.read_ageable_nbt(nbt);
        if let Some(main) = nbt.get_string("MainGene") {
            self.set_main_gene(PandaGene::from_name(main));
        }
        if let Some(hidden) = nbt.get_string("HiddenGene") {
            self.set_hidden_gene(PandaGene::from_name(hidden));
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
                    pumpkin_data::tracked_data::panda::DATA_BABY_ID,
                    true,
                )],
                None,
            );
        }
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::panda::MAIN_GENE_ID,
                self.main_gene.load(Ordering::Relaxed) as i8,
            )],
            None,
        );
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::panda::HIDDEN_GENE_ID,
                self.hidden_gene.load(Ordering::Relaxed) as i8,
            )],
            None,
        );
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::panda::DATA_ID_FLAGS,
                self.flags.load(Ordering::Relaxed) as i8,
            )],
            None,
        );
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::panda::UNHAPPY_COUNTER,
                VarInt(self.unhappy_counter.load(Ordering::Relaxed)),
            )],
            None,
        );
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::panda::SNEEZE_COUNTER,
                VarInt(self.sneeze_counter.load(Ordering::Relaxed)),
            )],
            None,
        );
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::panda::EAT_COUNTER,
                VarInt(self.eat_counter.load(Ordering::Relaxed)),
            )],
            None,
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        self.animal_interact(player, item_stack, Sound::EntityPandaAmbient)
    }
}
