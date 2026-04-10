use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::argument_types::coordinates::Coordinates;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use crate::world::World;
use pumpkin_data::translation;
use pumpkin_util::math::position::BlockPos;

pub const NOT_LOADED_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_POS_UNLOADED);
pub const OUT_OF_WORLD_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_POS_OUTOFWORLD);
pub const OUT_OF_BOUNDS_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_POS_OUTOFBOUNDS);

/// An argument type for a 3-dimensional vector representing a block position.
///
/// The parsed [`Coordinates`] can be converted to a [`BlockPos`] via one of the
/// following associated methods:
/// - [`BlockPosArgumentType::get_block_pos`]: Normal conversion.
/// - [`BlockPosArgumentType::get_loaded_block_pos`]: Converts the coordinates to a *loaded* `BlockPos`.
///   This is what you want to use most of the time (if you need to update the loaded position somehow).
/// - [`BlockPosArgumentType::get_loaded_block_pos_in_world`]: Converts the coordinates to a *loaded* `BlockPos`
///   in the provided world.
/// - [`BlockPosArgumentType::get_spawnable_pos`]: Converts the coordinates to a `BlockPos` where
///   a player can spawn.
pub struct BlockPosArgumentType;

impl ArgumentType for BlockPosArgumentType {
    type Item = Coordinates;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        if reader.peek() == Some('^') {
            Coordinates::parse_local(reader)
        } else {
            Coordinates::parse_world_integers(reader)
        }
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::Vec3
    }

    fn examples(&self) -> Vec<String> {
        examples!("1 3 5", "-3 ~24 ~-1", "80 80 80", "^ ^9 ^56")
    }
}

impl BlockPosArgumentType {
    /// Returns a [`CommandContext`]'s parsed coordinate argument in the form of a [`BlockPos`].
    ///
    /// # Arguments
    /// * `context` - The [`CommandContext`] that has the parsed `Coordinates` with the provided argument name.
    /// * `name` - The name of the argument that was parsed.
    ///
    /// # Returns
    /// The loaded `BlockPos` containing the position represented by the parsed argument, wrapped in an `Ok`,
    /// or an `Err` with the appropriate [`CommandSyntaxError`] if it could not be resolved.
    pub fn get_block_pos(
        context: &CommandContext,
        name: &str,
    ) -> Result<BlockPos, CommandSyntaxError> {
        Ok(BlockPos::floored_v(
            context
                .get_argument::<Coordinates>(name)?
                .resolve(context.source.as_ref()),
        ))
    }

    /// Returns a [`CommandContext`]'s parsed coordinate argument in the form of a loaded [`BlockPos`].
    ///
    /// # Arguments
    /// * `context` - The [`CommandContext`] that has the parsed `Coordinates` with the provided argument name.
    ///   Its `CommandSource` also decides the world to check the load status in.
    /// * `name` - The name of the argument that was parsed.
    ///
    /// # Returns
    /// The loaded `BlockPos` containing the position represented by the parsed argument, wrapped in an `Ok`,
    /// or an `Err` with the appropriate [`CommandSyntaxError`] if it could not be resolved.
    pub fn get_loaded_block_pos(
        context: &CommandContext,
        name: &str,
    ) -> Result<BlockPos, CommandSyntaxError> {
        Self::get_loaded_block_pos_in_world(context, name, context.source.world())
    }

    /// Returns a [`CommandContext`]'s parsed coordinate argument in the form of a loaded [`BlockPos`]
    /// in the provided world.
    ///
    /// # Arguments
    /// * `context` - The [`CommandContext`] that has the parsed `Coordinates` with the provided argument name.
    /// * `name` - The name of the argument that was parsed.
    /// * `world` - The world to check the load status in.
    ///
    /// # Returns
    /// The loaded `BlockPos` containing the position represented by the parsed argument, wrapped in an `Ok`,
    /// or an `Err` with the appropriate [`CommandSyntaxError`] if it could not be resolved.
    pub fn get_loaded_block_pos_in_world(
        context: &CommandContext,
        name: &str,
        world: &World,
    ) -> Result<BlockPos, CommandSyntaxError> {
        let pos = Self::get_block_pos(context, name)?;
        if world.level.try_get_chunk(&pos.chunk_position()).is_none() {
            Err(NOT_LOADED_ERROR_TYPE.create_without_context())
        } else if !world.is_in_build_limit(pos) {
            Err(OUT_OF_WORLD_ERROR_TYPE.create_without_context())
        } else {
            Ok(pos)
        }
    }

    /// Returns a [`CommandContext`]'s parsed coordinate argument in the form of a [`BlockPos`]
    /// where players can spawn.
    ///
    /// # Arguments
    /// * `context` - The [`CommandContext`] that has the parsed `Coordinates` with the provided argument name.
    /// * `name` - The name of the argument that was parsed.
    ///
    /// # Returns
    /// The loaded `BlockPos` containing the position represented by the parsed argument, wrapped in an `Ok`,
    /// or an `Err` with the appropriate [`CommandSyntaxError`] if it could not be resolved.
    pub fn get_spawnable_pos(
        context: &CommandContext,
        name: &str,
    ) -> Result<BlockPos, CommandSyntaxError> {
        let pos = Self::get_block_pos(context, name)?;
        if World::is_valid(pos) {
            Ok(pos)
        } else {
            Err(OUT_OF_BOUNDS_ERROR_TYPE.create_without_context())
        }
    }
}
