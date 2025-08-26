use std::sync::Arc;

use crate::command::CommandSender;
use crate::command::args::{Arg, ArgumentConsumer, FindArg, GetClientSideArgParser};
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_protocol::java::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};
use pumpkin_util::text::content::TextContent;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::text::{TextComponent, TextComponentBase};

pub struct TextComponentArgConsumer;

impl GetClientSideArgParser for TextComponentArgConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
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
        sender: &CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let s = args.pop()?;

        let text_component = parse_text_component(s, sender);

        let Some(text_component) = text_component else {
            if s.starts_with('"') && s.ends_with('"') {
                let s = s.replace('"', "");
                return Some(Arg::TextComponent(Box::new(TextComponent::text(s))));
            }
            return None;
        };

        Some(Arg::TextComponent(Box::new(text_component)))
    }

    async fn suggest<'a>(
        &'a self,
        _sender: &CommandSender,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
        Ok(None)
    }
}

impl FindArg<'_> for TextComponentArgConsumer {
    type Data = Box<TextComponent>;

    fn find_arg(args: &super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::TextComponent(data)) => Ok(data.clone()),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}

fn parse_text_component(input: &str, sender: &CommandSender) -> Option<TextComponent> {
    let result = serde_json::from_str(input);
    if let Err(e) = result {
        log::debug!("Failed to parse text component: {e}");
        None
    } else {
        let component: Option<TextComponent> = result.unwrap();
        if let CommandSender::Player(player) = sender {
            return Some(TextComponent(add_sender_uuid(component?.0, player)));
        }
        Some(component?)
    }
}
fn add_sender_uuid(component: TextComponentBase, sender: &Arc<Player>) -> TextComponentBase {
    let mut component = match component.content {
        TextContent::Scoreboard { mut score } => {
            score.sender = Some(sender.get_entity().entity_uuid);
            TextComponentBase {
                content: TextContent::Scoreboard { score },
                style: component.style,
                extra: component.extra,
            }
        }
        TextContent::EntityNames {
            selector,
            separator,
            ..
        } => TextComponentBase {
            content: TextContent::EntityNames {
                selector,
                separator,
                sender: Some(sender.get_entity().entity_uuid),
            },
            style: component.style,
            extra: component.extra,
        },
        TextContent::Nbt { mut value } => {
            value.sender = Some(sender.get_entity().entity_uuid);
            TextComponentBase {
                content: TextContent::Nbt { value },
                style: component.style,
                extra: component.extra,
            }
        }
        _ => component,
    };
    let mut extra = vec![];
    for component in component.extra {
        extra.push(add_sender_uuid(component, sender));
    }
    component.extra = extra;
    match component.style.hover_event {
        None => return component,
        Some(hover) => {
            component.style.hover_event = match hover {
                HoverEvent::ShowText { value } => {
                    let mut hover_components = vec![];
                    for hover_component in value {
                        hover_components.push(add_sender_uuid(hover_component, sender));
                    }
                    Some(HoverEvent::ShowText {
                        value: hover_components,
                    })
                }
                HoverEvent::ShowEntity { name, id, uuid } => match name {
                    None => Some(HoverEvent::ShowEntity {
                        name: None,
                        id,
                        uuid,
                    }),
                    Some(name) => {
                        let mut translated_names = Vec::new();
                        for part in name {
                            translated_names.push(add_sender_uuid(part, sender));
                        }
                        Some(HoverEvent::ShowEntity {
                            name: Some(translated_names),
                            id,
                            uuid,
                        })
                    }
                },
                HoverEvent::ShowItem { id, count } => Some(HoverEvent::ShowItem { id, count }),
            }
        }
    }
    component
}
