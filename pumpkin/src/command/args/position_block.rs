use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;

use crate::command::CommandSender;
use crate::command::args::ConsumeResult;
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::server::Server;
use crate::world::World;

use super::super::args::ArgumentConsumer;
use super::coordinate::MaybeRelativeBlockCoordinate;
use super::{Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

/// x, y and z coordinates
pub struct BlockPosArgumentConsumer;

impl GetClientSideArgParser for BlockPosArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::BlockPos
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for BlockPosArgumentConsumer {
    fn consume<'a, 'b>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let x_str_opt = args.pop().map(|arg| arg.value);
        let y_str_opt = args.pop().map(|arg| arg.value);
        let z_str_opt = args.pop().map(|arg| arg.value);

        let (Some(x_str), Some(y_str), Some(z_str)) = (x_str_opt, y_str_opt, z_str_opt) else {
            return Box::pin(async move { None });
        };

        let result: Option<Arg<'a>> = MaybeRelativeBlockPos::try_new(x_str, y_str, z_str)
            .and_then(|pos| pos.try_to_absolute(sender))
            .map(Arg::BlockPos);

        Box::pin(async move { result })
    }
}

enum MaybeRelativeBlockPos {
    World(
        MaybeRelativeBlockCoordinate<false>,
        MaybeRelativeBlockCoordinate<true>,
        MaybeRelativeBlockCoordinate<false>,
    ),
    Local(f64, f64, f64),
}

impl MaybeRelativeBlockPos {
    fn try_new(x: &str, y: &str, z: &str) -> Option<Self> {
        if x.starts_with('^') || y.starts_with('^') || z.starts_with('^') {
            Some(Self::Local(
                parse_local_coordinate(x)?,
                parse_local_coordinate(y)?,
                parse_local_coordinate(z)?,
            ))
        } else {
            Some(Self::World(
                x.try_into().ok()?,
                y.try_into().ok()?,
                z.try_into().ok()?,
            ))
        }
    }

    fn try_to_absolute(self, sender: &CommandSender) -> Option<BlockPos> {
        match self {
            Self::World(x, y, z) => {
                let origin = sender.position();
                Some(BlockPos(Vector3::new(
                    x.into_absolute(origin.map(|o| o.x))?,
                    y.into_absolute(origin.map(|o| o.y))?,
                    z.into_absolute(origin.map(|o| o.z))?,
                )))
            }
            Self::Local(left, up, forwards) => {
                let origin = sender.position()?;
                let rotation = sender.rotation()?;
                let offset = apply_local_coordinates(rotation, Vector3::new(left, up, forwards));
                Some(BlockPos::floored_v(origin.add(&offset)))
            }
        }
    }
}

fn parse_local_coordinate(s: &str) -> Option<f64> {
    let value = s.strip_prefix('^')?;
    if value.is_empty() {
        Some(0.0)
    } else {
        value.parse().ok()
    }
}

fn apply_local_coordinates(rotation: (f32, f32), direction: Vector3<f64>) -> Vector3<f64> {
    let (yaw, pitch) = rotation;
    let yaw = f64::from(yaw);
    let pitch = f64::from(pitch);

    let y_cos = (yaw + 90.0).to_radians().cos();
    let y_sin = (yaw + 90.0).to_radians().sin();
    let x_cos = (-pitch).to_radians().cos();
    let x_sin = (-pitch).to_radians().sin();
    let x_cos_up = (-pitch + 90.0).to_radians().cos();
    let x_sin_up = (-pitch + 90.0).to_radians().sin();

    let forwards = Vector3::new(y_cos * x_cos, x_sin, y_sin * x_cos);
    let up = Vector3::new(y_cos * x_cos_up, x_sin_up, y_sin * x_cos_up);
    let left = Vector3::new(
        forwards.y * up.z - forwards.z * up.y,
        forwards.z * up.x - forwards.x * up.z,
        forwards.x * up.y - forwards.y * up.x,
    ) * -1.0;

    Vector3::new(
        forwards.x * direction.z + up.x * direction.y + left.x * direction.x,
        forwards.y * direction.z + up.y * direction.y + left.y * direction.x,
        forwards.z * direction.z + up.z * direction.y + left.z * direction.x,
    )
}

impl DefaultNameArgConsumer for BlockPosArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "block_pos"
    }
}

impl<'a> FindArg<'a> for BlockPosArgumentConsumer {
    type Data = BlockPos;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::BlockPos(data)) => Ok(*data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}

impl BlockPosArgumentConsumer {
    pub fn find_loaded_arg(
        args: &super::ConsumedArgs,
        name: &str,
        world: &World,
    ) -> Result<BlockPos, CommandError> {
        let pos = Self::find_arg(args, name)?;

        if world.level.try_get_chunk(&pos.chunk_position()).is_none() {
            return Err(CommandError::CommandFailed(TextComponent::translate(
                "argument.pos.unloaded",
                [],
            )));
        }

        if !world.is_in_build_limit(pos) {
            return Err(CommandError::CommandFailed(TextComponent::translate(
                "argument.pos.outofworld",
                [],
            )));
        }

        Ok(pos)
    }

    pub fn find_spawnable_arg(
        args: &super::ConsumedArgs,
        name: &str,
    ) -> Result<BlockPos, CommandError> {
        let pos = Self::find_arg(args, name)?;

        if !World::is_valid(pos) {
            return Err(CommandError::CommandFailed(TextComponent::translate(
                "argument.pos.outofbounds",
                [],
            )));
        }

        Ok(pos)
    }
}
