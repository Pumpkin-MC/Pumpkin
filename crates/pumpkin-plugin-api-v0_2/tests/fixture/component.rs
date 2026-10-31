//! Test fixture compiled to a component by `tests/component.rs`, not a distributable plugin

// The component exports only link on wasm, so on native targets this example is empty
#![cfg(target_family = "wasm")]

use pumpkin_plugin_api_v0_2::{
    Context, EventPriority, Plugin, PluginMetadata, Result, Server,
    ai::{AiGoal, register_ai_goal},
    commands::{CommandHandler, CommandSuggestionHandler},
    display::EntityDisplayExt,
    events::{EventData, EventHandler, PlayerChatEvent},
    gametest::{register_async_test, register_entity_predicate, register_void_callback},
    host::{
        command::{
            Command, CommandError, CommandNode, CommandSender, CommandSuggestion,
            CommandSuggestions, ConsumedArgs, SuggestionRequest,
        },
        ipc, scheduler,
        world::{ChunkBuffer, Entity},
    },
    register_plugin,
    scheduler::{schedule_delayed_task, schedule_repeating_task},
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
        // has_permission is an async import, so the handler suspends here
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

struct Suggest;

impl CommandSuggestionHandler for Suggest {
    async fn suggest(
        &self,
        _sender: CommandSender,
        _server: Server,
        request: SuggestionRequest,
    ) -> CommandSuggestions {
        CommandSuggestions {
            start: request.start,
            length: request.remaining.len() as u32,
            values: vec![CommandSuggestion {
                value: "fixture".into(),
                tooltip: None,
            }],
        }
    }
}

struct NotDisplay;

impl AiGoal for NotDisplay {
    async fn can_start(&self, _server: Server, entity: Entity) -> bool {
        // A failed cast leaves the original entity available for another cast
        entity.as_display().is_none() && entity.as_interaction().is_none()
    }

    async fn start(&self, _server: Server, _entity: Entity) {
        scheduler::cancel_task(101);
    }
    async fn tick(&self, _server: Server, _entity: Entity) {
        scheduler::cancel_task(102);
    }
    async fn stop(&self, _server: Server, _entity: Entity) {
        scheduler::cancel_task(103);
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
        let node = CommandNode::literal("complete").suggest(Suggest);
        context.register_command(ping.then(node), "fixture.ping");
        use pumpkin_plugin_api_v0_2::{
            Item, ItemStack, ItemStackExt, TextComponent, enchantment::EnchantmentBuilder,
            recipe::ShapedRecipeBuilder,
        };
        ShapedRecipeBuilder::new(
            "fixture:diamond",
            ItemStack::of(Item::from_name("diamond").ok_or("unknown diamond item")?, 2),
        )
        .pattern(["DD"])
        .key('D', "diamond")
        .register_to_context(&context)
        .await
        .map_err(|error| error.to_string())?;
        EnchantmentBuilder::new("fixture:test", TextComponent::text("Test"))
            .register_to_context(&context)
            .await
            .map_err(|error| error.to_string())?;
        tracing::info!("fixture loaded");
        register_ai_goal(NotDisplay);
        register_entity_predicate(Entity::is_on_ground);
        register_chunk_generator(Flat);
        let done = register_void_callback(|| {});
        register_async_test(move |test| async move {
            let _ = test.succeed_when(done);
        });
        let message = std::cell::Cell::new("fixture task ran".to_string());
        schedule_delayed_task(20, move |server| async move {
            server.broadcast(message.into_inner()).await;
        });
        schedule_repeating_task(20, 40, |server| async move {
            server
                .broadcast("fixture repeating task ran".to_string())
                .await;
        });
        Ok(())
    }

    async fn on_unload(&self, _context: Context) -> Result<()> {
        scheduler::cancel_task(77);
        Ok(())
    }

    async fn handle_ipc_message(
        &self,
        sender: ipc::PluginId,
        message: ipc::IpcMessage,
    ) -> Result<ipc::IpcMessage> {
        if sender == "helpers" {
            use pumpkin_plugin_api_v0_2::{
                Item, ItemStack, ItemStackExt, PersistentDataHolder, TextComponent,
                TransformationBuilder,
                enchantment::EnchantmentBuilder,
                forms::{FormResponse, SimpleFormBuilder},
                recipe::{Ingredient, ShapedRecipeBuilder},
            };
            let item = ItemStack::of(Item::from_name("diamond").ok_or("unknown diamond item")?, 2);
            assert_eq!(item.get_item().await, Item::from_name("minecraft:diamond"));
            assert!(item.matches_item("diamond").await);
            use pumpkin_plugin_api_v0_2::{BlockType, IntoBlockKey, IntoItemKey};
            assert_eq!("diamond".into_item_key(), "minecraft:diamond");
            assert_eq!("custom:item".into_item_key(), "custom:item");
            assert_eq!("stone".into_block_key(), "minecraft:stone");
            assert_eq!("custom:block".into_block_key(), "custom:block");
            let stone = BlockType::from_name("stone").ok_or("unknown stone block")?;
            assert_eq!(stone.resource_location(), "minecraft:stone");
            assert_eq!(stone.as_str(), "stone");
            assert_eq!(Some(stone), BlockType::from_registry_key("minecraft:stone"));
            assert!(Item::from_name("custom:diamond").is_none());
            assert!(BlockType::from_name("custom:stone").is_none());
            item.set_string("fixture", "owner", "guest").await;
            assert_eq!(
                item.get_string("fixture", "owner").await.as_deref(),
                Some("guest")
            );
            assert!(
                ShapedRecipeBuilder::new("fixture:invalid", item)
                    .pattern(["DD"])
                    .key('D', "minecraft:diamond")
                    .validate()
                    .is_ok()
            );
            assert_eq!(
                Ingredient::tag("planks"),
                Ingredient::Tag("minecraft:planks".into())
            );
            assert_eq!(
                Ingredient::from("custom:ruby"),
                Ingredient::Item("custom:ruby".into())
            );
            assert_eq!(
                Ingredient::from("#logs"),
                Ingredient::Tag("minecraft:logs".into())
            );
            assert_eq!(
                Ingredient::from("#minecraft:planks"),
                Ingredient::Tag("minecraft:planks".into())
            );
            assert_eq!(
                Ingredient::from(["coal", "charcoal"]),
                Ingredient::OneOf(vec!["minecraft:coal".into(), "minecraft:charcoal".into()])
            );
            assert!(
                EnchantmentBuilder::new("", TextComponent::text("Invalid"))
                    .validate()
                    .is_err()
            );
            let _form =
                SimpleFormBuilder::new(TextComponent::text("Title"), TextComponent::text("Body"))
                    .button(TextComponent::text("OK"), None)
                    .build();
            assert!(matches!(
                FormResponse::parse(Some("0".into())),
                FormResponse::Simple(0)
            ));
            assert_eq!(
                TransformationBuilder::new()
                    .uniform_scale(2.0)
                    .build()
                    .scale
                    .x,
                2.0
            );
            return Ok(b"helpers passed".to_vec());
        }
        // cancel_task is a synchronous import; the send is an async one and has to be awaited
        scheduler::cancel_task(0);
        ipc::send_ipc_message(sender, message)
            .await
            .map_err(|()| "ipc send failed".to_string())?
    }
}

register_plugin!(Fixture);
