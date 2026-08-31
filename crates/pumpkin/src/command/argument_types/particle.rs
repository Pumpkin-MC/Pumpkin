use crate::command::argument_types::FromStringReader;
use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use pumpkin_data::particle::Particle;
use pumpkin_data::translation;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::text::TextComponent;

pub const ERROR_UNKNOWN_PARTICLE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::PARTICLE_NOTFOUND,
    translation::bedrock::COMMANDS_PARTICLE_NOTFOUND,
);

pub struct ParticleArgumentType;

impl ArgumentType for ParticleArgumentType {
    type Item = Particle;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let identifier = Identifier::from_reader(reader)?;
        Particle::from_name(identifier.path())
            .or_else(|| Particle::from_name(&identifier.to_string()))
            .ok_or_else(|| {
                ERROR_UNKNOWN_PARTICLE.create(reader, TextComponent::text(identifier.to_string()))
            })
    }

    fn client_side_parser(&self) -> JavaClientArgumentType {
        JavaClientArgumentType::Particle
    }

    fn examples(&self) -> Vec<String> {
        vec![
            "foo".to_string(),
            "foo:bar".to_string(),
            "particle".to_string(),
        ]
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        let particles = (0..=124u16)
            .filter_map(Particle::from_id)
            .map(|p| format!("{p:?}").to_lowercase())
            .collect();
        builder.filter_and_suggest_lowercase(particles).build()
    }
}

impl ParticleArgumentType {
    pub fn get(context: &CommandContext, name: &str) -> Result<Particle, CommandSyntaxError> {
        Ok(*context.get_argument::<Particle>(name)?)
    }
}
