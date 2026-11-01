use pumpkin_data::entity::EntityType;

use crate::entity::ai::goal::avoid_entity::{
    AvoidEntityGoal, ThreatDistances, ThreatFilter, ThreatSelector, not_spectator,
};
use crate::entity::ai::goal::goal_selector::GoalSelector;
use crate::entity::ai::goal::revenge::MobFilter;
use crate::entity::ai::target_match::TargetMatch;

/// One vanilla `AvoidEntityGoal`. Entries sharing a priority keep vanilla's order:
/// the first registered threat wins over a nearer later one.
#[derive(Clone, Copy)]
pub struct Flee {
    priority: u8,
    from: TargetMatch,
    distance: f64,
    walk_speed: f64,
    sprint_speed: f64,
    when: Option<MobFilter>,
    keep_when: bool,
    confirm: Option<ThreatFilter>,
    selector: Option<ThreatSelector>,
    drop_target: bool,
    per_type: Option<ThreatDistances>,
}

impl Flee {
    #[must_use]
    pub const fn new(
        priority: u8,
        from: TargetMatch,
        distance: f64,
        walk_speed: f64,
        sprint_speed: f64,
    ) -> Self {
        Self {
            priority,
            from,
            distance,
            walk_speed,
            sprint_speed,
            when: None,
            keep_when: false,
            confirm: None,
            selector: None,
            drop_target: false,
            per_type: None,
        }
    }

    /// See [`AvoidEntityGoal::nearest_of`].
    #[must_use]
    pub const fn nearest_of(
        priority: u8,
        threats: ThreatDistances,
        walk_speed: f64,
        sprint_speed: f64,
    ) -> Self {
        let mut flee = Self::new(priority, TargetMatch::Any, 0.0, walk_speed, sprint_speed);
        flee.per_type = Some(threats);
        flee
    }

    /// See [`AvoidEntityGoal::when`].
    #[must_use]
    pub const fn when(mut self, filter: MobFilter) -> Self {
        self.when = Some(filter);
        self
    }

    /// See [`AvoidEntityGoal::as_long_as`].
    #[must_use]
    pub const fn as_long_as(mut self, filter: MobFilter) -> Self {
        self.when = Some(filter);
        self.keep_when = true;
        self
    }

    /// See [`AvoidEntityGoal::matching`].
    #[must_use]
    pub const fn matching(mut self, selector: ThreatSelector) -> Self {
        self.selector = Some(selector);
        self
    }

    /// See [`AvoidEntityGoal::confirmed_by`].
    #[must_use]
    pub const fn confirmed_by(mut self, filter: ThreatFilter) -> Self {
        self.confirm = Some(filter);
        self
    }

    /// See [`AvoidEntityGoal::dropping_target`].
    #[must_use]
    pub const fn dropping_target(mut self) -> Self {
        self.drop_target = true;
        self
    }

    fn build(self) -> AvoidEntityGoal {
        let mut goal = self.per_type.map_or_else(
            || AvoidEntityGoal::new(self.from, self.distance, self.walk_speed, self.sprint_speed),
            |threats| AvoidEntityGoal::nearest_of(threats, self.walk_speed, self.sprint_speed),
        );
        if let Some(when) = self.when {
            goal = if self.keep_when {
                goal.as_long_as(when)
            } else {
                goal.when(when)
            };
        }
        if let Some(selector) = self.selector {
            goal = goal.matching(selector);
        }
        if let Some(confirm) = self.confirm {
            goal = goal.confirmed_by(confirm);
        }
        if self.drop_target {
            goal = goal.dropping_target();
        }
        goal
    }
}

/// Every fish keeps 8 blocks from players, creative ones included.
pub const FISH_FLEES: &[Flee] =
    &[Flee::new(2, TargetMatch::Type(&EntityType::PLAYER), 8.0, 1.6, 1.4).matching(not_spectator)];

/// Registers every entry in order.
pub fn apply(goals: &GoalSelector, flees: &[Flee]) {
    for flee in flees {
        goals.add_goal(flee.priority, Box::new(flee.build()));
    }
}
