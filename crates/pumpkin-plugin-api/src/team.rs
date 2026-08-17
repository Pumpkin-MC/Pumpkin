use crate::wit::pumpkin::plugin::common::NamedColor;
use crate::wit::pumpkin::plugin::player::Player;
use crate::wit::pumpkin::plugin::scoreboard::{
    CollisionRule, NametagVisibility, Scoreboard, TeamSettings,
};
use crate::wit::pumpkin::plugin::text::TextComponent;

/// Builder for constructing [`TeamSettings`].
pub struct TeamSettingsBuilder {
    display_name: Option<TextComponent>,
    friendly_fire: bool,
    see_friendly_invisibles: bool,
    nametag_visibility: NametagVisibility,
    collision_rule: CollisionRule,
    color: NamedColor,
    prefix: Option<TextComponent>,
    suffix: Option<TextComponent>,
}

impl Default for TeamSettingsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TeamSettingsBuilder {
    /// Creates a new `TeamSettingsBuilder` with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            display_name: None,
            friendly_fire: true,
            see_friendly_invisibles: false,
            nametag_visibility: NametagVisibility::Always,
            collision_rule: CollisionRule::Always,
            color: NamedColor::White,
            prefix: None,
            suffix: None,
        }
    }

    /// Sets the team's display name.
    #[must_use]
    pub fn display_name(mut self, name: impl Into<TextComponent>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Sets whether friendly fire is enabled for members of this team.
    #[must_use]
    pub fn friendly_fire(mut self, allow: bool) -> Self {
        self.friendly_fire = allow;
        self
    }

    /// Sets whether teammates can see invisible friendly players.
    #[must_use]
    pub fn see_friendly_invisibles(mut self, see: bool) -> Self {
        self.see_friendly_invisibles = see;
        self
    }

    /// Sets nametag visibility for this team.
    #[must_use]
    pub fn nametag_visibility(mut self, vis: NametagVisibility) -> Self {
        self.nametag_visibility = vis;
        self
    }

    /// Sets the collision rule for members of this team.
    #[must_use]
    pub fn collision_rule(mut self, rule: CollisionRule) -> Self {
        self.collision_rule = rule;
        self
    }

    /// Sets the display and glowing color for this team.
    #[must_use]
    pub fn color(mut self, color: NamedColor) -> Self {
        self.color = color;
        self
    }

    /// Sets the player prefix shown before member names.
    #[must_use]
    pub fn prefix(mut self, prefix: impl Into<TextComponent>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Sets the player suffix shown after member names.
    #[must_use]
    pub fn suffix(mut self, suffix: impl Into<TextComponent>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    /// Builds the [`TeamSettings`].
    #[must_use]
    pub fn build(self) -> TeamSettings {
        TeamSettings {
            display_name: self.display_name.unwrap_or_else(|| TextComponent::text("")),
            friendly_fire: self.friendly_fire,
            see_friendly_invisibles: self.see_friendly_invisibles,
            nametag_visibility: self.nametag_visibility,
            collision_rule: self.collision_rule,
            color: self.color,
            prefix: self.prefix.unwrap_or_else(|| TextComponent::text("")),
            suffix: self.suffix.unwrap_or_else(|| TextComponent::text("")),
        }
    }
}

/// A high-level representation of a scoreboard team.
pub struct Team<'a> {
    scoreboard: &'a Scoreboard,
    name: String,
}

impl<'a> Team<'a> {
    /// Creates a handle to a team on the given scoreboard.
    #[must_use]
    pub fn new(scoreboard: &'a Scoreboard, name: impl Into<String>) -> Self {
        Self {
            scoreboard,
            name: name.into(),
        }
    }

    /// Returns the team's internal identifier name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the current settings for this team, if it exists on the scoreboard.
    #[must_use]
    pub fn get_settings(&self) -> Option<TeamSettings> {
        self.scoreboard.get_team(&self.name)
    }

    /// Updates the settings for this team on the scoreboard.
    pub fn update_settings(&self, settings: TeamSettings) {
        self.scoreboard.update_team(&self.name, settings);
    }

    /// Gets the display name of this team.
    #[must_use]
    pub fn display_name(&self) -> Option<TextComponent> {
        self.get_settings().map(|s| s.display_name)
    }

    /// Sets the display name of this team.
    pub fn set_display_name(&self, name: TextComponent) {
        if let Some(mut s) = self.get_settings() {
            s.display_name = name;
            self.update_settings(s);
        }
    }

    /// Gets the prefix of this team.
    #[must_use]
    pub fn prefix(&self) -> Option<TextComponent> {
        self.get_settings().map(|s| s.prefix)
    }

    /// Sets the prefix of this team.
    pub fn set_prefix(&self, prefix: TextComponent) {
        if let Some(mut s) = self.get_settings() {
            s.prefix = prefix;
            self.update_settings(s);
        }
    }

    /// Gets the suffix of this team.
    #[must_use]
    pub fn suffix(&self) -> Option<TextComponent> {
        self.get_settings().map(|s| s.suffix)
    }

    /// Sets the suffix of this team.
    pub fn set_suffix(&self, suffix: TextComponent) {
        if let Some(mut s) = self.get_settings() {
            s.suffix = suffix;
            self.update_settings(s);
        }
    }

    /// Gets the team color.
    #[must_use]
    pub fn color(&self) -> Option<NamedColor> {
        self.get_settings().map(|s| s.color)
    }

    /// Sets the team color.
    pub fn set_color(&self, color: NamedColor) {
        if let Some(mut s) = self.get_settings() {
            s.color = color;
            self.update_settings(s);
        }
    }

    /// Gets whether friendly fire is enabled.
    #[must_use]
    pub fn allow_friendly_fire(&self) -> bool {
        self.get_settings().map_or(true, |s| s.friendly_fire)
    }

    /// Sets whether friendly fire is enabled.
    pub fn set_allow_friendly_fire(&self, allow: bool) {
        if let Some(mut s) = self.get_settings() {
            s.friendly_fire = allow;
            self.update_settings(s);
        }
    }

    /// Gets whether teammates can see friendly invisible players.
    #[must_use]
    pub fn can_see_friendly_invisibles(&self) -> bool {
        self.get_settings()
            .map_or(false, |s| s.see_friendly_invisibles)
    }

    /// Sets whether teammates can see friendly invisible players.
    pub fn set_can_see_friendly_invisibles(&self, see: bool) {
        if let Some(mut s) = self.get_settings() {
            s.see_friendly_invisibles = see;
            self.update_settings(s);
        }
    }

    /// Gets nametag visibility for this team.
    #[must_use]
    pub fn nametag_visibility(&self) -> Option<NametagVisibility> {
        self.get_settings().map(|s| s.nametag_visibility)
    }

    /// Sets nametag visibility for this team.
    pub fn set_nametag_visibility(&self, vis: NametagVisibility) {
        if let Some(mut s) = self.get_settings() {
            s.nametag_visibility = vis;
            self.update_settings(s);
        }
    }

    /// Gets the collision rule for this team.
    #[must_use]
    pub fn collision_rule(&self) -> Option<CollisionRule> {
        self.get_settings().map(|s| s.collision_rule)
    }

    /// Sets the collision rule for this team.
    pub fn set_collision_rule(&self, rule: CollisionRule) {
        if let Some(mut s) = self.get_settings() {
            s.collision_rule = rule;
            self.update_settings(s);
        }
    }

    /// Returns a list of all player / entity names in this team.
    #[must_use]
    pub fn get_players(&self) -> Vec<String> {
        self.scoreboard.get_team_players(&self.name)
    }

    /// Adds a player or entity name to this team.
    pub fn add_player(&self, player_name: &str) {
        self.scoreboard.add_player_to_team(&self.name, player_name);
    }

    /// Removes a player or entity name from this team.
    pub fn remove_player(&self, player_name: &str) {
        self.scoreboard
            .remove_player_from_team(&self.name, player_name);
    }

    /// Checks if a player or entity name is in this team.
    #[must_use]
    pub fn has_player(&self, player_name: &str) -> bool {
        self.get_players().iter().any(|p| p == player_name)
    }

    /// Removes all members from this team.
    pub fn clear_players(&self) {
        self.scoreboard.clear_team_players(&self.name);
    }

    /// Removes this team from the scoreboard.
    pub fn unregister(self) {
        self.scoreboard.remove_team(&self.name);
    }
}

/// Extension trait for [`Scoreboard`] providing team operations.
pub trait ScoreboardTeamExt {
    /// Registers and creates a new team on the scoreboard.
    fn register_new_team(&self, name: &str, settings: TeamSettings) -> Team<'_>;
    /// Gets a team by name, if it exists on the scoreboard.
    fn get_team_handle(&self, name: &str) -> Option<Team<'_>>;
    /// Returns all teams on the scoreboard.
    fn get_all_teams(&self) -> Vec<Team<'_>>;
    /// Gets the team that a player belongs to, if any.
    fn get_player_team_handle(&self, player_name: &str) -> Option<Team<'_>>;
}

impl ScoreboardTeamExt for Scoreboard {
    fn register_new_team(&self, name: &str, settings: TeamSettings) -> Team<'_> {
        self.create_team(name, settings);
        Team::new(self, name)
    }

    fn get_team_handle(&self, name: &str) -> Option<Team<'_>> {
        self.get_team(name).is_some().then(|| Team::new(self, name))
    }

    fn get_all_teams(&self) -> Vec<Team<'_>> {
        self.get_teams()
            .into_iter()
            .map(|name| Team::new(self, name))
            .collect()
    }

    fn get_player_team_handle(&self, player_name: &str) -> Option<Team<'_>> {
        self.get_player_team(player_name)
            .map(|name| Team::new(self, name))
    }
}

/// Extension trait for [`Player`] team operations.
pub trait PlayerTeamExt {
    /// Gets the active team name for this player, if any.
    fn get_team_name(&self) -> Option<String>;
}

impl PlayerTeamExt for Player {
    fn get_team_name(&self) -> Option<String> {
        self.get_team()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_settings_builder_defaults() {
        let dummy_text: TextComponent = unsafe { std::mem::zeroed() };
        let dummy_text2: TextComponent = unsafe { std::mem::zeroed() };
        let dummy_text3: TextComponent = unsafe { std::mem::zeroed() };
        let settings = TeamSettingsBuilder::new()
            .display_name(dummy_text)
            .color(NamedColor::Red)
            .prefix(dummy_text2)
            .suffix(dummy_text3)
            .friendly_fire(false)
            .see_friendly_invisibles(true)
            .nametag_visibility(NametagVisibility::HideForOtherTeams)
            .collision_rule(CollisionRule::PushOwnTeam)
            .build();

        assert!(!settings.friendly_fire);
        assert!(settings.see_friendly_invisibles);
        assert_eq!(settings.color, NamedColor::Red);
        assert_eq!(
            settings.nametag_visibility,
            NametagVisibility::HideForOtherTeams
        );
        assert_eq!(settings.collision_rule, CollisionRule::PushOwnTeam);
        std::mem::forget(settings);
    }
}
