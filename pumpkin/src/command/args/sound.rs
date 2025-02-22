use async_trait::async_trait;
use pumpkin_data::sound::Sound;
use pumpkin_protocol::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};

use crate::{command::dispatcher::CommandError, server::Server};

use super::{
    super::{
        CommandSender,
        args::{ArgumentConsumer, RawArgs},
    },
    Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser,
};

pub(crate) struct SoundArgumentConsumer;

impl GetClientSideArgParser for SoundArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        ArgumentType::ResourceLocation
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        Some(SuggestionProviders::AvailableSounds)
    }
}

#[async_trait]
impl ArgumentConsumer for SoundArgumentConsumer {
    async fn consume<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let s = args.pop()?;
        Some(Arg::Block(s))
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

impl DefaultNameArgConsumer for SoundArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "available_sounds"
    }
}

impl<'a> FindArg<'a> for SoundArgumentConsumer {
    type Data = Sound;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Block(name)) => Sound::from_name(&name.replace("minecraft:", ""))
                .map_or_else(
                    || {
                        Err(CommandError::GeneralCommandIssue(format!(
                            "Sound {name} does not exist."
                        )))
                    },
                    Result::Ok,
                ),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
