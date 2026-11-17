use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::block_properties::WheatLikeProperties;
use pumpkin_data::chunk::Biome;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{DataComponentImpl, RabbitVariantImpl};
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::entity_variant::RabbitVariant;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable, WorldgenBiome};
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, translation};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

use crate::block::blocks::plant::crop::MAX_AGE;
use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::control::move_control::{MoveControl, Operation};
use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::ai::goal::move_to_target_pos::{MoveToTargetPos, MoveToTargetPosGoal};
use crate::entity::ai::goal::revenge::RevengeGoal;
use crate::entity::ai::goal::{Controls, Goal, GoalReach, ParentHandle};
use crate::entity::attributes::{Modifier, ModifierOperation};
use crate::entity::mob::spawn::{EntitySpawnReason, SpawnGroupData};
use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::behavior::fleeing::{self, Flee},
    ai::goal::{
        breed::BreedGoal, climb_on_top_of_powder_snow::ClimbOnTopOfPowderSnowGoal,
        escape_danger::EscapeDangerGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    ai::target_match::TargetMatch,
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};
use crate::world::World;
use crate::world::game_event::GameEventContext;

const STROLL_SPEED_MOD: f64 = 0.6;
const BREED_SPEED_MOD: f64 = 0.8;
const FOLLOW_SPEED_MOD: f64 = 1.0;
const FLEE_SPEED_MOD: f64 = 2.2;
const ATTACK_SPEED_MOD: f64 = 1.4;
const BABY_JUMP_HEIGHT: f64 = 0.5;
const ADULT_JUMP_HEIGHT: f64 = 1.5;
const JUMP_DELAY_TICKS: i32 = 10;
const PANIC_JUMP_DELAY_TICKS: i32 = 3;
const JUMP_DURATION_IN_TICKS: i32 = 15;
const EVIL_ATTACK_POWER_MODIFIER: &str = "minecraft:evil";
const EVIL_ATTACK_POWER_INCREMENT: f64 = 5.0;
const EVIL_ARMOR_VALUE: f64 = 8.0;
const MORE_CARROTS_DELAY: i32 = 40;

const RABBIT_FLEES: &[Flee] = &[
    Flee::new(
        4,
        TargetMatch::Type(&EntityType::PLAYER),
        8.0,
        FLEE_SPEED_MOD,
        FLEE_SPEED_MOD,
    )
    .when(is_not_evil),
    Flee::new(
        4,
        TargetMatch::Type(&EntityType::WOLF),
        10.0,
        FLEE_SPEED_MOD,
        FLEE_SPEED_MOD,
    )
    .when(is_not_evil),
    Flee::new(4, TargetMatch::Monster, 4.0, FLEE_SPEED_MOD, FLEE_SPEED_MOD).when(is_not_evil),
];

const KILLER_BUNNY_HUNTS: &[Hunt] = &[
    Hunt::new(2, TargetMatch::Type(&EntityType::PLAYER), true),
    Hunt::new(2, TargetMatch::Type(&EntityType::WOLF), true),
];

fn as_rabbit(mob: &dyn Mob) -> Option<&RabbitEntity> {
    mob.cast_any().downcast_ref::<RabbitEntity>()
}

fn is_not_evil(mob: &dyn Mob) -> bool {
    as_rabbit(mob).is_some_and(|rabbit| rabbit.get_variant() != RabbitVariant::Evil)
}

fn random_rabbit_variant(biome: &Biome, random_val: i32) -> RabbitVariant {
    if biome.has_tag(&WorldgenBiome::MINECRAFT_SPAWNS_WHITE_RABBITS) {
        if random_val < 80 {
            RabbitVariant::White
        } else {
            RabbitVariant::WhiteSplotched
        }
    } else if biome.has_tag(&WorldgenBiome::MINECRAFT_SPAWNS_GOLD_RABBITS) {
        RabbitVariant::Gold
    } else if random_val < 50 {
        RabbitVariant::Brown
    } else if random_val < 90 {
        RabbitVariant::Salt
    } else {
        RabbitVariant::Black
    }
}

pub struct RabbitEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    variant: AtomicI32,
    more_carrot_ticks: AtomicI32,
    jump_ticks: AtomicI32,
    jump_duration: AtomicI32,
    was_on_ground: AtomicBool,
    jump_delay_ticks: AtomicI32,
}

impl RabbitEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let rabbit = Self {
            mob_entity: MobEntity::new(entity),
            ageable_data: AgeableData::default(),
            variant: AtomicI32::new(RabbitVariant::DEFAULT.id()),
            more_carrot_ticks: AtomicI32::new(0),
            jump_ticks: AtomicI32::new(0),
            jump_duration: AtomicI32::new(0),
            was_on_ground: AtomicBool::new(false),
            jump_delay_ticks: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(rabbit);
        *mob_arc
            .mob_entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(RabbitMoveControl::new(Arc::downgrade(&mob_arc)));
        mob_arc.set_speed_modifier(0.0);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(ClimbOnTopOfPowderSnowGoal));
            goal_selector.add_goal(1, Box::new(RabbitPanicGoal::new(FLEE_SPEED_MOD)));
            goal_selector.add_goal(2, BreedGoal::new(BREED_SPEED_MOD));
            goal_selector.add_goal(
                3,
                Box::new(TemptGoal::with_tag(
                    FOLLOW_SPEED_MOD,
                    &tag::Item::MINECRAFT_RABBIT_FOOD,
                    false,
                )),
            );
            fleeing::apply(goal_selector, RABBIT_FLEES);
            goal_selector.add_goal(5, Box::new(RaidGardenGoal::new()));
            goal_selector.add_goal(
                6,
                Box::new(WanderAroundGoal::water_avoiding(STROLL_SPEED_MOD)),
            );
            goal_selector.add_goal(
                11,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 10.0),
            );
        };

        mob_arc
    }

    #[must_use]
    pub fn get_variant(&self) -> RabbitVariant {
        RabbitVariant::by_id(self.variant.load(Ordering::Relaxed))
    }

    pub fn set_variant(&self, variant: RabbitVariant) {
        let living = &self.mob_entity.living_entity;
        if variant == RabbitVariant::Evil {
            living.set_attribute_base(&Attributes::ARMOR, EVIL_ARMOR_VALUE);
            self.mob_entity
                .goals_selector
                .add_goal(4, Box::new(MeleeAttackGoal::new(ATTACK_SPEED_MOD, true)));
            let target_selector = &self.mob_entity.target_selector;
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true).alerting_others()));
            hunting::apply(target_selector, &self.mob_entity, KILLER_BUNNY_HUNTS);
            living.update_attribute(&Attributes::ATTACK_DAMAGE, |inst| {
                inst.add_or_replace_modifier(Modifier {
                    id: EVIL_ATTACK_POWER_MODIFIER.to_string(),
                    amount: EVIL_ATTACK_POWER_INCREMENT,
                    operation: ModifierOperation::Add,
                });
            });
            let entity = self.get_entity();
            if entity.custom_name.load().is_none() {
                entity.set_custom_name(TextComponent::translate(
                    translation::java::ENTITY_MINECRAFT_KILLER_BUNNY,
                    [],
                ));
            }
        } else {
            living.update_attribute(&Attributes::ATTACK_DAMAGE, |inst| {
                inst.remove_modifier(EVIL_ATTACK_POWER_MODIFIER);
            });
        }
        crate::entity::attributes::send_attribute_updates_for_living(
            living,
            vec![Attributes::ARMOR],
        );

        self.variant.store(variant.id(), Ordering::Relaxed);
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::rabbit::DATA_TYPE_ID,
            VarInt(variant.id()),
        );
    }

    fn get_random_rabbit_variant(&self, world: &World, pos: &BlockPos) -> RabbitVariant {
        let random_val = self.get_random().random_range(0..100);
        random_rabbit_variant(world.get_biome(pos), random_val)
    }

    fn wants_more_food(&self) -> bool {
        self.more_carrot_ticks.load(Ordering::Relaxed) <= 0
    }

    fn move_control_state(&self) -> (bool, Vector3<f64>, f64) {
        let steered = self.mob_entity.steered();
        let move_control = steered.move_control();
        (
            move_control.has_wanted(),
            move_control.get_wanted_position(),
            move_control.get_speed_modifier(),
        )
    }

    fn set_speed_modifier(&self, speed: f64) {
        let steered = self.mob_entity.steered();
        steered.navigator().set_speed(speed);
        let mut move_control = steered.move_control();
        let wanted = move_control.get_wanted_position();
        move_control.set_wanted_position(wanted.x, wanted.y, wanted.z, speed);
    }

    fn set_jumping(&self, jump: bool) {
        self.mob_entity
            .living_entity
            .jumping
            .store(jump, Ordering::SeqCst);
        if jump {
            let mut rng = self.get_random();
            let pitch = (rng.random::<f32>() - rng.random::<f32>()).mul_add(0.2, 1.0) * 0.8;
            self.play_entity_sound(Sound::EntityRabbitJump, 1.0, pitch);
        }
    }

    fn start_jumping(&self) {
        self.set_jumping(true);
        self.jump_duration
            .store(JUMP_DURATION_IN_TICKS, Ordering::Relaxed);
        self.jump_ticks.store(0, Ordering::Relaxed);
    }

    fn face_point(&self, face_x: f64, face_z: f64) {
        let entity = self.get_entity();
        let pos = entity.pos.load();
        entity
            .yaw
            .store((face_z - pos.z).atan2(face_x - pos.x).to_degrees() as f32 - 90.0);
    }

    fn next_path_entity_pos(&self) -> Option<Vector3<f64>> {
        let width = self.get_entity().entity_dimension.load().width;
        let steered = self.mob_entity.steered();
        let navigator = steered.navigator();
        navigator
            .get_path()
            .filter(|path| !path.is_done())
            .and_then(|path| path.get_next_entity_pos(width))
    }

    fn set_landing_delay(&self) {
        let (_, _, speed) = self.move_control_state();
        let delay = if speed < FLEE_SPEED_MOD {
            JUMP_DELAY_TICKS
        } else {
            PANIC_JUMP_DELAY_TICKS
        };
        self.jump_delay_ticks.store(delay, Ordering::Relaxed);
    }

    fn jump_at_target(&self) {
        let Some(target) = self.mob_entity.get_target() else {
            return;
        };
        let target_pos = target.get_entity().pos.load();
        if self
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&target_pos)
            >= 16.0
        {
            return;
        }
        self.face_point(target_pos.x, target_pos.z);
        let steered = self.mob_entity.steered();
        let mut move_control = steered.move_control();
        let speed = move_control.get_speed_modifier();
        move_control.set_wanted_position(target_pos.x, target_pos.y, target_pos.z, speed);
        drop(move_control);
        self.start_jumping();
        self.was_on_ground.store(true, Ordering::Relaxed);
    }
}

impl AgeableMob for RabbitEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Animal for RabbitEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_RABBIT_FOOD)
    }

    fn get_breed_offspring(&self, partner: &dyn EntityBase) -> Arc<dyn EntityBase> {
        let entity = self.get_entity();
        let world = entity.world.load_full();
        let baby = Self::new(Entity::new(
            world.clone(),
            entity.pos.load(),
            &EntityType::RABBIT,
        ));
        let mut variant = self.get_random_rabbit_variant(&world, &entity.block_pos.load());
        let mut rng = self.get_random();
        if rng.random_range(0..20) != 0 {
            variant = match partner.cast_any().downcast_ref::<Self>() {
                Some(partner) if rng.random::<bool>() => partner.get_variant(),
                _ => self.get_variant(),
            };
        }
        baby.set_variant(variant);
        baby
    }
}

impl Mob for RabbitEntity {
    fn mob_sound_source(&self) -> SoundCategory {
        if self.get_variant() == RabbitVariant::Evil {
            SoundCategory::Hostile
        } else {
            SoundCategory::Neutral
        }
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn on_attack(&self, _target: &dyn EntityBase) {
        if self.get_variant() == RabbitVariant::Evil {
            let mut rng = self.get_random();
            let pitch = (rng.random::<f32>() - rng.random::<f32>()).mul_add(0.2, 1.0);
            self.play_entity_sound(Sound::EntityRabbitAttack, 1.0, pitch);
        }
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        let entity = self.get_entity();
        if self.jump_delay_ticks.load(Ordering::Relaxed) > 0 {
            self.jump_delay_ticks.fetch_sub(1, Ordering::Relaxed);
        }

        let more_carrot_ticks = self.more_carrot_ticks.load(Ordering::Relaxed);
        if more_carrot_ticks > 0 {
            let ticks = more_carrot_ticks - self.get_random().random_range(0..3);
            self.more_carrot_ticks
                .store(ticks.max(0), Ordering::Relaxed);
        }

        if entity.on_ground.load(Ordering::Relaxed) {
            if !self.was_on_ground.load(Ordering::Relaxed) {
                self.set_jumping(false);
                self.set_landing_delay();
            }

            if self.get_variant() == RabbitVariant::Evil
                && self.jump_delay_ticks.load(Ordering::Relaxed) == 0
            {
                self.jump_at_target();
            }

            let wants_jump = self
                .mob_entity
                .jump_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .wants_jump();
            if !wants_jump {
                let (has_wanted, wanted, _) = self.move_control_state();
                if has_wanted && self.jump_delay_ticks.load(Ordering::Relaxed) == 0 {
                    let pos = self.next_path_entity_pos().unwrap_or(wanted);
                    self.face_point(pos.x, pos.z);
                    self.start_jumping();
                }
            }
        }

        self.was_on_ground
            .store(entity.on_ground.load(Ordering::Relaxed), Ordering::Relaxed);
    }

    fn mob_ai_step(&self) {
        let jump_ticks = self.jump_ticks.load(Ordering::Relaxed);
        let jump_duration = self.jump_duration.load(Ordering::Relaxed);
        if jump_ticks != jump_duration {
            self.jump_ticks.store(jump_ticks + 1, Ordering::Relaxed);
        } else if jump_duration != 0 {
            self.jump_ticks.store(0, Ordering::Relaxed);
            self.jump_duration.store(0, Ordering::Relaxed);
            self.set_jumping(false);
        }
    }

    fn tick_jump_control(&self, jump: bool) {
        if jump {
            self.start_jumping();
        }
    }

    fn jump_power_multiplier(&self) -> f32 {
        let entity = self.get_entity();
        let y = entity.pos.load().y;
        let (_, wanted, speed) = self.move_control_state();
        let climbing = self
            .next_path_entity_pos()
            .is_some_and(|current_pos| current_pos.y > y + 0.5)
            || entity.horizontal_collision.load(Ordering::Relaxed)
            || self.mob_entity.living_entity.jumping.load(Ordering::SeqCst) && wanted.y > y + 0.5;
        let base_jump_power: f32 = if climbing {
            0.5
        } else if speed <= STROLL_SPEED_MOD {
            0.2
        } else {
            0.3
        };

        base_jump_power / 0.42
    }

    fn on_jump_from_ground(&self) {
        let entity = self.get_entity();
        let (_, _, speed) = self.move_control_state();
        if speed > 0.0 && entity.velocity.load().horizontal_length_squared() < 0.01 {
            let jump_height = if self.is_baby() {
                BABY_JUMP_HEIGHT
            } else {
                ADULT_JUMP_HEIGHT
            };
            entity
                .update_velocity_from_input(Vector3::new(0.0, jump_height, 1.0), f64::from(0.1f32));
        }

        entity
            .world
            .load()
            .send_entity_status(entity, EntityStatus::Jump, ActorEventID::Jump);
    }

    fn mob_get_component(&self, component: DataComponent) -> Option<Box<dyn DataComponentImpl>> {
        match component {
            DataComponent::RabbitVariant => Some(Box::new(RabbitVariantImpl {
                value: self.get_variant().name().into(),
            })),
            _ => None,
        }
    }

    fn mob_set_variant_name(&self, name: &str) {
        if let Some(variant) = RabbitVariant::from_name(name) {
            self.set_variant(variant);
        }
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        _reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        let mut variant =
            self.get_random_rabbit_variant(world, &self.get_entity().block_pos.load());
        let group_size = match group_data {
            Some(SpawnGroupData::Rabbit {
                variant: group_variant,
                group_size,
            }) => {
                variant = group_variant;
                group_size
            }
            _ => 0,
        };
        self.set_variant(variant);
        let group_size = match self.finalize_ageable_spawn(Some(SpawnGroupData::Ageable {
            group_size,
            should_spawn_baby: true,
            baby_spawn_chance: 1.0,
        })) {
            Some(SpawnGroupData::Ageable { group_size, .. }) => group_size,
            _ => group_size + 1,
        };
        Some(SpawnGroupData::Rabbit {
            variant,
            group_size,
        })
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("RabbitType", self.get_variant().id());
        nbt.put_int(
            "MoreCarrotTicks",
            self.more_carrot_ticks.load(Ordering::Relaxed),
        );
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.read_ageable_nbt(nbt);
        self.set_variant(
            nbt.get_int("RabbitType")
                .map_or(RabbitVariant::DEFAULT, RabbitVariant::by_id),
        );
        self.more_carrot_ticks.store(
            nbt.get_int("MoreCarrotTicks").unwrap_or(0),
            Ordering::Relaxed,
        );
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::rabbit::DATA_BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::rabbit::DATA_TYPE_ID,
            VarInt(self.get_variant().id()),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        self.animal_interact(player, item_stack)
    }
}

struct RabbitMoveControl {
    inner: MoveControl,
    next_jump_speed: f64,
    rabbit: Weak<RabbitEntity>,
}

impl RabbitMoveControl {
    fn new(rabbit: Weak<RabbitEntity>) -> Self {
        Self {
            inner: MoveControl::default(),
            next_jump_speed: 0.0,
            rabbit,
        }
    }

    fn set_speed_modifier(&mut self, mob: &dyn Mob, speed: f64) {
        mob.get_mob_entity().steered().navigator().set_speed(speed);
        let wanted = MoveControlTrait::get_wanted_position(&self.inner);
        self.set_wanted_position(wanted.x, wanted.y, wanted.z, speed);
    }
}

impl Control for RabbitMoveControl {}

impl MoveControlTrait for RabbitMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        let living = &mob_entity.living_entity;
        let wants_jump = mob_entity
            .jump_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .wants_jump();
        if living.entity.on_ground.load(Ordering::Relaxed)
            && !living.jumping.load(Ordering::SeqCst)
            && !wants_jump
        {
            self.set_speed_modifier(mob, 0.0);
        } else if self.inner.has_wanted() || self.inner.operation == Operation::Jumping {
            self.set_speed_modifier(mob, self.next_jump_speed);
        }

        MoveControlTrait::tick(&mut self.inner, mob);
    }

    fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, mut speed_modifier: f64) {
        if self
            .rabbit
            .upgrade()
            .is_some_and(|rabbit| rabbit.get_entity().touching_water.load(Ordering::Relaxed))
        {
            speed_modifier = 1.5;
        }

        self.inner.set_wanted_position(x, y, z, speed_modifier);
        if speed_modifier > 0.0 {
            self.next_jump_speed = speed_modifier;
        }
    }

    fn strafe(&mut self, forward: f32, right: f32) {
        self.inner.strafe(forward, right);
    }

    fn has_wanted(&self) -> bool {
        self.inner.has_wanted()
    }

    fn get_speed_modifier(&self) -> f64 {
        self.inner.get_speed_modifier()
    }

    fn get_wanted_position(&self) -> Vector3<f64> {
        MoveControlTrait::get_wanted_position(&self.inner)
    }
}

struct RabbitPanicGoal {
    inner: Box<EscapeDangerGoal>,
    speed_modifier: f64,
}

impl RabbitPanicGoal {
    fn new(speed_modifier: f64) -> Self {
        Self {
            inner: EscapeDangerGoal::new(speed_modifier),
            speed_modifier,
        }
    }
}

impl Goal for RabbitPanicGoal {
    fn tick_reach(&self, mob: &dyn Mob, running: bool) -> Option<GoalReach> {
        self.inner.tick_reach(mob, running)
    }

    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.inner.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.inner.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.inner.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.inner.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.inner.tick(mob);
        if let Some(rabbit) = as_rabbit(mob) {
            rabbit.set_speed_modifier(self.speed_modifier);
        }
    }

    fn should_run_every_tick(&self) -> bool {
        self.inner.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.inner.controls()
    }
}

#[derive(Default)]
struct RaidGardenTarget {
    wants_to_raid: AtomicBool,
    can_raid: AtomicBool,
}

struct RaidGardenGoal {
    inner: MoveToTargetPosGoal<RaidGardenTarget>,
    target: Box<RaidGardenTarget>,
}

impl RaidGardenGoal {
    fn new() -> Self {
        let target = Box::<RaidGardenTarget>::default();
        let mut inner =
            MoveToTargetPosGoal::with_default(ParentHandle::none(), f64::from(0.7f32), 16);
        // SAFETY: `target` is boxed, so its address stays fixed for as long as the goal owns it.
        inner.move_to_target_pos = unsafe { ParentHandle::new(&*target) };
        Self { inner, target }
    }

    fn raid(&self, rabbit: &RabbitEntity) {
        let entity = rabbit.get_entity();
        let world = entity.world.load_full();
        let crops_pos = self.inner.target_pos.up();
        let (block, state_id) = world.get_block_and_state_id(&crops_pos);
        if self.target.can_raid.load(Ordering::Relaxed) && block == &Block::CARROTS {
            let mut props = WheatLikeProperties::from_state_id(state_id);
            if props.age == 0 {
                world.set_block_state(
                    &crops_pos,
                    Block::AIR.default_state.id,
                    BlockFlags::NOTIFY_LISTENERS,
                );
            } else {
                props.age -= 1;
                world.set_block_state(
                    &crops_pos,
                    props.to_state_id(block),
                    BlockFlags::NOTIFY_LISTENERS,
                );
                world.emit_game_event_with_context(
                    GameEvent::BlockChange.name(),
                    crops_pos.to_centered_f64(),
                    &GameEventContext {
                        source_entity: Some(rabbit),
                    },
                );
                world.sync_world_event(
                    WorldEvent::ParticlesAndSoundDestroyBlock,
                    crops_pos,
                    i32::from(state_id.as_u16()),
                );
            }

            rabbit
                .more_carrot_ticks
                .store(MORE_CARROTS_DELAY, Ordering::Relaxed);
        }
    }
}

impl MoveToTargetPos for RaidGardenTarget {
    fn is_target_pos(&self, world: &World, block_pos: BlockPos) -> bool {
        if world
            .get_block(&block_pos)
            .has_tag(&tag::Block::MINECRAFT_SUPPORTS_CROPS)
            && self.wants_to_raid.load(Ordering::Relaxed)
            && !self.can_raid.load(Ordering::Relaxed)
        {
            let above = block_pos.up();
            let (block, state_id) = world.get_block_and_state_id(&above);
            if block == &Block::CARROTS
                && WheatLikeProperties::from_state_id(state_id).age >= MAX_AGE
            {
                self.can_raid.store(true, Ordering::Relaxed);
                return true;
            }
        }

        false
    }
}

impl Goal for RaidGardenGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if self.inner.cooldown <= 0 {
            if !mob
                .get_entity()
                .world
                .load()
                .level_info
                .load()
                .game_rules
                .mob_griefing
            {
                return false;
            }

            self.target.can_raid.store(false, Ordering::Relaxed);
            self.target.wants_to_raid.store(
                as_rabbit(mob).is_some_and(RabbitEntity::wants_more_food),
                Ordering::Relaxed,
            );
        }

        self.inner.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.target.can_raid.load(Ordering::Relaxed) && self.inner.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.inner.start(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.inner.tick(mob);
        let target = self.inner.target_pos;
        mob.get_mob_entity()
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .look_at_with_range(
                f64::from(target.0.x) + 0.5,
                f64::from(target.0.y + 1),
                f64::from(target.0.z) + 0.5,
                10.0,
                mob.get_max_look_pitch_change(),
            );
        if self.inner.is_reached_target() {
            if let Some(rabbit) = as_rabbit(mob) {
                self.raid(rabbit);
            }

            self.target.can_raid.store(false, Ordering::Relaxed);
            self.inner.cooldown = 10;
        }
    }

    fn should_run_every_tick(&self) -> bool {
        self.inner.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.inner.controls()
    }
}
