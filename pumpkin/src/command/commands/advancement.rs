use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::resource_key::{ADVANCEMENT_REGISTRY, ResourceKeyArgument};
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;
use crate::entity::player::Player;
use pumpkin_data::{Advancement, translation, ADVANCEMENT_TREE};
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use std::sync::Arc;
use pumpkin_data::advancement_data::AdvancementNode;

const NAME: &str = "advancement";
const DESCRIPTION: &str = "manage advancement of the player";
const PERMISSION: &str = "minecraft:command.advancement";

const ARG_TARGETS: &str = "targets";
const ARG_ADVANCEMENT: &str = "advancement";

#[allow(unused)]
const ERROR_CRITERION_NOT_FOUND: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_ADVANCEMENT_CRITERIONNOTFOUND,
    translation::java::COMMANDS_ADVANCEMENT_CRITERIONNOTFOUND,
);
const ERROR_GRANT_ONE_TO_ONE: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_ADVANCEMENT_GRANT_ONE_TO_ONE_FAILURE,
    translation::java::COMMANDS_ADVANCEMENT_GRANT_ONE_TO_ONE_FAILURE,
);
const ERROR_REVOKE_ONE_TO_ONE: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_ADVANCEMENT_REVOKE_ONE_TO_ONE_FAILURE,
    translation::java::COMMANDS_ADVANCEMENT_REVOKE_ONE_TO_ONE_FAILURE,
);
const ERROR_GRANT_ONE_TO_MANY: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_ADVANCEMENT_GRANT_ONE_TO_MANY_FAILURE,
    translation::java::COMMANDS_ADVANCEMENT_GRANT_ONE_TO_MANY_FAILURE,
);
const ERROR_REVOKE_ONE_TO_MANY: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_ADVANCEMENT_REVOKE_ONE_TO_MANY_FAILURE,
    translation::java::COMMANDS_ADVANCEMENT_REVOKE_ONE_TO_MANY_FAILURE,
);
const ERROR_GRANT_MANY_TO_ONE: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_ADVANCEMENT_GRANT_MANY_TO_ONE_FAILURE,
    translation::java::COMMANDS_ADVANCEMENT_GRANT_MANY_TO_ONE_FAILURE,
);
const ERROR_REVOKE_MANY_TO_ONE: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_ADVANCEMENT_REVOKE_MANY_TO_ONE_FAILURE,
    translation::java::COMMANDS_ADVANCEMENT_REVOKE_MANY_TO_ONE_FAILURE,
);
const ERROR_GRANT_MANY_TO_MANY: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_ADVANCEMENT_GRANT_MANY_TO_MANY_FAILURE,
    translation::java::COMMANDS_ADVANCEMENT_GRANT_MANY_TO_MANY_FAILURE,
);
const ERROR_REVOKE_MANY_TO_MANY: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_ADVANCEMENT_REVOKE_MANY_TO_MANY_FAILURE,
    translation::java::COMMANDS_ADVANCEMENT_REVOKE_MANY_TO_MANY_FAILURE,
);

#[derive(Clone, Copy)]
enum Action {
    Grant,
    Revoke,
}

impl Action {
    async fn perform(
        &self,
        player: &Arc<Player>,
        advancements: &Vec<&'static Advancement>,
        grant_everything: bool,
    ) -> i32 {
        let mut count = 0;

        if !grant_everything {
            player.advancements.lock().await.flush_dirty(player,true).await;
        }

        for advancement in advancements {
            if self.perform_single(player, advancement).await {
                count += 1;
            }
        }

        if !grant_everything {
            player.advancements.lock().await.flush_dirty(player,false).await;
        }
        count
    }

    async fn perform_single(
        &self,
        player: &Arc<Player>,
        advancement: &'static Advancement,
    ) -> bool {
        let mut guard = player.advancements.lock().await;
        match self {
            Self::Grant => {
                let progress = guard.get_or_start_progress(advancement);
                if progress.is_done() {
                    false
                } else {
                    guard.award(advancement).await;
                    true
                }
            }

            Self::Revoke => {
                let progress = guard.get_or_start_progress(advancement);
                if progress.has_progress() {
                    guard.revoke(advancement);
                    true
                } else {
                    false
                }
            }
        }
    }
    const fn get_key(&self) -> &str {
        match self {
            Self::Grant => "grant",
            Self::Revoke => "revoke",
        }
    }
}
#[derive(Clone, Copy)]
#[allow(unused)]
enum Mode {
    Only,
    Through,
    From,
    Until,
    Everything,
}

impl Mode {
    const fn parents(self) -> bool {
        match self {
            Self::Only | Self::From => false,
            Self::Through | Self::Until | Self::Everything => true,
        }
    }

    const fn children(self) -> bool {
        match self {
            Self::Only | Self::Until => false,
            Self::Through | Self::From | Self::Everything => true,
        }
    }
}

fn get_advancements(
    target: &Advancement,
    mode: Mode,
) -> Vec<&Advancement> {
    let tree = ADVANCEMENT_TREE;
    let target_node = tree.get_node_from_id(&target.id);
    if let Some(target_node) = target_node {
        let mut advancements = Vec::new();
        if mode.parents() {
            let parent = target_node.parent;
            while let Some(parent) = parent {
                advancements.push(tree.nodes_vector[parent].value);
            }
        }
        advancements.push(target);
        if mode.children() {
            add_children(target_node,&mut advancements);
        }
        advancements
    }else {
        vec![target]
    }
}

fn add_children(parent: &AdvancementNode, output:&mut Vec<&Advancement>) {
    for child in parent.children {
        let node = &ADVANCEMENT_TREE.nodes_vector[child];
        output.push(node.value);
        add_children(node, output);
    }
}

async fn perform_everything(
    context: Arc<CommandSource>,
    players: Vec<Arc<Player>>,
    action: Action,
    advancements: Vec<&'static Advancement>,
) -> Result<i32, CommandSyntaxError> {
    perform(context, players, action, advancements, true).await
}

#[allow(clippy::too_many_lines)]
async fn perform(
    context: Arc<CommandSource>,
    targets: Vec<Arc<Player>>,
    action: Action,
    advancements: Vec<&'static Advancement>,
    grant_everything: bool,
) -> Result<i32, CommandSyntaxError> {
    let mut i = 0;
    for player in &targets {
        i += action
            .perform(player, &advancements, grant_everything)
            .await;
    }
    if i == 0 {
        return if let [first_advancement] = advancements[..] {
            if let [first_player] = targets.as_slice() {
                Err(match action {
                    Action::Grant => &ERROR_GRANT_ONE_TO_ONE,
                    Action::Revoke => &ERROR_REVOKE_ONE_TO_ONE,
                }
                .create_without_context_args_slice(&[
                    first_advancement.name(),
                    first_player.get_display_name().await,
                ]))
            } else {
                Err(match action {
                    Action::Grant => &ERROR_GRANT_ONE_TO_MANY,
                    Action::Revoke => &ERROR_REVOKE_ONE_TO_MANY,
                }
                .create_without_context_args_slice(&[
                    first_advancement.name(),
                    TextComponent::text(targets.len().to_string()),
                ]))
            }
        } else if let [first_player] = targets.as_slice() {
            Err(match action {
                Action::Grant => &ERROR_GRANT_MANY_TO_ONE,
                Action::Revoke => &ERROR_REVOKE_MANY_TO_ONE,
            }
            .create_without_context_args_slice(&[
                TextComponent::text(advancements.len().to_string()),
                first_player.get_display_name().await,
            ]))
        } else {
            Err(match action {
                Action::Grant => &ERROR_GRANT_MANY_TO_MANY,
                Action::Revoke => &ERROR_REVOKE_MANY_TO_MANY,
            }
            .create_without_context_args_slice(&[
                TextComponent::text(advancements.len().to_string()),
                TextComponent::text(targets.len().to_string()),
            ]))
        };
    }
    if let [first_advancement] = advancements[..] {
        if let [first_player] = targets.as_slice() {
            context
                .send_feedback(
                    TextComponent::translate(
                        format!(
                            "commands.advancement.{}.one.to.one.success",
                            action.get_key()
                        ),
                        [
                            first_advancement.name(),
                            first_player.get_display_name().await,
                        ],
                    ),
                    true,
                )
                .await;
        } else {
            context
                .send_feedback(
                    TextComponent::translate(
                        format!(
                            "commands.advancement.{}.one.to.many.success",
                            action.get_key()
                        ),
                        [
                            first_advancement.name(),
                            TextComponent::text(targets.len().to_string()),
                        ],
                    ),
                    true,
                )
                .await;
        }
    } else if let [first] = targets.as_slice() {
        context
            .send_feedback(
                TextComponent::translate(
                    format!(
                        "commands.advancement.{}.many.to.many.success",
                        action.get_key()
                    ),
                    [
                        TextComponent::text(advancements.len().to_string()),
                        first.get_display_name().await,
                    ],
                ),
                true,
            )
            .await;
    } else {
        context
            .send_feedback(
                TextComponent::translate(
                    format!(
                        "commands.advancement.{}.many.to.many.success",
                        action.get_key()
                    ),
                    [
                        TextComponent::text(advancements.len().to_string()),
                        TextComponent::text(targets.len().to_string()),
                    ],
                ),
                true,
            )
            .await;
    }
    Ok(i)
}

struct AdvancementExecutor {
    action: Action,
}

impl CommandExecutor for AdvancementExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        let action = self.action;
        Box::pin(async move {
            perform_everything(
                context.source.clone(),
                EntityArgumentType::get_players(context, ARG_TARGETS).await?,
                action,
                get_advancements(
                    ResourceKeyArgument::get_advancement(context, ARG_ADVANCEMENT)?,
                    Mode::Only,
                ),
            )
            .await
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let build_action = |name: &'static str, action: Action| {
        literal(name).then(
            argument(ARG_TARGETS, EntityArgumentType::Players).then(
                literal("only").then(
                    argument(
                        ARG_ADVANCEMENT,
                        ResourceKeyArgument(ADVANCEMENT_REGISTRY.clone()),
                    )
                    .executes(AdvancementExecutor { action }),
                ),
            ),
        )
    };

    dispatcher.register(
        command(NAME, DESCRIPTION)
            .requires(PERMISSION)
            .then(build_action("grant", Action::Grant))
            .then(build_action("revoke", Action::Revoke)),
    );
}
