use pumpkin_data::sound::Sound;
use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};
use pumpkin_util::text::TextComponent;

use crate::{
    command::{args::ConsumeResult, dispatcher::CommandError},
    server::Server,
};

use super::{
    super::{
        CommandSender,
        args::{ArgumentConsumer, RawArgs},
    },
    Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser,
};

pub struct SoundArgumentConsumer;

impl GetClientSideArgParser for SoundArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        ArgumentType::ResourceLocation
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        Some(SuggestionProviders::AvailableSounds)
    }
}

impl ArgumentConsumer for SoundArgumentConsumer {
    fn consume<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        args.pop().map(|arg| Arg::Block(arg.value))
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
            Some(Arg::Block(name)) => {
                Sound::from_name(name.strip_prefix("minecraft:").unwrap_or(name)).map_or_else(
                    || {
                        Err(CommandError::CommandFailed(TextComponent::translate_cross(
                            pumpkin_data::translation::java::ARGUMENT_ID_INVALID,
                            pumpkin_data::translation::java::ARGUMENT_ID_INVALID,
                            [],
                        )))
                    },
                    Result::Ok,
                )
            }
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
