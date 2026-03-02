use crate::entity::player::Player;
use crate::server::Server;
use crate::world::bossbar::{Bossbar, BossbarColor, BossbarDivisions};
use pumpkin_util::identifier::Identifier;
use pumpkin_util::text::TextComponent;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum BossbarUpdateError {
    #[error("Unknown bossbar {0}")]
    UnknownBossbar(Identifier),
    #[error("No changes")]
    NoChanges(&'static str, Option<&'static str>),
}

/// Representing the stored custom boss bars from level.dat
#[derive(Clone)]
pub struct CustomBossbar {
    pub bossbar_data: Bossbar,
    pub max: i32,
    pub value: i32,
    pub visible: bool,
    pub players: Vec<Uuid>,
}

impl CustomBossbar {
    #[deny(clippy::new_without_default)]
    #[must_use]
    pub const fn new(bossbar_data: Bossbar) -> Self {
        Self {
            bossbar_data,
            max: 100,
            value: 0,
            visible: true,
            players: vec![],
        }
    }

    pub async fn update_health(
        &mut self,
        server: &Server,
        max_value: i32,
        value: i32,
    ) -> Result<(), BossbarUpdateError> {
        if self.value == value && self.max == max_value {
            return Err(BossbarUpdateError::NoChanges("value", None));
        }

        let ratio = f64::from(value) / f64::from(max_value);
        let health: f32;

        if ratio < 0.0 {
            health = 0.0;
        } else if ratio > 1.0 {
            health = 1.0;
        } else {
            health = ratio as f32;
        }

        self.value = value;
        self.max = max_value;
        self.bossbar_data.health = health;

        if self.visible {
            let players: Vec<Arc<Player>> = server.get_all_players();
            let matching_players = players
                .iter()
                .filter(|player| self.players.contains(&player.gameprofile.id));
            for player in matching_players {
                player
                    .update_bossbar_health(&self.bossbar_data.uuid, self.bossbar_data.health)
                    .await;
            }
        }

        Ok(())
    }

    pub async fn update_visibility(
        &mut self,
        server: &Server,
        new_visibility: bool,
    ) -> Result<(), BossbarUpdateError> {
        if self.visible == new_visibility && new_visibility {
            return Err(BossbarUpdateError::NoChanges("visibility", Some("visible")));
        }

        if self.visible == new_visibility && !new_visibility {
            return Err(BossbarUpdateError::NoChanges("visibility", Some("hidden")));
        }

        self.visible = new_visibility;

        let players: Vec<Arc<Player>> = server.get_all_players();
        let online_players = players
            .iter()
            .filter(|player| self.players.contains(&player.gameprofile.id));

        for player in online_players {
            if self.visible {
                player.send_bossbar(&self.bossbar_data).await;
            } else {
                player.remove_bossbar(self.bossbar_data.uuid).await;
            }
        }

        Ok(())
    }

    pub async fn update_name(
        &mut self,
        server: &Server,
        new_title: TextComponent,
    ) -> Result<(), BossbarUpdateError> {
        if self.bossbar_data.title == new_title {
            return Err(BossbarUpdateError::NoChanges("name", None));
        }

        self.bossbar_data.title = new_title;

        if !self.visible {
            return Ok(());
        }

        let players: Vec<Arc<Player>> = server.get_all_players();
        let matching_players = players
            .iter()
            .filter(|player| self.players.contains(&player.gameprofile.id));
        for player in matching_players {
            player
                .update_bossbar_title(&self.bossbar_data.uuid, self.bossbar_data.title.clone())
                .await;
        }

        Ok(())
    }

    pub async fn update_color(
        &mut self,
        server: &Server,
        new_color: BossbarColor,
    ) -> Result<(), BossbarUpdateError> {
        if self.bossbar_data.color == new_color {
            return Err(BossbarUpdateError::NoChanges("color", None));
        }

        self.bossbar_data.color = new_color;

        if self.visible {
            let players: Vec<Arc<Player>> = server.get_all_players();
            let matching_players = players
                .iter()
                .filter(|player| self.players.contains(&player.gameprofile.id));
            for player in matching_players {
                player
                    .update_bossbar_style(
                        &self.bossbar_data.uuid,
                        self.bossbar_data.color,
                        self.bossbar_data.division,
                    )
                    .await;
            }
        }

        Ok(())
    }

    pub async fn update_division(
        &mut self,
        server: &Server,
        new_division: BossbarDivisions,
    ) -> Result<(), BossbarUpdateError> {
        if self.bossbar_data.division == new_division {
            return Err(BossbarUpdateError::NoChanges("style", None));
        }

        self.bossbar_data.division = new_division;

        if self.visible {
            let players: Vec<Arc<Player>> = server.get_all_players();
            let matching_players = players
                .iter()
                .filter(|player| self.players.contains(&player.gameprofile.id));
            for player in matching_players {
                player
                    .update_bossbar_style(
                        &self.bossbar_data.uuid,
                        self.bossbar_data.color,
                        self.bossbar_data.division,
                    )
                    .await;
            }
        }

        Ok(())
    }

    pub async fn update_players(
        &mut self,
        server: &Server,
        new_players: Vec<Uuid>,
    ) -> Result<(), BossbarUpdateError> {
        // Get the difference between the old and new player list and remove bossbars from old players.
        let removed_players: Vec<Uuid> = self
            .players
            .iter()
            .filter(|item| !new_players.contains(item))
            .copied()
            .collect();

        let added_players: Vec<Uuid> = new_players
            .iter()
            .filter(|item| !self.players.contains(item))
            .copied()
            .collect();

        if removed_players.is_empty() && added_players.is_empty() {
            return Err(BossbarUpdateError::NoChanges("players", None));
        }

        if self.visible {
            for uuid in removed_players {
                let Some(player) = server.get_player_by_uuid(uuid) else {
                    continue;
                };

                player.remove_bossbar(self.bossbar_data.uuid).await;
            }
        }

        self.players = new_players;

        if self.visible {
            for uuid in added_players {
                let Some(player) = server.get_player_by_uuid(uuid) else {
                    continue;
                };

                player.send_bossbar(&self.bossbar_data).await;
            }
        }

        Ok(())
    }
}

pub struct CustomBossbars {
    pub custom_bossbars: HashMap<Identifier, CustomBossbar>,
}

impl Default for CustomBossbars {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomBossbars {
    #[must_use]
    pub fn new() -> Self {
        Self {
            custom_bossbars: HashMap::new(),
        }
    }

    #[must_use]
    pub fn get_player_bars(&self, uuid: &Uuid) -> Option<Vec<&Bossbar>> {
        let mut player_bars: Vec<&Bossbar> = Vec::new();
        for bossbar in &self.custom_bossbars {
            if bossbar.1.players.contains(uuid) {
                player_bars.push(&bossbar.1.bossbar_data);
            }
        }
        if !player_bars.is_empty() {
            return Some(player_bars);
        }
        None
    }

    pub fn create_bossbar(&mut self, identifier: Identifier, bossbar_data: Bossbar) {
        self.custom_bossbars
            .insert(identifier.clone(), CustomBossbar::new(bossbar_data));
    }

    #[must_use]
    pub fn get_all_bossbars(&self) -> Vec<(&Identifier, &CustomBossbar)> {
        self.custom_bossbars.iter().collect()
    }

    #[must_use]
    pub fn get_bossbars_len(&self) -> usize {
        self.custom_bossbars.len()
    }

    #[must_use]
    pub fn get_bossbar(&self, identifier: &Identifier) -> Option<&CustomBossbar> {
        self.custom_bossbars.get(identifier)
    }

    #[must_use]
    pub fn get_bossbar_or_err(
        &self,
        identifier: &Identifier,
    ) -> Result<&CustomBossbar, BossbarUpdateError> {
        self.custom_bossbars
            .get(identifier)
            .ok_or_else(|| BossbarUpdateError::UnknownBossbar(identifier.clone()))
    }

    #[must_use]
    pub fn get_bossbar_mut_or_err(
        &mut self,
        identifier: &Identifier,
    ) -> Result<&mut CustomBossbar, BossbarUpdateError> {
        self.custom_bossbars
            .get_mut(identifier)
            .ok_or_else(|| BossbarUpdateError::UnknownBossbar(identifier.clone()))
    }

    pub async fn remove_bossbar(
        &mut self,
        server: &Server,
        identifier: &Identifier,
    ) -> Result<CustomBossbar, BossbarUpdateError> {
        if let Some(bossbar) = self.custom_bossbars.remove(identifier) {
            let players: Vec<Arc<Player>> = server.get_all_players();

            let online_players = players
                .iter()
                .filter(|player| bossbar.players.contains(&player.gameprofile.id));

            if bossbar.visible {
                for player in online_players {
                    player.remove_bossbar(bossbar.bossbar_data.uuid).await;
                }
            }

            Ok(bossbar)
        } else {
            Err(BossbarUpdateError::UnknownBossbar(identifier.clone()))
        }
    }

    #[must_use]
    pub fn has_bossbar(&self, identifier: &Identifier) -> bool {
        self.custom_bossbars.contains_key(identifier)
    }
}

impl<'a> IntoIterator for &'a CustomBossbars {
    type Item = (&'a Identifier, &'a CustomBossbar);

    type IntoIter = std::collections::hash_map::Iter<'a, Identifier, CustomBossbar>;

    fn into_iter(self) -> Self::IntoIter {
        self.custom_bossbars.iter()
    }
}
