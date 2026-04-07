use std::collections::HashMap;

use pumpkin_data::scoreboard::ScoreboardDisplaySlot;
use pumpkin_protocol::{
    NumberFormat,
    codec::var_int::VarInt,
    java::client::play::{
        CDisplayObjective, CSetPlayerTeam, CUpdateObjectives, CUpdateScore, RenderType, TeamMethod,
        TeamParameters,
    },
};
use pumpkin_util::text::{TextComponent, color::NamedColor};
use tracing::warn;

use super::World;

#[derive(Default)]
pub struct Scoreboard {
    objectives: HashMap<String, ScoreboardObjective<'static>>,
    teams: HashMap<String, Team>,
    scores: HashMap<String, HashMap<String, ScoreboardScore<'static>>>,
}

impl Scoreboard {
    pub async fn add_objective(&mut self, world: &World, objective: ScoreboardObjective<'static>) {
        if self.objectives.contains_key(objective.name) {
            warn!(
                "Tried to create an objective which already exists: {}",
                &objective.name
            );
            return;
        }
        world
            .broadcast_packet_all(&CUpdateObjectives::new(
                objective.name.to_string(),
                pumpkin_protocol::java::client::play::Mode::Add,
                objective.display_name.clone(),
                objective.render_type,
                objective.number_format.clone(),
            ))
            .await;
        world
            .broadcast_packet_all(&CDisplayObjective::new(
                ScoreboardDisplaySlot::Sidebar,
                objective.name.to_string(),
            ))
            .await;
        self.objectives
            .insert(objective.name.to_string(), objective);
    }

    pub async fn remove_objective(&mut self, world: &World, name: &str) {
        if !self.objectives.contains_key(name) {
            warn!(
                "Tried to remove an objective which does not exist: {}",
                name
            );
            return;
        }
        world
            .broadcast_packet_all(&CUpdateObjectives::new(
                name.to_string(),
                pumpkin_protocol::java::client::play::Mode::Remove,
                TextComponent::empty(),
                pumpkin_protocol::java::client::play::RenderType::Integer,
                None,
            ))
            .await;
        self.objectives.remove(name);
        self.scores.remove(name);
    }

    pub async fn update_score(&mut self, world: &World, score: ScoreboardScore<'static>) {
        if !self.objectives.contains_key(score.objective_name) {
            warn!(
                "Tried to place a score into an objective which does not exist: {}",
                &score.objective_name
            );
            return;
        }
        world
            .broadcast_packet_all(&CUpdateScore::new(
                score.entity_name.to_string(),
                score.objective_name.to_string(),
                score.value,
                score.display_name.clone(),
                score.number_format.clone(),
            ))
            .await;

        self.scores
            .entry(score.objective_name.to_string())
            .or_default()
            .insert(score.entity_name.to_string(), score);
    }

    pub async fn remove_score(&mut self, world: &World, entity_name: &str, objective_name: &str) {
        world
            .broadcast_packet_all(&CUpdateScore::new_remove(
                entity_name.to_string(),
                objective_name.to_string(),
            ))
            .await;

        if let Some(objective_scores) = self.scores.get_mut(objective_name) {
            objective_scores.remove(entity_name);
        }
    }

    pub async fn add_team(&mut self, world: &World, team: Team) {
        if self.teams.contains_key(&team.name) {
            warn!(
                "Tried to create Team which does already exist, {}",
                team.name
            );
            return;
        }

        let parameters = TeamParameters {
            display_name: &team.display_name,
            options: team.options,
            nametag_visibility: team.nametag_visibility.to_str(),
            collision_rule: team.collision_rule.to_str(),
            color: team.color as i32,
            player_prefix: &team.player_prefix,
            player_suffix: &team.player_suffix,
        };

        world
            .broadcast_packet_all(&CSetPlayerTeam {
                team_name: team.name.clone(),
                method: TeamMethod::Create,
                parameters: Some(parameters),
                players: team.players.clone(),
            })
            .await;

        self.teams.insert(team.name.clone(), team);
    }

    pub async fn update_team(&mut self, world: &World, team: Team) {
        if !self.teams.contains_key(&team.name) {
            warn!("Tried to update Team which does not exist, {}", team.name);
            return;
        }

        let parameters = TeamParameters {
            display_name: &team.display_name,
            options: team.options,
            nametag_visibility: team.nametag_visibility.to_str(),
            collision_rule: team.collision_rule.to_str(),
            color: team.color as i32,
            player_prefix: &team.player_prefix,
            player_suffix: &team.player_suffix,
        };

        world
            .broadcast_packet_all(&CSetPlayerTeam {
                team_name: team.name.clone(),
                method: TeamMethod::Update,
                parameters: Some(parameters),
                players: Vec::new(),
            })
            .await;

        self.teams.insert(team.name.clone(), team);
    }

    pub async fn remove_team(&mut self, world: &World, name: &str) {
        if !self.teams.contains_key(name) {
            warn!("Tried to remove Team which does not exist, {}", name);
            return;
        }

        world
            .broadcast_packet_all(&CSetPlayerTeam {
                team_name: name.to_string(),
                method: TeamMethod::Remove,
                parameters: None,
                players: Vec::new(),
            })
            .await;

        self.teams.remove(name);
    }

    pub async fn add_player_to_team(&mut self, world: &World, team_name: &str, player: String) {
        let Some(team) = self.teams.get_mut(team_name) else {
            warn!(
                "Tried to add player to Team which does not exist, {}",
                team_name
            );
            return;
        };

        if team.players.contains(&player) {
            return;
        }

        world
            .broadcast_packet_all(&CSetPlayerTeam {
                team_name: team_name.to_string(),
                method: TeamMethod::AddPlayers,
                parameters: None,
                players: vec![player.clone()],
            })
            .await;

        team.players.push(player);
    }

    pub async fn remove_player_from_team(&mut self, world: &World, team_name: &str, player: &str) {
        let Some(team) = self.teams.get_mut(team_name) else {
            warn!(
                "Tried to remove player from Team which does not exist, {}",
                team_name
            );
            return;
        };

        if !team.players.contains(&player.to_string()) {
            return;
        }

        world
            .broadcast_packet_all(&CSetPlayerTeam {
                team_name: team_name.to_string(),
                method: TeamMethod::RemovePlayers,
                parameters: None,
                players: vec![player.to_string()],
            })
            .await;

        team.players.retain(|p| p != player);
    }
}

pub struct ScoreboardObjective<'a> {
    pub name: &'a str,
    pub display_name: TextComponent,
    pub render_type: RenderType,
    pub number_format: Option<NumberFormat>,
}

impl<'a> ScoreboardObjective<'a> {
    #[must_use]
    pub const fn new(
        name: &'a str,
        display_name: TextComponent,
        render_type: RenderType,
        number_format: Option<NumberFormat>,
    ) -> Self {
        Self {
            name,
            display_name,
            render_type,
            number_format,
        }
    }
}

pub struct ScoreboardScore<'a> {
    pub entity_name: &'a str,
    pub objective_name: &'a str,
    pub value: VarInt,
    pub display_name: Option<TextComponent>,
    pub number_format: Option<NumberFormat>,
}

impl<'a> ScoreboardScore<'a> {
    #[must_use]
    pub const fn new(
        entity_name: &'a str,
        objective_name: &'a str,
        value: VarInt,
        display_name: Option<TextComponent>,
        number_format: Option<NumberFormat>,
    ) -> Self {
        Self {
            entity_name,
            objective_name,
            value,
            display_name,
            number_format,
        }
    }
}

pub enum NameTagVisibility {
    Always,
    Never,
    HideForOtherTeams,
    HideForOwnTeam,
}

impl NameTagVisibility {
    #[must_use]
    pub const fn to_str(&self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Never => "never",
            Self::HideForOtherTeams => "hideForOtherTeams",
            Self::HideForOwnTeam => "hideForOwnTeam",
        }
    }
}

pub enum CollisionRule {
    Always,
    Never,
    PushOtherTeams,
    PushOwnTeam,
}

impl CollisionRule {
    #[must_use]
    pub const fn to_str(&self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Never => "never",
            Self::PushOtherTeams => "pushOtherTeams",
            Self::PushOwnTeam => "pushOwnTeam",
        }
    }
}

pub struct Team {
    pub name: String,
    pub display_name: TextComponent,
    pub options: i8,
    pub nametag_visibility: NameTagVisibility,
    pub collision_rule: CollisionRule,
    pub color: NamedColor,
    pub player_prefix: TextComponent,
    pub player_suffix: TextComponent,
    pub players: Vec<String>,
}
