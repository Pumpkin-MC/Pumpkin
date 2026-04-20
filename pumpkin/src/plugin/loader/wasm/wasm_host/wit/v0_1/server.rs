use uuid::Uuid;
use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::{PluginHostState, ServerResource},
    wit::v0_1::pumpkin::{
        self,
        plugin::{
            player::Player,
            server::{Difficulty, Server},
        },
    },
};

impl PluginHostState {
    fn get_server_res(&self, res: &Resource<Server>) -> wasmtime::Result<&ServerResource> {
        self.resource_table
            .get::<ServerResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }
}

impl pumpkin::plugin::server::Host for PluginHostState {}

impl pumpkin::plugin::server::HostServer for PluginHostState {
    async fn get_difficulty(&mut self, res: Resource<Server>) -> wasmtime::Result<Difficulty> {
        let resource = self.get_server_res(&res)?;

        Ok(match resource.provider.get_difficulty() {
            pumpkin_util::Difficulty::Peaceful => Difficulty::Peaceful,
            pumpkin_util::Difficulty::Easy => Difficulty::Easy,
            pumpkin_util::Difficulty::Normal => Difficulty::Normal,
            pumpkin_util::Difficulty::Hard => Difficulty::Hard,
        })
    }

    async fn get_player_count(&mut self, _res: Resource<Server>) -> wasmtime::Result<u32> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        Ok(server.get_player_count() as u32)
    }

    async fn get_mspt(&mut self, _res: Resource<Server>) -> wasmtime::Result<f64> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        Ok(server.get_mspt())
    }

    async fn get_tps(&mut self, _res: Resource<Server>) -> wasmtime::Result<f64> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        Ok(server.get_tps())
    }

    async fn get_all_players(
        &mut self,
        _res: Resource<Server>,
    ) -> wasmtime::Result<Vec<Resource<Player>>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server
            .get_all_players()
            .into_iter()
            .map(|player| {
                self.add_player(player)
                    .expect("failed to add player resource")
            })
            .collect())
    }

    async fn get_player_by_name(
        &mut self,
        _rep: Resource<Server>,
        name: String,
    ) -> wasmtime::Result<Option<Resource<Player>>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        server
            .get_player_by_name(&name)
            .map(|player| self.add_player(player))
            .transpose()
    }

    async fn get_player_by_uuid(
        &mut self,
        _rep: Resource<Server>,
        id: String,
    ) -> wasmtime::Result<Option<Resource<Player>>> {
        let Ok(uuid) = Uuid::parse_str(&id) else {
            return Ok(None);
        };

        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        server
            .get_player_by_uuid(uuid)
            .map(|player| self.add_player(player))
            .transpose()
    }

    async fn get_all_worlds(
        &mut self,
        _rep: Resource<Server>,
    ) -> wasmtime::Result<Vec<Resource<pumpkin::plugin::world::World>>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server
            .worlds
            .load()
            .iter()
            .map(|world| {
                self.add_world(world.clone())
                    .expect("failed to add world resource")
            })
            .collect())
    }

    async fn get_world_by_name(
        &mut self,
        _rep: Resource<Server>,
        name: String,
    ) -> wasmtime::Result<Option<Resource<pumpkin::plugin::world::World>>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server
            .worlds
            .load()
            .iter()
            .find(|world| world.dimension.minecraft_name == name)
            .map(|world| {
                self.add_world(world.clone())
                    .expect("failed to add world resource")
            }))
    }

    async fn broadcast(&mut self, _rep: Resource<Server>, message: String) -> wasmtime::Result<()> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        server
            .broadcast_message(
                &pumpkin_util::text::TextComponent::text(message),
                &pumpkin_util::text::TextComponent::text("Server"),
                0,
                None,
            )
            .await;

        Ok(())
    }

    async fn get_max_players(&mut self, _rep: Resource<Server>) -> wasmtime::Result<u32> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.basic_config.max_players)
    }

    async fn is_hardcore(&mut self, _rep: Resource<Server>) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.basic_config.hardcore)
    }

    async fn is_online_mode(&mut self, _rep: Resource<Server>) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.advanced_config.networking.authentication.enabled)
    }

    async fn get_motd(&mut self, _rep: Resource<Server>) -> wasmtime::Result<String> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.basic_config.motd.clone())
    }

    async fn has_whitelist(&mut self, _rep: Resource<Server>) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.basic_config.white_list)
    }

    async fn get_allow_nether(&mut self, _rep: Resource<Server>) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.basic_config.allow_nether)
    }

    async fn get_allow_end(&mut self, _rep: Resource<Server>) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.basic_config.allow_end)
    }

    async fn get_view_distance(&mut self, _rep: Resource<Server>) -> wasmtime::Result<u8> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.basic_config.view_distance.get())
    }

    async fn get_simulation_distance(&mut self, _rep: Resource<Server>) -> wasmtime::Result<u8> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.basic_config.simulation_distance.get())
    }

    async fn get_default_gamemode(
        &mut self,
        _rep: Resource<Server>,
    ) -> wasmtime::Result<pumpkin::plugin::common::GameMode> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(super::events::to_wasm_game_mode(
            server.basic_config.default_gamemode,
        ))
    }

    async fn drop(&mut self, rep: Resource<Server>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<ServerResource>(Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}
