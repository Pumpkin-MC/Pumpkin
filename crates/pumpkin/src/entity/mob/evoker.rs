use std::sync::Mutex;
use std::sync::atomic::{AtomicI32, AtomicU8, Ordering};
use std::sync::{Arc, Weak};
use uuid::Uuid;

use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::Sound;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        Controls, Goal, active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{
        Mob, MobEntity,
        patrol::{PatrolData, PatrollingMonster},
        raider::{
            ObtainRaidLeaderBannerGoal, PathfindToRaidGoal, Raider, RaiderCelebrationGoal,
            RaiderData, RaiderMoveThroughVillageGoal,
        },
    },
    projectile::evoker_fangs::EvokerFangsEntity,
    r#type::from_type,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum IllagerSpell {
    None = 0,
    SummonVex = 1,
    Fangs = 2,
    Wololo = 3,
    Disappear = 4,
    Blindness = 5,
}

impl IllagerSpell {
    #[must_use]
    pub const fn from_u8(val: u8) -> Self {
        match val {
            1 => Self::SummonVex,
            2 => Self::Fangs,
            3 => Self::Wololo,
            4 => Self::Disappear,
            5 => Self::Blindness,
            _ => Self::None,
        }
    }
}

pub struct EvokerEntity {
    pub mob_entity: MobEntity,
    pub raider_data: RaiderData,
    spell_casting_tick_count: AtomicI32,
    current_spell: AtomicU8,
    wololo_target_id: Arc<Mutex<Option<i32>>>,
}

impl EvokerEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let evoker = Self {
            mob_entity,
            raider_data: RaiderData::default(),
            spell_casting_tick_count: AtomicI32::new(0),
            current_spell: AtomicU8::new(IllagerSpell::None as u8),
            wololo_target_id: Arc::new(Mutex::new(None)),
        };
        let mob_arc = Arc::new(evoker);
        let mob_weak: Weak<Self> = Arc::downgrade(&mob_arc);
        let mob_interface_weak: Weak<dyn Mob> = {
            let mob_dyn: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_dyn)
        };

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(EvokerCastingSpellGoal::new(mob_weak.clone())));
            goal_selector.add_goal(2, Box::new(ObtainRaidLeaderBannerGoal));
            goal_selector.add_goal(3, Box::new(RaiderMoveThroughVillageGoal::new(1.05)));
            goal_selector.add_goal(3, Box::new(PathfindToRaidGoal::default()));
            goal_selector.add_goal(4, Box::new(EvokerSummonSpellGoal::new(mob_weak.clone())));
            goal_selector.add_goal(5, Box::new(EvokerAttackSpellGoal::new(mob_weak.clone())));
            goal_selector.add_goal(6, Box::new(EvokerWololoSpellGoal::new(mob_weak)));
            goal_selector.add_goal(7, Box::new(RaiderCelebrationGoal));
            goal_selector.add_goal(8, Box::new(WanderAroundGoal::new(0.6)));
            goal_selector.add_goal(
                9,
                LookAtEntityGoal::with_default(mob_interface_weak, &EntityType::PLAYER, 3.0),
            );
            goal_selector.add_goal(10, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::VILLAGER, true),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::IRON_GOLEM, true),
            );
        };

        mob_arc
    }

    pub fn is_casting_spell(&self) -> bool {
        self.spell_casting_tick_count.load(Ordering::Relaxed) > 0
    }

    pub fn set_is_casting_spell(&self, spell: IllagerSpell) {
        self.current_spell.store(spell as u8, Ordering::Relaxed);
        let entity = &self.mob_entity.living_entity.entity;
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::evoker::SPELL_CASTING_ID,
                spell as u8 as i8,
            )],
            None,
        );
    }

    pub fn get_current_spell(&self) -> IllagerSpell {
        IllagerSpell::from_u8(self.current_spell.load(Ordering::Relaxed))
    }

    pub fn get_spell_casting_time(&self) -> i32 {
        self.spell_casting_tick_count.load(Ordering::Relaxed)
    }

    pub fn set_spell_casting_time(&self, ticks: i32) {
        self.spell_casting_tick_count
            .store(ticks, Ordering::Relaxed);
    }
}

impl Mob for EvokerEntity {
    fn as_patrolling_monster(&self) -> Option<&dyn PatrollingMonster> {
        Some(self)
    }

    fn as_raider(&self) -> Option<&dyn Raider> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_raider_nbt(nbt);
        nbt.put_int("SpellTicks", self.get_spell_casting_time());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.read_raider_nbt(nbt);
        if let Some(ticks) = nbt.get_int("SpellTicks") {
            self.set_spell_casting_time(ticks);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let ticks = self.spell_casting_tick_count.load(Ordering::Relaxed);
        if ticks > 0 {
            self.spell_casting_tick_count
                .store(ticks - 1, Ordering::Relaxed);
        }
    }
}

impl PatrollingMonster for EvokerEntity {
    fn get_patrol_data(&self) -> &PatrolData {
        &self.raider_data.patrol_data
    }
}

impl Raider for EvokerEntity {
    fn get_raider_data(&self) -> &RaiderData {
        &self.raider_data
    }

    fn get_celebrate_sound(&self) -> Sound {
        Sound::EntityEvokerCelebrate
    }
}

pub struct EvokerCastingSpellGoal {
    evoker: Weak<EvokerEntity>,
}

impl EvokerCastingSpellGoal {
    #[must_use]
    pub const fn new(evoker: Weak<EvokerEntity>) -> Self {
        Self { evoker }
    }
}

impl Goal for EvokerCastingSpellGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(evoker) = self.evoker.upgrade() else {
            return false;
        };
        evoker.is_casting_spell()
    }

    fn should_continue(&self, _mob: &dyn Mob) -> bool {
        let Some(evoker) = self.evoker.upgrade() else {
            return false;
        };
        evoker.is_casting_spell()
    }

    fn start(&mut self, _mob: &dyn Mob) {
        if let Some(evoker) = self.evoker.upgrade() {
            evoker
                .mob_entity
                .living_entity
                .entity
                .velocity
                .store(Vector3::new(0.0, 0.0, 0.0));
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        if let Some(evoker) = self.evoker.upgrade() {
            evoker.set_is_casting_spell(IllagerSpell::None);
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

pub struct EvokerAttackSpellGoal {
    evoker: Weak<EvokerEntity>,
    warmup_delay: i32,
    next_attack_tick: i32,
}

impl EvokerAttackSpellGoal {
    #[must_use]
    pub const fn new(evoker: Weak<EvokerEntity>) -> Self {
        Self {
            evoker,
            warmup_delay: 0,
            next_attack_tick: 0,
        }
    }
}

impl Goal for EvokerAttackSpellGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(evoker) = self.evoker.upgrade() else {
            return false;
        };
        if evoker.is_casting_spell() {
            return false;
        }
        let entity = &evoker.mob_entity.living_entity.entity;
        if entity.age.load(Ordering::Relaxed) < self.next_attack_tick {
            return false;
        }
        let target = evoker.mob_entity.get_target();
        target.is_some()
    }

    fn should_continue(&self, _mob: &dyn Mob) -> bool {
        self.warmup_delay > 0
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.warmup_delay = 20;
        if let Some(evoker) = self.evoker.upgrade() {
            evoker.set_spell_casting_time(40);
            let age = evoker
                .mob_entity
                .living_entity
                .entity
                .age
                .load(Ordering::Relaxed);
            self.next_attack_tick = age + 100;
            evoker.set_is_casting_spell(IllagerSpell::Fangs);
            evoker
                .mob_entity
                .living_entity
                .entity
                .play_sound(Sound::EntityEvokerPrepareAttack);
        }
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        self.warmup_delay -= 1;
        if self.warmup_delay == 0 {
            let Some(evoker) = self.evoker.upgrade() else {
                return;
            };
            let target = evoker.mob_entity.get_target();
            let Some(target) = target else {
                return;
            };

            let evoker_ent = &evoker.mob_entity.living_entity.entity;
            evoker_ent.play_sound(Sound::EntityEvokerCastSpell);

            let evoker_pos = evoker_ent.pos.load();
            let target_pos = target.get_entity().pos.load();

            let dx = target_pos.x - evoker_pos.x;
            let dz = target_pos.z - evoker_pos.z;
            let angle_towards_target = (dz.atan2(dx)) as f32;

            let min_y = evoker_pos.y.min(target_pos.y);
            let dist_sq = evoker_pos.squared_distance_to_vec(&target_pos);
            let world = evoker_ent.world.load();
            let evoker_id = evoker_ent.entity_id;

            if dist_sq < 81.0 {
                // Close range: concentric rings around Evoker
                for i in 0..5 {
                    let angle = angle_towards_target + (i as f32) * std::f32::consts::PI * 0.4;
                    let spawn_x = evoker_pos.x + (angle.cos() as f64) * 1.5;
                    let spawn_z = evoker_pos.z + (angle.sin() as f64) * 1.5;
                    let pos = Vector3::new(spawn_x, min_y, spawn_z);

                    let entity = Entity::from_uuid(
                        Uuid::new_v4(),
                        world.clone(),
                        pos,
                        &EntityType::EVOKER_FANGS,
                    );
                    let fangs = Arc::new(EvokerFangsEntity::new(entity, 0, angle, Some(evoker_id)));
                    world.spawn_entity(fangs);
                }

                for i in 0..8 {
                    let angle = angle_towards_target
                        + (i as f32) * std::f32::consts::PI * 2.0 / 8.0
                        + 1.256_637_1;
                    let spawn_x = evoker_pos.x + (angle.cos() as f64) * 2.5;
                    let spawn_z = evoker_pos.z + (angle.sin() as f64) * 2.5;
                    let pos = Vector3::new(spawn_x, min_y, spawn_z);

                    let entity = Entity::from_uuid(
                        Uuid::new_v4(),
                        world.clone(),
                        pos,
                        &EntityType::EVOKER_FANGS,
                    );
                    let fangs = Arc::new(EvokerFangsEntity::new(entity, 3, angle, Some(evoker_id)));
                    world.spawn_entity(fangs);
                }
            } else {
                // Long range: line of fangs towards target
                for i in 0..16 {
                    let reach = 1.25 * ((i + 1) as f64);
                    let spawn_x = evoker_pos.x + (angle_towards_target.cos() as f64) * reach;
                    let spawn_z = evoker_pos.z + (angle_towards_target.sin() as f64) * reach;
                    let pos = Vector3::new(spawn_x, min_y, spawn_z);

                    let entity = Entity::from_uuid(
                        Uuid::new_v4(),
                        world.clone(),
                        pos,
                        &EntityType::EVOKER_FANGS,
                    );
                    let fangs = Arc::new(EvokerFangsEntity::new(
                        entity,
                        i as u32,
                        angle_towards_target,
                        Some(evoker_id),
                    ));
                    world.spawn_entity(fangs);
                }
            }
        }
    }
}

pub struct EvokerSummonSpellGoal {
    evoker: Weak<EvokerEntity>,
    warmup_delay: i32,
    next_attack_tick: i32,
}

impl EvokerSummonSpellGoal {
    #[must_use]
    pub const fn new(evoker: Weak<EvokerEntity>) -> Self {
        Self {
            evoker,
            warmup_delay: 0,
            next_attack_tick: 0,
        }
    }
}

impl Goal for EvokerSummonSpellGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(evoker) = self.evoker.upgrade() else {
            return false;
        };
        if evoker.is_casting_spell() {
            return false;
        }
        let entity = &evoker.mob_entity.living_entity.entity;
        if entity.age.load(Ordering::Relaxed) < self.next_attack_tick {
            return false;
        }
        let target = evoker.mob_entity.get_target();
        if target.is_none() {
            return false;
        }

        // Count nearby Vexes
        let bb = entity.bounding_box.load().expand(16.0, 16.0, 16.0);
        let world = entity.world.load();
        let nearby = world.get_entities_at_box(&bb);
        let vex_count = nearby
            .iter()
            .filter(|e| *e.get_entity().entity_type == EntityType::VEX)
            .count();

        let max_allowed = (rand::random::<u8>() % 8 + 1) as usize;
        vex_count < max_allowed
    }

    fn should_continue(&self, _mob: &dyn Mob) -> bool {
        self.warmup_delay > 0
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.warmup_delay = 20;
        if let Some(evoker) = self.evoker.upgrade() {
            evoker.set_spell_casting_time(100);
            let age = evoker
                .mob_entity
                .living_entity
                .entity
                .age
                .load(Ordering::Relaxed);
            self.next_attack_tick = age + 340;
            evoker.set_is_casting_spell(IllagerSpell::SummonVex);
            evoker
                .mob_entity
                .living_entity
                .entity
                .play_sound(Sound::EntityEvokerPrepareSummon);
        }
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        self.warmup_delay -= 1;
        if self.warmup_delay == 0 {
            let Some(evoker) = self.evoker.upgrade() else {
                return;
            };
            let evoker_ent = &evoker.mob_entity.living_entity.entity;
            evoker_ent.play_sound(Sound::EntityEvokerCastSpell);

            let world = evoker_ent.world.load();
            let evoker_pos = evoker_ent.pos.load();

            for _ in 0..3 {
                let offset_x = (rand::random::<i32>() % 5 - 2) as f64;
                let offset_z = (rand::random::<i32>() % 5 - 2) as f64;
                let spawn_pos = Vector3::new(
                    evoker_pos.x + offset_x,
                    evoker_pos.y + 1.0,
                    evoker_pos.z + offset_z,
                );

                let vex = from_type(&EntityType::VEX, spawn_pos, &world, Uuid::new_v4());
                world.spawn_entity(vex);
            }
        }
    }
}

pub struct EvokerWololoSpellGoal {
    evoker: Weak<EvokerEntity>,
    warmup_delay: i32,
    next_attack_tick: i32,
}

impl EvokerWololoSpellGoal {
    #[must_use]
    pub const fn new(evoker: Weak<EvokerEntity>) -> Self {
        Self {
            evoker,
            warmup_delay: 0,
            next_attack_tick: 0,
        }
    }
}

impl Goal for EvokerWololoSpellGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(evoker) = self.evoker.upgrade() else {
            return false;
        };
        if evoker.is_casting_spell() {
            return false;
        }
        let entity = &evoker.mob_entity.living_entity.entity;
        if entity.age.load(Ordering::Relaxed) < self.next_attack_tick {
            return false;
        }
        let target = evoker.mob_entity.get_target();
        if target.is_some() {
            return false;
        }

        // Find blue sheep within 16 blocks
        let bb = entity.bounding_box.load().expand(16.0, 4.0, 16.0);
        let world = entity.world.load();
        let candidates = world.get_entities_at_box(&bb);

        for cand in candidates {
            if *cand.get_entity().entity_type == EntityType::SHEEP
                && let Some(sheep) = cand
                    .cast_any()
                    .downcast_ref::<crate::entity::passive::sheep::SheepEntity>()
            {
                // Blue color is 11 in Minecraft
                if sheep.get_color() == 11 {
                    *evoker
                        .wololo_target_id
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) =
                        Some(cand.get_entity().entity_id);
                    return true;
                }
            }
        }

        false
    }

    fn should_continue(&self, _mob: &dyn Mob) -> bool {
        self.warmup_delay > 0
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.warmup_delay = 40;
        if let Some(evoker) = self.evoker.upgrade() {
            evoker.set_spell_casting_time(60);
            let age = evoker
                .mob_entity
                .living_entity
                .entity
                .age
                .load(Ordering::Relaxed);
            self.next_attack_tick = age + 140;
            evoker.set_is_casting_spell(IllagerSpell::Wololo);
            evoker
                .mob_entity
                .living_entity
                .entity
                .play_sound(Sound::EntityEvokerPrepareWololo);
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        if let Some(evoker) = self.evoker.upgrade() {
            *evoker
                .wololo_target_id
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        }
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        self.warmup_delay -= 1;
        if self.warmup_delay == 0 {
            let Some(evoker) = self.evoker.upgrade() else {
                return;
            };
            let evoker_ent = &evoker.mob_entity.living_entity.entity;
            evoker_ent.play_sound(Sound::EntityEvokerCastSpell);

            let target_id = *evoker
                .wololo_target_id
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(target_id) = target_id else {
                return;
            };

            let world = evoker_ent.world.load();
            let bb = evoker_ent.bounding_box.load().expand(16.0, 4.0, 16.0);
            let candidates = world.get_entities_at_box(&bb);

            for cand in candidates {
                if cand.get_entity().entity_id == target_id
                    && let Some(sheep) = cand
                        .cast_any()
                        .downcast_ref::<crate::entity::passive::sheep::SheepEntity>()
                {
                    // Convert color to Red (14)
                    sheep.set_color(14);
                    break;
                }
            }
        }
    }
}
