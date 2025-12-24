use pumpkin_data::packet::CURRENT_MC_PROTOCOL;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_util::translation::get_translation_text;
use pumpkin_world::CURRENT_MC_VERSION;
use std::borrow::Cow;

use crate::command::CommandResult;
use crate::command::{CommandExecutor, CommandSender, args::ConsumedArgs, tree::CommandTree};

const NAMES: [&str; 2] = ["pumpkin", "version"];

const DESCRIPTION: &str = "Display information about Pumpkin.";

struct Executor;

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[allow(clippy::too_many_lines)]
impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let locale = sender.get_locale().await;
            sender
                .send_message(
                    TextComponent::custom(
                        "pumpkin",
                        "commands.pumpkin.version",
                        locale,
                        vec![TextComponent::text(CARGO_PKG_VERSION)],
                    )
                    .hover_event(HoverEvent::show_text(TextComponent::custom(
                        "pumpkin",
                        "commands.pumpkin.version.hover",
                        locale,
                        vec![],
                    )))
                    .click_event(ClickEvent::CopyToClipboard {
                        value: Cow::from(
                            get_translation_text(
                                "pumpkin:commands.pumpkin.version",
                                locale,
                                vec![TextComponent::text(CARGO_PKG_VERSION).0],
                            )
                            .replace('\n', ""),
                        ),
                    })
                    .color_named(NamedColor::Green)
                    .add_child(
                        TextComponent::custom(
                            "pumpkin",
                            "commands.pumpkin.description",
                            locale,
                            vec![],
                        )
                        .click_event(ClickEvent::CopyToClipboard {
                            value: Cow::from(
                                get_translation_text(
                                    "pumpkin:commands.pumpkin.description",
                                    locale,
                                    vec![],
                                )
                                .replace('\n', ""),
                            ),
                        })
                        .hover_event(HoverEvent::show_text(TextComponent::custom(
                            "pumpkin",
                            "commands.pumpkin.description.hover",
                            locale,
                            vec![],
                        )))
                        .color_named(NamedColor::White),
                    )
                    .add_child(
                        TextComponent::custom(
                            "pumpkin",
                            "commands.pumpkin.minecraft_version",
                            locale,
                            vec![
                                TextComponent::text(CURRENT_MC_VERSION),
                                TextComponent::text(format!("{CURRENT_MC_PROTOCOL}")),
                            ],
                        )
                        .click_event(ClickEvent::CopyToClipboard {
                            value: Cow::from(
                                get_translation_text(
                                    "pumpkin:commands.pumpkin.minecraft_version",
                                    locale,
                                    vec![
                                        TextComponent::text(CURRENT_MC_VERSION).0,
                                        TextComponent::text(format!("{CURRENT_MC_PROTOCOL}")).0,
                                    ],
                                )
                                .replace('\n', ""),
                            ),
                        })
                        .hover_event(HoverEvent::show_text(TextComponent::custom(
                            "pumpkin",
                            "commands.pumpkin.minecraft_version.hover",
                            locale,
                            vec![],
                        )))
                        .color_named(NamedColor::Gold),
                    )
                    // https://pumpkinmc.org/
                    .add_child(
                        TextComponent::custom("pumpkin", "commands.pumpkin.github", locale, vec![])
                            .click_event(ClickEvent::OpenUrl {
                                url: Cow::from("https://github.com/Pumpkin-MC/Pumpkin"),
                            })
                            .hover_event(HoverEvent::show_text(TextComponent::custom(
                                "pumpkin",
                                "commands.pumpkin.github.hover",
                                locale,
                                vec![],
                            )))
                            .color_named(NamedColor::Blue)
                            .bold()
                            .underlined(),
                    )
                    // Added docs. and a space for spacing
                    .add_child(TextComponent::text("  "))
                    .add_child(
                        TextComponent::custom(
                            "pumpkin",
                            "commands.pumpkin.website",
                            locale,
                            vec![],
                        )
                        .click_event(ClickEvent::OpenUrl {
                            url: Cow::from("https://pumpkinmc.org/"),
                        })
                        .hover_event(HoverEvent::show_text(TextComponent::custom(
                            "pumpkin",
                            "commands.pumpkin.website.hover",
                            locale,
                            vec![],
                        )))
                        .color_named(NamedColor::Blue)
                        .bold()
                        .underlined(),
                    ),
                )
                .await;
            Ok(())
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(Executor)
}
