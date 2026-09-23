use crate::command::CommandSender;
use crate::command::argument_types::entity_anchor::{EntityAnchor, EntityAnchorExt};
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use pumpkin_command::errors::command_syntax_error::CommandSyntaxError;
use pumpkin_command::errors::error_types::CommandErrorType;
pub use pumpkin_command::source::{
    ResultValueTaker, ReturnValue, ReturnValueCallable, ReturnValueCallback,
};
use pumpkin_data::translation;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::{Color, NamedColor};
use std::sync::Arc;

pub const REQUIRES_PLAYER: CommandErrorType<0> = CommandErrorType::new(
    translation::java::PERMISSIONS_REQUIRES_PLAYER,
    translation::java::PERMISSIONS_REQUIRES_PLAYER,
);
pub const REQUIRES_ENTITY: CommandErrorType<0> = CommandErrorType::new(
    translation::java::PERMISSIONS_REQUIRES_ENTITY,
    translation::java::PERMISSIONS_REQUIRES_ENTITY,
);

/// Represents a source of a command, which
/// contains its own state, which could keep track of its:
/// - position
/// - rotation
/// - world
/// - permissions
/// - name
/// - display name
/// - the internal server
/// - whether it is silent or not
/// - entity which it could represent
///
/// Not to be confused with [`CommandSender`], [`CommandSource`]
/// can be modified by commands to change how another
/// command works later in the command chain.
///
/// A source having a player and a particular position does
/// not necessarily mean that the player does have that position;
/// but rather is a state for more complex functionality.
#[derive(Clone)]
pub struct CommandSource {
    pub output: CommandSender,
    pub world: Option<Arc<World>>,
    pub entity: Option<Arc<dyn EntityBase>>,
    pub position: Vector3<f64>,
    pub rotation: Vector2<f32>,
    pub name: String,
    pub display_name: TextComponent,
    pub server: Option<Arc<Server>>,
    pub silent: bool,
    pub command_result_taker: ResultValueTaker,
    pub entity_anchor: EntityAnchor,
}

impl CommandSource {
    /// Creates a dummy [`CommandSource`], great for unit testing.
    ///
    /// # Note
    /// **This should only be used for unit tests!!!**
    ///
    /// The returned [`CommandSource`] does not contain
    /// a server or a world. If there is attempt to fetch the server or a world from
    /// the returned source, there will be a panic!
    #[must_use]
    // We use it in tests
    #[allow(dead_code)]
    pub(crate) fn dummy() -> Self {
        Self {
            output: CommandSender::Dummy,
            world: None,
            entity: None,
            position: Vector3::default(),
            rotation: Vector2::default(),
            name: String::new(),
            display_name: TextComponent::empty(),
            server: None,
            silent: false,
            command_result_taker: ResultValueTaker::new(),
            entity_anchor: EntityAnchor::Feet,
        }
    }

    /// Creates a usable [`CommandSource`] for running commands in an actual environment.
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        output: CommandSender,
        world: Arc<World>,
        entity: Option<Arc<dyn EntityBase>>,
        position: Vector3<f64>,
        rotation: Vector2<f32>,
        name: String,
        display_name: TextComponent,
        server: Arc<Server>,
    ) -> Self {
        Self {
            output,
            world: Some(world),
            entity,
            position,
            rotation,
            name,
            display_name,
            server: Some(server),
            silent: false,
            command_result_taker: ResultValueTaker(Vec::new()),
            entity_anchor: EntityAnchor::Feet,
        }
    }

    /// Returns a new [`CommandSource`] with the specified output and
    /// everything else from the `source` provided.
    #[must_use]
    pub fn with_output(self, output: CommandSender) -> Self {
        Self {
            output,
            world: self.world,
            entity: self.entity,
            position: self.position,
            rotation: self.rotation,
            name: self.name,
            display_name: self.display_name,
            server: self.server,
            silent: self.silent,
            command_result_taker: self.command_result_taker,
            entity_anchor: self.entity_anchor,
        }
    }

    /// Returns a new [`CommandSource`] with the specified world and
    /// everything else from the `source` provided.
    #[must_use]
    pub fn with_world(self, world: Arc<World>) -> Self {
        Self {
            output: self.output,
            world: Some(world),
            entity: self.entity,
            position: self.position,
            rotation: self.rotation,
            name: self.name,
            display_name: self.display_name,
            server: self.server,
            silent: true,
            command_result_taker: self.command_result_taker,
            entity_anchor: self.entity_anchor,
        }
    }

    /// Returns a new [`CommandSource`] with the specified entity and
    /// everything else from the `source` provided.
    #[must_use]
    pub fn with_entity(self, entity: Arc<dyn EntityBase>) -> Self {
        let name = entity.get_name().get_text();
        let display_name = entity.get_display_name();
        Self {
            output: self.output,
            world: self.world,
            entity: Some(entity),
            position: self.position,
            rotation: self.rotation,
            name,
            display_name,
            server: self.server,
            silent: self.silent,
            command_result_taker: self.command_result_taker,
            entity_anchor: self.entity_anchor,
        }
    }

    /// Returns a new [`CommandSource`] with the specified position and
    /// everything else from the `source` provided.
    #[must_use]
    pub fn with_position(self, position: Vector3<f64>) -> Self {
        Self {
            output: self.output,
            world: self.world,
            entity: self.entity,
            position,
            rotation: self.rotation,
            name: self.name,
            display_name: self.display_name,
            server: self.server,
            silent: self.silent,
            command_result_taker: self.command_result_taker,
            entity_anchor: self.entity_anchor,
        }
    }

    /// Returns a new [`CommandSource`] with the specified rotation and
    /// everything else from the `source` provided.
    #[must_use]
    pub fn with_rotation(self, rotation: Vector2<f32>) -> Self {
        Self {
            output: self.output,
            world: self.world,
            entity: self.entity,
            position: self.position,
            rotation,
            name: self.name,
            display_name: self.display_name,
            server: self.server,
            silent: self.silent,
            command_result_taker: self.command_result_taker,
            entity_anchor: self.entity_anchor,
        }
    }

    /// Merges the given takers with this one, returning a new [`CommandSource`] with
    /// the merged taker.
    #[must_use]
    pub fn merge_command_result_taker(self, command_result_taker: &ResultValueTaker) -> Self {
        let merged = ResultValueTaker::merge(&self.command_result_taker, command_result_taker);
        self.with_command_result_taker(merged)
    }

    /// Returns a new [`CommandSource`] with the specified silent state and
    /// everything else from the `source` provided.
    #[must_use]
    pub fn with_silent(self) -> Self {
        Self {
            output: self.output,
            world: self.world,
            entity: self.entity,
            position: self.position,
            rotation: self.rotation,
            name: self.name,
            display_name: self.display_name,
            server: self.server,
            silent: true,
            command_result_taker: self.command_result_taker,
            entity_anchor: self.entity_anchor,
        }
    }

    /// Returns a new [`CommandSource`] with the specified command result taker and
    /// everything else from the `source` provided.
    #[must_use]
    pub fn with_command_result_taker(self, command_result_taker: ResultValueTaker) -> Self {
        Self {
            output: self.output,
            world: self.world,
            entity: self.entity,
            position: self.position,
            rotation: self.rotation,
            name: self.name,
            display_name: self.display_name,
            server: self.server,
            silent: self.silent,
            command_result_taker,
            entity_anchor: self.entity_anchor,
        }
    }

    /// Returns a new [`CommandSource`] with the specified entity anchor and
    /// everything else from the `source` provided.
    #[must_use]
    pub fn with_entity_anchor(self, entity_anchor: EntityAnchor) -> Self {
        Self {
            output: self.output,
            world: self.world,
            entity: self.entity,
            position: self.position,
            rotation: self.rotation,
            name: self.name,
            display_name: self.display_name,
            server: self.server,
            silent: true,
            command_result_taker: self.command_result_taker,
            entity_anchor,
        }
    }

    /// Returns a new [`CommandSource`] with the rotation changed in such
    /// a way that the source faces the anchor of the entity and
    /// everything else from the `source` provided.
    #[must_use]
    pub fn with_looking_at_entity(
        self,
        entity: &Arc<dyn EntityBase>,
        anchor: EntityAnchor,
    ) -> Self {
        self.with_looking_at_pos(anchor.position_at_entity(entity.get_entity()))
    }

    /// Returns a new [`CommandSource`] with the rotation changed in such
    /// a way that the source faces the provided position and
    /// everything else from the `source` provided.
    #[must_use]
    pub fn with_looking_at_pos(self, pos: Vector3<f64>) -> Self {
        let source_pos = self.entity_anchor.position_at_source(&self);
        let delta = pos.sub(&source_pos);
        let horizontal_len = delta.horizontal_length();
        let pitch = -delta.y.atan2(horizontal_len).to_degrees();
        let yaw = delta.z.atan2(delta.x).to_degrees() - 90.0;
        self.with_rotation(Vector2::new(
            wrap_degrees(pitch as f32),
            wrap_degrees(yaw as f32),
        ))
    }

    /// Gets the entity as a result:
    ///
    /// - If this source actually contains an entity, it returns that wrapped in an [`Ok`].
    /// - If it doesn't, a command error is provided instead, wrapped in an [`Err`].
    pub fn entity_or_err(&self) -> Result<Arc<dyn EntityBase>, CommandSyntaxError> {
        self.entity
            .clone()
            .ok_or(REQUIRES_ENTITY.create_without_context())
    }

    /// Gets the world as a result:
    ///
    /// - If this source actually contains a server, it returns that.
    /// - If it doesn't, this function **panics**. Ideally, a source should contain a world, but it may not in a unit test.
    #[must_use]
    pub const fn world(&self) -> &Arc<World> {
        self.world.as_ref().expect("Expected world to exist")
    }

    /// Gets the server as a result:
    ///
    /// - If this source actually contains a server, it returns that.
    /// - If it doesn't, this function **panics**. Ideally, a source should contain the server, but it may not in a unit test.
    #[must_use]
    pub const fn server(&self) -> &Arc<Server> {
        self.server.as_ref().expect("Expected server to exist")
    }

    /// Gets the player as an option:
    ///
    /// - If this source actually contains a player, it returns that wrapped in a [`Some`].
    /// - If it doesn't, a [`None`] is returned instead.
    #[must_use]
    pub fn player_or_none(&self) -> Option<&Player> {
        self.entity.as_ref().and_then(|entity| entity.get_player())
    }

    /// Gets the player as an `Arc<Player>` from the underlying output sender.
    #[must_use]
    pub fn as_player(&self) -> Option<Arc<Player>> {
        self.output.as_player()
    }

    /// Gets the player as a result:
    ///
    /// - If this source actually contains a player, it returns that wrapped in an [`Ok`].
    /// - If it doesn't, a command error is provided instead, wrapped in an [`Err`].
    pub fn player_or_err(&self) -> Result<&Player, CommandSyntaxError> {
        self.player_or_none()
            .ok_or(REQUIRES_PLAYER.create_without_context())
    }

    /// Returns if the command was executed by a player.
    #[must_use]
    pub fn executed_by_player(&self) -> bool {
        self.player_or_none().is_some()
    }

    /// Sends a message to this source.
    pub fn send_message(&self, message: TextComponent) {
        if !self.silent {
            self.output.send_message(message);
        }
    }

    /// Sends a message to all online operators.
    fn send_to_ops(&self, message: TextComponent) {
        let text = TextComponent::translate_cross(
            "chat.type.admin",
            "chat.type.admin",
            &[self.display_name.clone(), message],
        )
        .color(Color::Named(NamedColor::Gray))
        .italic();
        let Some(server) = &self.server else {
            return;
        };
        if server.level_info.load().game_rules.send_command_feedback {
            let output_player = match &self.output {
                CommandSender::Player(sender) => Some(sender),
                _ => None,
            };
            for player in server.get_all_players() {
                if output_player != Some(&player)
                    && player.permission_lvl.load() >= server.basic_config.op_permission_level
                {
                    player.send_system_message(&text);
                }
            }
        }
    }

    /// Sends feedback to this source.
    pub fn send_feedback(&self, message: TextComponent, broadcast_to_ops: bool) {
        if !self.silent {
            let should_send_to_output = self.output.should_receive_feedback();
            let should_send_to_ops =
                broadcast_to_ops && self.output.should_broadcast_console_to_ops();

            if should_send_to_output {
                self.output.send_message(message.clone());
            }
            if should_send_to_ops {
                self.send_to_ops(message);
            }
        }
    }

    /// Sends an error message to the console.
    ///
    /// # Note
    /// Do not use this function if you want to report a [`CommandSyntaxError`].
    /// Instead, wrap the error in an [`Err`] and return that (or use the `?` operator)
    ///
    /// However, there are still use cases of this function to send an error
    /// without reporting command failure directly.
    pub fn send_error(&self, error: TextComponent) {
        if !self.silent && self.output.should_track_output() {
            self.output.send_message(
                TextComponent::empty()
                    .add_child(error)
                    .color(Color::Named(NamedColor::Red)),
            );
        }
    }

    /// Returns whether this source has the permission provided.
    ///
    /// # Panics
    ///
    /// Panics if this source does not have a reference to the
    /// server (i.e. this is a dummy [`CommandSource`].)
    #[must_use]
    pub fn has_permission(&self, permission: &str) -> bool {
        self.server.as_ref().map_or(
            matches!(
                self.output,
                crate::command::CommandSender::Console
                    | crate::command::CommandSender::Rcon(_)
                    | crate::command::CommandSender::Dummy
            ),
            |server| self.output.has_permission(server, permission),
        )
    }

    /// Returns whether this source has the permission provided.
    ///
    /// # Panics
    ///
    /// Panics **if, and only if** both the following conditions are met:
    ///
    /// - permission is not [`None`].
    /// - this source does not have a reference to the server (i.e. this is a dummy [`CommandSource`].)
    #[must_use]
    pub fn has_permission_from_option(&self, permission: Option<&str>) -> bool {
        permission.is_none_or(|permission| self.has_permission(permission))
    }
}

impl pumpkin_command::source::CommandSource for CommandSource {
    fn send_message(&self, message: TextComponent) {
        self.send_message(message);
    }

    fn send_error(&self, error: TextComponent) {
        self.send_error(error);
    }

    fn call_result(&self, result: ReturnValue) {
        self.command_result_taker.call(result);
    }

    fn has_permission(&self, permission: &str) -> bool {
        self.has_permission(permission)
    }

    fn position(&self) -> Vector3<f64> {
        self.position
    }

    fn rotation(&self) -> Vector2<f32> {
        self.rotation
    }

    fn check_block_loaded(&self, pos: &BlockPos) -> Result<(), CommandSyntaxError> {
        let world = self.world();
        if world
            .level
            .read_chunk_sync(&pos.chunk_position(), |_| ())
            .is_none()
        {
            Err(
                pumpkin_command::argument_types::coordinates::block_pos::NOT_LOADED_ERROR_TYPE
                    .create_without_context(),
            )
        } else if !world.is_in_build_limit(*pos) {
            Err(
                pumpkin_command::argument_types::coordinates::block_pos::OUT_OF_WORLD_ERROR_TYPE
                    .create_without_context(),
            )
        } else {
            Ok(())
        }
    }

    fn entity_anchor(&self) -> EntityAnchor {
        self.entity_anchor
    }

    fn anchor_position(&self, anchor: EntityAnchor) -> Vector3<f64> {
        let pos = self.position;
        self.entity
            .as_ref()
            .map_or_else(|| pos, |e| anchor.position_at_entity(e.get_entity()))
    }
}
