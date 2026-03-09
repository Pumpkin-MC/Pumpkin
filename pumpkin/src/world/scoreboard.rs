use std::collections::HashMap;

use pumpkin_data::scoreboard::ScoreboardDisplaySlot;
use pumpkin_protocol::{
    NumberFormat,
    codec::var_int::VarInt,
    java::client::play::{CDisplayObjective, CUpdateObjectives, CUpdateScore, RenderType},
};
use pumpkin_util::text::TextComponent;
use tracing::warn;

use super::World;

#[derive(Default)]
pub struct Scoreboard {
    objectives: HashMap<String, StoredObjective>,
    scores: HashMap<String, HashMap<String, i32>>, // entity_name -> objective_name -> value
    display_slots: HashMap<u8, String>,            // slot (as u8) -> objective_name
}

pub struct StoredObjective {
    pub display_name: TextComponent,
    pub render_type: RenderType,
    pub number_format: Option<NumberFormat>,
}

impl Scoreboard {
    #[must_use]
    pub fn has_objective(&self, name: &str) -> bool {
        self.objectives.contains_key(name)
    }

    #[must_use]
    pub const fn get_objectives(&self) -> &HashMap<String, StoredObjective> {
        &self.objectives
    }

    pub async fn add_objective(&mut self, world: &World, objective: ScoreboardObjective<'_>) {
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
                RenderType::Integer,
                None,
            ))
            .await;
        self.objectives.insert(
            objective.name.to_string(),
            StoredObjective {
                display_name: objective.display_name,
                render_type: RenderType::Integer,
                number_format: objective.number_format,
            },
        );
    }

    pub async fn remove_objective(&mut self, world: &World, name: &str) {
        if self.objectives.remove(name).is_none() {
            return;
        }
        // Remove display slot references
        self.display_slots.retain(|_, v| v != name);
        // Remove all scores for this objective
        for scores in self.scores.values_mut() {
            scores.remove(name);
        }
        world
            .broadcast_packet_all(&CUpdateObjectives::new(
                name.to_string(),
                pumpkin_protocol::java::client::play::Mode::Remove,
                TextComponent::text(""),
                RenderType::Integer,
                None,
            ))
            .await;
    }

    pub async fn set_display_slot(
        &mut self,
        world: &World,
        slot: ScoreboardDisplaySlot,
        objective_name: Option<&str>,
    ) {
        let slot_id = slot as u8;
        if let Some(name) = objective_name {
            self.display_slots.insert(slot_id, name.to_string());
        } else {
            self.display_slots.remove(&slot_id);
        }
        world
            .broadcast_packet_all(&CDisplayObjective::new(
                ScoreboardDisplaySlot::Sidebar,
                objective_name.unwrap_or("").to_string(),
            ))
            .await;
    }

    pub async fn set_score(&mut self, world: &World, entity: &str, objective: &str, value: i32) {
        if !self.objectives.contains_key(objective) {
            warn!(
                "Tried to set a score for an objective which does not exist: {}",
                objective
            );
            return;
        }
        self.scores
            .entry(entity.to_string())
            .or_default()
            .insert(objective.to_string(), value);
        world
            .broadcast_packet_all(&CUpdateScore::new(
                entity.to_string(),
                objective.to_string(),
                VarInt(value),
                None,
                None,
            ))
            .await;
    }

    #[must_use]
    pub fn get_score(&self, entity: &str, objective: &str) -> Option<i32> {
        self.scores.get(entity)?.get(objective).copied()
    }

    pub fn reset_scores(&mut self, entity: &str, objective: Option<&str>) {
        if let Some(obj) = objective {
            if let Some(scores) = self.scores.get_mut(entity) {
                scores.remove(obj);
            }
        } else {
            self.scores.remove(entity);
        }
    }

    #[must_use]
    pub fn get_entity_scores(&self, entity: &str) -> Option<&HashMap<String, i32>> {
        self.scores.get(entity)
    }

    pub async fn update_score(&self, world: &World, score: ScoreboardScore<'_>) {
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
                score.display_name,
                score.number_format,
            ))
            .await;
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
