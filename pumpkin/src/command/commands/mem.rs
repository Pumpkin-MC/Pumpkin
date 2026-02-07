use pumpkin_util::text::{TextComponent, color::NamedColor};
use memory_stats::memory_stats;

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
            // Get process memory usage (RSS)
            let used_mb = if let Some(usage) = memory_stats() {
                usage.physical_mem as f64 / 1024.0 / 1024.0
            } else {
                0.0
            };

            // Pre-calculate totals
            let mut total_chunks = 0;
            let mut total_players = 0;
            let mut total_entities = 0;
            let mut world_lines = Vec::new();

            for world_arc in server.worlds.load().iter() {
                let world = world_arc.clone();
                let c = world.level.loaded_chunk_count();
                let p = world.players.load().len();
                let e = world.entities.load().len();

                total_chunks += c;
                total_players += p;
                total_entities += e;

                let dim_name = if world.dimension == pumpkin_data::dimension::Dimension::OVERWORLD {
                    "Overworld"
                } else if world.dimension == pumpkin_data::dimension::Dimension::THE_NETHER {
                    "Nether"
                } else if world.dimension == pumpkin_data::dimension::Dimension::THE_END {
                    "End"
                } else {
                    "Unknown"
                };

                // Store formatted string for later
                world_lines.push(format!("{}: {}c {}p {}e", dim_name, c, p, e));
            }

            // Compact msg with totals
            let mut message = TextComponent::text("[")
                .color_named(NamedColor::Gray)
                .add_child(TextComponent::text(format!("RAM: {:.1}MB", used_mb)).color_named(NamedColor::Gold))
                .add_child(TextComponent::text(" | ").color_named(NamedColor::Gray))
                .add_child(TextComponent::text(format!("Chunks: {}", total_chunks)).color_named(NamedColor::Green))
                .add_child(TextComponent::text(" | ").color_named(NamedColor::Gray))
                .add_child(TextComponent::text(format!("Players: {}", total_players)).color_named(NamedColor::Aqua))
                .add_child(TextComponent::text(" | ").color_named(NamedColor::Gray))
                .add_child(TextComponent::text(format!("Entities: {}", total_entities)).color_named(NamedColor::Red))
                .add_child(TextComponent::text("]").color_named(NamedColor::Gray));

            // Append per-world details on new lines
            for line in world_lines {
                message = message
                    .add_child(TextComponent::text("\n > ").color_named(NamedColor::DarkGray))
                    .add_child(TextComponent::text(line).color_named(NamedColor::White));
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