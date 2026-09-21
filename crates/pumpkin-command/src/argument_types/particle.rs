use crate::argument_types::FromStringReader;
use crate::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::context::command_context::CommandContext;
use crate::errors::command_syntax_error::CommandSyntaxError;
use crate::errors::error_types::{CommandErrorType, LiteralCommandErrorType};
use crate::string_reader::StringReader;
use crate::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use pumpkin_data::particle::Particle;
use pumpkin_data::translation;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::text::TextComponent;

pub const ERROR_UNKNOWN_PARTICLE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::PARTICLE_NOTFOUND,
    translation::bedrock::COMMANDS_PARTICLE_NOTFOUND,
);

pub const ERROR_NEEDS_OPTIONS: LiteralCommandErrorType = LiteralCommandErrorType::new(
    "This particle needs extra options, which this server cannot read yet",
);

/// Particles whose id needs option data on the wire, which can't be read off the command line, so the command is refused.
const fn takes_options(particle: Particle) -> bool {
    matches!(
        particle,
        Particle::Block
            | Particle::BlockMarker
            | Particle::BlockCrumble
            | Particle::DustPillar
            | Particle::FallingDust
            | Particle::Item
            | Particle::Dust
            | Particle::DustColorTransition
            | Particle::EntityEffect
            | Particle::TintedLeaves
            | Particle::SculkCharge
            | Particle::Shriek
            | Particle::Vibration
            | Particle::Trail
    )
}

pub struct ParticleArgumentType;

impl<S: crate::source::CommandSource> ArgumentType<S> for ParticleArgumentType {
    type Item = Particle;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let identifier = Identifier::from_reader(reader)?;
        let particle = Particle::from_name(identifier.path())
            .or_else(|| Particle::from_name(&identifier.to_string()))
            .ok_or_else(|| {
                ERROR_UNKNOWN_PARTICLE.create(reader, TextComponent::text(identifier.to_string()))
            })?;
        if takes_options(particle) {
            return Err(ERROR_NEEDS_OPTIONS.create(reader));
        }
        Ok(particle)
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
        _context: &CommandContext<S>,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        let particles = (0..=Particle::SulfurCubeGoo.to_id())
            .filter_map(Particle::from_id)
            .map(|p| format!("{p:?}").to_lowercase())
            .collect();
        builder.filter_and_suggest_lowercase(particles).build()
    }
}

impl ParticleArgumentType {
    pub fn get<S: crate::source::CommandSource>(
        context: &CommandContext<S>,
        name: &str,
    ) -> Result<Particle, CommandSyntaxError> {
        Ok(*context.get_argument::<Particle>(name)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::DummySource;

    fn parse(input: &str) -> Result<Particle, CommandSyntaxError> {
        ArgumentType::<DummySource>::parse(&ParticleArgumentType, &mut StringReader::new(input))
    }

    #[test]
    fn a_particle_with_options_is_refused() {
        assert!(parse("minecraft:flame").is_ok());
        assert!(parse("minecraft:block").is_err());
        assert!(parse("minecraft:dust").is_err());
        assert!(parse("minecraft:trail").is_err());
    }

    #[test]
    fn the_suggestion_range_covers_every_particle() {
        let last = Particle::SulfurCubeGoo.to_id();
        assert!(Particle::from_id(last).is_some());
        assert!(Particle::from_id(last + 1).is_none());
    }
}
