use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};

use crate::command::{
    CommandSender,
    args::{
        Arg, ArgumentConsumer, ConsumeResult, ConsumeResultWithSyntax, DefaultNameArgConsumer,
        FindArg, GetClientSideArgParser,
    },
    argument_types::{argument_type::ArgumentType as _, time::TimeArgumentType},
    dispatcher::CommandError,
    errors::command_syntax_error::{CommandSyntaxError, CommandSyntaxErrorContext},
    string_reader::StringReader,
    tree::{RawArg, RawArgs},
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

fn map_local_syntax_error(error: CommandSyntaxError, raw_arg: RawArg<'_>) -> CommandSyntaxError {
    let local_cursor = error.context.map_or(0, |context| context.cursor);
    let mut clamped_local_cursor = local_cursor.min(raw_arg.value.len());
    while clamped_local_cursor > 0 && !raw_arg.value.is_char_boundary(clamped_local_cursor) {
        clamped_local_cursor -= 1;
    }

    CommandSyntaxError {
        error_type: error.error_type,
        message: error.message,
        context: Some(CommandSyntaxErrorContext {
            input: raw_arg.input.to_string(),
            cursor: raw_arg.start + clamped_local_cursor,
        }),
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
    fn consume<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let s_opt: Option<&'a str> = args.pop().map(|arg| arg.value);

        s_opt.and_then(|s| {
            let mut reader = StringReader::new(s);
            let parser = TimeArgumentType::new(self.min);
            parser.parse(&mut reader).ok().map(Arg::Time)
        })
    }

    fn consume_with_syntax<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResultWithSyntax<'a> {
        let Some(raw_arg) = args.pop() else {
            return Ok(None);
        };

        let mut reader = StringReader::new(raw_arg.value);
        let parser = TimeArgumentType::new(self.min);
        parser
            .parse(&mut reader)
            .map(|ticks| Some(Arg::Time(ticks)))
            .map_err(|error| map_local_syntax_error(error, raw_arg))
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
