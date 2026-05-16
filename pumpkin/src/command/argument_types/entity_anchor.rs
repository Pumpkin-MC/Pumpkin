use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_source::CommandSource;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use crate::entity::Entity;
use pumpkin_data::translation;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;

pub const INVALID_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_ANCHOR_INVALID,
    translation::java::ARGUMENT_ANCHOR_INVALID,
);

pub struct EntityAnchorArgumentType;

impl ArgumentType for EntityAnchorArgumentType {
    type Item = EntityAnchor;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let i = reader.cursor();
        let anchor = reader.read_unquoted_string();
        EntityAnchor::from_id(anchor.as_str()).map_or_else(
            || {
                reader.set_cursor(i);
                Err(INVALID_ERROR_TYPE.create(reader, TextComponent::text(anchor)))
            },
            Ok,
        )
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::EntityAnchor
    }

    fn examples(&self) -> Vec<String> {
        examples!("eyes", "feet")
    }
}

impl_copy_get!(EntityAnchorArgumentType, EntityAnchor);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum EntityAnchor {
    Feet,
    Eyes,
}

impl EntityAnchor {
    /// Gets the [`EntityAnchor`] whose identity is the ID provided.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "feet" => Some(Self::Feet),
            "eyes" => Some(Self::Eyes),
            _ => None,
        }
    }

    /// Gets the ID of this [`EntityAnchor`]
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Feet => "feet",
            Self::Eyes => "eyes",
        }
    }

    fn transform_position(self, position: Vector3<f64>, entity: &Entity) -> Vector3<f64> {
        match self {
            Self::Feet => position,
            Self::Eyes => position.add(&Vector3::new(0.0, entity.get_eye_height(), 0.0)),
        }
    }

    /// Gets the position of an entity with respect to this anchor.
    pub fn position_at_entity(self, entity: &Entity) -> Vector3<f64> {
        self.transform_position(entity.pos.load(), entity)
    }

    /// Gets the position of a source with respect to this anchor.
    #[must_use]
    pub fn position_at_source(self, command_source: &CommandSource) -> Vector3<f64> {
        let pos = command_source.position;
        command_source
            .entity
            .as_ref()
            .map_or_else(|| pos, |e| self.position_at_entity(e.get_entity()))
    }
}
