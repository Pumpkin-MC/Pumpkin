use crate::entity::player::Player;
use crate::server::Server;
use crate::world::bossbar::{Bossbar, BossbarColor, BossbarDivisions};
use pumpkin_util::text::TextComponent;
use rustc_hash::FxHashMap;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum BossbarUpdateError {
    #[error("Invalid resource location")]
    InvalidResourceLocation(String),
    #[error("No changes")]
    NoChanges(&'static str, Option<&'static str>),
}

/// Representing the stored custom boss bars from level.dat
#[derive(Clone)]
pub struct CustomBossbar {
    pub namespace: String,
    pub bossbar_data: Bossbar,
    pub max: i32,
    pub value: i32,
    pub visible: bool,
    pub players: Vec<Uuid>,
}

impl CustomBossbar {
    #[deny(clippy::new_without_default)]
    #[must_use]
    pub const fn new(namespace: String, bossbar_data: Bossbar) -> Self {
        Self {
            namespace,
            bossbar_data,
            max: 100,
            value: 0,
            visible: true,
            players: vec![],
        }
    }
}

pub struct CustomBossbars {
    pub custom_bossbars: FxHashMap<String, CustomBossbar>,
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
            custom_bossbars: FxHashMap::default(),
        }
    }

    #[must_use]
    pub fn get_player_bars(&self, uuid: &Uuid) -> Option<Vec<&Bossbar>> {
        let mut player_bars: Vec<&Bossbar> = Vec::new();
        for bossbar in &self.custom_bossbars {
            if bossbar.1.visible && bossbar.1.players.contains(uuid) {
                player_bars.push(&bossbar.1.bossbar_data);
            }
        }
        if !player_bars.is_empty() {
            return Some(player_bars);
        }
        None
    }

    pub fn create_bossbar(&mut self, namespace: String, bossbar_data: Bossbar) {
        self.custom_bossbars.insert(
            namespace.clone(),
            CustomBossbar::new(namespace, bossbar_data),
        );
    }

    pub fn replace_bossbar(&mut self, resource_location: String, bossbar_data: CustomBossbar) {
        self.custom_bossbars.insert(resource_location, bossbar_data);
    }

    #[must_use]
    pub fn get_all_bossbars(&self) -> Vec<CustomBossbar> {
        let mut bossbars: Vec<CustomBossbar> = Vec::new();
        for bossbar in self.custom_bossbars.clone() {
            bossbars.push(bossbar.1);
        }
        bossbars
    }

    #[must_use]
    pub fn get_bossbars_len(&self) -> usize {
        self.custom_bossbars.len()
    }

    #[must_use]
    pub fn get_bossbar(&self, resource_location: &str) -> Option<CustomBossbar> {
        let bossbar = self.custom_bossbars.get(resource_location);
        if let Some(bossbar) = bossbar {
            return Some(bossbar.clone());
        }
        None
    }

    pub fn remove_bossbar(
        &mut self,
        server: &Server,
        resource_location: String,
    ) -> Result<(), BossbarUpdateError> {
        let bossbar = self.custom_bossbars.get(&resource_location).cloned();
        if let Some(bossbar) = bossbar {
            self.custom_bossbars.remove(&resource_location);

            let players: Vec<Arc<Player>> = server.get_all_players();

            let online_players = players
                .iter()
                .filter(|player| bossbar.players.contains(&player.gameprofile.id));

            if bossbar.visible {
                for player in online_players {
                    player.remove_bossbar(bossbar.bossbar_data.uuid);
                }
            }

            return Ok(());
        }
        Err(BossbarUpdateError::InvalidResourceLocation(
            resource_location,
        ))
    }

    #[must_use]
    pub fn has_bossbar(&self, resource_location: &str) -> bool {
        self.custom_bossbars.contains_key(resource_location)
    }

    pub fn update_value(
        &mut self,
        server: &Server,
        resource_location: String,
        value: i32,
    ) -> Result<(), BossbarUpdateError> {
        let bossbar = self.custom_bossbars.get_mut(&resource_location);
        if let Some(bossbar) = bossbar {
            if bossbar.value == value {
                return Err(BossbarUpdateError::NoChanges("value", None));
            }

            let ratio = f64::from(value) / f64::from(bossbar.max);
            let health: f32 = if ratio < 0.0 {
                0.0
            } else if ratio > 1.0 {
                1.0
            } else {
                ratio as f32
            };

            bossbar.value = value;
            bossbar.bossbar_data.health = health;

            if !bossbar.visible {
                return Ok(());
            }

            let players: Vec<Arc<Player>> = server.get_all_players();
            let matching_players = players
                .iter()
                .filter(|player| bossbar.players.contains(&player.gameprofile.id));
            for player in matching_players {
                player
                    .update_bossbar_health(&bossbar.bossbar_data.uuid, bossbar.bossbar_data.health);
            }

            return Ok(());
        }
        Err(BossbarUpdateError::InvalidResourceLocation(
            resource_location,
        ))
    }

    pub fn update_max(
        &mut self,
        server: &Server,
        resource_location: String,
        max_value: i32,
    ) -> Result<(), BossbarUpdateError> {
        let bossbar = self.custom_bossbars.get_mut(&resource_location);
        if let Some(bossbar) = bossbar {
            if bossbar.max == max_value {
                return Err(BossbarUpdateError::NoChanges("max", None));
            }

            let ratio = f64::from(bossbar.value) / f64::from(max_value);
            let health: f32 = if ratio < 0.0 {
                0.0
            } else if ratio > 1.0 {
                1.0
            } else {
                ratio as f32
            };

            bossbar.max = max_value;
            bossbar.bossbar_data.health = health;

            if !bossbar.visible {
                return Ok(());
            }

            let players: Vec<Arc<Player>> = server.get_all_players();
            let matching_players = players
                .iter()
                .filter(|player| bossbar.players.contains(&player.gameprofile.id));
            for player in matching_players {
                player
                    .update_bossbar_health(&bossbar.bossbar_data.uuid, bossbar.bossbar_data.health);
            }

            return Ok(());
        }
        Err(BossbarUpdateError::InvalidResourceLocation(
            resource_location,
        ))
    }

    pub fn update_health(
        &mut self,
        server: &Server,
        resource_location: String,
        max_value: i32,
        value: i32,
    ) -> Result<(), BossbarUpdateError> {
        let bossbar = self.custom_bossbars.get_mut(&resource_location);
        if let Some(bossbar) = bossbar {
            if bossbar.value == value && bossbar.max == max_value {
                return Err(BossbarUpdateError::NoChanges("value", None));
            }

            let ratio = f64::from(value) / f64::from(max_value);

            let health: f32 = if ratio < 0.0 {
                0.0
            } else if ratio > 1.0 {
                1.0
            } else {
                ratio as f32
            };

            bossbar.value = value;
            bossbar.max = max_value;
            bossbar.bossbar_data.health = health;

            if !bossbar.visible {
                return Ok(());
            }

            let players: Vec<Arc<Player>> = server.get_all_players();
            let matching_players = players
                .iter()
                .filter(|player| bossbar.players.contains(&player.gameprofile.id));
            for player in matching_players {
                player
                    .update_bossbar_health(&bossbar.bossbar_data.uuid, bossbar.bossbar_data.health);
            }

            return Ok(());
        }
        Err(BossbarUpdateError::InvalidResourceLocation(
            resource_location,
        ))
    }

    pub fn update_visibility(
        &mut self,
        server: &Server,
        resource_location: String,
        new_visibility: bool,
    ) -> Result<(), BossbarUpdateError> {
        let bossbar = self.custom_bossbars.get_mut(&resource_location);
        if let Some(bossbar) = bossbar {
            if bossbar.visible == new_visibility && new_visibility {
                return Err(BossbarUpdateError::NoChanges("visibility", Some("visible")));
            }

            if bossbar.visible == new_visibility && !new_visibility {
                return Err(BossbarUpdateError::NoChanges("visibility", Some("hidden")));
            }

            bossbar.visible = new_visibility;

            let players: Vec<Arc<Player>> = server.get_all_players();
            let online_players = players
                .iter()
                .filter(|player| bossbar.players.contains(&player.gameprofile.id));

            for player in online_players {
                if bossbar.visible {
                    player.send_bossbar(&bossbar.bossbar_data);
                } else {
                    player.remove_bossbar(bossbar.bossbar_data.uuid);
                }
            }

            return Ok(());
        }
        Err(BossbarUpdateError::InvalidResourceLocation(
            resource_location,
        ))
    }

    pub fn update_name(
        &mut self,
        server: &Server,
        resource_location: &str,
        new_title: &TextComponent,
    ) -> Result<(), BossbarUpdateError> {
        let bossbar = self.custom_bossbars.get_mut(resource_location);
        if let Some(bossbar) = bossbar {
            bossbar.bossbar_data.title = new_title.clone();

            let players: Vec<Arc<Player>> = server.get_all_players();
            let online_players = players
                .iter()
                .filter(|player| bossbar.players.contains(&player.gameprofile.id));

            if bossbar.visible {
                for player in online_players {
                    player.update_bossbar_title(&bossbar.bossbar_data.uuid, new_title.clone());
                }
            }

            return Ok(());
        }
        Err(BossbarUpdateError::InvalidResourceLocation(
            resource_location.to_string(),
        ))
    }

    pub fn update_color(
        &mut self,
        server: &Server,
        resource_location: &str,
        new_color: BossbarColor,
    ) -> Result<(), BossbarUpdateError> {
        let bossbar = self.custom_bossbars.get_mut(resource_location);
        if let Some(bossbar) = bossbar {
            bossbar.bossbar_data.color = new_color;

            let players: Vec<Arc<Player>> = server.get_all_players();
            let online_players = players
                .iter()
                .filter(|player| bossbar.players.contains(&player.gameprofile.id));

            if bossbar.visible {
                for player in online_players {
                    player.update_bossbar_style(
                        &bossbar.bossbar_data.uuid,
                        new_color,
                        bossbar.bossbar_data.division,
                    );
                }
            }

            return Ok(());
        }
        Err(BossbarUpdateError::InvalidResourceLocation(
            resource_location.to_string(),
        ))
    }

    pub fn update_style(
        &mut self,
        server: &Server,
        resource_location: &str,
        new_style: BossbarDivisions,
    ) -> Result<(), BossbarUpdateError> {
        let bossbar = self.custom_bossbars.get_mut(resource_location);
        if let Some(bossbar) = bossbar {
            bossbar.bossbar_data.division = new_style;

            let players: Vec<Arc<Player>> = server.get_all_players();
            let online_players = players
                .iter()
                .filter(|player| bossbar.players.contains(&player.gameprofile.id));

            if bossbar.visible {
                for player in online_players {
                    player.update_bossbar_style(
                        &bossbar.bossbar_data.uuid,
                        bossbar.bossbar_data.color,
                        new_style,
                    );
                }
            }

            return Ok(());
        }
        Err(BossbarUpdateError::InvalidResourceLocation(
            resource_location.to_string(),
        ))
    }

    pub fn set_players(
        &mut self,
        server: &Server,
        resource_location: String,
        new_players: Vec<Uuid>,
    ) -> Result<(), BossbarUpdateError> {
        let bossbar = self.custom_bossbars.get_mut(&resource_location);
        if let Some(bossbar) = bossbar {
            // Get the difference between the old and new player list and remove bossbars from old players.
            let removed_players: Vec<Uuid> = bossbar
                .players
                .iter()
                .filter(|item| !new_players.contains(item))
                .copied()
                .collect();

            let added_players: Vec<Uuid> = new_players
                .iter()
                .filter(|item| !bossbar.players.contains(item))
                .copied()
                .collect();

            if removed_players.is_empty() && added_players.is_empty() {
                return Err(BossbarUpdateError::NoChanges("players", None));
            }

            if bossbar.visible {
                for uuid in removed_players {
                    let Some(player) = server.get_player_by_uuid(uuid) else {
                        continue;
                    };

                    player.remove_bossbar(bossbar.bossbar_data.uuid);
                }
            }

            bossbar.players = new_players;

            if !bossbar.visible {
                return Ok(());
            }

            for uuid in added_players {
                let Some(player) = server.get_player_by_uuid(uuid) else {
                    continue;
                };

                player.send_bossbar(&bossbar.bossbar_data);
            }

            return Ok(());
        }
        Err(BossbarUpdateError::InvalidResourceLocation(
            resource_location,
        ))
    }
}
