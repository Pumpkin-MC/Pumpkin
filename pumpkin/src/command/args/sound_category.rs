use crate::command::CommandSender;
use crate::command::args::{
    Arg, ArgumentConsumer, ConsumeResult, DefaultNameArgConsumer, FindArg, GetClientSideArgParser,
    SuggestResult,
};
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::server::Server;
use pumpkin_data::sound::SoundCategory;
use pumpkin_protocol::java::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};

/// `ArgumentConsumer` for Minecraft sound categories (master, music, record, etc.)
pub struct SoundCategoryArgumentConsumer;

impl GetClientSideArgParser for SoundCategoryArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        // ResourceLocation is used for enumerated string values
        ArgumentType::ResourceLocation
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        // Force server-side suggestions to show available sound categories
        Some(SuggestionProviders::AskServer)
    }
}

impl ArgumentConsumer for SoundCategoryArgumentConsumer {
    fn consume<'a, 'b>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let s_opt: Option<&'a str> = args.pop();

        let result: Option<Arg<'a>> = s_opt.and_then(|s| {
            let category = match s.to_lowercase().as_str() {
                "master" => Some(SoundCategory::Master),
                "music" => Some(SoundCategory::Music),
                // i don't use SoundCategory::from_name because its is record and not records :c
                "record" => Some(SoundCategory::Records),
                "weather" => Some(SoundCategory::Weather),
                "block" => Some(SoundCategory::Blocks),
                "hostile" => Some(SoundCategory::Hostile),
                "neutral" => Some(SoundCategory::Neutral),
                "player" => Some(SoundCategory::Players),
                "ambient" => Some(SoundCategory::Ambient),
                "voice" => Some(SoundCategory::Voice),
                _ => None,
            };

            category.map(Arg::SoundCategory)
        });

        Box::pin(async move { result })
    }

    fn suggest<'a>(
        &'a self,
        _sender: &CommandSender,
        _server: &'a Server,
        _input: &'a str,
    ) -> SuggestResult<'a> {
        Box::pin(async move {
            let categories = [
                "master", "music", "record", "weather", "block", "hostile", "neutral", "player",
                "ambient", "voice",
            ];
            let suggestions: Vec<CommandSuggestion> = categories
                .iter()
                .map(|cat| CommandSuggestion::new((*cat).to_string(), None))
                .collect();
            Ok(Some(suggestions))
        })
    }
}

impl DefaultNameArgConsumer for SoundCategoryArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "source"
    }
}

impl<'a> FindArg<'a> for SoundCategoryArgumentConsumer {
    type Data = &'a SoundCategory;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::SoundCategory(data)) => Ok(data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
