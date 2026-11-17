//! Arguments the Java client parses with vanilla's parser.
//!
//! The server reads them with an existing argument type, while the client
//! is told the parser vanilla declares for that argument, so its
//! highlighting, validation and completion match vanilla's.

use pumpkin_protocol::java::client::play::SuggestionProviders;
use pumpkin_util::identifier::Identifier;

use crate::argument_types::FromStringReader;
use crate::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::context::command_context::CommandContext;
use crate::errors::command_syntax_error::CommandSyntaxError;
use crate::source::CommandSource;
use crate::string_reader::StringReader;
use crate::suggestion::suggestions::{Suggestions, SuggestionsBuilder};

/// Parses like `inner`, declared to the client as `client`.
pub struct ClientParsed<A> {
    pub inner: A,
    pub client: JavaClientArgumentType,
}

impl<A> ClientParsed<A> {
    pub const fn new(inner: A, client: JavaClientArgumentType) -> Self {
        Self { inner, client }
    }
}

impl<S: CommandSource, A: ArgumentType<S>> ArgumentType<S> for ClientParsed<A> {
    type Item = A::Item;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        self.inner.parse(reader)
    }

    fn parse_with_source(
        &self,
        reader: &mut StringReader,
        source: &S,
    ) -> Result<Self::Item, CommandSyntaxError> {
        self.inner.parse_with_source(reader, source)
    }

    fn list_suggestions(
        &self,
        context: &CommandContext<S>,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        self.inner.list_suggestions(context, builder)
    }

    fn client_side_parser(&self) -> JavaClientArgumentType {
        self.client.clone()
    }

    fn override_suggestion_providers(&self) -> Option<SuggestionProviders> {
        self.inner.override_suggestion_providers()
    }

    fn examples(&self) -> Vec<String> {
        self.inner.examples()
    }
}

/// An identifier (`namespace:path`, the namespace defaulting to
/// `minecraft`), as its full string.
pub struct IdStringArgumentType;

impl<S: CommandSource> ArgumentType<S> for IdStringArgumentType {
    type Item = String;

    fn parse(&self, reader: &mut StringReader) -> Result<String, CommandSyntaxError> {
        Ok(Identifier::from_reader(reader)?.to_string())
    }

    fn client_side_parser(&self) -> JavaClientArgumentType {
        JavaClientArgumentType::ResourceLocation
    }

    fn examples(&self) -> Vec<String> {
        examples!("foo", "foo:bar")
    }
}

impl IdStringArgumentType {
    /// The parsed id, as `namespace:path`.
    pub fn get<'a, S: CommandSource>(
        context: &'a CommandContext<S>,
        name: &str,
    ) -> Result<&'a str, CommandSyntaxError> {
        Ok(context.get_argument::<String>(name)?.as_str())
    }
}

/// Everything up to the next space: what vanilla's criteria and similar
/// arguments read (`ObjectiveCriteriaArgument` reads up to whitespace).
pub struct WordArgumentType;

impl<S: CommandSource> ArgumentType<S> for WordArgumentType {
    type Item = String;

    fn parse(&self, reader: &mut StringReader) -> Result<String, CommandSyntaxError> {
        let start = reader.cursor();
        while reader.peek().is_some_and(|c| c != ' ') {
            reader.skip();
        }
        Ok(reader.string()[start..reader.cursor()].to_owned())
    }

    fn client_side_parser(&self) -> JavaClientArgumentType {
        JavaClientArgumentType::String(
            pumpkin_protocol::java::client::play::StringProtoArgBehavior::SingleWord,
        )
    }

    fn examples(&self) -> Vec<String> {
        examples!("foo", "foo.bar:baz")
    }
}

/// `MessageArgument`: the rest of the line, as the client's `message`
/// parser reads it.
pub struct MessageArgumentType;

impl<S: CommandSource> ArgumentType<S> for MessageArgumentType {
    type Item = String;

    fn parse(&self, reader: &mut StringReader) -> Result<String, CommandSyntaxError> {
        let text = reader.remaining_part().to_owned();
        reader.set_cursor(reader.total_length());
        Ok(text)
    }

    fn client_side_parser(&self) -> JavaClientArgumentType {
        JavaClientArgumentType::Message
    }

    fn examples(&self) -> Vec<String> {
        examples!("Hello world!", "foo", "@e", "Hello @p :)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::DummySource;

    #[test]
    fn ids_and_words_read_what_vanilla_does() {
        let mut reader = StringReader::new("minecraft:chests/simple_dungeon rest");
        assert_eq!(
            ArgumentType::<DummySource>::parse(&IdStringArgumentType, &mut reader).unwrap(),
            "minecraft:chests/simple_dungeon"
        );
        let mut reader = StringReader::new("chests/simple_dungeon");
        assert_eq!(
            ArgumentType::<DummySource>::parse(&IdStringArgumentType, &mut reader).unwrap(),
            "minecraft:chests/simple_dungeon"
        );
        let mut reader = StringReader::new("minecraft.mined:minecraft.stone next");
        assert_eq!(
            ArgumentType::<DummySource>::parse(&WordArgumentType, &mut reader).unwrap(),
            "minecraft.mined:minecraft.stone"
        );
        let mut reader = StringReader::new("hello there @a");
        assert_eq!(
            ArgumentType::<DummySource>::parse(&MessageArgumentType, &mut reader).unwrap(),
            "hello there @a"
        );
        let typed = ClientParsed::new(WordArgumentType, JavaClientArgumentType::ObjectiveCriteria);
        assert!(matches!(
            ArgumentType::<DummySource>::client_side_parser(&typed),
            JavaClientArgumentType::ObjectiveCriteria
        ));
    }
}
