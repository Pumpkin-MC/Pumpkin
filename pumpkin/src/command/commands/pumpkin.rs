use async_trait::async_trait;
use pumpkin_data::packet::CURRENT_MC_PROTOCOL;
use pumpkin_util::text::TextComponentBase;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::content::{NbtSource, TextContent};
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::text::style::Style;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_world::CURRENT_MC_VERSION;
use std::borrow::Cow;

use crate::command::{
    CommandError, CommandExecutor, CommandSender, args::ConsumedArgs, tree::CommandTree,
};
use crate::entity::EntityBase;
use crate::world::text::TextResolution;

const NAMES: [&str; 2] = ["pumpkin", "version"];

const DESCRIPTION: &str = "Display information about Pumpkin.";

struct Executor;

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[async_trait]
#[allow(clippy::too_many_lines)]
impl CommandExecutor for Executor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let player = sender.to_receiver();
        let uuid = player.as_ref().map(|p| p.get_entity().entity_uuid);
        sender
            .send_message(
                TextComponent::custom(
                    "pumpkin",
                    "commands.pumpkin.version",
                    vec![TextComponent::text(CARGO_PKG_VERSION)],
                )
                .hover_event(HoverEvent::show_text(TextComponent::custom(
                    "pumpkin",
                    "commands.pumpkin.version.hover",
                    vec![],
                )))
                .click_event(ClickEvent::CopyToClipboard {
                    value: TextComponent::custom(
                        "pumpkin",
                        "commands.pumpkin.version",
                        vec![TextComponent::text(CARGO_PKG_VERSION)],
                    )
                    .to_string(player, false)
                    .await
                    .replace('\n', "")
                    .into(),
                })
                .color_named(NamedColor::Green)
                .add_child(
                    TextComponent::custom("pumpkin", "commands.pumpkin.description", vec![])
                        .click_event(ClickEvent::CopyToClipboard {
                            value: TextComponent::custom(
                                "pumpkin",
                                "commands.pumpkin.description",
                                vec![],
                            )
                            .to_string(player, false)
                            .await
                            .replace('\n', "")
                            .into(),
                        })
                        .hover_event(HoverEvent::show_text(TextComponent::custom(
                            "pumpkin",
                            "commands.pumpkin.description.hover",
                            vec![],
                        )))
                        .color_named(NamedColor::White),
                )
                .add_child(
                    TextComponent::custom(
                        "pumpkin",
                        "commands.pumpkin.minecraft_version",
                        vec![
                            TextComponent::text(CURRENT_MC_VERSION),
                            TextComponent::text(format!("{CURRENT_MC_PROTOCOL}")),
                        ],
                    )
                    .click_event(ClickEvent::CopyToClipboard {
                        value: TextComponent::custom(
                            "pumpkin",
                            "commands.pumpkin.minecraft_version",
                            vec![
                                TextComponent::text(CURRENT_MC_VERSION),
                                TextComponent::text(format!("{CURRENT_MC_PROTOCOL}")),
                            ],
                        )
                        .to_string(player, false)
                        .await
                        .replace('\n', "")
                        .into(),
                    })
                    .hover_event(HoverEvent::show_text(TextComponent::custom(
                        "pumpkin",
                        "commands.pumpkin.minecraft_version.hover",
                        vec![],
                    )))
                    .color_named(NamedColor::Gold),
                )
                // https://pumpkinmc.org/
                .add_child(
                    TextComponent::custom("pumpkin", "commands.pumpkin.github", vec![])
                        .click_event(ClickEvent::OpenUrl {
                            url: Cow::from("https://github.com/Pumpkin-MC/Pumpkin"),
                        })
                        .hover_event(HoverEvent::show_text(TextComponent::custom(
                            "pumpkin",
                            "commands.pumpkin.github.hover",
                            vec![],
                        )))
                        .color_named(NamedColor::Blue)
                        .bold()
                        .underlined(),
                )
                // Added docs. and a space for spacing
                .add_text("  ")
                .add_child(
                    TextComponent::custom("pumpkin", "commands.pumpkin.website", vec![])
                        .click_event(ClickEvent::OpenUrl {
                            url: Cow::from("https://pumpkinmc.org/"),
                        })
                        .hover_event(HoverEvent::show_text(TextComponent::custom(
                            "pumpkin",
                            "commands.pumpkin.website.hover",
                            vec![],
                        )))
                        .color_named(NamedColor::Blue)
                        .bold()
                        .underlined(),
                )
                .add_text("\n\nScoreboard:\n")
                .add_child(TextComponent(TextComponentBase {
                    content: TextContent::Scoreboard {
                        score: pumpkin_util::text::content::ScoreboardValue {
                            name: "*".into(),
                            objective: "dummy".into(),
                            sender: None,
                        },
                    },
                    style: Style::default(),
                    extra: vec![],
                }))
                .add_text("\n\nMobs:\n")
                .add_child(TextComponent(TextComponentBase {
                    content: TextContent::EntityNames {
                        selector: "@e[limit=3]".into(),
                        separator: None,
                        sender: uuid,
                    },
                    style: Style::default(),
                    extra: vec![],
                }))
                .add_text("\n\nNbt (UUID):\n")
                .add_child(TextComponent(TextComponentBase {
                    content: TextContent::Nbt {
                        value: pumpkin_util::text::content::NbtValue {
                            source: Some(NbtSource::Entity),
                            nbt: "UUID".into(),
                            interpret: None,
                            separator: None,
                            block: None,
                            entity: Some("@s".into()),
                            storage: None,
                            sender: uuid,
                        },
                    },
                    style: Style::default(),
                    extra: vec![],
                })),
            )
            .await;
        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(Executor)
}
