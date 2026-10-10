use std::sync::Arc;

use super::{Controls, Goal, to_goal_ticks};
use crate::entity::EntityBase;
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob, player::Player};
use pumpkin_data::attributes::Attributes;
use pumpkin_data::item::Item;
use pumpkin_util::math::vector3::Vector3;

const DEFAULT_STOP_DISTANCE: f64 = 2.5;
/// Following stops once the player moves further than this while being watched.
const SCARE_RANGE_SQ: f64 = 36.0;
const SCARE_MOVE_SQ: f64 = 0.010_000_000_000_000_002;
const SCARE_ROTATION: f32 = 5.0;

/// Vanilla subclass hooks, ported as closures: `CatTemptGoal#canUse` (tame state) and
/// `OcelotTemptGoal#canScare` (trust state).
type TemptGate = dyn Fn(&dyn Mob) -> bool + Send + Sync;

pub struct TemptGoal {
    goal_control: Controls,
    speed: f64,
    tempt_items: &'static [&'static Item],
    can_scare: bool,
    stop_distance: f64,
    target_player: Option<Arc<Player>>,
    watched_pos: Vector3<f64>,
    watched_yaw: f32,
    watched_pitch: f32,
    cooldown: i32,
    running: bool,
    /// Extra condition asked on every start/continue attempt (`CatTemptGoal#canUse`).
    can_use_gate: Option<Box<TemptGate>>,
    /// Extra condition combined with `can_scare` on every continue check
    /// (`OcelotTemptGoal#canScare`).
    scare_condition: Option<Box<TemptGate>>,
}

impl TemptGoal {
    #[must_use]
    pub fn new(speed: f64, tempt_items: &'static [&'static Item], can_scare: bool) -> Self {
        Self::with_stop_distance(speed, tempt_items, can_scare, DEFAULT_STOP_DISTANCE)
    }

    /// Like [`Self::new`], with a custom distance at which following stops once reached.
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
            tempt_items,
            can_scare,
            stop_distance,
            target_player: None,
            watched_pos: Vector3::new(0.0, 0.0, 0.0),
            watched_yaw: 0.0,
            watched_pitch: 0.0,
            cooldown: 0,
            running: false,
            can_use_gate: None,
            scare_condition: None,
        }
    }

    /// Like [`Self::new`], but the goal refuses to start while `gate` returns false.
    ///
    /// Mirrors vanilla `CatTemptGoal#canUse`'s `&& !cat.isTame()`: a tamed cat ignores the
    /// tempt item instead of following it. Consulted on every start and continue attempt,
    /// like the vanilla override (which `canContinueToUse` reaches through virtual dispatch).
    #[must_use]
    pub fn with_can_use_gate<F>(mut self, gate: F) -> Self
    where
        F: Fn(&dyn Mob) -> bool + Send + Sync + 'static,
    {
        self.can_use_gate = Some(Box::new(gate));
        self
    }

    /// Like [`Self::new`], but the startle check additionally requires `condition`.
    ///
    /// Mirrors vanilla `OcelotTemptGoal#canScare`'s `&& !ocelot.isTrusting()`: a trusting
    /// ocelot keeps following the fish instead of being startled by player movement.
    #[must_use]
    pub fn with_scare_condition<F>(mut self, condition: F) -> Self
    where
        F: Fn(&dyn Mob) -> bool + Send + Sync + 'static,
    {
        self.scare_condition = Some(Box::new(condition));
        self
    }

    /// Whether the can-use gate (if any) allows a start right now; no gate always allows.
    fn can_use_gate_allows(&self, mob: &dyn Mob) -> bool {
        self.can_use_gate.as_ref().is_none_or(|gate| gate(mob))
    }

    /// Whether the startle check applies right now: `can_scare` combined with the optional
    /// subclass condition.
    fn can_scare_now(&self, mob: &dyn Mob) -> bool {
        self.can_scare
            && self
                .scare_condition
                .as_ref()
                .is_none_or(|condition| condition(mob))
    }

    #[must_use]
    pub const fn is_running(&self) -> bool {
        self.running
    }

    fn is_tempt_item(&self, stack: &pumpkin_data::item_stack::ItemStack) -> bool {
        stack.item_count > 0 && self.tempt_items.iter().any(|i| i.id == stack.item.id)
    }

    fn is_holding_tempt_item(&self, player: &Player) -> bool {
        self.is_tempt_item(&player.inventory().held_item())
            || self.is_tempt_item(&player.inventory().off_hand_item())
    }

    /// Non-combat, line of sight ignored, ranged by `tempt_range` and filtered on the held item.
    fn find_tempting_player(&self, mob: &dyn Mob) -> Option<Arc<Player>> {
        let mob_entity = mob.get_mob_entity();
        let range = mob_entity
            .living_entity
            .get_attribute_value(&Attributes::TEMPT_RANGE);
        let world = mob_entity.living_entity.entity.world.load();

        world.get_nearest_player(
            mob_entity.living_entity.entity.pos.load(),
            range,
            |player| player.living_entity.is_part_of_game() && self.is_holding_tempt_item(player),
        )
    }

    fn watch(&mut self, player: &Player) {
        let entity = player.get_entity();
        self.watched_pos = entity.pos.load();
        self.watched_yaw = entity.yaw.load();
        self.watched_pitch = entity.pitch.load();
    }
}

impl Goal for TemptGoal {
    /// Finds the nearest player holding a tempt item; the startle cooldown and the optional
    /// can-use gate can both veto.
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if self.cooldown > 0 {
            self.cooldown -= 1;
            return false;
        }
        self.target_player = self.find_tempting_player(mob);
        self.target_player.is_some() && self.can_use_gate_allows(mob)
    }

    /// Continues while [`Self::can_start`] agrees; while `can_scare_now` holds, a moving or
    /// turning player within six blocks ends the temptation.
    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        if self.can_scare_now(mob) {
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
        let mut navigator = mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if mob_pos.squared_distance_to_vec(&player_pos) < self.stop_distance * self.stop_distance {
            navigator.stop();
        } else {
            navigator.set_progress(NavigatorGoal::new(mob_pos, player_pos, self.speed));
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.target_player = None;
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();
        self.cooldown = to_goal_ticks(100);
        self.running = false;
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
