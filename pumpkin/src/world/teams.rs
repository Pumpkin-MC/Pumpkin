use std::collections::{HashMap, HashSet};

use pumpkin_protocol::java::client::play::CUpdateTeams;
use pumpkin_util::text::TextComponent;

use crate::net::java::JavaClient;

use super::World;

#[derive(Default)]
pub struct Teams {
    teams: HashMap<String, Team>,
}

pub struct Team {
    pub display_name: TextComponent,
    pub prefix: TextComponent,
    pub suffix: TextComponent,
    pub friendly_fire: bool,
    pub see_friendly_invisibles: bool,
    pub name_tag_visibility: String,
    pub collision_rule: String,
    /// Vanilla color index (0-15 for colors, 21 for reset/white)
    pub color: i32,
    pub members: HashSet<String>,
}

impl Team {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            display_name: TextComponent::text(name.to_string()),
            prefix: TextComponent::text(String::new()),
            suffix: TextComponent::text(String::new()),
            friendly_fire: true,
            see_friendly_invisibles: true,
            name_tag_visibility: "always".to_string(),
            collision_rule: "always".to_string(),
            color: 21, // Reset (white)
            members: HashSet::new(),
        }
    }

    const fn friendly_flags(&self) -> u8 {
        let mut flags = 0u8;
        if self.friendly_fire {
            flags |= 0x01;
        }
        if self.see_friendly_invisibles {
            flags |= 0x02;
        }
        flags
    }
}

impl Teams {
    /// Send all existing teams to a joining player
    pub async fn send_to_player(&self, client: &JavaClient) {
        for (name, team) in &self.teams {
            let members: Vec<String> = team.members.iter().cloned().collect();
            client
                .enqueue_packet(&CUpdateTeams::create(
                    name.clone(),
                    team.display_name.clone(),
                    team.friendly_flags(),
                    team.name_tag_visibility.clone(),
                    team.collision_rule.clone(),
                    team.color,
                    team.prefix.clone(),
                    team.suffix.clone(),
                    members,
                ))
                .await;
        }
    }

    #[must_use]
    pub fn has_team(&self, name: &str) -> bool {
        self.teams.contains_key(name)
    }

    #[must_use]
    pub const fn get_teams(&self) -> &HashMap<String, Team> {
        &self.teams
    }

    #[must_use]
    pub fn get_team(&self, name: &str) -> Option<&Team> {
        self.teams.get(name)
    }

    #[must_use]
    pub fn get_team_mut(&mut self, name: &str) -> Option<&mut Team> {
        self.teams.get_mut(name)
    }

    /// Find which team a member belongs to
    #[must_use]
    pub fn get_member_team(&self, member: &str) -> Option<&str> {
        for (name, team) in &self.teams {
            if team.members.contains(member) {
                return Some(name);
            }
        }
        None
    }

    pub async fn add_team(&mut self, world: &World, name: &str) {
        let team = Team::new(name);
        let packet = CUpdateTeams::create(
            name.to_string(),
            team.display_name.clone(),
            team.friendly_flags(),
            team.name_tag_visibility.clone(),
            team.collision_rule.clone(),
            team.color,
            team.prefix.clone(),
            team.suffix.clone(),
            Vec::new(),
        );
        self.teams.insert(name.to_string(), team);
        world.broadcast_packet_all(&packet).await;
    }

    pub async fn remove_team(&mut self, world: &World, name: &str) -> Option<Team> {
        let team = self.teams.remove(name)?;
        world
            .broadcast_packet_all(&CUpdateTeams::remove(name.to_string()))
            .await;
        Some(team)
    }

    pub async fn add_members(
        &mut self,
        world: &World,
        team_name: &str,
        members: &[String],
    ) -> usize {
        let mut added = 0;

        // Remove members from any existing teams first
        for member in members {
            for (name, team) in &mut self.teams {
                if *name != team_name && team.members.remove(member) {
                    world
                        .broadcast_packet_all(&CUpdateTeams::remove_entities(
                            name.clone(),
                            vec![member.clone()],
                        ))
                        .await;
                }
            }
        }

        if let Some(team) = self.teams.get_mut(team_name) {
            let mut new_members = Vec::new();
            for member in members {
                if team.members.insert(member.clone()) {
                    new_members.push(member.clone());
                    added += 1;
                }
            }
            if !new_members.is_empty() {
                world
                    .broadcast_packet_all(&CUpdateTeams::add_entities(
                        team_name.to_string(),
                        new_members,
                    ))
                    .await;
            }
        }
        added
    }

    pub async fn remove_members(
        &mut self,
        world: &World,
        team_name: &str,
        members: &[String],
    ) -> usize {
        let mut removed = 0;
        if let Some(team) = self.teams.get_mut(team_name) {
            let mut removed_members = Vec::new();
            for member in members {
                if team.members.remove(member) {
                    removed_members.push(member.clone());
                    removed += 1;
                }
            }
            if !removed_members.is_empty() {
                world
                    .broadcast_packet_all(&CUpdateTeams::remove_entities(
                        team_name.to_string(),
                        removed_members,
                    ))
                    .await;
            }
        }
        removed
    }

    /// Remove members from whatever team they're on (for `/team leave`)
    pub async fn leave_members(&mut self, world: &World, members: &[String]) -> usize {
        let mut removed = 0;
        for (team_name, team) in &mut self.teams {
            let mut left = Vec::new();
            for member in members {
                if team.members.remove(member) {
                    left.push(member.clone());
                    removed += 1;
                }
            }
            if !left.is_empty() {
                world
                    .broadcast_packet_all(&CUpdateTeams::remove_entities(team_name.clone(), left))
                    .await;
            }
        }
        removed
    }

    /// Broadcast a team update packet (mode 2) for the given team
    pub async fn broadcast_team_update(&self, world: &World, team_name: &str) {
        if let Some(team) = self.teams.get(team_name) {
            let packet = CUpdateTeams::update(
                team_name.to_string(),
                team.display_name.clone(),
                team.friendly_flags(),
                team.name_tag_visibility.clone(),
                team.collision_rule.clone(),
                team.color,
                team.prefix.clone(),
                team.suffix.clone(),
            );
            world.broadcast_packet_all(&packet).await;
        }
    }

    pub async fn empty_team(&mut self, world: &World, team_name: &str) -> usize {
        if let Some(team) = self.teams.get_mut(team_name) {
            let members: Vec<String> = team.members.drain().collect();
            let count = members.len();
            if !members.is_empty() {
                world
                    .broadcast_packet_all(&CUpdateTeams::remove_entities(
                        team_name.to_string(),
                        members,
                    ))
                    .await;
            }
            count
        } else {
            0
        }
    }
}
