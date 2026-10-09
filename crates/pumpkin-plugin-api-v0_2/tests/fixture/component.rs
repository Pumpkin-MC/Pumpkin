//! Test fixture compiled to a component by `tests/component.rs`, not a distributable plugin

// The component exports only link on wasm, so the native build of this example is empty
#![cfg(target_family = "wasm")]

use pumpkin_plugin_api_v0_2::{
    Context, EventPriority, Plugin, PluginMetadata, Result, Server,
    ai::{AiGoal, register_ai_goal},
    commands::CommandHandler,
    display::EntityDisplayExt,
    events::{EventData, EventHandler, PlayerChatEvent},
    gametest::{register_async_test, register_entity_predicate, register_void_callback},
    host::{
        command::{Command, CommandError, CommandSender, ConsumedArgs},
        ipc, scheduler,
        world::{ChunkBuffer, Entity},
    },
    register_plugin,
    scheduler::schedule_delayed_task,
    worldgen::{ChunkGenerator, register_chunk_generator},
};

struct Fixture;

struct ChatGuard;

impl EventHandler<PlayerChatEvent> for ChatGuard {
    async fn handle(
        &self,
        _server: Server,
        mut event: EventData<PlayerChatEvent>,
    ) -> EventData<PlayerChatEvent> {
        // The permission check is an async host import with the node as its only argument
        if !event
            .player
            .has_permission("fixture.chat".to_string())
            .await
        {
            event.cancelled = true;
        }
        event
    }
}

struct Ping;

impl CommandHandler for Ping {
    async fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        Ok(i32::from(
            sender.has_permission("fixture.ping".to_string()).await,
        ))
    }
}

struct NotDisplay;

impl AiGoal for NotDisplay {
    async fn can_start(&self, _server: Server, entity: Entity) -> bool {
        // A failed conversion hands the original entity back
        entity.into_display_entity().is_err()
    }
}

struct Flat;

impl ChunkGenerator for Flat {
    async fn generate_surface(&self, chunk: ChunkBuffer) {
        chunk.fill_layer(chunk.get_min_y(), 1);
    }
}

impl Plugin for Fixture {
    fn new() -> Self {
        Self
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "fixture".into(),
            version: "0.0.0".into(),
            authors: Vec::new(),
            description: "pumpkin-plugin-api-v0_2 test fixture".into(),
            dependencies: Vec::new(),
            permissions: Vec::new(),
        }
    }

    async fn on_load(&self, context: Context) -> Result<()> {
        context.register_event_handler::<PlayerChatEvent, _>(
            ChatGuard,
            EventPriority::Normal,
            true,
        );
        let ping = Command::new(&["ping".to_string()], "replies when allowed").execute(Ping);
        context.register_command(ping, "fixture.ping");
        register_ai_goal(NotDisplay);
        register_entity_predicate(Entity::is_on_ground);
        register_chunk_generator(Flat);
        let done = register_void_callback(|| {});
        register_async_test(move |test| async move {
            let _ = test.succeed_when(done);
        });
        schedule_delayed_task(20, |server| async move {
            server.broadcast("fixture task ran".to_string()).await;
        });
        Ok(())
    }

    async fn handle_ipc_message(
        &self,
        sender: ipc::PluginId,
        message: ipc::IpcMessage,
    ) -> Result<ipc::IpcMessage> {
        // Task cancellation is a synchronous import, the send is awaited
        scheduler::cancel_task(0);
        ipc::send_ipc_message(sender, message)
            .await
            .map_err(|()| "ipc send failed".to_string())?
    }
}

register_plugin!(Fixture);
