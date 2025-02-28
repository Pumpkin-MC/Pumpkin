use async_trait::async_trait;
use pumpkin_protocol::CURRENT_MC_PROTOCOL;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use std::borrow::Cow;

use crate::{
    GIT_VERSION,
    command::{
        CommandError, CommandExecutor, CommandSender, args::ConsumedArgs, tree::CommandTree,
    },
    server::CURRENT_MC_VERSION,
};

const NAMES: [&str; 2] = ["pumpkin", "version"];

const DESCRIPTION: &str = "Display information about Pumpkin.";

struct Executor;

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const CARGO_PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[async_trait]
impl CommandExecutor for Executor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        sender
            .send_message(
                TextComponent::text(format!("Pumpkin {CARGO_PKG_VERSION} ({GIT_VERSION})\n"))
                    .hover_event(HoverEvent::show_text(TextComponent::text(Cow::from(
                        "Click to Copy Version",
                    ))))
                    .click_event(ClickEvent::CopyToClipboard(Cow::from(format!(
                        "Pumpkin {CARGO_PKG_VERSION} ({GIT_VERSION})"
                    ))))
                    .color_named(NamedColor::Green)
                    .add_child(
                        TextComponent::text(format!(
                            "{}\n{}\n",
                            &CARGO_PKG_DESCRIPTION[0..35],
                            &CARGO_PKG_DESCRIPTION[37..]
                        ))
                        .click_event(ClickEvent::CopyToClipboard(Cow::from(
                            CARGO_PKG_DESCRIPTION,
                        )))
                        .hover_event(HoverEvent::show_text(TextComponent::text(Cow::from(
                            "Click to Copy Description",
                        ))))
                        .color_named(NamedColor::White),
                    )
                    .add_child(
                        TextComponent::text(format!(
                            "(Minecraft {CURRENT_MC_VERSION}, Protocol {CURRENT_MC_PROTOCOL})\n"
                        ))
                        .click_event(ClickEvent::CopyToClipboard(Cow::from(format!(
                            "(Minecraft {CURRENT_MC_VERSION}, Protocol {CURRENT_MC_PROTOCOL})"
                        ))))
                        .hover_event(HoverEvent::show_text(TextComponent::text(Cow::from(
                            "Click to Copy Minecraft Version",
                        ))))
                        .color_named(NamedColor::Gold),
                    )
                    // https://pumpkinmc.org/
                    .add_child(
                        TextComponent::text("[Github Repository]")
                            .click_event(ClickEvent::OpenUrl(Cow::from(
                                "https://github.com/Pumpkin-MC/Pumpkin",
                            )))
                            .hover_event(HoverEvent::show_text(TextComponent::text(Cow::from(
                                "Click to open repository.",
                            ))))
                            .color_named(NamedColor::Blue)
                            .bold()
                            .underlined(),
                    )
                    // Added docs. and a space for spacing
                    .add_child(TextComponent::text("  "))
                    .add_child(
                        TextComponent::text("[Website]")
                            .click_event(ClickEvent::OpenUrl(Cow::from("https://pumpkinmc.org/")))
                            .hover_event(HoverEvent::show_text(TextComponent::text(Cow::from(
                                "Click to open website.",
                            ))))
                            .color_named(NamedColor::Blue)
                            .bold()
                            .underlined(),
                    ),
            )
            .await;
        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(Executor)
}
