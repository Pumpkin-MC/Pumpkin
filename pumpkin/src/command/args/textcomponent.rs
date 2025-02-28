use crate::command::CommandSender;
use crate::command::args::{Arg, ArgumentConsumer, FindArg, GetClientSideArgParser};
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_protocol::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};
use pumpkin_util::text::{TextComponent, TextContent};

pub(crate) struct TextComponentArgConsumer;

impl GetClientSideArgParser for TextComponentArgConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        ArgumentType::Component
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

#[async_trait]
impl ArgumentConsumer for TextComponentArgConsumer {
    async fn consume<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let s = args.pop()?;

        let text_component = parse_text_component(s);

        let Some(text_component) = text_component else {
            if s.starts_with('"') && s.ends_with('"') {
                let s = s.replace('"', "");
                return Some(Arg::TextComponent(TextComponent::text(s)));
            }
            return None;
        };

        Some(Arg::TextComponent(text_component))
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
    if input.starts_with('{') && input.ends_with('}') {
        let text_component: Option<TextContent> = serde_json::from_str(input).unwrap_or(None);
        Some(TextComponent::from_content(text_component?))
    } else {
        serde_json::from_str(input).unwrap_or(None)
    }
}
