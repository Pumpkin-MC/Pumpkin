use async_trait::async_trait;
use pumpkin_data::game_rules::{GameRule, GameRuleRegistry, GameRuleValue};

use crate::command::args::FindArg;
use crate::command::args::bool::BoolArgConsumer;
use crate::command::args::bounded_num::BoundedNumArgumentConsumer;

use crate::TextComponent;

use crate::command::args::ConsumedArgs;
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandSender};
use crate::server::Server;

const NAMES: [&str; 1] = ["gamerule"];

const DESCRIPTION: &str = "Sets or queries a game rule value.";

const ARG_NAME: &str = "value";

struct QueryExecutor(GameRule);

#[async_trait]
impl CommandExecutor for QueryExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let key = TextComponent::text(self.0.to_string());
        let level_info = server.level_info.read().await;
        let value = TextComponent::text(level_info.game_rules.get(&self.0).to_string());
        drop(level_info);

        sender
            .send_message(TextComponent::translate(
                "commands.gamerule.query",
                [key, value],
            ))
            .await;
        Ok(())
    }
}

struct SetExecutor(GameRule);

#[async_trait]
impl CommandExecutor for SetExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let key = TextComponent::text(self.0.to_string());
        let mut level_info = server.level_info.write().await;
        let raw_value = level_info.game_rules.get_mut(&self.0);

        let value = TextComponent::text(match raw_value {
            GameRuleValue::Int(value) => {
                let arg_value = BoundedNumArgumentConsumer::<i64>::find_arg(args, ARG_NAME)??;
                *value = arg_value;
                arg_value.to_string()
            }
            GameRuleValue::Bool(value) => {
                let arg_value = BoolArgConsumer::find_arg(args, ARG_NAME)?;
                *value = arg_value;
                arg_value.to_string()
            }
        });
        drop(level_info);

        sender
            .send_message(TextComponent::translate(
                "commands.gamerule.set",
                [key, value],
            ))
            .await;
        Ok(())
    }
}

#[allow(clippy::redundant_closure_for_method_calls)]
pub fn init_command_tree() -> CommandTree {
    let mut command_tree = CommandTree::new(NAMES, DESCRIPTION);
    let rule_registry = GameRuleRegistry::default();
    for rule in GameRule::all() {
        let arg = match rule_registry.get(rule) {
            GameRuleValue::Int(_) => argument(ARG_NAME, BoundedNumArgumentConsumer::<i64>::new()),
            GameRuleValue::Bool(_) => argument(ARG_NAME, BoolArgConsumer),
        };
        command_tree = command_tree.then(
            literal(rule.to_string())
                .execute(QueryExecutor(rule.clone()))
                .then(arg.execute(SetExecutor(rule.clone()))),
        );
    }
    command_tree
}
