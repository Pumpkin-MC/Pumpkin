use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};

use crate::command::CommandSender;
use crate::command::args::ConsumeResult;
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

/// yaw and pitch
pub struct RotationArgumentConsumer;

impl GetClientSideArgParser for RotationArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::Rotation
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for RotationArgumentConsumer {
    fn consume<'a, 'b>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let yaw_str_opt = args.pop();
        let pitch_str_opt = args.pop();

        let (Some(yaw_str), Some(pitch_str)) = (yaw_str_opt, pitch_str_opt) else {
            return Box::pin(async move { None });
        };

        let result: Option<Arg<'a>> = yaw_str.parse::<f32>().ok().and_then(|mut yaw| {
            pitch_str.parse::<f32>().ok().map(|mut pitch| {
                yaw %= 360.0;
                if yaw >= 180.0 {
                    yaw -= 360.0;
                }
                pitch %= 360.0;
                if pitch >= 180.0 {
                    pitch -= 360.0;
                }

                Arg::Rotation(yaw, pitch)
            })
        });

        Box::pin(async move { result })
    }
}

impl DefaultNameArgConsumer for RotationArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "rotation"
    }
}

impl<'a> FindArg<'a> for RotationArgumentConsumer {
    type Data = (f32, f32);

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Rotation(yaw, pitch)) => Ok((*yaw, *pitch)),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
