use pumpkin_util::text::TextComponent;

use crate::command::{
    errors::{command_syntax_error::CommandSyntaxError, error_types::CommandErrorType},
    string_reader::StringReader,
};

/// A trait so that a parser specializing
/// to keep track of errors don't need to keep track
/// of suggestions, and vice versa.
pub trait ErrorEntries: Default {
    fn simple(
        &mut self,
        reader: &StringReader,
        error_type: &'static CommandErrorType<0>,
        suggestions: impl FnOnce() -> Vec<String>,
    );

    fn dynamic(
        &mut self,
        reader: &StringReader,
        error_type: &'static CommandErrorType<1>,
        arg1: impl FnOnce() -> TextComponent,
        suggestions: impl FnOnce() -> Vec<String>,
    );
}

/// A data structure that keeps track of errors. However,
/// this structure only stores at most 1 error as only up to 1
/// error will be thrown to the sender.
#[derive(Debug, Default)]
pub struct CommandErrorEntries(Option<CommandSyntaxError>);
impl ErrorEntries for CommandErrorEntries {
    fn simple(
        &mut self,
        reader: &StringReader,
        error_type: &'static CommandErrorType<0>,
        _: impl FnOnce() -> Vec<String>,
    ) {
        // We only store the first 'longest' error that occurred.
        // 'Longest' here means the error that occurred the furthest in the string.
        if self.0.is_none() {
            self.0 = Some(error_type.create(reader));
        }
    }

    fn dynamic(
        &mut self,
        reader: &StringReader,
        error_type: &'static CommandErrorType<1>,
        arg1: impl FnOnce() -> TextComponent,
        _: impl FnOnce() -> Vec<String>,
    ) {
        // We only store the first 'longest' error that occurred.
        // 'Longest' here means the error that occurred the furthest in the string.
        if self.0.is_none() {
            self.0 = Some(error_type.create(reader, arg1()));
        }
    }
}

/// A data structure that keeps track of suggestions to fix errors.
#[derive(Debug, Default)]
pub struct SuggestionsErrorEntries(Vec<String>);
impl ErrorEntries for SuggestionsErrorEntries {
    fn simple(
        &mut self,
        _: &StringReader,
        _: &'static CommandErrorType<0>,
        suggestions: impl FnOnce() -> Vec<String>,
    ) {
        self.0.append(&mut suggestions());
    }

    fn dynamic(
        &mut self,
        _: &StringReader,
        _: &'static CommandErrorType<1>,
        _: impl FnOnce() -> TextComponent,
        suggestions: impl FnOnce() -> Vec<String>,
    ) {
        self.0.append(&mut suggestions());
    }
}
