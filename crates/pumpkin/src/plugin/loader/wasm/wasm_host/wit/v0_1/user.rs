use std::sync::Arc;
use wasmtime::component::Resource;

use super::{
    pumpkin::plugin::{player, user as wit, uuid::Uuid},
    uuid::UuidExt,
};
use crate::{
    net::user::{Edition, User},
    plugin::loader::wasm::wasm_host::state::PluginHostState,
};

#[must_use]
pub const fn to_wasm_state(state: pumpkin_protocol::ConnectionState) -> wit::ConnectionState {
    use pumpkin_protocol::ConnectionState as C;
    match state {
        C::HandShake => wit::ConnectionState::Handshake,
        C::Status => wit::ConnectionState::Status,
        C::Login => wit::ConnectionState::Login,
        C::Transfer => wit::ConnectionState::Transfer,
        C::Config => wit::ConnectionState::Config,
        C::Play => wit::ConnectionState::Play,
    }
}

impl PluginHostState {
    pub fn add_user(&mut self, user: Arc<User>) -> wasmtime::Result<Resource<wit::User>> {
        Ok(Resource::new_own(self.resource_table.push(user)?.rep()))
    }

    pub(crate) fn packet_user(
        &self,
        resource: &Resource<wit::User>,
    ) -> wasmtime::Result<Arc<User>> {
        Ok(self
            .resource_table
            .get::<Arc<User>>(&Resource::new_borrow(resource.rep()))?
            .clone())
    }
}

impl wit::Host for PluginHostState {}

impl wit::HostUser for PluginHostState {
    async fn get_info(&mut self, resource: Resource<wit::User>) -> wasmtime::Result<wit::UserInfo> {
        let user = self.packet_user(&resource)?;
        let info = user.info();
        Ok(wit::UserInfo {
            id: Uuid::to_wit(&user.id),
            edition: match user.edition {
                Edition::Java => wit::Edition::Java,
                Edition::Bedrock => wit::Edition::Bedrock,
            },
            decoder_state: to_wasm_state(user.decoder_state.load()),
            encoder_state: to_wasm_state(user.encoder_state.load()),
            address: info.address.to_string(),
            server_address: info.server_address,
            server_port: info.server_port,
            protocol_version: user.protocol_version.load(),
            name: info.profile.as_ref().map(|p| p.name.clone()),
            uuid: info.profile.as_ref().map(|p| Uuid::to_wit(&p.id)),
            properties: info
                .profile
                .as_ref()
                .map(|p| {
                    p.properties
                        .load()
                        .iter()
                        .map(|p| wit::ProfileProperty {
                            name: p.name.to_string(),
                            value: p.value.to_string(),
                            signature: p.signature.as_ref().map(ToString::to_string),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            settings: info.config.as_ref().map(super::player::to_wasm_settings),
            brand: info.brand,
            closed: user.is_closed(),
        })
    }

    async fn get_player(
        &mut self,
        resource: Resource<wit::User>,
    ) -> wasmtime::Result<Option<Resource<player::Player>>> {
        self.packet_user(&resource)?
            .player()
            .map(|player| self.add_player(player))
            .transpose()
    }

    async fn send_packet(
        &mut self,
        resource: Resource<wit::User>,
        packet_id: i32,
        payload: Vec<u8>,
        silent: bool,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(self
            .packet_user(&resource)?
            .send_packet(packet_id, &payload, silent)
            .map_err(|e| e.to_string()))
    }

    async fn close(&mut self, resource: Resource<wit::User>) -> wasmtime::Result<()> {
        self.packet_user(&resource)?.close();
        Ok(())
    }

    async fn clone(
        &mut self,
        resource: Resource<wit::User>,
    ) -> wasmtime::Result<Resource<wit::User>> {
        self.add_user(self.packet_user(&resource)?)
    }

    async fn drop(&mut self, resource: Resource<wit::User>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<Arc<User>>(Resource::new_own(resource.rep()))?;
        Ok(())
    }
}
