use memory_stats::memory_stats;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use std::borrow::Cow;

use crate::command::CommandResult;
use crate::command::{CommandExecutor, CommandSender, args::ConsumedArgs, tree::CommandTree};

const NAMES: [&str; 3] = ["mem", "memory", "chunks"];
const DESCRIPTION: &str = "Display process memory and chunk usage.";

struct Executor;

impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let used_mb = memory_stats().map_or(0.0, |u| u.physical_mem as f64 / 1024.0 / 1024.0);

            let mut world_stats = Vec::new();
            let mut total = (0, 0, 0); // (Chunks, Players, Entities)

            for world in server.worlds.load().iter() {
                let c = world.level.loaded_chunk_count();
                let p = world.players.load().len();
                let e = world.entities.load().len();

                total.0 += c;
                total.1 += p;
                total.2 += e;

                let dimension = if world.dimension == pumpkin_data::dimension::Dimension::OVERWORLD
                {
                    "NORMAL"
                } else if world.dimension == pumpkin_data::dimension::Dimension::THE_NETHER {
                    "NETHER"
                } else if world.dimension == pumpkin_data::dimension::Dimension::THE_END {
                    "END"
                } else {
                    "UNKNOWN"
                };

                let display_name =
                    format!("{} ({})", world.level_info.load().level_name, dimension);
                world_stats.push((display_name, c, p, e));
            }

            let memory_text = if used_mb >= 1024.0 {
                format!("{:.2} GB", used_mb / 1024.0) // 2 decimals for GB precision
            } else {
                format!("{used_mb:.1} MB")
            };

            let mut message = TextComponent::text("Memory Usage - ")
                .add_child(
                    TextComponent::text("[Something Wrong?]\n")
                        .click_event(ClickEvent::OpenUrl {
                            url: Cow::from("https://github.com/Pumpkin-MC/Pumpkin/issues/"),
                        })
                        .hover_event(HoverEvent::show_text(TextComponent::text(
                            "Report an issue on GitHub!",
                        )))
                        .color_named(NamedColor::Blue)
                        .bold()
                        .underlined(),
                )
                .color_named(NamedColor::White)
                .add_child(TextComponent::text("[").color_named(NamedColor::Gray))
                .add_child(TextComponent::text(memory_text).color_named(NamedColor::Gold))
                .add_child(TextComponent::text(" | ").color_named(NamedColor::Gray))
                .add_child(
                    TextComponent::text(format!("Chunks: {}", total.0))
                        .color_named(NamedColor::Green),
                )
                .add_child(TextComponent::text(" | ").color_named(NamedColor::Gray))
                .add_child(
                    TextComponent::text(format!("Players: {}", total.1))
                        .color_named(NamedColor::Aqua),
                )
                .add_child(TextComponent::text(" | ").color_named(NamedColor::Gray))
                .add_child(
                    TextComponent::text(format!("Entities: {}", total.2))
                        .color_named(NamedColor::Red),
                )
                .add_child(TextComponent::text("]").color_named(NamedColor::Gray));

            for (name, c, p, e) in world_stats {
                message = message
                    .add_child(TextComponent::text("\n > ").color_named(NamedColor::DarkGray))
                    .add_child(
                        TextComponent::text(format!("{name}: ")).color_named(NamedColor::Yellow),
                    )
                    .add_child(TextComponent::text(format!("{c}c")).color_named(NamedColor::Green))
                    .add_child(TextComponent::text(" | ").color_named(NamedColor::Gray))
                    .add_child(TextComponent::text(format!("{p}p")).color_named(NamedColor::Aqua))
                    .add_child(TextComponent::text(" | ").color_named(NamedColor::Gray))
                    .add_child(TextComponent::text(format!("{e}e")).color_named(NamedColor::Red));
            }

            sender.send_message(message).await;
            Ok(1)
        })
    }
}

#[must_use]
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(Executor)
}
