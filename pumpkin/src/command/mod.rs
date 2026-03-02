use std::fmt;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;

use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use args::ConsumedArgs;

use dispatcher::CommandError;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::permission::{PermissionDefault, PermissionLvl};
use pumpkin_util::text::TextComponent;
use pumpkin_util::translation::Locale;
use pumpkin_world::block::entities::BlockEntity;
use pumpkin_world::block::entities::command_block::CommandBlockEntity;

pub mod args;
pub mod argument_builder;
pub mod argument_types;
pub mod client_suggestions;
pub mod commands;
pub mod context;
pub mod dispatcher;
pub mod errors;
pub mod node;
pub mod string_reader;
pub mod suggestion;
pub mod tree;

/// Represents the source of a command execution.
///
/// Different senders have different permissions, output targets, and
/// positions in the world. This enum abstracts those differences for the
/// command dispatcher.
#[derive(Clone)]
pub enum CommandSender {
    /// A remote console connection via the RCON protocol.
    ///
    /// Stores an asynchronous buffer to capture command output
    /// so it can be sent back over the network to the RCON client.
    Rcon(Arc<tokio::sync::Mutex<Vec<String>>>),
    /// The local server terminal/console.
    ///
    /// This sender typically has absolute permissions (bypass) and
    /// outputs directly to the server logs.
    Console,
    /// A player currently connected to the server.
    ///
    /// Contains a reference to the [Player] struct to access their
    /// location, permissions, and session.
    Player(Arc<Player>),
    /// A Command Block or Command Block Minecart.
    ///
    /// Contains the block entity responsible for the command and the
    /// world context it exists in for coordinate-relative execution (e.g., `~ ~ ~`).
    CommandBlock(Arc<CommandBlockEntity>, Arc<World>),
    /// Nothingness. Anything sent to this sender is void.
    /// Has the same permissions as that of `CommandBlock`.
    Dummy,
}

impl fmt::Display for CommandSender {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Console => "Server",
                Self::Rcon(_) => "Rcon",
                Self::Player(p) => &p.gameprofile.name,
                Self::CommandBlock(..) => "@",
                Self::Dummy => "",
            }
        )
    }
}

impl CommandSender {
    pub async fn send_message(&self, text: TextComponent) {
        match self {
            #[allow(clippy::print_stdout)]
            Self::Console => println!("{}", text.to_pretty_console()),
            Self::Player(c) => c.send_system_message(&text).await,
            Self::Rcon(s) => s.lock().await.push(text.to_pretty_console()),
            Self::CommandBlock(block_entity, _) => {
                let mut last_output = block_entity.last_output.lock().await;

                let now = time::OffsetDateTime::now_utc();
                let format = time::macros::format_description!("[hour]:[minute]:[second]");
                let timestamp = now.format(&format).unwrap();

                *last_output = format!("[{}] {}", timestamp, text.get_text());
            }
            Self::Dummy => {}
        }
    }

    pub fn set_success_count(&self, count: u32) {
        if let Self::CommandBlock(c, _) = self {
            c.success_count
                .store(count, std::sync::atomic::Ordering::SeqCst);
        }
    }

    #[must_use]
    pub const fn is_player(&self) -> bool {
        matches!(self, Self::Player(_))
    }

    #[must_use]
    pub const fn is_console(&self) -> bool {
        matches!(self, Self::Console)
    }
    #[must_use]
    pub fn as_player(&self) -> Option<Arc<Player>> {
        match self {
            Self::Player(player) => Some(player.clone()),
            _ => None,
        }
    }

    /// prefer using `has_permission_lvl(lvl)`
    #[must_use]
    pub fn permission_lvl(&self) -> PermissionLvl {
        match self {
            Self::Console | Self::Rcon(_) => PermissionLvl::Four,
            Self::Player(p) => p.permission_lvl.load(),
            Self::CommandBlock(..) | Self::Dummy => PermissionLvl::Two,
        }
    }

    #[must_use]
    pub fn has_permission_lvl(&self, lvl: PermissionLvl) -> bool {
        match self {
            Self::Console | Self::Rcon(_) => true,
            Self::Player(p) => p.permission_lvl.load().ge(&lvl),
            Self::CommandBlock(..) | Self::Dummy => PermissionLvl::Two >= lvl,
        }
    }

    pub async fn has_permission(
        &self,
        server: &Server,
        node: &str,
    ) -> (bool, Option<TextComponent>) {
        match self {
            Self::Console | Self::Rcon(_) => (true, None),
            Self::Player(p) => p.has_permission(server, node).await,
            Self::CommandBlock(..) | Self::Dummy => {
                let perm_reg = server.permission_registry.read().await;
                let Some(p) = perm_reg.get_permission(node) else {
                    return (false, None);
                };
                let allowed = match p.default {
                    PermissionDefault::Allow => true,
                    PermissionDefault::Deny => false,
                    PermissionDefault::Op(o) => o <= PermissionLvl::Two,
                };
                (allowed, None)
            }
        }
    }

    #[must_use]
    pub fn position(&self) -> Option<Vector3<f64>> {
        match self {
            Self::Console | Self::Rcon(..) | Self::Dummy => None,
            Self::Player(p) => Some(p.living_entity.entity.pos.load()),
            Self::CommandBlock(c, _) => Some(c.get_position().to_centered_f64()),
        }
    }

    #[must_use]
    pub fn world(&self) -> Option<Arc<World>> {
        match self {
            // TODO: maybe return first world when console
            Self::Console | Self::Rcon(..) | Self::Dummy => None,
            Self::Player(p) => Some(p.living_entity.entity.world.load_full()),
            Self::CommandBlock(_, w) => Some(w.clone()),
        }
    }

    #[must_use]
    pub fn get_locale(&self) -> Locale {
        match self {
            Self::CommandBlock(..) | Self::Console | Self::Rcon(..) | Self::Dummy => Locale::EnUs, // Default locale for console and RCON
            Self::Player(player) => {
                Locale::from_str(&player.config.load().locale).unwrap_or(Locale::EnUs)
            }
        }
    }

    #[must_use]
    pub fn should_receive_feedback(&self) -> bool {
        match self {
            Self::CommandBlock(_, world) => {
                world.level_info.load().game_rules.send_command_feedback
            }
            Self::Player(player) => {
                player
                    .world()
                    .level_info
                    .load()
                    .game_rules
                    .send_command_feedback
            }
            Self::Console | Self::Rcon(_) => true,
            Self::Dummy => false,
        }
    }

    #[must_use]
    pub fn should_broadcast_console_to_ops(&self) -> bool {
        match self {
            Self::CommandBlock(_, world) => world.level_info.load().game_rules.command_block_output,
            // TODO: should Console and Rcon be decided by server config?
            Self::Player(..) | Self::Console | Self::Rcon(_) => true,
            Self::Dummy => false,
        }
    }

    #[must_use]
    pub const fn should_track_output(&self) -> bool {
        match self {
            Self::Dummy => false,
            Self::Player(..) | Self::Console | Self::Rcon(_) | Self::CommandBlock(..) => true,
        }
    }
}

/// Represents the result of running a command after completion.
///
/// If the command **ran successfully**, an [`Ok`] is returned containing an [`i32`].
/// This represents the 'output value' of the command, which is *homologous* to the
/// `int` that command executors in vanilla return **upon success**.
///
/// **You should choose the successful result as `1` if**:
/// - you don't know what value to use for a success for your
///   own commands, or
/// - you don't understand what this value means, or
/// - you just simply don't care about this value at all
///
/// If the command **fails**, an [`Err`] is returned, containing the [`CommandError`]
/// that led to this result.
pub type CommandResult<'a> = Pin<Box<dyn Future<Output = Result<i32, CommandError>> + Send + 'a>>;

pub trait CommandExecutor: Sync + Send {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a>;
}
