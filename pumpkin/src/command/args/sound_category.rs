use crate::command::CommandSender;
use crate::command::args::{
    Arg, ArgumentConsumer, DefaultNameArgConsumer, FindArg, GetClientSideArgParser,
};
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_data::sound::SoundCategory;
use pumpkin_protocol::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};

/// `ArgumentConsumer` for Minecraft sound categories (master, music, record, etc.)
pub struct SoundCategoryArgumentConsumer;

impl GetClientSideArgParser for SoundCategoryArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        // ResourceLocation is used for enumerated string values
        ArgumentType::ResourceLocation
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        // Force server-side suggestions to show available sound categories
        Some(SuggestionProviders::AskServer)
    }
}

#[async_trait]
impl ArgumentConsumer for SoundCategoryArgumentConsumer {
    async fn consume<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let s = args.pop()?;

        // Convert string input to SoundCategory enum
        // Uses lowercase to make the command case-insensitive
        let category = match s.to_lowercase().as_str() {
            "master" => Some(SoundCategory::Master), // Default category, affects all sounds
            "music" => Some(SoundCategory::Music),   // Background music
            "record" => Some(SoundCategory::Records), // Music discs
            "weather" => Some(SoundCategory::Weather), // Rain, thunder
            "block" => Some(SoundCategory::Blocks),  // Block sounds
            "hostile" => Some(SoundCategory::Hostile), // Hostile mob sounds
            "neutral" => Some(SoundCategory::Neutral), // Neutral mob sounds
            "player" => Some(SoundCategory::Players), // Player sounds
            "ambient" => Some(SoundCategory::Ambient), // Ambient environment
            "voice" => Some(SoundCategory::Voice),   // Voice/speech
            _ => None,
        };

        category.map(Arg::SoundCategory) // Simplified by removing redundant closure
    }

    async fn suggest<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
        let categories = [
            "master", "music", "record", "weather", "block", "hostile", "neutral", "player",
            "ambient", "voice",
        ];
        let suggestions: Vec<CommandSuggestion> = categories
            .iter()
            .map(|cat| CommandSuggestion::new((*cat).to_string(), None))
            .collect();
        Ok(Some(suggestions))
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
