use std::sync::Arc;
use std::sync::Weak;
use std::sync::atomic::Ordering;

use super::{Controls, Goal, GoalReach, to_goal_ticks};
use crate::entity::EntityBase;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::cube_mob::CubeMob;
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob, player::Player};
use pumpkin_data::attributes::Attributes;
use pumpkin_data::item::Item;
use pumpkin_data::tag::{Tag, Taggable};
use pumpkin_util::math::vector3::Vector3;

const DEFAULT_STOP_DISTANCE: f64 = 2.5;
/// Following stops once the player moves further than this while being watched.
const SCARE_RANGE_SQ: f64 = 36.0;
const SCARE_MOVE_SQ: f64 = 0.010_000_000_000_000_002;
const SCARE_ROTATION: f32 = 5.0;

enum TemptItems {
    Items(&'static [&'static Item]),
    Tag(&'static Tag),
    Predicate(fn(&dyn Mob, &pumpkin_data::item_stack::ItemStack) -> bool),
}

pub enum TemptNavigation {
    Path,
    CubeMob(Weak<dyn CubeMob>),
}

pub struct TemptGoal {
    goal_control: Controls,
    speed: f64,
    tempt_items: TemptItems,
    can_scare: bool,
    stop_distance: f64,
    target_player: Option<Arc<Player>>,
    watched_pos: Vector3<f64>,
    watched_yaw: f32,
    watched_pitch: f32,
    cooldown: i32,
    running: bool,
    navigation: TemptNavigation,
}

impl TemptGoal {
    #[must_use]
    pub fn new(speed: f64, tempt_items: &'static [&'static Item], can_scare: bool) -> Self {
        Self::with_stop_distance(speed, tempt_items, can_scare, DEFAULT_STOP_DISTANCE)
    }

    #[must_use]
    pub fn with_tag(speed: f64, tag: &'static Tag, can_scare: bool) -> Self {
        Self {
            tempt_items: TemptItems::Tag(tag),
            ..Self::new(speed, &[], can_scare)
        }
    }

    #[must_use]
    pub fn with_stop_distance(
        speed: f64,
        tempt_items: &'static [&'static Item],
        can_scare: bool,
        stop_distance: f64,
    ) -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            speed,
            tempt_items: TemptItems::Items(tempt_items),
            can_scare,
            stop_distance,
            target_player: None,
            watched_pos: Vector3::new(0.0, 0.0, 0.0),
            watched_yaw: 0.0,
            watched_pitch: 0.0,
            cooldown: 0,
            running: false,
            navigation: TemptNavigation::Path,
        }
    }

    #[must_use]
    pub fn for_cube_mob(
        speed: f64,
        predicate: fn(&dyn Mob, &pumpkin_data::item_stack::ItemStack) -> bool,
        can_scare: bool,
        stop_distance: f64,
        cube: Weak<dyn CubeMob>,
    ) -> Self {
        Self {
            goal_control: Controls::LOOK,
            tempt_items: TemptItems::Predicate(predicate),
            navigation: TemptNavigation::CubeMob(cube),
            ..Self::with_stop_distance(speed, &[], can_scare, stop_distance)
        }
    }

    #[must_use]
    pub const fn is_running(&self) -> bool {
        self.running
    }

    #[must_use]
    pub const fn target_player(&self) -> Option<&Arc<Player>> {
        self.target_player.as_ref()
    }

    pub const fn set_can_scare(&mut self, can_scare: bool) {
        self.can_scare = can_scare;
    }

    fn is_tempt_item(&self, mob: &dyn Mob, stack: &pumpkin_data::item_stack::ItemStack) -> bool {
        stack.item_count > 0
            && match self.tempt_items {
                TemptItems::Items(items) => items.iter().any(|i| i.id == stack.item.id),
                TemptItems::Tag(tag) => stack.item.has_tag(tag),
                TemptItems::Predicate(predicate) => predicate(mob, stack),
            }
    }

    fn is_holding_tempt_item(&self, mob: &dyn Mob, player: &Player) -> bool {
        self.is_tempt_item(mob, &player.inventory().held_item())
            || self.is_tempt_item(mob, &player.inventory().off_hand_item())
    }

    /// Non-combat, line of sight ignored, ranged by `tempt_range` and filtered on the held item.
    fn find_tempting_player(&self, mob: &dyn Mob) -> Option<Arc<Player>> {
        let mob_entity = mob.get_mob_entity();
        let range = mob_entity
            .living_entity
            .get_attribute_value(&Attributes::TEMPT_RANGE);
        let world = mob_entity.living_entity.entity.world.load();
        let conditions = TargetPredicate::create_non_attackable()
            .ignore_visibility()
            .set_base_max_distance(range);

        // The conditions scale the range down, so sneaking shortens it.
        world.get_nearest_player(
            mob_entity.living_entity.entity.pos.load(),
            range,
            |player| {
                self.is_holding_tempt_item(mob, player)
                    && conditions.test(&world, Some(mob), player.as_ref())
            },
        )
    }

    fn watch(&mut self, player: &Player) {
        let entity = player.get_entity();
        self.watched_pos = entity.pos.load();
        self.watched_yaw = entity.yaw.load();
        self.watched_pitch = entity.pitch.load();
    }

    fn stop_navigation(&self, mob: &dyn Mob) {
        match &self.navigation {
            TemptNavigation::Path => mob.get_mob_entity().steered().navigator().stop(),
            TemptNavigation::CubeMob(cube) => {
                if let Some(cube) = cube.upgrade() {
                    cube.cube_data().speed_modifier.store(Some(0.0));
                }
            }
        }
    }

    fn navigate_towards(&self, mob: &dyn Mob, player: &Player) {
        match &self.navigation {
            TemptNavigation::Path => {
                let mob_entity = mob.get_mob_entity();
                let mob_pos = mob_entity.living_entity.entity.pos.load();
                let player_pos = player.get_entity().pos.load();
                let steered = mob_entity.steered();
                let mut navigator = steered.navigator();
                navigator.set_progress(NavigatorGoal::new(mob_pos, player_pos, self.speed));
            }
            TemptNavigation::CubeMob(cube) => {
                mob.get_mob_entity().look_at_entity(player, 10.0, 10.0);
                if let Some(cube) = cube.upgrade() {
                    let data = cube.cube_data();
                    data.target_yaw.store(cube.get_entity().yaw.load());
                    data.is_aggressive.store(true, Ordering::Relaxed);
                }
            }
        }
    }
}

impl Goal for TemptGoal {
    fn tick_reach(&self, mob: &dyn Mob, running: bool) -> Option<GoalReach> {
        let mob_entity = mob.get_mob_entity();
        let range = mob_entity
            .living_entity
            .get_attribute_value(&Attributes::TEMPT_RANGE);
        let tempted = running
            || !mob_entity
                .living_entity
                .entity
                .world
                .load()
                .get_nearby_players(mob_entity.living_entity.entity.pos.load(), range)
                .is_empty();
        Some(GoalReach {
            players: range,
            speed: if tempted { self.speed } else { 0.0 },
            links: self
                .target_player
                .iter()
                .map(|player| player.entity_id())
                .collect(),
            ..GoalReach::default()
        })
    }

    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if self.cooldown > 0 {
            self.cooldown -= 1;
            return false;
        }
        self.target_player = self.find_tempting_player(mob);
        self.target_player.is_some()
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        if self.can_scare {
            let Some(player) = self.target_player.clone() else {
                return false;
            };
            let entity = player.get_entity();
            let player_pos = entity.pos.load();
            let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();

            if mob_pos.squared_distance_to_vec(&player_pos) < SCARE_RANGE_SQ {
                if player_pos.squared_distance_to_vec(&self.watched_pos) > SCARE_MOVE_SQ {
                    return false;
                }
                if (entity.pitch.load() - self.watched_pitch).abs() > SCARE_ROTATION
                    || (entity.yaw.load() - self.watched_yaw).abs() > SCARE_ROTATION
                {
                    return false;
                }
            } else {
                self.watched_pos = player_pos;
            }

            self.watched_yaw = entity.yaw.load();
            self.watched_pitch = entity.pitch.load();
        }

        self.can_start(mob)
    }

    fn start(&mut self, _mob: &dyn Mob) {
        if let Some(player) = self.target_player.clone() {
            self.watch(&player);
        }
        self.running = true;
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(player) = &self.target_player else {
            return;
        };
        let mob_entity = mob.get_mob_entity();
        let player_pos = player.get_entity().pos.load();

        mob_entity
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .look_at_with_range(
                player_pos.x,
                player.get_entity().get_eye_y(),
                player_pos.z,
                mob.get_max_head_rotation() + 20.0,
                mob.get_max_look_pitch_change(),
            );

        let mob_pos = mob_entity.living_entity.entity.pos.load();
        if mob_pos.squared_distance_to_vec(&player_pos) < self.stop_distance * self.stop_distance {
            self.stop_navigation(mob);
        } else {
            self.navigate_towards(mob, player);
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.target_player = None;
        self.stop_navigation(mob);
        self.cooldown = to_goal_ticks(100);
        self.running = false;
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
