use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::Block;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

use crate::entity::custom_sound::CustomSound;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        Goal, active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, revenge::RevengeGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct SilverfishEntity {
    pub entity: Arc<MobEntity>,
    wake_up_friends_ticks: AtomicI32,
}

impl SilverfishEntity {
    pub const XP_REWARD: u32 = 5;

    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = Arc::new(MobEntity::new(entity));
        let silverfish = Self {
            entity,
            wake_up_friends_ticks: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(silverfish);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc
                .entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut target_selector = mob_arc
                .entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            goal_selector.add_goal(
                3,
                Box::new(SilverfishWakeUpFriendsGoal::new(Arc::downgrade(&mob_arc))),
            );
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, false)));
            goal_selector.add_goal(
                5,
                Box::new(SilverfishMergeWithStoneGoal::new(Arc::downgrade(&mob_arc))),
            );
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }
}

impl CustomSound for SilverfishEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntitySilverfishDeath)
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntitySilverfishHurt)
    }
}

impl Mob for SilverfishEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let entity = &self.entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }

        let yaw = entity.yaw.load();
        entity.body_yaw.store(yaw);
        entity.head_yaw.store(yaw);

        // Vanilla ambient sound
        if rand::random_range(0..100) == 0 && !entity.silent.load(Ordering::Relaxed) {
            entity.world.load().play_sound_fine(
                Sound::EntitySilverfishAmbient,
                SoundCategory::Hostile,
                &entity.pos.load(),
                1.0,
                1.0,
            );
        }
    }

    fn on_damage(&self, _damage_type: DamageType, source: Option<&dyn EntityBase>) {
        if source.is_some() && self.wake_up_friends_ticks.load(Ordering::Relaxed) == 0 {
            self.wake_up_friends_ticks.store(20, Ordering::Relaxed);
        }
    }

    fn get_base_experience_reward(&self) -> u32 {
        Self::XP_REWARD
    }
}

fn get_infested_block_for_host(block: &Block) -> Option<&'static Block> {
    if block.id == Block::STONE.id {
        Some(&Block::INFESTED_STONE)
    } else if block.id == Block::COBBLESTONE.id {
        Some(&Block::INFESTED_COBBLESTONE)
    } else if block.id == Block::STONE_BRICKS.id {
        Some(&Block::INFESTED_STONE_BRICKS)
    } else if block.id == Block::MOSSY_STONE_BRICKS.id {
        Some(&Block::INFESTED_MOSSY_STONE_BRICKS)
    } else if block.id == Block::CRACKED_STONE_BRICKS.id {
        Some(&Block::INFESTED_CRACKED_STONE_BRICKS)
    } else if block.id == Block::CHISELED_STONE_BRICKS.id {
        Some(&Block::INFESTED_CHISELED_STONE_BRICKS)
    } else if block.id == Block::DEEPSLATE.id {
        Some(&Block::INFESTED_DEEPSLATE)
    } else {
        None
    }
}

fn get_host_block_for_infested(block: &Block) -> Option<&'static Block> {
    if block.id == Block::INFESTED_STONE.id {
        Some(&Block::STONE)
    } else if block.id == Block::INFESTED_COBBLESTONE.id {
        Some(&Block::COBBLESTONE)
    } else if block.id == Block::INFESTED_STONE_BRICKS.id {
        Some(&Block::STONE_BRICKS)
    } else if block.id == Block::INFESTED_MOSSY_STONE_BRICKS.id {
        Some(&Block::MOSSY_STONE_BRICKS)
    } else if block.id == Block::INFESTED_CRACKED_STONE_BRICKS.id {
        Some(&Block::CRACKED_STONE_BRICKS)
    } else if block.id == Block::INFESTED_CHISELED_STONE_BRICKS.id {
        Some(&Block::CHISELED_STONE_BRICKS)
    } else if block.id == Block::INFESTED_DEEPSLATE.id {
        Some(&Block::DEEPSLATE)
    } else {
        None
    }
}

pub struct SilverfishMergeWithStoneGoal {
    silverfish: Weak<SilverfishEntity>,
    merge_cooldown: i32,
}

impl SilverfishMergeWithStoneGoal {
    #[must_use]
    pub const fn new(silverfish: Weak<SilverfishEntity>) -> Self {
        Self {
            silverfish,
            merge_cooldown: 0,
        }
    }
}

impl Goal for SilverfishMergeWithStoneGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(silverfish) = self.silverfish.upgrade() else {
            return false;
        };
        if silverfish.entity.get_target().is_some() {
            return false;
        }
        self.merge_cooldown -= 1;
        if self.merge_cooldown > 0 {
            return false;
        }
        self.merge_cooldown = 20;

        let entity = &silverfish.entity.living_entity.entity;
        let world = entity.world.load();
        if !world.level_info.load().game_rules.mob_griefing {
            return false;
        }

        rand::random_range(0..10) == 0
    }

    fn start(&mut self, _mob: &dyn Mob) {
        let Some(silverfish) = self.silverfish.upgrade() else {
            return;
        };
        let entity = &silverfish.entity.living_entity.entity;
        let world = entity.world.load();
        let base_pos = entity.block_pos.load();

        let directions = [
            (0, -1, 0),
            (0, 1, 0),
            (-1, 0, 0),
            (1, 0, 0),
            (0, 0, -1),
            (0, 0, 1),
        ];
        let offset = directions[rand::random_range(0..6)];
        let target_pos = BlockPos::new(
            base_pos.0.x + offset.0,
            base_pos.0.y + offset.1,
            base_pos.0.z + offset.2,
        );

        let block = world.get_block(&target_pos);
        if let Some(infested) = get_infested_block_for_host(block) {
            world.set_block_state(&target_pos, infested.default_state.id, BlockFlags::NOTIFY_ALL);
            world.play_sound(
                Sound::EntitySilverfishHurt,
                SoundCategory::Hostile,
                &entity.pos.load(),
            );
            entity.remove();
        }
    }
}

pub struct SilverfishWakeUpFriendsGoal {
    silverfish: Weak<SilverfishEntity>,
}

impl SilverfishWakeUpFriendsGoal {
    #[must_use]
    pub const fn new(silverfish: Weak<SilverfishEntity>) -> Self {
        Self { silverfish }
    }
}

impl Goal for SilverfishWakeUpFriendsGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(silverfish) = self.silverfish.upgrade() else {
            return false;
        };
        silverfish.wake_up_friends_ticks.load(Ordering::Relaxed) > 0
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        let Some(silverfish) = self.silverfish.upgrade() else {
            return false;
        };
        silverfish.wake_up_friends_ticks.load(Ordering::Relaxed) > 0
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        let Some(silverfish) = self.silverfish.upgrade() else {
            return;
        };
        let remaining = silverfish.wake_up_friends_ticks.fetch_sub(1, Ordering::Relaxed) - 1;
        if remaining > 0 {
            return;
        }

        let entity = &silverfish.entity.living_entity.entity;
        let world = entity.world.load();
        let base_pos = entity.block_pos.load();
        let mob_griefing = world.level_info.load().game_rules.mob_griefing;

        'outer: for dy in -5..=5 {
            for dx in -10..=10 {
                for dz in -10..=10 {
                    let test_pos = BlockPos::new(
                        base_pos.0.x + dx,
                        base_pos.0.y + dy,
                        base_pos.0.z + dz,
                    );
                    let block = world.get_block(&test_pos);
                    if let Some(host) = get_host_block_for_infested(block) {
                        if mob_griefing {
                            world.break_block(&test_pos, None, BlockFlags::NOTIFY_ALL);
                            let spawn_pos = Vector3::new(
                                f64::from(test_pos.0.x) + 0.5,
                                f64::from(test_pos.0.y),
                                f64::from(test_pos.0.z) + 0.5,
                            );
                            let new_entity = crate::entity::r#type::from_type(
                                &EntityType::SILVERFISH,
                                spawn_pos,
                                &world,
                                uuid::Uuid::new_v4(),
                            );
                            world.spawn_entity(new_entity);
                        } else {
                            world.set_block_state(
                                &test_pos,
                                host.default_state.id,
                                BlockFlags::NOTIFY_ALL,
                            );
                        }

                        if rand::random::<bool>() {
                            break 'outer;
                        }
                    }
                }
            }
        }
    }
}
