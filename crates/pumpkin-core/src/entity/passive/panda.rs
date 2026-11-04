use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, AtomicU8, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::BlockState;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use pumpkin_data::entity_variant::PandaGene;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    mob::{
        Mob, MobEntity,
        spawn::{EntitySpawnReason, SpawnGroupData},
    },
    passive::animal::Animal,
    player::Player,
};
use crate::world::World;

const TEMPT_ITEMS: &[&Item] = &[&Item::BAMBOO];

pub const FLAG_SNEEZE: u8 = 2;
pub const FLAG_ROLL: u8 = 4;
pub const FLAG_SIT: u8 = 8;
pub const FLAG_ON_BACK: u8 = 16;

const GENE_RANDOM_BOUND: i32 = 16;
const MUTATION_CHANCE: i32 = 32;
const WEAK_MAX_HEALTH: f32 = 10.0;
const LAZY_MOVEMENT_SPEED: f32 = 0.07;

/// A recessive main gene only shows when the hidden gene is the same one.
fn get_variant_from_genes(main_gene: PandaGene, hidden_gene: PandaGene) -> PandaGene {
    if main_gene.recessive() && main_gene != hidden_gene {
        PandaGene::Normal
    } else {
        main_gene
    }
}

/// `next_int(bound)` stands for vanilla's `RandomSource.nextInt`.
fn get_random_gene(mut next_int: impl FnMut(i32) -> i32) -> PandaGene {
    match next_int(GENE_RANDOM_BOUND) {
        0 => PandaGene::Lazy,
        1 => PandaGene::Worried,
        2 => PandaGene::Playful,
        4 => PandaGene::Aggressive,
        3 | 5..=8 => PandaGene::Weak,
        9 | 10 => PandaGene::Brown,
        _ => PandaGene::Normal,
    }
}

fn next_boolean(next_int: &mut impl FnMut(i32) -> i32) -> bool {
    next_int(2) != 0
}

/// Vanilla `getOneOfGenesRandomly` for a panda's (main, hidden) genes.
fn get_one_of_genes_randomly(
    genes: (PandaGene, PandaGene),
    next_int: &mut impl FnMut(i32) -> i32,
) -> PandaGene {
    if next_boolean(next_int) {
        genes.0
    } else {
        genes.1
    }
}

/// Vanilla `setGeneFromParents`: the (main, hidden) genes of a foal of two pandas.
fn genes_from_parents(
    parent1: (PandaGene, PandaGene),
    parent2: (PandaGene, PandaGene),
    mut next_int: impl FnMut(i32) -> i32,
) -> (PandaGene, PandaGene) {
    let (first, second) = if next_boolean(&mut next_int) {
        (parent1, parent2)
    } else {
        (parent2, parent1)
    };
    let mut main_gene = get_one_of_genes_randomly(first, &mut next_int);
    let mut hidden_gene = get_one_of_genes_randomly(second, &mut next_int);
    if next_int(MUTATION_CHANCE) == 0 {
        main_gene = get_random_gene(&mut next_int);
    }
    if next_int(MUTATION_CHANCE) == 0 {
        hidden_gene = get_random_gene(&mut next_int);
    }
    (main_gene, hidden_gene)
}

pub struct PandaEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub main_gene: AtomicCell<PandaGene>,
    pub hidden_gene: AtomicCell<PandaGene>,
    pub flags: AtomicU8,
    pub eat_counter: AtomicI32,
    pub sneeze_counter: AtomicI32,
    pub unhappy_counter: AtomicI32,
}

impl PandaEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        mob_entity.set_can_pick_up_loot(true);
        let panda = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            main_gene: AtomicCell::new(PandaGene::Normal),
            hidden_gene: AtomicCell::new(PandaGene::Normal),
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
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(2.0));
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.2, TEMPT_ITEMS, false)));
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
        self.main_gene.load()
    }

    pub fn set_main_gene(&self, gene: PandaGene) {
        self.main_gene.store(gene);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::panda::MAIN_GENE_ID,
            gene.id() as i8,
        );
    }

    #[must_use]
    pub fn get_hidden_gene(&self) -> PandaGene {
        self.hidden_gene.load()
    }

    pub fn set_hidden_gene(&self, gene: PandaGene) {
        self.hidden_gene.store(gene);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::panda::HIDDEN_GENE_ID,
            gene.id() as i8,
        );
    }

    #[must_use]
    pub fn get_variant(&self) -> PandaGene {
        get_variant_from_genes(self.get_main_gene(), self.get_hidden_gene())
    }

    #[must_use]
    pub fn is_lazy(&self) -> bool {
        self.get_variant() == PandaGene::Lazy
    }

    #[must_use]
    pub fn is_worried(&self) -> bool {
        self.get_variant() == PandaGene::Worried
    }

    #[must_use]
    pub fn is_playful(&self) -> bool {
        self.get_variant() == PandaGene::Playful
    }

    #[must_use]
    pub fn is_brown(&self) -> bool {
        self.get_variant() == PandaGene::Brown
    }

    #[must_use]
    pub fn is_weak(&self) -> bool {
        self.get_variant() == PandaGene::Weak
    }

    #[must_use]
    pub fn is_aggressive(&self) -> bool {
        self.get_variant() == PandaGene::Aggressive
    }

    pub fn set_attributes(&self) {
        let living = &self.mob_entity.living_entity;
        if self.is_weak() {
            living.set_max_health(WEAK_MAX_HEALTH);
        }
        if self.is_lazy() {
            living.set_attribute_base(&Attributes::MOVEMENT_SPEED, f64::from(LAZY_MOVEMENT_SPEED));
        }
    }

    pub fn set_gene_from_parents(&self, parent1: &Self, parent2: &Self) {
        let mut rng = self.get_random();
        let (main_gene, hidden_gene) = genes_from_parents(
            (parent1.get_main_gene(), parent1.get_hidden_gene()),
            (parent2.get_main_gene(), parent2.get_hidden_gene()),
            |bound| rng.random_range(0..bound),
        );
        self.set_main_gene(main_gene);
        self.set_hidden_gene(hidden_gene);
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
        entity.set_synced_data(
            pumpkin_data::tracked_data::panda::DATA_ID_FLAGS,
            new_flags as i8,
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

    fn default_baby_spawn_chance(&self) -> f32 {
        crate::entity::ageable::PANDA_BABY_SPAWN_CHANCE
    }
}

impl Animal for PandaEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_PANDA_FOOD)
    }

    fn get_breed_offspring(&self, partner: &dyn EntityBase) -> Arc<dyn EntityBase> {
        let entity = self.get_entity();
        let baby = Self::new(Entity::new(
            entity.world.load_full(),
            entity.pos.load(),
            &EntityType::PANDA,
        ));
        if let Some(partner) = partner.cast_any().downcast_ref::<Self>() {
            baby.set_gene_from_parents(self, partner);
        }
        baby.set_attributes();
        baby
    }
}

impl Mob for PandaEntity {
    fn can_dispenser_equip_into_slot(
        &self,
        slot: &pumpkin_data::data_component_impl::EquipmentSlot,
    ) -> bool {
        *slot == pumpkin_data::data_component_impl::EquipmentSlot::MAIN_HAND
            && self.mob_entity.can_pick_up_loot()
    }

    // Needs pandas eating from their hand first.
    fn wants_to_pick_up(&self, _stack: &ItemStack) -> bool {
        false
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityPandaStep, 0.15, 1.0);
    }

    fn can_be_leashed(&self) -> bool {
        false
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(if self.is_aggressive() {
            Sound::EntityPandaAggressiveAmbient
        } else if self.is_worried() {
            Sound::EntityPandaWorriedAmbient
        } else {
            Sound::EntityPandaAmbient
        })
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_ageable_nbt(nbt);
        nbt.put_string("MainGene", self.get_main_gene().name().to_string());
        nbt.put_string("HiddenGene", self.get_hidden_gene().name().to_string());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.read_ageable_nbt(nbt);
        self.set_main_gene(
            nbt.get_string("MainGene")
                .and_then(PandaGene::from_name)
                .unwrap_or(PandaGene::Normal),
        );
        self.set_hidden_gene(
            nbt.get_string("HiddenGene")
                .and_then(PandaGene::from_name)
                .unwrap_or(PandaGene::Normal),
        );
    }

    fn finalize_spawn(
        &self,
        _world: &Arc<World>,
        _reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        let mut rng = self.get_random();
        let mut next_int = |bound| rng.random_range(0..bound);
        self.set_main_gene(get_random_gene(&mut next_int));
        self.set_hidden_gene(get_random_gene(&mut next_int));
        self.set_attributes();
        self.finalize_spawn_super(group_data)
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::panda::DATA_BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::panda::MAIN_GENE_ID,
            self.main_gene.load().id() as i8,
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::panda::HIDDEN_GENE_ID,
            self.hidden_gene.load().id() as i8,
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::panda::DATA_ID_FLAGS,
            self.flags.load(Ordering::Relaxed) as i8,
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::panda::UNHAPPY_COUNTER,
            VarInt(self.unhappy_counter.load(Ordering::Relaxed)),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::panda::SNEEZE_COUNTER,
            VarInt(self.sneeze_counter.load(Ordering::Relaxed)),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::panda::EAT_COUNTER,
            VarInt(self.eat_counter.load(Ordering::Relaxed)),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        self.animal_interact(player, item_stack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;
    use pumpkin_util::math::vector3::Vector3;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn panda(world: &Arc<World>) -> Arc<PandaEntity> {
        PandaEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::PANDA,
        ))
    }

    fn set_genes(panda: &PandaEntity, main_gene: PandaGene, hidden_gene: PandaGene) {
        panda.set_main_gene(main_gene);
        panda.set_hidden_gene(hidden_gene);
    }

    /// Runs `genes_from_parents` against scripted `(bound, result)` random calls.
    fn inherit(
        parent1: (PandaGene, PandaGene),
        parent2: (PandaGene, PandaGene),
        calls: &[(i32, i32)],
    ) -> (PandaGene, PandaGene) {
        let mut script = calls.iter();
        let genes = genes_from_parents(parent1, parent2, |bound| {
            let (expected_bound, result) = script.next().copied().unwrap_or((-1, -1));
            assert_eq!(bound, expected_bound);
            result
        });
        assert!(script.next().is_none(), "unused scripted random calls");
        genes
    }

    #[test]
    fn foal_genes_come_from_the_parents_and_mutate_independently() {
        let parent1 = (PandaGene::Lazy, PandaGene::Worried);
        let parent2 = (PandaGene::Playful, PandaGene::Aggressive);
        // First boolean true: main from parent 1, hidden from parent 2.
        let genes = inherit(
            parent1,
            parent2,
            &[(2, 1), (2, 1), (2, 0), (32, 5), (32, 7)],
        );
        assert_eq!(genes, (PandaGene::Lazy, PandaGene::Aggressive));
        // First boolean false: main from parent 2, hidden from parent 1.
        let genes = inherit(
            parent1,
            parent2,
            &[(2, 0), (2, 0), (2, 1), (32, 5), (32, 7)],
        );
        assert_eq!(genes, (PandaGene::Aggressive, PandaGene::Lazy));
        // A 1/32 mutation hits one gene and not the other.
        let genes = inherit(
            parent1,
            parent2,
            &[(2, 1), (2, 1), (2, 0), (32, 0), (16, 3), (32, 4)],
        );
        assert_eq!(genes, (PandaGene::Weak, PandaGene::Aggressive));
    }

    #[tokio::test]
    async fn weak_variant_halves_health_and_lazy_variant_slows_down() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let default_speed = panda(&world)
            .mob_entity
            .living_entity
            .get_attribute_value(&Attributes::MOVEMENT_SPEED);

        let weak = panda(&world);
        set_genes(&weak, PandaGene::Weak, PandaGene::Weak);
        weak.set_attributes();
        let living = &weak.mob_entity.living_entity;
        assert!((living.get_max_health() - 10.0).abs() < f32::EPSILON);
        assert!((living.health.load() - 10.0).abs() < f32::EPSILON);
        assert!(
            (living.get_attribute_value(&Attributes::MOVEMENT_SPEED) - default_speed).abs() < 1e-9
        );

        let lazy = panda(&world);
        set_genes(&lazy, PandaGene::Lazy, PandaGene::Normal);
        lazy.set_attributes();
        let living = &lazy.mob_entity.living_entity;
        assert!((living.get_attribute_value(&Attributes::MOVEMENT_SPEED) - 0.07).abs() < 1e-6);
        assert!((living.get_max_health() - 20.0).abs() < f32::EPSILON);

        // A hidden recessive gene does not make the panda weak.
        let carrier = panda(&world);
        set_genes(&carrier, PandaGene::Weak, PandaGene::Normal);
        carrier.set_attributes();
        assert!((carrier.mob_entity.living_entity.get_max_health() - 20.0).abs() < f32::EPSILON);
        Ok(())
    }

    #[tokio::test]
    async fn genes_round_trip_through_nbt_and_default_to_normal() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let source = panda(&world);
        set_genes(&source, PandaGene::Brown, PandaGene::Weak);
        let mut nbt = NbtCompound::new();
        source.write_custom_nbt(&mut nbt);
        assert_eq!(nbt.get_string("MainGene"), Some("brown"));
        assert_eq!(nbt.get_string("HiddenGene"), Some("weak"));

        let loaded = panda(&world);
        set_genes(&loaded, PandaGene::Lazy, PandaGene::Lazy);
        loaded.read_custom_nbt(&nbt);
        assert_eq!(loaded.get_main_gene(), PandaGene::Brown);
        assert_eq!(loaded.get_hidden_gene(), PandaGene::Weak);

        set_genes(&loaded, PandaGene::Weak, PandaGene::Brown);
        loaded.read_custom_nbt(&NbtCompound::new());
        assert_eq!(loaded.get_main_gene(), PandaGene::Normal);
        assert_eq!(loaded.get_hidden_gene(), PandaGene::Normal);
        Ok(())
    }

    #[tokio::test]
    async fn offspring_gets_attributes_for_its_genes() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let parent = panda(&world);
        set_genes(&parent, PandaGene::Weak, PandaGene::Weak);
        let mate = panda(&world);
        set_genes(&mate, PandaGene::Weak, PandaGene::Weak);

        let baby = parent.get_breed_offspring(mate.as_ref());
        let foal = baby
            .cast_any()
            .downcast_ref::<PandaEntity>()
            .ok_or("offspring is not a panda")?;
        // A mutation can make the foal normal, so compare against its own variant.
        let expected = if foal.is_weak() { 10.0 } else { 20.0 };
        assert!((foal.mob_entity.living_entity.get_max_health() - expected).abs() < f32::EPSILON);
        Ok(())
    }
}
