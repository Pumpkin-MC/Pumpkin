use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};

use crate::command::{
    CommandSender,
    args::{
        Arg, ArgumentConsumer, ConsumeResult, DefaultNameArgConsumer, FindArg,
        GetClientSideArgParser,
    },
    dispatcher::CommandError,
    tree::RawArgs,
};
use crate::server::Server;

pub struct TimeArgumentConsumer;

impl GetClientSideArgParser for TimeArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::Time { min: 0 }
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for TimeArgumentConsumer {
    fn consume<'a, 'b>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let s_opt: Option<&'a str> = args.pop();

        let result: Option<Arg<'a>> = s_opt.and_then(|s| {
            let (num_str, unit) = s
                .find(|c: char| c.is_alphabetic())
                .map_or((s, "t"), |pos| (&s[..pos], &s[pos..]));

            let number = num_str.parse::<f32>().ok()?; // Replaces .ok()?

            if number < 0.0 {
                return None;
            }

            let ticks = match unit {
                "d" => number * 24000.0,
                "s" => number * 20.0,
                "t" => number,
                _ => return None,
            };

            let ticks = ticks.round() as i32;

            Some(Arg::Time(ticks))
        });

        Box::pin(async move { result })
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
