use std::borrow::Cow;

use pumpkin_util::text::TextComponent;

use crate::command::{
    errors::{
        command_syntax_error::{CommandSyntaxError, CommandSyntaxErrorContext, ContextProvider},
        error_types::{AnyCommandErrorType, CommandErrorType},
    },
    string_reader::StringReader,
    suggestion::suggestions::{Suggestions, SuggestionsBuilder},
};

/// A delayed version of [`CommandSyntaxError`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelayedCommandSyntaxError {
    pub error_type: &'static dyn AnyCommandErrorType,
    pub translation_key: &'static str,
    pub arguments: Vec<Cow<'static, str>>,
}

#[derive(Debug, Default)]
pub struct SnbtErrors {
    cursor: usize,
    command_error: Option<DelayedCommandSyntaxError>,
    suggestions: Vec<Cow<'static, str>>,
}

/// A trait so that a parser specializing
/// to keep track of errors don't need to keep track
/// of suggestions, and vice versa.
impl SnbtErrors {
    pub fn simple_static(
        &mut self,
        reader: &StringReader,
        error_type: &'static CommandErrorType<0>,
        suggestions: &[&'static str],
    ) {
        self.store(
            reader,
            || DelayedCommandSyntaxError {
                error_type,
                translation_key: error_type.translation_key,
                arguments: vec![],
            },
            |entries| {
                for suggestion in suggestions {
                    entries.push(Cow::Borrowed(*suggestion));
                }
            },
        );
    }

    pub fn dynamic_static<A: Into<Cow<'static, str>>>(
        &mut self,
        reader: &StringReader,
        error_type: &'static CommandErrorType<1>,
        arg1: A,
        suggestions: &[&'static str],
    ) {
        self.store(
            reader,
            || DelayedCommandSyntaxError {
                error_type,
                translation_key: error_type.translation_key,
                arguments: vec![arg1.into()],
            },
            |entries| {
                for suggestion in suggestions {
                    entries.push(Cow::Borrowed(*suggestion));
                }
            },
        );
    }

    pub fn simple(
        &mut self,
        reader: &StringReader,
        error_type: &'static CommandErrorType<0>,
        suggestions: Vec<String>,
    ) {
        self.store(
            reader,
            || DelayedCommandSyntaxError {
                error_type,
                translation_key: error_type.translation_key,
                arguments: vec![],
            },
            |entries| {
                for suggestion in suggestions {
                    entries.push(Cow::Owned(suggestion));
                }
            },
        );
    }

    pub fn dynamic<A: Into<Cow<'static, str>>>(
        &mut self,
        reader: &StringReader,
        error_type: &'static CommandErrorType<1>,
        arg1: A,
        suggestions: Vec<String>,
    ) {
        self.store(
            reader,
            || DelayedCommandSyntaxError {
                error_type,
                translation_key: error_type.translation_key,
                arguments: vec![arg1.into()],
            },
            |entries| {
                for suggestion in suggestions {
                    entries.push(Cow::Owned(suggestion));
                }
            },
        );
    }

    #[inline]
    fn store(
        &mut self,
        reader: &StringReader,
        error: impl FnOnce() -> DelayedCommandSyntaxError,
        suggestions: impl FnOnce(&mut Vec<Cow<'static, str>>),
    ) {
        let current = self.cursor;
        let new = reader.cursor();

        if self.command_error.is_none() || new > current {
            self.command_error = Some(error());
            self.suggestions.clear();
            suggestions(&mut self.suggestions);
        } else if new >= current {
            suggestions(&mut self.suggestions);
        }
    }

    // to be removed later
    /*
     * fn into_command_syntax_error(self, read) -> Option<CommandSyntaxError> {
        let delayed_error = self.command_error?;
        Some(CommandSyntaxError {
            error_type: delayed_error.error_type,
            message: TextComponent::translate(
                delayed_error.translation_key,
                delayed_error
                    .arguments
                    .into_iter()
                    .map(|text| TextComponent::text(text))
                    .collect::<Vec<_>>(),
            ),
            context: Some(CommandSyntaxErrorContext { input: self., cursor: () }),
        })
    }
     */
}
