use crate::command::argument_builder::{ArgumentBuilder, command};
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use pumpkin_data::translation::{CHAT_COPY_CLICK, COMMANDS_SEED_SUCCESS};
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use std::borrow::Cow;

const DESCRIPTION: &str = "Displays the world seed.";
const PERMISSION: &str = "minecraft:command.seed";

struct SeedCommandExecutor;

fn create_copy_on_click_text(content: String) -> TextComponent {
    TextComponent::translate(
        COMMANDS_SEED_SUCCESS,
        [TextComponent::wrap_in_square_brackets(
            TextComponent::text(content.clone())
                .hover_event(HoverEvent::show_text(TextComponent::translate(
                    CHAT_COPY_CLICK,
                    [],
                )))
                .click_event(ClickEvent::CopyToClipboard {
                    value: Cow::from(content),
                })
                .color_named(NamedColor::Green),
        )],
    )
}

impl CommandExecutor for SeedCommandExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let seed = context.world().level.seed.0;
            let seed_string = seed.to_string();

            context
                .source
                .send_feedback(create_copy_on_click_text(seed_string), false)
                .await;

            Ok(seed as i32)
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry
        .register_permission_or_panic(Permission::new(
            PERMISSION,
            DESCRIPTION,
            // For integrated servers, the permission level is 0,
            // but Pumpkin is always a dedicated server. For dedicated servers,
            // /seed is limited to level 2.
            PermissionDefault::Op(PermissionLvl::Two),
        ));

    dispatcher.register(
        command("seed", DESCRIPTION)
            .requires(PERMISSION)
            .executes(SeedCommandExecutor),
    );
}
