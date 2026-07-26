use super::{CommandDispatcher, ParsingResult};
use crate::command::context::command_source::CommandSource;
use crate::command::node::tree::NodeIdClassification;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use futures::future;
use pumpkin_protocol::java::client::play::CommandSuggestion;
use std::pin::Pin;

impl CommandDispatcher {
    /// Returns a new [`Suggestions`] structure in the future
    /// from the given parsing result, which was a command that was parsed,
    /// assuming the cursor is at the end.
    ///
    /// This is useful to tell the client on what suggestions are there next.
    pub async fn get_completion_suggestions_at_end(
        &self,
        parsing_result: ParsingResult<'_>,
    ) -> Suggestions {
        let length = parsing_result.reader.total_length();
        self.get_completion_suggestions(parsing_result, length)
            .await
    }

    /// Returns a new [`Suggestions`] structure in the future
    /// from the given parsing result, which was a command that was parsed.
    ///
    /// This is useful to tell the client on what suggestions are there next.
    pub async fn get_completion_suggestions(
        &self,
        parsing_result: ParsingResult<'_>,
        cursor: usize,
    ) -> Suggestions {
        let context = parsing_result.context;
        let (parent, start) = {
            let node_before_cursor = context.find_suggestion_context(cursor);
            (
                node_before_cursor.parent,
                node_before_cursor.starting_position.min(cursor),
            )
        };

        let full_input = parsing_result.reader.string();

        let truncated_input = &full_input[0..cursor.min(full_input.len())];

        let children = self.tree.get_children(parent);
        let capacity = children.len();
        let mut futures = Vec::with_capacity(capacity);

        let context = context.build(truncated_input);
        let mut provided_suggestions = Vec::new();

        for child in children {
            let builder = SuggestionsBuilder::new(truncated_input, start);

            let future: Option<Pin<Box<dyn Future<Output = Suggestions> + Send>>> =
                match self.tree.classify_id(child) {
                    NodeIdClassification::Root => Some(Box::pin(async { Suggestions::empty() })),
                    NodeIdClassification::Literal(literal_node_id) => Some(Box::pin(async move {
                        let node = &self.tree[literal_node_id];
                        if node
                            .meta
                            .literal_lowercase
                            .starts_with(builder.remaining_lowercase())
                        {
                            builder.suggest(&*node.meta.literal).build()
                        } else {
                            Suggestions::empty()
                        }
                    })),
                    NodeIdClassification::Command(command_node_id) => Some(Box::pin(async move {
                        let node = &self.tree[command_node_id];
                        if node
                            .meta
                            .literal_lowercase
                            .starts_with(builder.remaining_lowercase())
                        {
                            builder.suggest(&*node.meta.literal).build()
                        } else {
                            Suggestions::empty()
                        }
                    })),
                    NodeIdClassification::Argument(argument_node_id) => {
                        let node = &self.tree[argument_node_id];
                        if let Some(provider) = &node.meta.suggestion_provider {
                            // For custom suggestions sent by the server, we simply
                            // wait instead of adding the future to join.
                            provided_suggestions.push(provider.suggest(&context, builder).await);
                        } else {
                            provided_suggestions.push(
                                node.meta
                                    .argument_type
                                    .list_suggestions(&context, builder)
                                    .await,
                            );
                        }
                        None
                    }
                };

            if let Some(future) = future {
                futures.push(future);
            }
        }

        let mut suggestions = future::join_all(futures).await;
        suggestions.append(&mut provided_suggestions);
        Suggestions::merge(full_input, suggestions)
    }

    /// Gets all the suggestions in the future as a [`Vec`] of [`CommandSuggestion`].
    ///
    /// # Panics
    ///
    /// This function currently panics if the source provided was a dummy source.
    /// This is subject to change in the future.
    pub async fn suggest(&self, input: &str, source: &CommandSource) -> Vec<CommandSuggestion> {
        let future1 = async move {
            let parsed = self.parse_input(input, source).await;
            let suggestions = self.get_completion_suggestions_at_end(parsed).await;

            suggestions
                .suggestions
                .into_iter()
                .map(|suggestion| CommandSuggestion {
                    suggestion: suggestion.text.cached_text().clone(),
                    tooltip: suggestion.tooltip,
                })
                .collect::<Vec<CommandSuggestion>>()
        };

        let future2 = async move {
            self.fallback_dispatcher
                .find_suggestions(&source.output, source.server(), input)
                .await
        };

        let (mut a, mut b) = future::join(future1, future2).await;
        a.append(&mut b);
        a
    }
}
