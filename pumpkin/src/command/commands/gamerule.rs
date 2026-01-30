use pumpkin_data::game_rules::{GameRule, GameRuleRegistry, GameRuleValue};

use crate::command::args::FindArg;
use crate::command::args::bool::BoolArgConsumer;
use crate::command::args::bounded_num::BoundedNumArgumentConsumer;

use crate::TextComponent;

use crate::command::args::ConsumedArgs;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 1] = ["gamerule"];

const DESCRIPTION: &str = "Sets or queries a game rule value.";

const ARG_NAME: &str = "value";

struct QueryExecutor(GameRule);

impl CommandExecutor for QueryExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let key = TextComponent::text(self.0.to_string());
            let level_info = server.level_info.load();
            let game_rule = level_info.game_rules.get(&self.0);
            let game_rule_i32_value = match game_rule {
                GameRuleValue::Int(value) => {
                    (*value).clamp(i32::MIN as i64, i32::MAX as i64) as i32
                }
                GameRuleValue::Bool(value) => {
                    *value as i32
                }
            };
            let value = TextComponent::text(game_rule.to_string());
            drop(level_info);

            sender
                .send_message(TextComponent::translate(
                    "commands.gamerule.query",
                    [key, value],
                ))
                .await;

            Ok(game_rule_i32_value)
        })
    }
}

struct SetExecutor(GameRule);

impl CommandExecutor for SetExecutor {
    #[expect(unused)]
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let key = TextComponent::text(self.0.to_string());
            let current_info = server.level_info.load();

            let mut new_info = (**current_info).clone();

            let mut output_value = String::new();
            let mut result_i32: i32;

            let raw_value = new_info.game_rules.get_mut(&self.0);

            match raw_value {
                GameRuleValue::Int(value) => {
                    let arg_value = BoundedNumArgumentConsumer::<i64>::find_arg(args, ARG_NAME)??;
                    *value = arg_value;
                    output_value = arg_value.to_string();
                    // TODO: Should integer gamerule values be kept as a `i64` or should it be changed to an `i32`?
                    // For now, we can cast it
                    result_i32 = arg_value.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
                }
                GameRuleValue::Bool(value) => {
                    let arg_value = BoolArgConsumer::find_arg(args, ARG_NAME)?;
                    *value = arg_value;
                    output_value = arg_value.to_string();
                    result_i32 = *value as i32;
                }
            }

            server.level_info.store(std::sync::Arc::new(new_info));

            let value_component = TextComponent::text(output_value);
            sender
                .send_message(TextComponent::translate(
                    "commands.gamerule.set",
                    [key, value_component],
                ))
                .await;

            Ok(result_i32)
        })
    }
}

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
