use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_util::{
    GameMode, Hand,
    math::{position::BlockPos, vector3::Vector3},
};
use wasmtime::component::Resource;

use crate::{
    entity::player::Player,
    plugin::{
        BoxFuture, EventHandler, Payload,
        loader::wasm::wasm_host::{
            PluginInstance, WasmPlugin,
            state::{PlayerResource, PluginHostState, TextComponentResource, WorldResource},
            wit::{self, v0_1_0::pumpkin},
        },
    },
    server::Server,
    world::World,
};

pub mod block;
pub mod player;
pub mod server;
pub mod world;

impl pumpkin::plugin::event::Host for PluginHostState {}

pub struct WasmPluginV0_1_0EventHandler {
    pub handler_id: u32,
    pub plugin: Arc<WasmPlugin>,
}

pub trait ToFromV0_1_0WasmEvent {
    fn to_v0_1_0_wasm_event(
        &self,
        state: &mut PluginHostState,
    ) -> wit::v0_1_0::pumpkin::plugin::event::Event;

    fn from_v0_1_0_wasm_event(
        event: wit::v0_1_0::pumpkin::plugin::event::Event,
        state: &mut PluginHostState,
    ) -> Self;
}

pub(super) const fn to_wasm_position(position: Vector3<f64>) -> pumpkin::plugin::common::Position {
    (position.x, position.y, position.z)
}

pub(super) const fn from_wasm_position(
    position: pumpkin::plugin::common::Position,
) -> Vector3<f64> {
    Vector3::new(position.0, position.1, position.2)
}

pub(super) const fn to_wasm_block_position(
    position: BlockPos,
) -> pumpkin::plugin::common::BlockPosition {
    (position.0.x, position.0.y, position.0.z)
}

pub(super) const fn from_wasm_block_position(
    position: pumpkin::plugin::common::BlockPosition,
) -> BlockPos {
    BlockPos::new(position.0, position.1, position.2)
}

pub(super) fn to_wasm_block_name(block: &'static Block) -> String {
    format!("minecraft:{}", block.name)
}

pub(super) fn from_wasm_block_name(block_name: &str) -> &'static Block {
    Block::from_registry_key(block_name.strip_prefix("minecraft:").unwrap_or(block_name))
        .expect("invalid block name")
}

pub(super) const fn to_wasm_hand(hand: Hand) -> pumpkin::plugin::common::Hand {
    match hand {
        Hand::Left => pumpkin::plugin::common::Hand::Left,
        Hand::Right => pumpkin::plugin::common::Hand::Right,
    }
}

pub(super) const fn from_wasm_hand(hand: pumpkin::plugin::common::Hand) -> Hand {
    match hand {
        pumpkin::plugin::common::Hand::Left => Hand::Left,
        pumpkin::plugin::common::Hand::Right => Hand::Right,
    }
}

pub(super) const fn to_wasm_game_mode(game_mode: GameMode) -> pumpkin::plugin::common::GameMode {
    match game_mode {
        GameMode::Survival => pumpkin::plugin::common::GameMode::Survival,
        GameMode::Creative => pumpkin::plugin::common::GameMode::Creative,
        GameMode::Adventure => pumpkin::plugin::common::GameMode::Adventure,
        GameMode::Spectator => pumpkin::plugin::common::GameMode::Spectator,
    }
}

pub(super) const fn from_wasm_game_mode(game_mode: pumpkin::plugin::common::GameMode) -> GameMode {
    match game_mode {
        pumpkin::plugin::common::GameMode::Survival => GameMode::Survival,
        pumpkin::plugin::common::GameMode::Creative => GameMode::Creative,
        pumpkin::plugin::common::GameMode::Adventure => GameMode::Adventure,
        pumpkin::plugin::common::GameMode::Spectator => GameMode::Spectator,
    }
}

pub(super) fn consume_player(
    state: &mut PluginHostState,
    player: &Resource<pumpkin::plugin::player::Player>,
) -> Arc<Player> {
    state
        .resource_table
        .delete::<PlayerResource>(Resource::new_own(player.rep()))
        .expect("invalid player resource handle")
        .provider
}

pub(super) fn consume_text_component(
    state: &mut PluginHostState,
    text_component: &Resource<pumpkin::plugin::text::TextComponent>,
) -> pumpkin_util::text::TextComponent {
    state
        .resource_table
        .delete::<TextComponentResource>(Resource::new_own(text_component.rep()))
        .expect("invalid text-component resource handle")
        .provider
}

pub(super) fn consume_world(
    state: &mut PluginHostState,
    world: &Resource<pumpkin::plugin::world::World>,
) -> Arc<World> {
    state
        .resource_table
        .delete::<WorldResource>(Resource::new_own(world.rep()))
        .expect("invalid world resource handle")
        .provider
}

impl<E: Payload + ToFromV0_1_0WasmEvent> EventHandler<E> for WasmPluginV0_1_0EventHandler {
    fn handle<'a>(&'a self, server: &'a Arc<Server>, event: &'a E) -> BoxFuture<'a, ()> {
        Box::pin(async {
            let mut store = self.plugin.store.lock().await;
            let event = event.to_v0_1_0_wasm_event(store.data_mut());
            match self.plugin.plugin_instance {
                PluginInstance::V0_1_0(ref plugin) => {
                    let server = store.data_mut().add_server(server.clone()).unwrap();
                    plugin
                        .call_handle_event(&mut *store, self.handler_id, server, &event)
                        .await
                        .unwrap();
                }
            }
        })
    }

    fn handle_blocking<'a>(
        &'a self,
        server: &'a Arc<Server>,
        event: &'a mut E,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async {
            let mut store = self.plugin.store.lock().await;
            let wasm_event = event.to_v0_1_0_wasm_event(store.data_mut());
            match self.plugin.plugin_instance {
                PluginInstance::V0_1_0(ref plugin) => {
                    let server = store.data_mut().add_server(server.clone()).unwrap();
                    let returned_event = plugin
                        .call_handle_event(&mut *store, self.handler_id, server, &wasm_event)
                        .await
                        .unwrap();

                    *event = E::from_v0_1_0_wasm_event(returned_event, store.data_mut());
                }
            }
        })
    }
}
