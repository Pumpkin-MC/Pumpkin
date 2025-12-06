use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};
use pumpkin_util::math::vector3::Vector3;

use crate::command::CommandSender;
use crate::command::args::ConsumeResult;
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::coordinate::MaybeRelativeCoordinate;
use super::{Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

/// x, y and z coordinates
pub struct Position3DArgumentConsumer;

impl GetClientSideArgParser for Position3DArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::Vec3
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for Position3DArgumentConsumer {
    fn consume<'a, 'b>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let x_str_opt = args.pop();
        let y_str_opt = args.pop();
        let z_str_opt = args.pop();

        let (Some(x_str), Some(y_str), Some(z_str)) = (x_str_opt, y_str_opt, z_str_opt) else {
            return Box::pin(async move { None });
        };

        let result: Option<Arg<'a>> = MaybeRelativePosition3D::try_new(x_str, y_str, z_str)
            .and_then(|pos| pos.try_to_absolute(sender.position()))
            .map(Arg::Pos3D);

        Box::pin(async move { result })
    }
}

struct MaybeRelativePosition3D(
    MaybeRelativeCoordinate<false>,
    MaybeRelativeCoordinate<true>,
    MaybeRelativeCoordinate<false>,
);

impl MaybeRelativePosition3D {
    fn try_new(x: &str, y: &str, z: &str) -> Option<Self> {
        Some(Self(
            x.try_into().ok()?,
            y.try_into().ok()?,
            z.try_into().ok()?,
        ))
    }

    fn try_to_absolute(self, origin: Option<Vector3<f64>>) -> Option<Vector3<f64>> {
        Some(Vector3::new(
            self.0.into_absolute(origin.map(|o| o.x))?,
            self.1.into_absolute(origin.map(|o| o.y))?,
            self.2.into_absolute(origin.map(|o| o.z))?,
        ))
    }
}

impl DefaultNameArgConsumer for Position3DArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "pos"
    }
}

impl<'a> FindArg<'a> for Position3DArgumentConsumer {
    type Data = Vector3<f64>;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Pos3D(data)) => Ok(*data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
