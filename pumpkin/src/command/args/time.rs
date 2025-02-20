use async_trait::async_trait;
use pumpkin_protocol::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};

use crate::command::{
    CommandSender,
    args::{Arg, ArgumentConsumer, DefaultNameArgConsumer, FindArg, GetClientSideArgParser},
    dispatcher::CommandError,
    tree::RawArgs,
};
use crate::server::Server;

pub struct TimeArgumentConsumer;

impl GetClientSideArgParser for TimeArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        ArgumentType::Time { min: 0 }
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

#[async_trait]
impl ArgumentConsumer for TimeArgumentConsumer {
    async fn consume<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let s = args.pop()?;

        // Parse number and unit
        let (num_str, unit) = s
            .find(|c: char| c.is_alphabetic())
            .map_or((s, "t"), |pos| (&s[..pos], &s[pos..]));

        let number = num_str.parse::<f32>().ok()?;
        if number < 0.0 {
            return None;
        }

        // Convert to ticks based on unit
        let ticks = match unit {
            "d" => number * 24000.0,
            "s" => number * 20.0,
            "t" => number,
            _ => return None,
        };

        // Round to nearest integer
        let ticks = ticks.round() as i32;

        Some(Arg::Time(ticks))
    }

    async fn suggest<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
        Ok(None)
    }
}

impl DefaultNameArgConsumer for TimeArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "time"
    }
}

impl<'a> FindArg<'a> for TimeArgumentConsumer {
    type Data = i32;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Time(ticks)) => Ok(*ticks),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
