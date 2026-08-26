use crate::command::CommandSender;
use crate::command::args::{Arg, ArgumentConsumer, ConsumeResult, FindArg, GetClientSideArgParser};
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::server::Server;
use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};
use pumpkin_util::text::TextComponent;
use tracing::debug;

pub struct TextComponentArgConsumer;

impl GetClientSideArgParser for TextComponentArgConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        ArgumentType::Component
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for TextComponentArgConsumer {
    fn consume<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let s = args.pop().map(|arg| arg.value)?;

        let text_component_opt = parse_text_component(s);

        // TODO: Allow identifiers (starting with alphabetic or _, then alphanumeric+-_.) as display names
        text_component_opt.map_or_else(
            || {
                (s.starts_with('"') && s.ends_with('"')).then(|| {
                    let s_owned = s.replace('"', "");
                    Arg::TextComponent(TextComponent::text(s_owned))
                })
            },
            |text_component| Some(Arg::TextComponent(text_component)),
        )
    }
}

impl FindArg<'_> for TextComponentArgConsumer {
    type Data = TextComponent;

    fn find_arg(args: &super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::TextComponent(data)) => Ok(data.clone()),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}

fn parse_text_component(input: &str) -> Option<TextComponent> {
    serde_json::from_str(input)
        .map_err(|e| debug!("Failed to parse text component: {e}"))
        .ok()
}
