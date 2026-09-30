use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use std::sync::Arc;

use crate::argument_types::entity_anchor::EntityAnchor;
use crate::errors::command_syntax_error::CommandSyntaxError;

pub trait ReturnValueCallable: Send + Sync {
    fn call(&self, value: ReturnValue);
}

pub type ReturnValueCallback = Arc<dyn ReturnValueCallable>;

#[derive(Clone, Default)]
pub struct ResultValueTaker(pub Vec<ReturnValueCallback>);

impl ResultValueTaker {
    #[must_use]
    pub fn merge(taker_1: &Self, taker_2: &Self) -> Self {
        let mut takers = Vec::with_capacity(taker_1.0.len() + taker_2.0.len());
        for taker in &taker_1.0 {
            takers.push(taker.clone());
        }
        for taker in &taker_2.0 {
            takers.push(taker.clone());
        }
        Self(takers)
    }

    #[must_use]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn call(&self, return_value: ReturnValue) {
        for callback in &self.0 {
            callback.call(return_value);
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ReturnValue {
    Success(i32),
    Failure,
}

impl ReturnValue {
    #[must_use]
    pub const fn success_value(self) -> bool {
        match self {
            Self::Success(_) => true,
            Self::Failure => false,
        }
    }

    #[must_use]
    pub const fn result_value(self) -> i32 {
        match self {
            Self::Success(value) => value,
            Self::Failure => 0,
        }
    }
}

/// Abstract command source trait for command dispatch and execution.
pub trait CommandSource: Clone + Send + Sync + 'static {
    /// Sends a feedback message to the command sender.
    fn send_message(&self, message: TextComponent);

    /// Sends an error message to the command sender.
    fn send_error(&self, error: TextComponent) {
        self.send_message(error);
    }

    /// Invokes callbacks for the command's return value.
    fn call_result(&self, _result: ReturnValue) {}

    /// Checks if this command source has the specified permission.
    fn has_permission(&self, _permission: &str) -> bool {
        true
    }

    /// Returns the position of this command source in the world.
    fn position(&self) -> Vector3<f64> {
        Vector3::default()
    }

    /// Returns the rotation (pitch, yaw) of this command source.
    fn rotation(&self) -> Vector2<f32> {
        Vector2::default()
    }

    /// Checks if the block at the given position is loaded.
    fn check_block_loaded(&self, _pos: &BlockPos) -> Result<(), CommandSyntaxError> {
        Ok(())
    }

    /// Returns the anchor position for the given entity anchor.
    fn anchor_position(&self, _anchor: EntityAnchor) -> Vector3<f64> {
        self.position()
    }

    /// Returns the current entity anchor of this command source.
    fn entity_anchor(&self) -> EntityAnchor {
        EntityAnchor::Feet
    }
}

#[derive(Clone, Default)]
pub struct DummySource {
    pub position: Vector3<f64>,
    pub rotation: Vector2<f32>,
    pub entity_anchor: EntityAnchor,
    pub command_result_taker: ResultValueTaker,
}

impl DummySource {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn dummy() -> Self {
        Self::default()
    }
}

impl CommandSource for DummySource {
    fn send_message(&self, _message: TextComponent) {}
    fn send_error(&self, _error: TextComponent) {}
    fn position(&self) -> Vector3<f64> {
        self.position
    }
    fn rotation(&self) -> Vector2<f32> {
        self.rotation
    }
    fn entity_anchor(&self) -> EntityAnchor {
        self.entity_anchor
    }
    fn call_result(&self, result: ReturnValue) {
        self.command_result_taker.call(result);
    }
}

impl CommandSource for () {
    fn send_message(&self, _message: TextComponent) {}
}

impl<S: CommandSource> CommandSource for Arc<S> {
    fn send_message(&self, message: TextComponent) {
        (**self).send_message(message);
    }

    fn send_error(&self, error: TextComponent) {
        (**self).send_error(error);
    }

    fn has_permission(&self, permission: &str) -> bool {
        (**self).has_permission(permission)
    }

    fn position(&self) -> Vector3<f64> {
        (**self).position()
    }

    fn rotation(&self) -> Vector2<f32> {
        (**self).rotation()
    }

    fn entity_anchor(&self) -> EntityAnchor {
        (**self).entity_anchor()
    }

    fn anchor_position(&self, anchor: EntityAnchor) -> Vector3<f64> {
        (**self).anchor_position(anchor)
    }

    fn call_result(&self, result: ReturnValue) {
        (**self).call_result(result);
    }

    fn check_block_loaded(&self, pos: &BlockPos) -> Result<(), CommandSyntaxError> {
        (**self).check_block_loaded(pos)
    }
}
