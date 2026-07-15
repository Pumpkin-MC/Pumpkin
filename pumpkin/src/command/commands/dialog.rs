use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::dialog::{DialogArg, DialogArgumentType};
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::LiteralCommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use pumpkin_protocol::IdOr;
use pumpkin_protocol::java::client::dialog::DialogNBT;
use pumpkin_protocol::java::client::play::{CPlayClearDialog, CPlayShowDialog};
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::translation::localized_text;

const DESCRIPTION: &str = "commands.dialog.description";
const PERMISSION: &str = "minecraft:command.dialog";

const ARG_TARGETS: &str = "targets";
const ARG_DIALOG: &str = "dialog";

struct DialogClearExecutor;

impl CommandExecutor for DialogClearExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_players(context, ARG_TARGETS).await?;

            let count = targets.len();
            let packet = CPlayClearDialog::new();
            for player in &targets {
                player.client.send_packet_now(&packet).await;
            }

            let msg = if count == 1 {
                localized_text(
                    "commands.dialog.clear.success.single",
                    [TextComponent::text(targets[0].gameprofile.name.clone())],
                )
            } else {
                localized_text(
                    "commands.dialog.clear.success.multiple",
                    [TextComponent::text(count.to_string())],
                )
            };
            context.source.send_feedback(msg, true).await;

            Ok(count as i32)
        })
    }
}

static REGISTRY_ERROR: LiteralCommandErrorType =
    LiteralCommandErrorType::new("commands.dialog.error.registry_unsupported");

struct DialogShowExecutor;

impl CommandExecutor for DialogShowExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_players(context, ARG_TARGETS).await?;
            let dialog_arg = DialogArgumentType::get(context, ARG_DIALOG)?;

            match dialog_arg {
                DialogArg::Nbt(compound) => {
                    let count = targets.len();
                    let dialog_nbt = DialogNBT::from_nbt(compound);
                    let packet = CPlayShowDialog::new(IdOr::Value(dialog_nbt));

                    for player in &targets {
                        player.client.send_packet_now(&packet).await;
                    }

                    let msg = if count == 1 {
                        localized_text(
                            "commands.dialog.show.success.single",
                            [TextComponent::text(targets[0].gameprofile.name.clone())],
                        )
                    } else {
                        localized_text(
                            "commands.dialog.show.success.multiple",
                            [TextComponent::text(count.to_string())],
                        )
                    };
                    context.source.send_feedback(msg, true).await;

                    Ok(count as i32)
                }
                DialogArg::Id(_id) => Err(REGISTRY_ERROR.create_without_context()),
            }
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("dialog", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("clear").then(
                argument(ARG_TARGETS, EntityArgumentType::Players).executes(DialogClearExecutor),
            ))
            .then(
                literal("show").then(
                    argument(ARG_TARGETS, EntityArgumentType::Players).then(
                        argument(ARG_DIALOG, DialogArgumentType).executes(DialogShowExecutor),
                    ),
                ),
            ),
    );
}
