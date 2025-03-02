use async_trait::async_trait;
use pumpkin_data::particle::Particle;
use pumpkin_protocol::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};

use crate::command::{
    CommandSender,
    args::{
        Arg, ArgumentConsumer, ConsumedArgs, DefaultNameArgConsumer, FindArg,
        GetClientSideArgParser,
    },
    dispatcher::CommandError,
    tree::RawArgs,
};
use crate::server::Server;

pub struct ParticleArgumentConsumer;

impl GetClientSideArgParser for ParticleArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        ArgumentType::Resource {
            identifier: "particle_type",
        }
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

#[async_trait]
impl ArgumentConsumer for ParticleArgumentConsumer {
    async fn consume<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let name = args.pop()?;

        // Create a static damage type first
        let particle = Particle::from_name(&name.replace("minecraft:", ""))?;
        // Find matching static damage type from values array
        Some(Arg::Particle(particle))
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

impl DefaultNameArgConsumer for ParticleArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "particle_type"
    }
}

impl<'a> FindArg<'a> for ParticleArgumentConsumer {
    type Data = &'a Particle;

    fn find_arg(args: &'a ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Particle(data)) => Ok(data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
