use pumpkin_data::particle::Particle;
use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};

use crate::command::{
    CommandSender,
    args::{
        Arg, ArgumentConsumer, ConsumeResult, ConsumedArgs, DefaultNameArgConsumer, FindArg,
        GetClientSideArgParser,
    },
    dispatcher::CommandError,
    tree::RawArgs,
};
use crate::server::Server;

pub struct ParticleArgumentConsumer;

impl GetClientSideArgParser for ParticleArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::Resource {
            identifier: "particle_type",
        }
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for ParticleArgumentConsumer {
    fn consume<'a, 'b>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let name_opt: Option<&'a str> = args.pop();

        let result: Option<Arg<'a>> = name_opt.map_or_else(
            || None,
            |name| {
                Particle::from_name(name.strip_prefix("minecraft:").unwrap_or(name))
                    .map(Arg::Particle)
            },
        );

        Box::pin(async move { result })
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
