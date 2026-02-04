use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_util::translation::get_translation_text;
use serde::Deserialize;
use std::borrow::Cow;

use crate::command::CommandResult;
use crate::command::{CommandExecutor, CommandSender, args::ConsumedArgs, tree::CommandTree};

const NAMES: [&str; 2] = ["pumpkin", "version"];

const DESCRIPTION: &str = "Display information about Pumpkin.";

struct Executor;

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_HASH: &str = env!("GIT_HASH");
const GIT_HASH_FULL: &str = env!("GIT_HASH_FULL");

#[derive(Deserialize)]
struct Contributor {
    login: String,
}

fn fetch_all_contributors() -> Vec<Contributor> {
    let mut all_contributors = Vec::new();
    let mut next_url = Some(
        "https://api.github.com/repos/Pumpkin-MC/Pumpkin/contributors?per_page=100".to_string(),
    );

    while let Some(url) = next_url {
        let response = ureq::get(&url).header("User-Agent", "Pumpkin-MC").call();

        match response {
            Ok(mut res) => {
                if let Ok(contributors) = res.body_mut().read_json::<Vec<Contributor>>() {
                    all_contributors.extend(contributors);
                } else {
                    break;
                }
                let link_header = res.headers().get("link").map(|s| s.to_str().unwrap_or(""));

                next_url = link_header.and_then(extract_next_url);
            }
            Err(_) => break,
        }
    }

    if all_contributors.is_empty() {
        return vec![];
    }

    all_contributors
}

fn extract_next_url(header: &str) -> Option<String> {
    header
        .split(',')
        .find(|part| part.contains("rel=\"next\""))
        .and_then(|part| {
            let start = part.find('<')? + 1;
            let end = part.find('>')?;
            Some(part[start..end].to_string())
        })
}

#[expect(clippy::too_many_lines)]
impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let contributors = fetch_all_contributors();
            let contributor_names = contributors
                .iter()
                .map(|c| c.login.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            let locale = sender.get_locale();
            let version_string = format!(
                "{} (Commit: {}) - {} Contributors",
                CARGO_PKG_VERSION,
                GIT_HASH,
                contributors.len()
            );
            sender
                .send_message(
                    TextComponent::custom(
                        "pumpkin",
                        "commands.pumpkin.version",
                        locale,
                        vec![TextComponent::text(version_string.clone())],
                    )
                    .hover_event(HoverEvent::show_text(
                        TextComponent::text(format!("Commit: {GIT_HASH_FULL}\n\nContributors:\n",))
                            .add_child(
                                TextComponent::text(contributor_names)
                                    .gradient_named(&[NamedColor::DarkGreen, NamedColor::Green])
                                    .new_line(),
                            ),
                    ))
                    .click_event(ClickEvent::CopyToClipboard {
                        value: Cow::from(
                            get_translation_text(
                                "pumpkin:commands.pumpkin.version",
                                locale,
                                vec![TextComponent::text(version_string).0],
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
                                TextComponent::text(CURRENT_MC_VERSION.to_string()),
                                TextComponent::text(
                                    CURRENT_MC_VERSION.protocol_version().to_string(),
                                ),
                            ],
                        )
                        .click_event(ClickEvent::CopyToClipboard {
                            value: Cow::from(
                                get_translation_text(
                                    "pumpkin:commands.pumpkin.minecraft_version",
                                    locale,
                                    vec![
                                        TextComponent::text(CURRENT_MC_VERSION.to_string()).0,
                                        TextComponent::text(
                                            CURRENT_MC_VERSION.protocol_version().to_string(),
                                        )
                                        .0,
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
                    // Spacing
                    .add_child(TextComponent::text("  "))
                    .add_child(
                        TextComponent::text("[Donate]")
                            .click_event(ClickEvent::OpenUrl {
                                url: Cow::from("https://github.com/sponsors/Snowiiii"),
                            })
                            .hover_event(HoverEvent::show_text(TextComponent::text(
                                "Click to open Donate",
                            )))
                            .rainbow()
                            .bold()
                            .underlined(),
                    )
                    // Spacing
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

            // It makes total sense to return the number of
            // contributors as the i32 result for this command.
            Ok(contributors.len() as i32)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(Executor)
}
