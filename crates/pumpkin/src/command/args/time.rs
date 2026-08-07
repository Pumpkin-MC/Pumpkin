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

#[derive(Clone, Copy, Debug)]
pub struct TimeArgumentConsumer {
    min: i32,
}

impl Default for TimeArgumentConsumer {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeArgumentConsumer {
    #[must_use]
    pub const fn new() -> Self {
        Self { min: 0 }
    }

    #[must_use]
    pub const fn min(min: i32) -> Self {
        Self { min }
    }
}

impl GetClientSideArgParser for TimeArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        ArgumentType::Time { min: self.min }
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
        let s_opt: Option<&'a str> = args.pop().map(|arg| arg.value);

        let result: Option<Arg<'a>> = s_opt.and_then(|s| {
            let (num_str, unit) = s
                .find(|c: char| c.is_alphabetic() && c != '-')
                .map_or((s, "t"), |pos| (&s[..pos], &s[pos..]));

            let number = num_str.parse::<f32>().ok()?;

            let factor = match unit {
                "d" => 24000.0,
                "s" => 20.0,
                "t" | "" => 1.0,
                _ => return None,
            };

            let ticks = (number * factor).round() as i32;

            if ticks < self.min {
                return None;
            }

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
