use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::{PluginHostState, ScoreboardResource},
    wit::v0_1::pumpkin::{
        self,
        plugin::scoreboard::{
            self, CollisionRule, DisplaySlot, NametagVisibility, RenderType, TeamSettings,
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
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        let display_name = self.get_text_provider(&display_name)?;

        let rt = match render_type {
            RenderType::Integer => pumpkin_protocol::java::client::play::RenderType::Integer,
            RenderType::Hearts => pumpkin_protocol::java::client::play::RenderType::Hearts,
        };

        let objective =
            ScoreboardObjective::new(Box::leak(name.into_boxed_str()), display_name, rt, None);
        world
            .scoreboard
            .lock()
            .await
            .add_objective(&world, objective)
            .await;
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
            .remove_objective(&world, &name)
            .await;
        Ok(())
    }

    async fn set_display_slot(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        slot: DisplaySlot,
        objective_name: String,
    ) -> wasmtime::Result<()> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        let slot = match slot {
            DisplaySlot::PlayerList => pumpkin_data::scoreboard::ScoreboardDisplaySlot::List,
            DisplaySlot::Sidebar => pumpkin_data::scoreboard::ScoreboardDisplaySlot::Sidebar,
            DisplaySlot::BelowName => pumpkin_data::scoreboard::ScoreboardDisplaySlot::BelowName,
            DisplaySlot::SidebarTeamBlack => {
                pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamBlack
            }
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
            DisplaySlot::SidebarTeamGold => {
                pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamGold
            }
            DisplaySlot::SidebarTeamGray => {
                pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamGray
            }
            DisplaySlot::SidebarTeamDarkGray => {
                pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamDarkGray
            }
            DisplaySlot::SidebarTeamBlue => {
                pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamBlue
            }
            DisplaySlot::SidebarTeamGreen => {
                pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamGreen
            }
            DisplaySlot::SidebarTeamAqua => {
                pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamAqua
            }
            DisplaySlot::SidebarTeamRed => pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamRed,
            DisplaySlot::SidebarTeamLightPurple => {
                pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamLightPurple
            }
            DisplaySlot::SidebarTeamYellow => {
                pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamYellow
            }
            DisplaySlot::SidebarTeamWhite => {
                pumpkin_data::scoreboard::ScoreboardDisplaySlot::TeamWhite
            }
        };

        world
            .broadcast_packet_all(
                &pumpkin_protocol::java::client::play::CDisplayObjective::new(slot, objective_name),
            )
            .await;
        Ok(())
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
            Box::leak(entity_name.into_boxed_str()),
            Box::leak(objective_name.into_boxed_str()),
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

    async fn create_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
        settings: TeamSettings,
    ) -> wasmtime::Result<()> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        let team = map_team_settings(name, &settings, self)?;
        world.scoreboard.lock().await.add_team(&world, team).await;
        Ok(())
    }

    async fn remove_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
    ) -> wasmtime::Result<()> {
        let world = self.get_scoreboard_res(&res)?.provider.clone();
        world
            .scoreboard
            .lock()
            .await
            .remove_team(&world, &name)
            .await;
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
        world
            .scoreboard
            .lock()
            .await
            .update_team(&world, team)
            .await;
        Ok(())
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
            .add_player_to_team(&world, &team_name, player_name)
            .await;
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
            .remove_player_from_team(&world, &team_name, &player_name)
            .await;
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
