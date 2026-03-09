use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::{
    CommandError, CommandExecutor, CommandResult, CommandSender,
    args::{
        ConsumedArgs, FindArg, position_3d::Position3DArgumentConsumer, simple::SimpleArgConsumer,
    },
    tree::{
        CommandTree,
        builder::{argument, literal},
    },
};

const NAMES: [&str; 1] = ["place"];
const DESCRIPTION: &str = "Places a structure, feature, jigsaw, or template.";
const ARG_NAME: &str = "name";
const ARG_POS: &str = "pos";

struct StructureExecutor;

impl CommandExecutor for StructureExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let name = SimpleArgConsumer::find_arg(args, ARG_NAME)?;
            // Position is optional in vanilla; defaults to sender position
            let _pos = Position3DArgumentConsumer::find_arg(args, ARG_POS).ok();

            // TODO: Implement structure placement when worldgen adapter is available
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_PLACE_STRUCTURE_FAILED,
                    [TextComponent::text(name.to_string())],
                ))
                .await;

            Err(CommandError::InvalidConsumption(Some(name.to_string())))
        })
    }
}

struct FeatureExecutor;

impl CommandExecutor for FeatureExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let name = SimpleArgConsumer::find_arg(args, ARG_NAME)?;
            let _pos = Position3DArgumentConsumer::find_arg(args, ARG_POS).ok();

            // TODO: Implement feature placement when worldgen adapter is available
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_PLACE_FEATURE_FAILED,
                    [TextComponent::text(name.to_string())],
                ))
                .await;

            Err(CommandError::InvalidConsumption(Some(name.to_string())))
        })
    }
}

struct JigsawExecutor;

impl CommandExecutor for JigsawExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let name = SimpleArgConsumer::find_arg(args, ARG_NAME)?;
            let _pos = Position3DArgumentConsumer::find_arg(args, ARG_POS).ok();

            // TODO: Implement jigsaw placement when worldgen adapter is available
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_PLACE_JIGSAW_FAILED,
                    [TextComponent::text(name.to_string())],
                ))
                .await;

            Err(CommandError::InvalidConsumption(Some(name.to_string())))
        })
    }
}

struct TemplateExecutor;

impl CommandExecutor for TemplateExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let name = SimpleArgConsumer::find_arg(args, ARG_NAME)?;
            let _pos = Position3DArgumentConsumer::find_arg(args, ARG_POS).ok();

            // TODO: Implement template placement when worldgen adapter is available
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_PLACE_TEMPLATE_FAILED,
                    [TextComponent::text(name.to_string())],
                ))
                .await;

            Err(CommandError::InvalidConsumption(Some(name.to_string())))
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("structure").then(
                argument(ARG_NAME, SimpleArgConsumer)
                    .then(argument(ARG_POS, Position3DArgumentConsumer).execute(StructureExecutor))
                    .execute(StructureExecutor),
            ),
        )
        .then(
            literal("feature").then(
                argument(ARG_NAME, SimpleArgConsumer)
                    .then(argument(ARG_POS, Position3DArgumentConsumer).execute(FeatureExecutor))
                    .execute(FeatureExecutor),
            ),
        )
        .then(
            literal("jigsaw").then(
                argument(ARG_NAME, SimpleArgConsumer)
                    .then(argument(ARG_POS, Position3DArgumentConsumer).execute(JigsawExecutor))
                    .execute(JigsawExecutor),
            ),
        )
        .then(
            literal("template").then(
                argument(ARG_NAME, SimpleArgConsumer)
                    .then(argument(ARG_POS, Position3DArgumentConsumer).execute(TemplateExecutor))
                    .execute(TemplateExecutor),
            ),
        )
}
