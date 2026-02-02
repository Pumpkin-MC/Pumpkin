use pumpkin_util::text::TextComponent;

/// A struct detailing the context of a syntax error, including where it happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSyntaxErrorContext {
    pub input: String,
    pub cursor: usize,
}

/// Indicates an object that can provide context of a command syntax error from itself.
pub trait ContextProvider {
    fn context(&self) -> CommandSyntaxErrorContext;
}

impl ContextProvider for CommandSyntaxErrorContext {
    fn context(&self) -> CommandSyntaxErrorContext {
        self.clone()
    }
}

/// A struct detailing a syntax error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSyntaxError {
    pub message: TextComponent,
    pub context: Option<CommandSyntaxErrorContext>,
}

impl CommandSyntaxError {
    /// Constructs a new [`CommandSyntaxError`] without any context of the error,
    /// and only the error message itself.
    ///
    /// This means this error will not print a context to the client, which
    /// includes the string and the location the error was caused.
    #[must_use]
    pub const fn new_without_context(message: TextComponent) -> Self {
        Self {
            message,
            context: None,
        }
    }

    /// Constructs a new [`CommandSyntaxError`] with a given context of the error,
    /// which includes the string and the location the error was caused,
    /// along with the error message itself.
    #[must_use]
    pub fn new_with_context<C>(message: TextComponent, context_provider: &C) -> Self
    where
        C: ContextProvider,
    {
        Self {
            message,
            context: Some(context_provider.context()),
        }
    }
}
