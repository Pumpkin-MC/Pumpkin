use wasmtime::component::Resource;

use std::sync::Arc;

use crate::plugin::loader::wasm::wasm_host::{
    state::{PluginHostState, ScoreboardResource},
    wit::v0_1::pumpkin::{
        self,
        plugin::scoreboard::{
            self, CollisionRule, DisplaySlot, NametagVisibility, RenderType as WitRenderType,
            TeamSettings,
        },
    },
};
use crate::world::scoreboard::{ScoreboardObjective, ScoreboardScore, Team};
use pumpkin_data::scoreboard::ScoreboardDisplaySlot;
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

    const fn to_wit_render_type(
        rt: pumpkin_protocol::java::client::play::RenderType,
    ) -> WitRenderType {
        match rt {
            pumpkin_protocol::java::client::play::RenderType::Integer => WitRenderType::Integer,
            pumpkin_protocol::java::client::play::RenderType::Hearts => WitRenderType::Hearts,
        }
    }

    const fn to_internal_render_type(
        rt: WitRenderType,
    ) -> pumpkin_protocol::java::client::play::RenderType {
        match rt {
            WitRenderType::Integer => pumpkin_protocol::java::client::play::RenderType::Integer,
            WitRenderType::Hearts => pumpkin_protocol::java::client::play::RenderType::Hearts,
        }
    }

    const fn to_internal_display_slot(slot: DisplaySlot) -> ScoreboardDisplaySlot {
        match slot {
            DisplaySlot::PlayerList => ScoreboardDisplaySlot::List,
            DisplaySlot::Sidebar => ScoreboardDisplaySlot::Sidebar,
            DisplaySlot::BelowName => ScoreboardDisplaySlot::BelowName,
            DisplaySlot::SidebarTeamBlack => ScoreboardDisplaySlot::TeamBlack,
            DisplaySlot::SidebarTeamDarkBlue => ScoreboardDisplaySlot::TeamDarkBlue,
            DisplaySlot::SidebarTeamDarkGreen => ScoreboardDisplaySlot::TeamDarkGreen,
            DisplaySlot::SidebarTeamDarkAqua => ScoreboardDisplaySlot::TeamDarkAqua,
            DisplaySlot::SidebarTeamDarkRed => ScoreboardDisplaySlot::TeamDarkRed,
            DisplaySlot::SidebarTeamDarkPurple => ScoreboardDisplaySlot::TeamDarkPurple,
            DisplaySlot::SidebarTeamGold => ScoreboardDisplaySlot::TeamGold,
            DisplaySlot::SidebarTeamGray => ScoreboardDisplaySlot::TeamGray,
            DisplaySlot::SidebarTeamDarkGray => ScoreboardDisplaySlot::TeamDarkGray,
            DisplaySlot::SidebarTeamBlue => ScoreboardDisplaySlot::TeamBlue,
            DisplaySlot::SidebarTeamGreen => ScoreboardDisplaySlot::TeamGreen,
            DisplaySlot::SidebarTeamAqua => ScoreboardDisplaySlot::TeamAqua,
            DisplaySlot::SidebarTeamRed => ScoreboardDisplaySlot::TeamRed,
            DisplaySlot::SidebarTeamLightPurple => ScoreboardDisplaySlot::TeamLightPurple,
            DisplaySlot::SidebarTeamYellow => ScoreboardDisplaySlot::TeamYellow,
            DisplaySlot::SidebarTeamWhite => ScoreboardDisplaySlot::TeamWhite,
        }
    }

    fn to_objective_info(
        obj: &crate::world::scoreboard::ScoreboardObjective,
    ) -> scoreboard::ObjectiveInfo {
        scoreboard::ObjectiveInfo {
            name: obj.name.to_string(),
            display_name: obj.display_name.clone().get_text(),
            criteria_name: obj.criterion.to_string(),
            render_type: Self::to_wit_render_type(obj.render_type),
            display_auto_update: obj.display_auto_update,
        }
    }

    fn to_team_info(team: &crate::world::scoreboard::Team) -> scoreboard::TeamInfo {
        scoreboard::TeamInfo {
            name: team.name.clone(),
            display_name: team.display_name.clone().get_text(),
            players: team.players.clone(),
        }
    }
}

impl scoreboard::Host for PluginHostState {}

impl scoreboard::HostScoreboard for PluginHostState {
    async fn add_objective(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
        display_name: Resource<pumpkin::plugin::text::TextComponent>,
        criteria: String,
        render_type: WitRenderType,
    ) -> wasmtime::Result<()> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        let display_name = self.get_text_provider(&display_name)?;
        let rt = Self::to_internal_render_type(render_type);

        let objective = ScoreboardObjective::new(
            Arc::from(name.as_str()),
            display_name,
            rt,
            None,
            Arc::from(criteria.as_str()),
        );
        world
            .scoreboard
            .lock()
            .await
            .add_objective(&world, objective);
        Ok(())
    }

    async fn remove_objective(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
    ) -> wasmtime::Result<()> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        world
            .scoreboard
            .lock()
            .await
            .remove_objective(&world, &name);
        Ok(())
    }

    async fn get_objective(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
    ) -> wasmtime::Result<Option<scoreboard::ObjectiveInfo>> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        let sb = world.scoreboard.lock().await;
        Ok(sb.get_objective(&name).map(Self::to_objective_info))
    }

    async fn get_objectives(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
    ) -> wasmtime::Result<Vec<scoreboard::ObjectiveInfo>> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        let sb = world.scoreboard.lock().await;
        Ok(sb
            .get_objectives()
            .values()
            .map(Self::to_objective_info)
            .collect())
    }

    async fn get_objectives_by_criteria(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        criteria: String,
    ) -> wasmtime::Result<Vec<scoreboard::ObjectiveInfo>> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        let sb = world.scoreboard.lock().await;
        Ok(sb
            .get_objectives()
            .values()
            .filter(|o| *o.criterion == *criteria)
            .map(Self::to_objective_info)
            .collect())
    }

    async fn set_display_slot(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        slot: DisplaySlot,
        objective_name: String,
    ) -> wasmtime::Result<()> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        let internal_slot = Self::to_internal_display_slot(slot);
        world.scoreboard.lock().await.set_display_objective(
            &world,
            internal_slot,
            Some(&objective_name),
        );
        Ok(())
    }

    async fn clear_slot(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        slot: DisplaySlot,
    ) -> wasmtime::Result<()> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        let internal_slot = Self::to_internal_display_slot(slot);
        world
            .scoreboard
            .lock()
            .await
            .set_display_objective(&world, internal_slot, None);
        Ok(())
    }

    async fn get_display_slot(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        slot: DisplaySlot,
    ) -> wasmtime::Result<Option<String>> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        let internal_slot = Self::to_internal_display_slot(slot);
        let sb = world.scoreboard.lock().await;
        Ok(sb.get_display_objective(&internal_slot).map(String::from))
    }

    async fn update_score(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        entity_name: String,
        objective_name: String,
        value: i32,
    ) -> wasmtime::Result<()> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        let score = ScoreboardScore::new(
            Arc::from(entity_name.as_str()),
            Arc::from(objective_name.as_str()),
            VarInt(value),
            None,
            None,
        );
        world
            .scoreboard
            .lock()
            .await
            .update_score(&world, score)
            .await;
        Ok(())
    }

    async fn remove_score(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        entity_name: String,
        objective_name: String,
    ) -> wasmtime::Result<()> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        world
            .scoreboard
            .lock()
            .await
            .remove_score(&world, &entity_name, &objective_name)
            .await;
        Ok(())
    }

    async fn get_scores(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        entry: String,
    ) -> wasmtime::Result<Vec<scoreboard::ScoreInfo>> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        let sb = world.scoreboard.lock().await;
        let raw_scores = sb.list_scores_for_player(&entry);
        let results: Vec<scoreboard::ScoreInfo> = raw_scores
            .into_iter()
            .filter_map(|(obj_name, _)| {
                let score_info = sb.get_player_score_info(&entry, obj_name)?;
                Some(scoreboard::ScoreInfo {
                    objective_name: obj_name.to_string(),
                    value: score_info.value.0,
                    locked: score_info.locked,
                })
            })
            .collect();
        Ok(results)
    }

    async fn reset_scores(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        entry: String,
    ) -> wasmtime::Result<()> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        world
            .scoreboard
            .lock()
            .await
            .reset_all_player_scores(&world, &entry);
        Ok(())
    }

    async fn get_tracked_entries(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
    ) -> wasmtime::Result<Vec<String>> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        let sb = world.scoreboard.lock().await;
        Ok(sb
            .get_tracked_players()
            .into_iter()
            .map(ToString::to_string)
            .collect())
    }

    async fn create_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
        settings: TeamSettings,
    ) -> wasmtime::Result<()> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        let team = map_team_settings(name, &settings, self)?;
        world.scoreboard.lock().await.add_team(&world, team);
        Ok(())
    }

    async fn remove_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
    ) -> wasmtime::Result<()> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        world.scoreboard.lock().await.remove_team(&world, &name);
        Ok(())
    }

    async fn update_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
        settings: TeamSettings,
    ) -> wasmtime::Result<()> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        let team = map_team_settings(name, &settings, self)?;
        world.scoreboard.lock().await.update_team(&world, team);
        Ok(())
    }

    async fn get_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
    ) -> wasmtime::Result<Option<scoreboard::TeamInfo>> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        let sb = world.scoreboard.lock().await;
        Ok(sb.get_teams().get(&name).map(Self::to_team_info))
    }

    async fn get_teams(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
    ) -> wasmtime::Result<Vec<scoreboard::TeamInfo>> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        let sb = world.scoreboard.lock().await;
        Ok(sb.get_teams().values().map(Self::to_team_info).collect())
    }

    async fn get_entry_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        entry: String,
    ) -> wasmtime::Result<Option<String>> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        let sb = world.scoreboard.lock().await;
        Ok(sb
            .get_teams()
            .values()
            .find(|t| t.players.contains(&entry))
            .map(|t| t.name.clone()))
    }

    async fn add_player_to_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        team_name: String,
        player_name: String,
    ) -> wasmtime::Result<()> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        world
            .scoreboard
            .lock()
            .await
            .add_player_to_team(&world, &team_name, player_name);
        Ok(())
    }

    async fn remove_player_from_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        team_name: String,
        player_name: String,
    ) -> wasmtime::Result<()> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        world
            .scoreboard
            .lock()
            .await
            .remove_player_from_team(&world, &team_name, &player_name);
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<scoreboard::Scoreboard>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<ScoreboardResource>(Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
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
