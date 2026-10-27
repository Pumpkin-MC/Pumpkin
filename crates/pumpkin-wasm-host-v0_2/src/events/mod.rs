use std::{marker::PhantomData, mem, ptr::NonNull, sync::Arc};

use pumpkin_data::{Block, entity::EntityType};
use pumpkin_inventory::screen_handler::ClickType;
use pumpkin_protocol::java::server::play::ActionType;
use pumpkin_util::{
    GameMode, Hand,
    math::{position::BlockPos, vector3::Vector3},
};
use wasmtime::component::Resource;

use crate::pumpkin;
use pumpkin_core::{
    entity::player::Player,
    plugin::{BoxFuture, EventHandler, Payload},
    server::Server,
    world::World,
};
use pumpkin_wasm_host_common::{plugin::WasmPlugin, state::PluginHostState};

pub mod block;
pub mod cleanup;
pub mod dialog;
pub mod enchantment;
pub mod entity;
pub mod hanging;
pub mod inventory;
pub mod player;
pub mod raid;
pub mod server;
pub mod vehicle;
pub mod world;

pub use cleanup::*;

impl pumpkin::plugin::event::Host for PluginHostState {}

pub struct WasmPluginEventHandler {
    pub handler_id: u32,
    pub plugin: Arc<WasmPlugin>,
}

pub trait ToFromWasmEvent {
    type Wit;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit;

    fn from_wasm_event(event: Self::Wit, state: &mut PluginHostState) -> Self;

    fn apply_wasm_event(&mut self, event: Self::Wit, state: &mut PluginHostState)
    where
        Self: Sized,
    {
        *self = Self::from_wasm_event(event, state);
    }
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
) -> pumpkin::plugin::common::BlockPos {
    pumpkin::plugin::common::BlockPos {
        x: position.0.x,
        y: position.0.y,
        z: position.0.z,
    }
}

pub(super) const fn from_wasm_block_position(
    position: pumpkin::plugin::common::BlockPos,
) -> BlockPos {
    BlockPos::new(position.x, position.y, position.z)
}

pub(super) fn to_wasm_block_name(block: &'static Block) -> String {
    format!("minecraft:{}", block.name)
}

pub(super) fn from_wasm_block_name(block_name: &str) -> &'static Block {
    Block::from_registry_key(block_name.strip_prefix("minecraft:").unwrap_or(block_name))
        .unwrap_or(&Block::AIR)
}

pub(super) fn to_wasm_entity_type(entity_type: &'static EntityType) -> String {
    format!("minecraft:{}", entity_type.resource_name)
}

pub(super) fn from_wasm_entity_type(entity_type: &str) -> &'static EntityType {
    EntityType::from_name(
        entity_type
            .strip_prefix("minecraft:")
            .unwrap_or(entity_type),
    )
    .unwrap_or(&EntityType::PLAYER)
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

pub(super) const fn to_wasm_entity_interaction_action(
    action: ActionType,
) -> pumpkin::plugin::event::EntityInteractionAction {
    match action {
        ActionType::Interact => pumpkin::plugin::event::EntityInteractionAction::Interact,
        ActionType::Attack => pumpkin::plugin::event::EntityInteractionAction::Attack,
        ActionType::InteractAt => pumpkin::plugin::event::EntityInteractionAction::InteractAt,
    }
}

pub(super) const fn from_wasm_entity_interaction_action(
    action: pumpkin::plugin::event::EntityInteractionAction,
) -> ActionType {
    match action {
        pumpkin::plugin::event::EntityInteractionAction::Interact => ActionType::Interact,
        pumpkin::plugin::event::EntityInteractionAction::Attack => ActionType::Attack,
        pumpkin::plugin::event::EntityInteractionAction::InteractAt => ActionType::InteractAt,
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

pub(super) const fn to_wasm_click_type(click_type: ClickType) -> pumpkin::plugin::gui::ClickType {
    match click_type {
        ClickType::Left => pumpkin::plugin::gui::ClickType::Left,
        ClickType::Right => pumpkin::plugin::gui::ClickType::Right,
        ClickType::ShiftLeft => pumpkin::plugin::gui::ClickType::ShiftLeft,
        ClickType::ShiftRight => pumpkin::plugin::gui::ClickType::ShiftRight,
        ClickType::Middle => pumpkin::plugin::gui::ClickType::Middle,
        ClickType::Drop => pumpkin::plugin::gui::ClickType::Drop,
        ClickType::ControlDrop => pumpkin::plugin::gui::ClickType::ControlDrop,
        ClickType::DoubleClick => pumpkin::plugin::gui::ClickType::DoubleClick,
        ClickType::NumberKey(_) => pumpkin::plugin::gui::ClickType::NumberKey,
        ClickType::Unknown => pumpkin::plugin::gui::ClickType::Unknown,
    }
}

pub(super) const fn from_wasm_click_type(click_type: pumpkin::plugin::gui::ClickType) -> ClickType {
    match click_type {
        pumpkin::plugin::gui::ClickType::Left => ClickType::Left,
        pumpkin::plugin::gui::ClickType::Right => ClickType::Right,
        pumpkin::plugin::gui::ClickType::ShiftLeft => ClickType::ShiftLeft,
        pumpkin::plugin::gui::ClickType::ShiftRight => ClickType::ShiftRight,
        pumpkin::plugin::gui::ClickType::Middle => ClickType::Middle,
        pumpkin::plugin::gui::ClickType::Drop => ClickType::Drop,
        pumpkin::plugin::gui::ClickType::ControlDrop => ClickType::ControlDrop,
        pumpkin::plugin::gui::ClickType::DoubleClick => ClickType::DoubleClick,
        pumpkin::plugin::gui::ClickType::NumberKey => ClickType::NumberKey(0), // Default to 0
        pumpkin::plugin::gui::ClickType::Unknown => ClickType::Unknown,
    }
}

pub(super) fn consume_player(
    state: &mut PluginHostState,
    player: &Resource<pumpkin::plugin::player::Player>,
) -> Arc<Player> {
    state
        .resource_table
        .delete::<Arc<Player>>(Resource::new_own(player.rep()))
        .expect("invalid player resource handle")
}

pub(super) fn consume_text_component(
    state: &mut PluginHostState,
    text_component: &Resource<pumpkin::plugin::text::TextComponent>,
) -> pumpkin_util::text::TextComponent {
    state
        .resource_table
        .delete::<pumpkin_util::text::TextComponent>(Resource::new_own(text_component.rep()))
        .expect("invalid text-component resource handle")
}

pub(super) fn consume_world(
    state: &mut PluginHostState,
    world: &Resource<pumpkin::plugin::world::World>,
) -> Arc<World> {
    state
        .resource_table
        .delete::<Arc<World>>(Resource::new_own(world.rep()))
        .expect("invalid world resource handle")
}

type WasmEvent = crate::pumpkin::plugin::event::Event;

pub(crate) struct EventHandle {
    data: NonNull<dyn Payload + 'static>,
    mutable: bool,
}

unsafe impl Send for EventHandle {}
unsafe impl Sync for EventHandle {}

impl EventHandle {
    fn from_ref(event: &(dyn Payload + 'static)) -> Self {
        Self {
            data: NonNull::from_ref(event),
            mutable: false,
        }
    }

    fn from_mut(event: &mut (dyn Payload + 'static)) -> Self {
        Self {
            data: NonNull::from_mut(event),
            mutable: true,
        }
    }

    fn get_ref<'a, T: Payload + 'static>(&'a self) -> Result<&'a T, ()> {
        unsafe { self.data.as_ref() }.downcast_ref::<T>().ok_or(())
    }

    fn get_mut<'a, T: Payload + 'static>(&'a mut self) -> Result<&'a mut T, ()> {
        if !self.mutable {
            return Err(());
        }
        unsafe { self.data.as_mut() }.downcast_mut::<T>().ok_or(())
    }
}

struct EventResource<'a> {
    data: Resource<WasmEvent>,
    _phantom: PhantomData<&'a mut (dyn Payload + 'static)>,
}

impl<'a> EventResource<'a> {
    async fn from_ref(
        event: &'a (dyn Payload + 'static),
        plugin: &'a Arc<WasmPlugin>,
    ) -> wasmtime::Result<Self> {
        let static_event: &'static (dyn Payload + 'static) = unsafe { mem::transmute(event) };
        let handle = EventHandle::from_ref(static_event);
        Ok(Self {
            data: plugin
                .store
                .call(move |state| state.add(handle))
                .await
                .flatten()?,
            _phantom: PhantomData,
        })
    }

    async fn from_mut(
        event: &'a mut (dyn Payload + 'static),
        plugin: &'a Arc<WasmPlugin>,
    ) -> wasmtime::Result<Self> {
        let static_event: &'static mut (dyn Payload + 'static) = unsafe { mem::transmute(event) };
        let handle = EventHandle::from_mut(static_event);
        Ok(Self {
            data: plugin
                .store
                .call(|state| state.add(handle))
                .await
                .flatten()?,
            _phantom: PhantomData,
        })
    }

    fn get_borrow(&self) -> Resource<WasmEvent> {
        Resource::new_borrow(self.data.rep())
    }

    async fn destroy(self, plugin: &Arc<WasmPlugin>) -> wasmtime::Result<()> {
        plugin
            .store
            .call(|state| state.drop(self.data))
            .await
            .flatten()
    }
}

impl WasmPluginEventHandler {
    async fn handle_dyn<'a, F, Fut>(&'a self, server: &'a Arc<Server>, create_event: F)
    where
        F: FnOnce(&'a Arc<WasmPlugin>) -> Fut,
        Fut: std::future::Future<Output = wasmtime::Result<EventResource<'a>>>,
    {
        let wasm_event = match create_event(&self.plugin).await {
            Ok(res) => res,
            Err(error) => {
                tracing::error!(
                    handler_id = self.handler_id,
                    %error,
                    "Failed to create resource for event."
                );
                return;
            }
        };

        let event_borrow = wasm_event.get_borrow();
        let server_clone = server.clone();

        let handler_id = self.handler_id;
        let function = *self
            .plugin
            .instance::<crate::Plugin>()
            .func_handle_event()
            .func();

        match self
            .plugin
            .store
            .call_guest(move |mut guest| {
                Box::pin(async move {
                    let server_res = guest.with(|mut store| store.data_mut().add(server_clone))?;

                    guest
                        .call_func::<(
                            u32,
                            Resource<crate::pumpkin::plugin::server::Server>,
                            Resource<WasmEvent>,
                        ), ()>(
                            function, (handler_id, server_res, event_borrow)
                        )
                        .await
                })
            })
            .await
        {
            Ok(_) => {}
            Err(error) => {
                tracing::error!(
                    handler_id = self.handler_id,
                    %error,
                    "Wasm event handler failed"
                );
            }
        }

        match wasm_event.destroy(&self.plugin).await {
            Ok(_) => {}
            Err(error) => {
                tracing::error!(
                    handler_id = self.handler_id,
                    %error,
                    "Failed to cleanup resource for event."
                );
            }
        };
    }
}

impl<E: Payload + ToFromWasmEvent + 'static> EventHandler<E> for WasmPluginEventHandler {
    fn handle<'a>(&'a self, server: &'a Arc<Server>, event: &'a E) -> BoxFuture<'a, ()> {
        let dyn_event: &(dyn Payload + 'static) = event;
        Box::pin(self.handle_dyn(server, move |plugin| {
            EventResource::from_ref(dyn_event, plugin)
        }))
    }

    fn handle_blocking<'a>(
        &'a self,
        server: &'a Arc<Server>,
        event: &'a mut E,
    ) -> BoxFuture<'a, ()> {
        let dyn_event: &mut (dyn Payload + 'static) = event;
        Box::pin(self.handle_dyn(server, move |plugin| {
            EventResource::from_mut(dyn_event, plugin)
        }))
    }
}
