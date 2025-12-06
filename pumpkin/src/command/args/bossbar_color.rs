use crate::command::CommandSender;
use crate::command::args::{
    Arg, ArgumentConsumer, ConsumeResult, DefaultNameArgConsumer, FindArg, GetClientSideArgParser,
    SuggestResult,
};
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::server::Server;
use crate::world::bossbar::BossbarColor;
use pumpkin_protocol::java::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};

pub struct BossbarColorArgumentConsumer;

impl GetClientSideArgParser for BossbarColorArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        // Not sure if this is right...
        ArgumentType::ResourceLocation
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        Some(SuggestionProviders::AskServer)
    }
}

impl ArgumentConsumer for BossbarColorArgumentConsumer {
    fn consume<'a, 'b>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let s_opt: Option<&'a str> = args.pop();

        let result: Option<Arg<'a>> = s_opt.map_or_else(
            || None,
            |s| {
                let color = match s {
                    "blue" => Some(BossbarColor::Blue),
                    "green" => Some(BossbarColor::Green),
                    "pink" => Some(BossbarColor::Pink),
                    "purple" => Some(BossbarColor::Purple),
                    "red" => Some(BossbarColor::Red),
                    "white" => Some(BossbarColor::White),
                    "yellow" => Some(BossbarColor::Yellow),
                    _ => None,
                };

                color.map(Arg::BossbarColor)
            },
        );

        Box::pin(async move { result })
    }

    fn suggest<'a>(
        &'a self,
        _sender: &CommandSender,
        _server: &'a Server,
        _input: &'a str,
    ) -> SuggestResult<'a> {
        Box::pin(async move {
            let colors = ["blue", "green", "pink", "purple", "red", "white", "yellow"];
            let suggestions: Vec<CommandSuggestion> = colors
                .iter()
                .map(|color| CommandSuggestion::new((*color).to_string(), None))
                .collect();
            Ok(Some(suggestions))
        })
    }
}

impl DefaultNameArgConsumer for BossbarColorArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "color"
    }
}

impl<'a> FindArg<'a> for BossbarColorArgumentConsumer {
    type Data = &'a BossbarColor;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::BossbarColor(data)) => Ok(data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
