use crate::command::CommandSender;
use crate::command::args::{Arg, ArgumentConsumer, ConsumeResult, FindArg, GetClientSideArgParser};
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::server::Server;
use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};
use pumpkin_util::text::TextComponent;

pub struct TextComponentArgConsumer;

impl GetClientSideArgParser for TextComponentArgConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::Component
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for TextComponentArgConsumer {
    fn consume<'a, 'b>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let s_opt: Option<&'a str> = args.pop();

        let Some(s) = s_opt else {
            return Box::pin(async move { None });
        };

        let text_component_opt = parse_text_component(s);

        let final_arg: Option<Arg<'a>> = text_component_opt.map_or_else(
            || {
                (s.starts_with('"') && s.ends_with('"')).then(|| {
                    let s_owned = s.replace('"', "");
                    Arg::TextComponent(TextComponent::text(s_owned))
                })
            },
            |text_component| Some(Arg::TextComponent(text_component)),
        );

        Box::pin(async move { final_arg })
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
    let result = serde_json::from_str(input);
    if let Err(e) = result {
        log::debug!("Failed to parse text component: {e}");
        None
    } else {
        result.unwrap()
    }
}
