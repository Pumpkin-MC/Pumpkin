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
pub mod client_suggestions;
pub mod commands;
pub mod dispatcher;
pub mod tree;

/// Represents the source of a command execution.
///
/// Different senders have different permissions, output targets, and
/// positions in the world. This enum abstracts those differences for the
/// command dispatcher.
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
    CommandBlock(Arc<dyn BlockEntity>, Arc<World>),
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
            }
        )
    }
}

impl CommandSender {
    pub async fn send_message(&self, text: TextComponent) {
        match self {
            Self::Console => log::info!("{}", text.to_pretty_console()),
            Self::Player(c) => c.send_system_message(&text).await,
            Self::Rcon(s) => s.lock().await.push(text.to_pretty_console()),
            Self::CommandBlock(block_entity, _) => {
                let command_entity: &CommandBlockEntity =
                    block_entity.as_any().downcast_ref().unwrap();
                let mut last_output = command_entity.last_output.lock().await;

                let now = time::OffsetDateTime::now_local().unwrap();
                let format = time::macros::format_description!("[hour]:[minute]:[second]");
                let timestamp = now.format(&format).unwrap();

                *last_output = format!("[{}] {}", timestamp, text.get_text());
            }
        }
    }

    pub fn set_success_count(&self, count: u32) {
        if let Self::CommandBlock(c, _) = self {
            let block: &CommandBlockEntity = c.as_any().downcast_ref().unwrap();
            block
                .success_count
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
            Self::CommandBlock(..) => PermissionLvl::Two,
        }
    }

    #[must_use]
    pub fn has_permission_lvl(&self, lvl: PermissionLvl) -> bool {
        match self {
            Self::Console | Self::Rcon(_) => true,
            Self::Player(p) => p.permission_lvl.load().ge(&lvl),
            Self::CommandBlock(..) => PermissionLvl::Two >= lvl,
        }
    }

    /// Check if the sender has a specific permission
    pub async fn has_permission(&self, server: &Server, node: &str) -> bool {
        match self {
            Self::Console | Self::Rcon(_) => true, // Console and RCON always have all permissions
            Self::Player(p) => p.has_permission(server, node).await,
            Self::CommandBlock(..) => {
                let perm_reg = server.permission_registry.read().await;
                let Some(p) = perm_reg.get_permission(node) else {
                    return false;
                };
                match p.default {
                    PermissionDefault::Allow => true,
                    PermissionDefault::Deny => false,
                    PermissionDefault::Op(o) => o <= PermissionLvl::Two,
                }
            }
        }
    }

    #[must_use]
    pub fn position(&self) -> Option<Vector3<f64>> {
        match self {
            Self::Console | Self::Rcon(..) => None,
            Self::Player(p) => Some(p.living_entity.entity.pos.load()),
            Self::CommandBlock(c, _) => Some(c.get_position().to_centered_f64()),
        }
    }

    #[must_use]
    pub fn world(&self) -> Option<Arc<World>> {
        match self {
            // TODO: maybe return first world when console
            Self::Console | Self::Rcon(..) => None,
            Self::Player(p) => Some(p.living_entity.entity.world.load_full()),
            Self::CommandBlock(_, w) => Some(w.clone()),
        }
    }

    pub async fn get_locale(&self) -> Locale {
        match self {
            Self::CommandBlock(..) | Self::Console | Self::Rcon(..) => Locale::EnUs, // Default locale for console and RCON
            Self::Player(player) => {
                Locale::from_str(&player.config.read().await.locale).unwrap_or(Locale::EnUs)
            }
        }
    }
}

pub type CommandResult<'a> = Pin<Box<dyn Future<Output = Result<(), CommandError>> + Send + 'a>>;

pub trait CommandExecutor: Sync + Send {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a>;
}
