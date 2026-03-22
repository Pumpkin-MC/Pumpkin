use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::{
    CommandError, CommandExecutor, CommandResult, CommandSender,
    args::{
        ConsumedArgs, FindArg, bounded_num::BoundedNumArgumentConsumer,
        position_3d::Position3DArgumentConsumer, simple::SimpleArgConsumer,
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
const ARG_POOL: &str = "pool";
const ARG_TARGET: &str = "target";
const ARG_MAX_DEPTH: &str = "max_depth";

const fn max_depth_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new()
        .name("max_depth")
        .min(1)
        .max(20)
}

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
            let _pos = Position3DArgumentConsumer::find_arg(args, ARG_POS).ok();

            // TODO: Implement structure placement when worldgen adapter is available
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_PLACE_STRUCTURE_FAILED,
                    [TextComponent::text(name.to_string())],
                ))
                .await;

            Err(CommandError::CommandFailed(TextComponent::text(format!(
                "Failed to place structure {name}"
            ))))
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

            Err(CommandError::CommandFailed(TextComponent::text(format!(
                "Failed to place feature {name}"
            ))))
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
            let pool = SimpleArgConsumer::find_arg(args, ARG_POOL)?;
            let _target = SimpleArgConsumer::find_arg(args, ARG_TARGET)?;
            let _max_depth = BoundedNumArgumentConsumer::<i32>::find_arg(args, ARG_MAX_DEPTH)?;
            let _pos = Position3DArgumentConsumer::find_arg(args, ARG_POS).ok();

            // TODO: Implement jigsaw placement when worldgen adapter is available
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_PLACE_JIGSAW_FAILED,
                    [TextComponent::text(pool.to_string())],
                ))
                .await;

            Err(CommandError::CommandFailed(TextComponent::text(format!(
                "Failed to place jigsaw {pool}"
            ))))
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

            Err(CommandError::CommandFailed(TextComponent::text(format!(
                "Failed to place template {name}"
            ))))
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
            // /place jigsaw <pool> <target> <max_depth> [<pos>]
            literal("jigsaw").then(
                argument(ARG_POOL, SimpleArgConsumer).then(
                    argument(ARG_TARGET, SimpleArgConsumer).then(
                        argument(ARG_MAX_DEPTH, max_depth_consumer())
                            .then(
                                argument(ARG_POS, Position3DArgumentConsumer)
                                    .execute(JigsawExecutor),
                            )
                            .execute(JigsawExecutor),
                    ),
                ),
            ),
        )
        .then(
            // /place template <name> [<pos>] [<rotation>] [<mirror>] [<integrity>] [<seed>]
            // TODO: Add rotation, mirror, integrity, seed optional arguments
            literal("template").then(
                argument(ARG_NAME, SimpleArgConsumer)
                    .then(argument(ARG_POS, Position3DArgumentConsumer).execute(TemplateExecutor))
                    .execute(TemplateExecutor),
            ),
        )
}
