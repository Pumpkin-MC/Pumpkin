use std::pin::Pin;
use std::sync::Arc;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;
use pumpkin_util::text::color::{Color, NamedColor};
use pumpkin_util::text::TextComponent;
use crate::command::CommandSender;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;

pub const REQUIRES_PLAYER: CommandErrorType<0> = CommandErrorType::new("permissions.requires.player");
pub const REQUIRES_ENTITY: CommandErrorType<0> = CommandErrorType::new("permissions.requires.entity");

trait ReturnValueCallable: Send + Sync {
    fn call(&self, value: ReturnValue) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

pub type ReturnValueCallback = Arc<dyn ReturnValueCallable>;

/// Represents a collection of 'return value callbacks'.
#[derive(Clone)]
pub struct ResultValueTaker(pub Vec<ReturnValueCallback>);

impl ResultValueTaker {
    /// Merges two takers, returning one.
    pub fn merge(taker_1: &ResultValueTaker, taker_2: &ResultValueTaker) -> ResultValueTaker {
        let mut takers = Vec::with_capacity(taker_1.0.len() + taker_2.0.len());
        for taker in taker_1.0.iter() {
            takers.push(taker.clone());
        }
        for taker in taker_2.0.iter() {
            takers.push(taker.clone());
        }
        ResultValueTaker(takers)
    }

    /// Constructs a new, empty result value taker.
    pub fn new() -> ResultValueTaker {
        ResultValueTaker(Vec::new())
    }
}

/// Represents a source of a command, which
/// contains its own state, which could keep track of its:
/// - position
/// - rotation
/// - world
/// - permissions
/// - name
/// - display name
/// - the internal server,
/// - whether it is silent or no
/// - entity which it could represent
///
/// Not to be confused with [`CommandSender`], [`CommandSource`]
/// stores contextual information which could change due
/// to subcommands as it passes along the command.
///
/// A source having a player and a particular position does
/// not necessarily mean that the player does have that position;
/// but rather is a state for more complex functionality.
///
/// The `/execute` command heavily takes advantage of this source structure,
/// and can help you get a better understanding of this structure.
#[derive(Clone)]
pub struct CommandSource {
    pub output: CommandSender,
    pub world: Arc<World>,
    pub entity: Option<Arc<dyn EntityBase>>,
    pub position: Vector3<f64>,
    pub rotation: Vector2<f32>,
    // pub permission_predicate: TODO,
    pub name: String,
    pub display_name: TextComponent,
    pub server: Arc<Server>,
    pub silent: bool,
    pub command_result_taker: ResultValueTaker,
    pub entity_anchor: EntityAnchor
}

impl CommandSource {
    /// Returns a new [`CommandSource`] with the specified output and
    /// everything else from the `source` provided.
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
            entity_anchor: self.entity_anchor
        }
    }

    /// Returns a new [`CommandSource`] with the specified entity and
    /// everything else from the `source` provided.
    pub fn with_entity(self, entity: Option<Arc<dyn EntityBase>>) -> Self {
        Self {
            output: self.output,
            world: self.world,
            entity,
            position: self.position,
            rotation: self.rotation,
            name: self.name,
            display_name: self.display_name,
            server: self.server,
            silent: self.silent,
            command_result_taker: self.command_result_taker,
            entity_anchor: self.entity_anchor
        }
    }

    /// Returns a new [`CommandSource`] with the specified position and
    /// everything else from the `source` provided.
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
            entity_anchor: self.entity_anchor
        }
    }

    /// Returns a new [`CommandSource`] with the specified rotation and
    /// everything else from the `source` provided.
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
            entity_anchor: self.entity_anchor
        }
    }

    /// Returns a new [`CommandSource`] with the specified command result taker and
    /// everything else from the `source` provided.
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
            entity_anchor: self.entity_anchor
        }
    }

    /// Merges the given takers with this one, returning a new [`CommandSource`] with
    /// the merged taker.
    pub fn merge_command_result_taker(self, command_result_taker: &ResultValueTaker) -> Self {
        let merged = ResultValueTaker::merge(&self.command_result_taker, command_result_taker);
        self.with_command_result_taker(merged)
    }

    /// Returns a new [`CommandSource`] with the specified silent state and
    /// everything else from the `source` provided.
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
            entity_anchor: self.entity_anchor
        }
    }

    /// Returns a new [`CommandSource`] with the specified entity anchor and
    /// everything else from the `source` provided.
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
            entity_anchor
        }
    }

    /// Returns a new [`CommandSource`] with the specified world and
    /// everything else from the `source` provided.
    pub fn with_world(self, world: Arc<World>) -> Self {
        Self {
            output: self.output,
            world,
            entity: self.entity,
            position: self.position,
            rotation: self.rotation,
            name: self.name,
            display_name: self.display_name,
            server: self.server,
            silent: true,
            command_result_taker: self.command_result_taker,
            entity_anchor: self.entity_anchor
        }
    }

    /// Returns a new [`CommandSource`] with the rotation changed in such
    /// a way that the source faces the anchor of the entity and
    /// everything else from the `source` provided.
    pub fn with_looking_at_entity(self, entity: &Arc<dyn EntityBase>, anchor: EntityAnchor) -> Self {
        self.with_looking_at_pos(anchor.position_at_entity(entity))
    }

    /// Returns a new [`CommandSource`] with the rotation changed in such
    /// a way that the source faces the provided position and
    /// everything else from the `source` provided.
    pub fn with_looking_at_pos(self, pos: Vector3<f64>) -> Self {
        let source_pos = self.entity_anchor.position_at_source(&self);
        let delta = pos.sub(&source_pos);
        let horizontal_len = delta.horizontal_length();
        let pitch = -delta.y.atan2(horizontal_len).to_degrees();
        let yaw = delta.z.atan2(delta.x).to_degrees() - 90.0;
        self.with_rotation(
            Vector2::new(
                wrap_degrees(pitch as f32),
                wrap_degrees(yaw as f32)
            )
        )
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

    /// Gets the player as an option:
    ///
    /// - If this source actually contains a player, it returns that wrapped in a [`Some`].
    /// - If it doesn't, a [`None`] is returned instead.
    pub fn player_or_none(&self) -> Option<&Player> {
        self.entity
            .as_ref()
            .and_then(|entity| entity.get_player())
    }

    /// Gets the player as a result:
    ///
    /// - If this source actually contains a player, it returns that wrapped in an [`Ok`].
    /// - If it doesn't, a command error is provided instead, wrapped in an [`Err`].
    pub fn player_or_err(&self) -> Result<&Player, CommandSyntaxError> {
        self.player_or_none()
            .clone()
            .ok_or(REQUIRES_PLAYER.create_without_context())
    }

    /// Returns if the command was executed by a player.
    pub fn executed_by_player(&self) -> bool {
        self.player_or_none().is_some()
    }

    /// Sends a message to this source.
    pub async fn send_message(&self, message: TextComponent) {
        if !self.silent {
            self.output.send_message(message).await;
        }
    }

    /// Sends a message to all online operators.
    async fn send_to_ops(&self, message: TextComponent) {
        let text = TextComponent::translate("chat.type.admin", &[self.display_name.clone(), message])
            .color(Color::Named(NamedColor::Gray))
            .italic();
        if self.world.level_info.load().game_rules.send_command_feedback {
            let output_player = match &self.output {
                CommandSender::Player(sender) => Some(sender),
                _ => None
            };
            for player in self.server.get_all_players() {
                if output_player != Some(&player) {
                    if player.permission_lvl.load() >= self.server.basic_config.op_permission_level {
                        player.send_system_message(&text).await;
                    }
                }
            }
        }
    }

    /// Sends feedback to this source.
    pub async fn send_feedback(&self, message: TextComponent, broadcast_to_ops: bool) {
        if !self.silent {
            let should_send_to_output = self.output.should_receive_feedback();
            let should_send_to_ops = broadcast_to_ops && self.output.should_broadcast_console_to_ops();

            if should_send_to_output {
                self.output.send_message(message.clone()).await
            }
            if should_send_to_ops {
                self.send_to_ops(message).await
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
    pub async fn send_error(&self, error: TextComponent) {
        if !self.silent && self.output.should_track_output() {
            // TODO: Use `TextComponent::empty` instead of `TextComponent::text` when implemented
            self.output.send_message(
                TextComponent::text("")
                    .add_child(error)
                    .color(Color::Named(NamedColor::Red))
            ).await;
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum EntityAnchor {
    Feet,
    Eyes
}


// TODO: Move this to the /execute command when implemented.
impl EntityAnchor {
    /// Gets the [`EntityAnchor`] whose identity is the ID provided.
    pub fn from_id(id: &str) -> Option<EntityAnchor> {
        match id {
            "feet" => Some(EntityAnchor::Feet),
            "eyes" => Some(EntityAnchor::Eyes),
            _ => None
        }
    }

    /// Gets the ID of this [`EntityAnchor`]
    pub fn id(self) -> &'static str {
        match self {
            EntityAnchor::Feet => "feet",
            EntityAnchor::Eyes => "eyes",
        }
    }

    /// Gets the position of an entity with respect to this anchor.
    pub fn position_at_entity(self, entity: &Arc<dyn EntityBase>) -> Vector3<f64> {
        let entity = entity.get_entity();
        let mut pos = entity.pos.load();
        pos.y = entity.get_entity().get_eye_y();
        pos
    }

    /// Gets the position of a source with respect to this anchor.
    pub fn position_at_source(self, command_source: &CommandSource) -> Vector3<f64> {
        if let Some(entity) = &command_source.entity {
            self.position_at_entity(&entity)
        } else {
            command_source.position
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ReturnValue {
    Success(i32),
    Failure
}

impl ReturnValue {
    /// Get the success value of this return value.
    pub fn success_value(self) -> bool {
        match self {
            ReturnValue::Success(_) => true,
            ReturnValue::Failure => false
        }
    }

    /// Get the result integral value of this return value.
    pub fn result_value(self) -> i32 {
        match self {
            ReturnValue::Success(value) => value,
            ReturnValue::Failure => 0
        }
    }
}