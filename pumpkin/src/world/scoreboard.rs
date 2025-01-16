use std::collections::HashMap;

use pumpkin_data::scoreboard::ScoreboardDisplaySlot;
use pumpkin_protocol::{
    client::play::{CDisplayObjective, CUpdateObjectives, CUpdateScore, RenderType},
    codec::var_int::VarInt,
    NumberFormat,
};
use pumpkin_util::text::TextComponent;

use super::World;

#[derive(Default)]
pub struct Scoreboard {
    objectives: HashMap<String, ScoreboardObjective<'static>>,
    //  teams: HashMap<String, Team>,
}

impl Scoreboard {
    #[must_use]
    pub fn new() -> Self {
        Self {
            objectives: HashMap::new(),
        }
    }

    pub async fn add_objective(&mut self, world: &World, objective: ScoreboardObjective<'_>) {
        if self.objectives.contains_key(objective.name) {
            // Maybe make this an error ?
            log::warn!(
                "Tried to create Objective which does already exist, {}",
                &objective.name
            );
            return;
        }
        world
            .broadcast_packet_all(&CUpdateObjectives::new(
                objective.name,
                pumpkin_protocol::client::play::Mode::Add,
                objective.display_name,
                objective.render_type,
                objective.number_format,
            ))
            .await;
        world
            .broadcast_packet_all(&CDisplayObjective::new(
                ScoreboardDisplaySlot::Sidebar,
                objective.name,
            ))
            .await;
    }

    pub async fn update_score(&self, world: &World, score: ScoreboardScore<'_>) {
        if self.objectives.contains_key(score.objective_name) {
            log::warn!(
                "Tried to place a score into a Objective which does not exist, {}",
                &score.objective_name
            );
            return;
        }
        world
            .broadcast_packet_all(&CUpdateScore::new(
                score.entity_name,
                score.objective_name,
                score.value,
                score.display_name,
                score.number_format,
            ))
            .await;
    }

    // pub fn add_team(&mut self, name: String) {
    //     if self.teams.contains_key(&name) {
    //         // Maybe make this an error ?
    //         log::warn!("Tried to create Team which does already exist, {}", name);
    //     }
    // }
}

pub struct ScoreboardObjective<'a> {
    name: &'a str,
    display_name: TextComponent,
    render_type: RenderType,
    number_format: Option<NumberFormat>,
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
    entity_name: &'a str,
    objective_name: &'a str,
    value: VarInt,
    display_name: Option<TextComponent>,
    number_format: Option<NumberFormat>,
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
