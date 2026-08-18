use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::{BedrockScoreboardResource, PluginHostState, ScoreboardProvider, ScoreboardResource},
    wit::v0_1::pumpkin::{
        self,
        plugin::scoreboard::{
            self, CollisionRule, DisplaySlot, HostBedrockScoreboard, NametagVisibility, RenderType,
            TeamSettings,
        },
    },
};
use crate::world::scoreboard::{ScoreboardObjective, ScoreboardScore, Team};
use pumpkin_protocol::codec::var_int::VarInt;

impl PluginHostState {
    fn get_scoreboard_res(
        &self,
        res: &Resource<scoreboard::Scoreboard>,
    ) -> wasmtime::Result<&ScoreboardResource> {
        self.resource_table
            .get::<ScoreboardResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }

    fn get_bedrock_scoreboard_res(
        &self,
        res: &Resource<scoreboard::BedrockScoreboard>,
    ) -> wasmtime::Result<&BedrockScoreboardResource> {
        self.resource_table
            .get::<BedrockScoreboardResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }
}

impl scoreboard::Host for PluginHostState {}

impl scoreboard::HostScoreboard for PluginHostState {
    async fn add_objective(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
        display_name: Resource<pumpkin::plugin::text::TextComponent>,
        render_type: RenderType,
    ) -> wasmtime::Result<()> {
        let provider = self.get_scoreboard_res(&res)?.provider.clone();
        let display_name = self.get_text_provider(&display_name)?;

        let rt = match render_type {
            RenderType::Integer => pumpkin_protocol::java::client::play::RenderType::Integer,
            RenderType::Hearts => pumpkin_protocol::java::client::play::RenderType::Hearts,
        };

        let objective = ScoreboardObjective::new(name, display_name, rt, None, "dummy");

        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .await
                    .add_objective(world.as_ref(), objective)
                    .await;
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player.custom_scoreboard.lock().await;
                if !matches!(
                    *custom_guard,
                    Some(crate::entity::player::CustomScoreboard::Java(_))
                ) {
                    *custom_guard = Some(crate::entity::player::CustomScoreboard::Java(
                        crate::world::scoreboard::Scoreboard::default(),
                    ));
                }
                if let Some(crate::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.add_objective(player.as_ref(), objective).await;
                }
            }
        }
        Ok(())
    }

    async fn update_objective(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
        display_name: Resource<pumpkin::plugin::text::TextComponent>,
        render_type: RenderType,
    ) -> wasmtime::Result<()> {
        let provider = self.get_scoreboard_res(&res)?.provider.clone();
        let display_name = self.get_text_provider(&display_name)?;

        let rt = match render_type {
            RenderType::Integer => pumpkin_protocol::java::client::play::RenderType::Integer,
            RenderType::Hearts => pumpkin_protocol::java::client::play::RenderType::Hearts,
        };

        let objective = ScoreboardObjective::new(name, display_name, rt, None, "dummy");

        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .await
                    .update_objective(world.as_ref(), objective)
                    .await;
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player.custom_scoreboard.lock().await;
                if !matches!(
                    *custom_guard,
                    Some(crate::entity::player::CustomScoreboard::Java(_))
                ) {
                    *custom_guard = Some(crate::entity::player::CustomScoreboard::Java(
                        crate::world::scoreboard::Scoreboard::default(),
                    ));
                }
                if let Some(crate::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.update_objective(player.as_ref(), objective).await;
                }
            }
        }
        Ok(())
    }

    async fn remove_objective(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
    ) -> wasmtime::Result<()> {
        let provider = self.get_scoreboard_res(&res)?.provider.clone();
        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .await
                    .remove_objective(world.as_ref(), &name)
                    .await;
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player.custom_scoreboard.lock().await;
                if let Some(crate::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.remove_objective(player.as_ref(), &name).await;
                }
            }
        }
        Ok(())
    }

    async fn set_display_slot(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        slot: DisplaySlot,
        objective_name: String,
    ) -> wasmtime::Result<()> {
        let provider = self.get_scoreboard_res(&res)?.provider.clone();
        let slot = map_display_slot(slot);

        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .await
                    .set_display_objective(world.as_ref(), slot, Some(&objective_name))
                    .await;
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player.custom_scoreboard.lock().await;
                if !matches!(
                    *custom_guard,
                    Some(crate::entity::player::CustomScoreboard::Java(_))
                ) {
                    *custom_guard = Some(crate::entity::player::CustomScoreboard::Java(
                        crate::world::scoreboard::Scoreboard::default(),
                    ));
                }
                if let Some(crate::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.set_display_objective(player.as_ref(), slot, Some(&objective_name))
                        .await;
                }
            }
        }
        Ok(())
    }

    async fn clear_display_slot(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        slot: DisplaySlot,
    ) -> wasmtime::Result<()> {
        let provider = self.get_scoreboard_res(&res)?.provider.clone();
        let slot = map_display_slot(slot);

        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .await
                    .clear_display_objective(world.as_ref(), slot)
                    .await;
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player.custom_scoreboard.lock().await;
                if let Some(crate::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.clear_display_objective(player.as_ref(), slot).await;
                }
            }
        }
        Ok(())
    }

    async fn update_score(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        entity_name: String,
        objective_name: String,
        value: i32,
    ) -> wasmtime::Result<()> {
        let provider = self.get_scoreboard_res(&res)?.provider.clone();
        let score = ScoreboardScore::new(entity_name, objective_name, VarInt(value), None, None);
        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .await
                    .update_score(world.as_ref(), score)
                    .await;
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player.custom_scoreboard.lock().await;
                if !matches!(
                    *custom_guard,
                    Some(crate::entity::player::CustomScoreboard::Java(_))
                ) {
                    *custom_guard = Some(crate::entity::player::CustomScoreboard::Java(
                        crate::world::scoreboard::Scoreboard::default(),
                    ));
                }
                if let Some(crate::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.update_score(player.as_ref(), score).await;
                }
            }
        }
        Ok(())
    }

    async fn add_score(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        entity_name: String,
        objective_name: String,
        delta: i32,
    ) -> wasmtime::Result<i32> {
        let provider = self.get_scoreboard_res(&res)?.provider.clone();
        let new_val = match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .await
                    .add_score(world.as_ref(), entity_name, objective_name, delta)
                    .await
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player.custom_scoreboard.lock().await;
                if !matches!(
                    *custom_guard,
                    Some(crate::entity::player::CustomScoreboard::Java(_))
                ) {
                    *custom_guard = Some(crate::entity::player::CustomScoreboard::Java(
                        crate::world::scoreboard::Scoreboard::default(),
                    ));
                }
                let Some(crate::entity::player::CustomScoreboard::Java(sb)) = custom_guard.as_mut()
                else {
                    return Err(wasmtime::Error::msg("Invalid scoreboard state"));
                };
                sb.add_score(player.as_ref(), entity_name, objective_name, delta)
                    .await
            }
        };
        Ok(new_val)
    }

    async fn remove_score(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        entity_name: String,
        objective_name: String,
    ) -> wasmtime::Result<()> {
        let provider = self.get_scoreboard_res(&res)?.provider.clone();
        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .await
                    .remove_score(world.as_ref(), &entity_name, &objective_name)
                    .await;
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player.custom_scoreboard.lock().await;
                if let Some(crate::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.remove_score(player.as_ref(), &entity_name, &objective_name)
                        .await;
                }
            }
        }
        Ok(())
    }

    async fn reset_entity_scores(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        entity_name: String,
    ) -> wasmtime::Result<()> {
        let provider = self.get_scoreboard_res(&res)?.provider.clone();
        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .await
                    .reset_scores_for_entity(world.as_ref(), &entity_name)
                    .await;
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player.custom_scoreboard.lock().await;
                if let Some(crate::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.reset_scores_for_entity(player.as_ref(), &entity_name)
                        .await;
                }
            }
        }
        Ok(())
    }

    async fn create_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
        settings: TeamSettings,
    ) -> wasmtime::Result<()> {
        let provider = self.get_scoreboard_res(&res)?.provider.clone();
        let team = map_team_settings(name, &settings, self)?;
        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .await
                    .add_team(world.as_ref(), team)
                    .await;
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player.custom_scoreboard.lock().await;
                if !matches!(
                    *custom_guard,
                    Some(crate::entity::player::CustomScoreboard::Java(_))
                ) {
                    *custom_guard = Some(crate::entity::player::CustomScoreboard::Java(
                        crate::world::scoreboard::Scoreboard::default(),
                    ));
                }
                if let Some(crate::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.add_team(player.as_ref(), team).await;
                }
            }
        }
        Ok(())
    }

    async fn remove_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
    ) -> wasmtime::Result<()> {
        let provider = self.get_scoreboard_res(&res)?.provider.clone();
        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .await
                    .remove_team(world.as_ref(), &name)
                    .await;
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player.custom_scoreboard.lock().await;
                if let Some(crate::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.remove_team(player.as_ref(), &name).await;
                }
            }
        }
        Ok(())
    }

    async fn update_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
        settings: TeamSettings,
    ) -> wasmtime::Result<()> {
        let provider = self.get_scoreboard_res(&res)?.provider.clone();
        let team = map_team_settings(name, &settings, self)?;
        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .await
                    .update_team(world.as_ref(), team)
                    .await;
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player.custom_scoreboard.lock().await;
                if let Some(crate::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.update_team(player.as_ref(), team).await;
                }
            }
        }
        Ok(())
    }

    async fn add_player_to_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        team_name: String,
        player_name: String,
    ) -> wasmtime::Result<()> {
        let provider = self.get_scoreboard_res(&res)?.provider.clone();
        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .await
                    .add_player_to_team(world.as_ref(), &team_name, player_name)
                    .await;
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player.custom_scoreboard.lock().await;
                if let Some(crate::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.add_player_to_team(player.as_ref(), &team_name, player_name)
                        .await;
                }
            }
        }
        Ok(())
    }

    async fn remove_player_from_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        team_name: String,
        player_name: String,
    ) -> wasmtime::Result<()> {
        let provider = self.get_scoreboard_res(&res)?.provider.clone();
        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .await
                    .remove_player_from_team(world.as_ref(), &team_name, &player_name)
                    .await;
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player.custom_scoreboard.lock().await;
                if let Some(crate::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.remove_player_from_team(player.as_ref(), &team_name, &player_name)
                        .await;
                }
            }
        }
        Ok(())
    }

    async fn clear_team_players(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        team_name: String,
    ) -> wasmtime::Result<()> {
        let provider = self.get_scoreboard_res(&res)?.provider.clone();
        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .await
                    .clear_team_players(world.as_ref(), &team_name)
                    .await;
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player.custom_scoreboard.lock().await;
                if let Some(crate::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.clear_team_players(player.as_ref(), &team_name).await;
                }
            }
        }
        Ok(())
    }

    async fn get_teams(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
    ) -> wasmtime::Result<Vec<String>> {
        let provider = self.get_scoreboard_res(&res)?.provider.clone();
        let teams = match provider {
            ScoreboardProvider::World(world) => world
                .scoreboard
                .lock()
                .await
                .get_teams()
                .keys()
                .cloned()
                .collect(),
            ScoreboardProvider::Player(player) => {
                let custom_guard = player.custom_scoreboard.lock().await;
                if let Some(crate::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_ref()
                {
                    sb.get_teams().keys().cloned().collect()
                } else {
                    Vec::new()
                }
            }
        };
        Ok(teams)
    }

    async fn get_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
    ) -> wasmtime::Result<Option<TeamSettings>> {
        let provider = self.get_scoreboard_res(&res)?.provider.clone();
        let team_opt = match provider {
            ScoreboardProvider::World(world) => {
                world.scoreboard.lock().await.get_team(&name).cloned()
            }
            ScoreboardProvider::Player(player) => {
                let custom_guard = player.custom_scoreboard.lock().await;
                if let Some(crate::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_ref()
                {
                    sb.get_team(&name).cloned()
                } else {
                    None
                }
            }
        };

        if let Some(team) = team_opt {
            Ok(Some(map_team_to_settings(&team, self)?))
        } else {
            Ok(None)
        }
    }

    async fn get_team_players(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        team_name: String,
    ) -> wasmtime::Result<Vec<String>> {
        let provider = self.get_scoreboard_res(&res)?.provider.clone();
        let players = match provider {
            ScoreboardProvider::World(world) => world
                .scoreboard
                .lock()
                .await
                .get_team(&team_name)
                .map(|t| t.players.clone())
                .unwrap_or_default(),
            ScoreboardProvider::Player(player) => {
                let custom_guard = player.custom_scoreboard.lock().await;
                if let Some(crate::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_ref()
                {
                    sb.get_team(&team_name)
                        .map(|t| t.players.clone())
                        .unwrap_or_default()
                } else {
                    Vec::new()
                }
            }
        };
        Ok(players)
    }

    async fn get_player_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        player_name: String,
    ) -> wasmtime::Result<Option<String>> {
        let provider = self.get_scoreboard_res(&res)?.provider.clone();
        let team_name = match provider {
            ScoreboardProvider::World(world) => world
                .scoreboard
                .lock()
                .await
                .get_entity_team(&player_name)
                .map(|t| t.name.clone()),
            ScoreboardProvider::Player(player) => {
                let custom_guard = player.custom_scoreboard.lock().await;
                if let Some(crate::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_ref()
                {
                    sb.get_entity_team(&player_name).map(|t| t.name.clone())
                } else {
                    None
                }
            }
        };
        Ok(team_name)
    }

    async fn drop(&mut self, rep: Resource<scoreboard::Scoreboard>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<ScoreboardResource>(Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}

const fn map_display_slot(slot: DisplaySlot) -> pumpkin_data::scoreboard::ScoreboardDisplaySlot {
    match slot {
        DisplaySlot::PlayerList => pumpkin_data::scoreboard::ScoreboardDisplaySlot::List,
        DisplaySlot::Sidebar => pumpkin_data::scoreboard::ScoreboardDisplaySlot::Sidebar,
        DisplaySlot::BelowName => pumpkin_data::scoreboard::ScoreboardDisplaySlot::BelowName,
        DisplaySlot::SidebarTeamBlack => pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamBlack,
        DisplaySlot::SidebarTeamDarkBlue => {
            pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamDarkBlue
        }
        DisplaySlot::SidebarTeamDarkGreen => {
            pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamDarkGreen
        }
        DisplaySlot::SidebarTeamDarkAqua => {
            pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamDarkAqua
        }
        DisplaySlot::SidebarTeamDarkRed => {
            pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamDarkRed
        }
        DisplaySlot::SidebarTeamDarkPurple => {
            pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamDarkPurple
        }
        DisplaySlot::SidebarTeamGold => pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamGold,
        DisplaySlot::SidebarTeamGray => pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamGray,
        DisplaySlot::SidebarTeamDarkGray => {
            pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamDarkGray
        }
        DisplaySlot::SidebarTeamBlue => pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamBlue,
        DisplaySlot::SidebarTeamGreen => pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamGreen,
        DisplaySlot::SidebarTeamAqua => pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamAqua,
        DisplaySlot::SidebarTeamRed => pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamRed,
        DisplaySlot::SidebarTeamLightPurple => {
            pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamLightPurple
        }
        DisplaySlot::SidebarTeamYellow => {
            pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamYellow
        }
        DisplaySlot::SidebarTeamWhite => pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamWhite,
    }
}

fn map_team_settings(
    name: String,
    settings: &TeamSettings,
    state: &PluginHostState,
) -> wasmtime::Result<Team> {
    let display_name = state.get_text_provider(&settings.display_name)?;
    let player_prefix = state.get_text_provider(&settings.prefix)?;
    let player_suffix = state.get_text_provider(&settings.suffix)?;

    let mut options = 0;
    if settings.friendly_fire {
        options |= 0x01;
    }
    if settings.see_friendly_invisibles {
        options |= 0x02;
    }

    Ok(Team {
        name,
        display_name,
        options,
        nametag_visibility: match settings.nametag_visibility {
            NametagVisibility::Always => crate::world::scoreboard::NameTagVisibility::Always,
            NametagVisibility::Never => crate::world::scoreboard::NameTagVisibility::Never,
            NametagVisibility::HideForOtherTeams => {
                crate::world::scoreboard::NameTagVisibility::HideForOtherTeams
            }
            NametagVisibility::HideForOwnTeam => {
                crate::world::scoreboard::NameTagVisibility::HideForOwnTeam
            }
        },
        collision_rule: match settings.collision_rule {
            CollisionRule::Always => crate::world::scoreboard::CollisionRule::Always,
            CollisionRule::Never => crate::world::scoreboard::CollisionRule::Never,
            CollisionRule::PushOtherTeams => {
                crate::world::scoreboard::CollisionRule::PushOtherTeams
            }
            CollisionRule::PushOwnTeam => crate::world::scoreboard::CollisionRule::PushOwnTeam,
        },
        color: map_named_color(settings.color),
        player_prefix,
        player_suffix,
        players: Vec::new(),
    })
}

const fn map_named_color(
    color: pumpkin::plugin::common::NamedColor,
) -> pumpkin_util::text::color::NamedColor {
    match color {
        pumpkin::plugin::common::NamedColor::Black => pumpkin_util::text::color::NamedColor::Black,
        pumpkin::plugin::common::NamedColor::DarkBlue => {
            pumpkin_util::text::color::NamedColor::DarkBlue
        }
        pumpkin::plugin::common::NamedColor::DarkGreen => {
            pumpkin_util::text::color::NamedColor::DarkGreen
        }
        pumpkin::plugin::common::NamedColor::DarkAqua => {
            pumpkin_util::text::color::NamedColor::DarkAqua
        }
        pumpkin::plugin::common::NamedColor::DarkRed => {
            pumpkin_util::text::color::NamedColor::DarkRed
        }
        pumpkin::plugin::common::NamedColor::DarkPurple => {
            pumpkin_util::text::color::NamedColor::DarkPurple
        }
        pumpkin::plugin::common::NamedColor::Gold => pumpkin_util::text::color::NamedColor::Gold,
        pumpkin::plugin::common::NamedColor::Gray => pumpkin_util::text::color::NamedColor::Gray,
        pumpkin::plugin::common::NamedColor::DarkGray => {
            pumpkin_util::text::color::NamedColor::DarkGray
        }
        pumpkin::plugin::common::NamedColor::Blue => pumpkin_util::text::color::NamedColor::Blue,
        pumpkin::plugin::common::NamedColor::Green => pumpkin_util::text::color::NamedColor::Green,
        pumpkin::plugin::common::NamedColor::Aqua => pumpkin_util::text::color::NamedColor::Aqua,
        pumpkin::plugin::common::NamedColor::Red => pumpkin_util::text::color::NamedColor::Red,
        pumpkin::plugin::common::NamedColor::LightPurple => {
            pumpkin_util::text::color::NamedColor::LightPurple
        }
        pumpkin::plugin::common::NamedColor::Yellow => {
            pumpkin_util::text::color::NamedColor::Yellow
        }
        pumpkin::plugin::common::NamedColor::White => pumpkin_util::text::color::NamedColor::White,
    }
}

fn map_team_to_settings(
    team: &Team,
    state: &mut PluginHostState,
) -> wasmtime::Result<TeamSettings> {
    let display_name = state.add_text_component(team.display_name.clone())?;
    let prefix = state.add_text_component(team.player_prefix.clone())?;
    let suffix = state.add_text_component(team.player_suffix.clone())?;

    let friendly_fire = (team.options & 0x01) != 0;
    let see_friendly_invisibles = (team.options & 0x02) != 0;

    let nametag_visibility = match team.nametag_visibility {
        crate::world::scoreboard::NameTagVisibility::Always => NametagVisibility::Always,
        crate::world::scoreboard::NameTagVisibility::Never => NametagVisibility::Never,
        crate::world::scoreboard::NameTagVisibility::HideForOtherTeams => {
            NametagVisibility::HideForOtherTeams
        }
        crate::world::scoreboard::NameTagVisibility::HideForOwnTeam => {
            NametagVisibility::HideForOwnTeam
        }
    };

    let collision_rule = match team.collision_rule {
        crate::world::scoreboard::CollisionRule::Always => CollisionRule::Always,
        crate::world::scoreboard::CollisionRule::Never => CollisionRule::Never,
        crate::world::scoreboard::CollisionRule::PushOtherTeams => CollisionRule::PushOtherTeams,
        crate::world::scoreboard::CollisionRule::PushOwnTeam => CollisionRule::PushOwnTeam,
    };

    let color = map_named_color_rev(team.color);

    Ok(TeamSettings {
        display_name,
        friendly_fire,
        see_friendly_invisibles,
        nametag_visibility,
        collision_rule,
        color,
        prefix,
        suffix,
    })
}

const fn map_named_color_rev(
    color: pumpkin_util::text::color::NamedColor,
) -> pumpkin::plugin::common::NamedColor {
    match color {
        pumpkin_util::text::color::NamedColor::Black => pumpkin::plugin::common::NamedColor::Black,
        pumpkin_util::text::color::NamedColor::DarkBlue => {
            pumpkin::plugin::common::NamedColor::DarkBlue
        }
        pumpkin_util::text::color::NamedColor::DarkGreen => {
            pumpkin::plugin::common::NamedColor::DarkGreen
        }
        pumpkin_util::text::color::NamedColor::DarkAqua => {
            pumpkin::plugin::common::NamedColor::DarkAqua
        }
        pumpkin_util::text::color::NamedColor::DarkRed => {
            pumpkin::plugin::common::NamedColor::DarkRed
        }
        pumpkin_util::text::color::NamedColor::DarkPurple => {
            pumpkin::plugin::common::NamedColor::DarkPurple
        }
        pumpkin_util::text::color::NamedColor::Gold => pumpkin::plugin::common::NamedColor::Gold,
        pumpkin_util::text::color::NamedColor::Gray => pumpkin::plugin::common::NamedColor::Gray,
        pumpkin_util::text::color::NamedColor::DarkGray => {
            pumpkin::plugin::common::NamedColor::DarkGray
        }
        pumpkin_util::text::color::NamedColor::Blue => pumpkin::plugin::common::NamedColor::Blue,
        pumpkin_util::text::color::NamedColor::Green => pumpkin::plugin::common::NamedColor::Green,
        pumpkin_util::text::color::NamedColor::Aqua => pumpkin::plugin::common::NamedColor::Aqua,
        pumpkin_util::text::color::NamedColor::Red => pumpkin::plugin::common::NamedColor::Red,
        pumpkin_util::text::color::NamedColor::LightPurple => {
            pumpkin::plugin::common::NamedColor::LightPurple
        }
        pumpkin_util::text::color::NamedColor::Yellow => {
            pumpkin::plugin::common::NamedColor::Yellow
        }
        pumpkin_util::text::color::NamedColor::White => pumpkin::plugin::common::NamedColor::White,
    }
}

impl HostBedrockScoreboard for PluginHostState {
    async fn add_objective(
        &mut self,
        res: Resource<scoreboard::BedrockScoreboard>,
        name: String,
        display_name: String,
        sort_order: scoreboard::BedrockSortOrder,
    ) -> wasmtime::Result<()> {
        let player = self.get_bedrock_scoreboard_res(&res)?.provider.clone();
        let mut custom_guard = player.custom_scoreboard.lock().await;
        if !matches!(
            *custom_guard,
            Some(crate::entity::player::CustomScoreboard::Bedrock(_))
        ) {
            *custom_guard = Some(crate::entity::player::CustomScoreboard::Bedrock(
                crate::world::scoreboard::BedrockScoreboard::default(),
            ));
        }
        let Some(crate::entity::player::CustomScoreboard::Bedrock(sb)) = custom_guard.as_mut()
        else {
            return Err(wasmtime::Error::msg("Invalid scoreboard state"));
        };
        sb.add_objective(
            player.as_ref(),
            crate::world::scoreboard::BedrockObjective {
                name,
                display_name,
                sort_order: match sort_order {
                    scoreboard::BedrockSortOrder::Ascending => {
                        crate::world::scoreboard::BedrockSortOrder::Ascending
                    }
                    scoreboard::BedrockSortOrder::Descending => {
                        crate::world::scoreboard::BedrockSortOrder::Descending
                    }
                },
            },
        )
        .await;
        Ok(())
    }

    async fn update_objective(
        &mut self,
        res: Resource<scoreboard::BedrockScoreboard>,
        name: String,
        display_name: String,
        sort_order: scoreboard::BedrockSortOrder,
    ) -> wasmtime::Result<()> {
        let player = self.get_bedrock_scoreboard_res(&res)?.provider.clone();
        let mut custom_guard = player.custom_scoreboard.lock().await;
        if !matches!(
            *custom_guard,
            Some(crate::entity::player::CustomScoreboard::Bedrock(_))
        ) {
            *custom_guard = Some(crate::entity::player::CustomScoreboard::Bedrock(
                crate::world::scoreboard::BedrockScoreboard::default(),
            ));
        }
        let Some(crate::entity::player::CustomScoreboard::Bedrock(sb)) = custom_guard.as_mut()
        else {
            return Err(wasmtime::Error::msg("Invalid scoreboard state"));
        };
        sb.update_objective(
            player.as_ref(),
            crate::world::scoreboard::BedrockObjective {
                name,
                display_name,
                sort_order: match sort_order {
                    scoreboard::BedrockSortOrder::Ascending => {
                        crate::world::scoreboard::BedrockSortOrder::Ascending
                    }
                    scoreboard::BedrockSortOrder::Descending => {
                        crate::world::scoreboard::BedrockSortOrder::Descending
                    }
                },
            },
        )
        .await;
        Ok(())
    }

    async fn remove_objective(
        &mut self,
        res: Resource<scoreboard::BedrockScoreboard>,
        name: String,
    ) -> wasmtime::Result<()> {
        let player = self.get_bedrock_scoreboard_res(&res)?.provider.clone();
        let mut custom_guard = player.custom_scoreboard.lock().await;
        if let Some(crate::entity::player::CustomScoreboard::Bedrock(sb)) = custom_guard.as_mut() {
            sb.remove_objective(player.as_ref(), &name).await;
        }
        Ok(())
    }

    async fn set_display_slot(
        &mut self,
        res: Resource<scoreboard::BedrockScoreboard>,
        slot: scoreboard::BedrockDisplaySlot,
        objective_name: String,
    ) -> wasmtime::Result<()> {
        let player = self.get_bedrock_scoreboard_res(&res)?.provider.clone();
        let mut custom_guard = player.custom_scoreboard.lock().await;
        if !matches!(
            *custom_guard,
            Some(crate::entity::player::CustomScoreboard::Bedrock(_))
        ) {
            *custom_guard = Some(crate::entity::player::CustomScoreboard::Bedrock(
                crate::world::scoreboard::BedrockScoreboard::default(),
            ));
        }
        let Some(crate::entity::player::CustomScoreboard::Bedrock(sb)) = custom_guard.as_mut()
        else {
            return Err(wasmtime::Error::msg("Invalid scoreboard state"));
        };
        let b_slot = match slot {
            scoreboard::BedrockDisplaySlot::PlayerList => {
                crate::world::scoreboard::BedrockDisplaySlot::PlayerList
            }
            scoreboard::BedrockDisplaySlot::Sidebar => {
                crate::world::scoreboard::BedrockDisplaySlot::Sidebar
            }
            scoreboard::BedrockDisplaySlot::BelowName => {
                crate::world::scoreboard::BedrockDisplaySlot::BelowName
            }
        };
        sb.set_display_objective(player.as_ref(), b_slot, Some(&objective_name))
            .await;
        Ok(())
    }

    async fn clear_display_slot(
        &mut self,
        res: Resource<scoreboard::BedrockScoreboard>,
        slot: scoreboard::BedrockDisplaySlot,
    ) -> wasmtime::Result<()> {
        let player = self.get_bedrock_scoreboard_res(&res)?.provider.clone();
        let mut custom_guard = player.custom_scoreboard.lock().await;
        if let Some(crate::entity::player::CustomScoreboard::Bedrock(sb)) = custom_guard.as_mut() {
            let b_slot = match slot {
                scoreboard::BedrockDisplaySlot::PlayerList => {
                    crate::world::scoreboard::BedrockDisplaySlot::PlayerList
                }
                scoreboard::BedrockDisplaySlot::Sidebar => {
                    crate::world::scoreboard::BedrockDisplaySlot::Sidebar
                }
                scoreboard::BedrockDisplaySlot::BelowName => {
                    crate::world::scoreboard::BedrockDisplaySlot::BelowName
                }
            };
            sb.clear_display_objective(player.as_ref(), b_slot).await;
        }
        Ok(())
    }

    async fn update_score(
        &mut self,
        res: Resource<scoreboard::BedrockScoreboard>,
        entity_name: String,
        objective_name: String,
        value: i32,
    ) -> wasmtime::Result<()> {
        let player = self.get_bedrock_scoreboard_res(&res)?.provider.clone();
        let mut custom_guard = player.custom_scoreboard.lock().await;
        if !matches!(
            *custom_guard,
            Some(crate::entity::player::CustomScoreboard::Bedrock(_))
        ) {
            *custom_guard = Some(crate::entity::player::CustomScoreboard::Bedrock(
                crate::world::scoreboard::BedrockScoreboard::default(),
            ));
        }
        let Some(crate::entity::player::CustomScoreboard::Bedrock(sb)) = custom_guard.as_mut()
        else {
            return Err(wasmtime::Error::msg("Invalid scoreboard state"));
        };
        sb.update_score(player.as_ref(), &entity_name, &objective_name, value)
            .await;
        Ok(())
    }

    async fn add_score(
        &mut self,
        res: Resource<scoreboard::BedrockScoreboard>,
        entity_name: String,
        objective_name: String,
        delta: i32,
    ) -> wasmtime::Result<i32> {
        let player = self.get_bedrock_scoreboard_res(&res)?.provider.clone();
        let mut custom_guard = player.custom_scoreboard.lock().await;
        if !matches!(
            *custom_guard,
            Some(crate::entity::player::CustomScoreboard::Bedrock(_))
        ) {
            *custom_guard = Some(crate::entity::player::CustomScoreboard::Bedrock(
                crate::world::scoreboard::BedrockScoreboard::default(),
            ));
        }
        let Some(crate::entity::player::CustomScoreboard::Bedrock(sb)) = custom_guard.as_mut()
        else {
            return Err(wasmtime::Error::msg("Invalid scoreboard state"));
        };
        let new_val = sb
            .add_score(player.as_ref(), entity_name, objective_name, delta)
            .await;
        Ok(new_val)
    }

    async fn remove_score(
        &mut self,
        res: Resource<scoreboard::BedrockScoreboard>,
        entity_name: String,
        objective_name: String,
    ) -> wasmtime::Result<()> {
        let player = self.get_bedrock_scoreboard_res(&res)?.provider.clone();
        let mut custom_guard = player.custom_scoreboard.lock().await;
        if let Some(crate::entity::player::CustomScoreboard::Bedrock(sb)) = custom_guard.as_mut() {
            sb.remove_score(player.as_ref(), &entity_name, &objective_name)
                .await;
        }
        Ok(())
    }

    async fn reset_entity_scores(
        &mut self,
        res: Resource<scoreboard::BedrockScoreboard>,
        entity_name: String,
    ) -> wasmtime::Result<()> {
        let player = self.get_bedrock_scoreboard_res(&res)?.provider.clone();
        let mut custom_guard = player.custom_scoreboard.lock().await;
        if let Some(crate::entity::player::CustomScoreboard::Bedrock(sb)) = custom_guard.as_mut() {
            sb.reset_scores_for_entity(player.as_ref(), &entity_name)
                .await;
        }
        Ok(())
    }

    async fn drop(
        &mut self,
        _res: Resource<scoreboard::BedrockScoreboard>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }
}
