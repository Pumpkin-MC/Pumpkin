use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};

use crate::command::CommandSender;
use crate::command::args::ConsumeResult;
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

/// Parses a rotation value that may be relative (prefixed with ~).
/// Returns (value, `is_relative`).
fn parse_rotation_component(s: &str) -> Option<(f32, bool)> {
    if let Some(rest) = s.strip_prefix('~') {
        // Relative rotation
        let value = if rest.is_empty() {
            0.0
        } else {
            rest.parse::<f32>().ok()?
        };
        Some((value, true))
    } else {
        // Absolute rotation
        let mut value = s.parse::<f32>().ok()?;
        value %= 360.0;
        if value >= 180.0 {
            value -= 360.0;
        }
        Some((value, false))
    }
}

/// yaw and pitch with optional relative prefix (~)
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

        let result: Option<Arg<'a>> = parse_rotation_component(yaw_str).and_then(|(yaw, yaw_rel)| {
            parse_rotation_component(pitch_str).map(|(pitch, pitch_rel)| {
                Arg::Rotation(yaw, yaw_rel, pitch, pitch_rel)
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
    /// (yaw, `is_yaw_relative`, pitch, `is_pitch_relative`)
    type Data = (f32, bool, f32, bool);

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Rotation(yaw, yaw_rel, pitch, pitch_rel)) => {
                Ok((*yaw, *yaw_rel, *pitch, *pitch_rel))
            }
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
