use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::resource_key::{ADVANCEMENT_REGISTRY, ResourceKeyArgument};
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::suggestion::provider::{SuggestionProvider, SuggestionProviderResult};
use crate::command::suggestion::suggestions::SuggestionsBuilder;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::entity::player::advancement::PlayerAdvancement;
use futures::future::join_all;
use pumpkin_data::advancement_data::AdvancementNode;
use pumpkin_data::{ADVANCEMENT_TREE, Advancement, translation};
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use std::sync::Arc;

const NAME: &str = "advancement";
const DESCRIPTION: &str = "manage advancement of players";
const PERMISSION: &str = "minecraft:command.advancement";

const ARG_TARGETS: &str = "targets";
const ARG_ADVANCEMENT: &str = "advancement";
const ARG_CRITERION: &str = "criterion";

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

const ERROR_GRANT_CRITERION_TO_ONE_FAILURE: CommandErrorType<3> = CommandErrorType::new(
    translation::java::COMMANDS_ADVANCEMENT_GRANT_CRITERION_TO_ONE_FAILURE,
    translation::java::COMMANDS_ADVANCEMENT_GRANT_CRITERION_TO_ONE_FAILURE,
);

const ERROR_REVOKE_CRITERION_TO_ONE_FAILURE: CommandErrorType<3> = CommandErrorType::new(
    translation::java::COMMANDS_ADVANCEMENT_REVOKE_CRITERION_TO_ONE_FAILURE,
    translation::java::COMMANDS_ADVANCEMENT_REVOKE_CRITERION_TO_ONE_FAILURE,
);

const ERROR_GRANT_CRITERION_TO_MANY_FAILURE: CommandErrorType<3> = CommandErrorType::new(
    translation::java::COMMANDS_ADVANCEMENT_GRANT_CRITERION_TO_MANY_FAILURE,
    translation::java::COMMANDS_ADVANCEMENT_GRANT_CRITERION_TO_MANY_FAILURE,
);

const ERROR_REVOKE_CRITERION_TO_MANY_FAILURE: CommandErrorType<3> = CommandErrorType::new(
    translation::java::COMMANDS_ADVANCEMENT_REVOKE_CRITERION_TO_MANY_FAILURE,
    translation::java::COMMANDS_ADVANCEMENT_REVOKE_CRITERION_TO_MANY_FAILURE,
);

#[derive(Clone, Copy)]
pub enum Action {
    Grant,
    Revoke,
}

impl Action {
    /// inner function that directly take the locked [`PlayerAdvancement`]
    fn perform_single_inner(
        self,
        guard: &mut PlayerAdvancement,
        advancement: &'static Advancement,
    ) -> bool {
        let progress = guard.progress.get_mut_or_start_progress(advancement);
        match self {
            Self::Grant => {
                if progress.is_done() {
                    return false;
                }
                let criteria: Vec<Arc<str>> = progress.get_remaining_criteria().collect();
                for criterion in criteria {
                    guard.award(advancement, &criterion);
                }
                true
            }
            Self::Revoke => {
                if !progress.is_done() {
                    return false;
                }
                let criteria: Vec<Arc<str>> = progress.get_completed_criteria().collect();
                for criterion in criteria {
                    guard.revoke(advancement, &criterion);
                }
                true
            }
        }
    }

    /// Performs the action (grant or revoke) on multiple advancements for a single player.
    ///
    /// This method applies the current action to each advancement in the provided slice for the
    /// specified player. It locks the player's advancements and processes each advancement
    /// sequentially, collecting the count of successful operations.
    ///
    /// # Arguments
    ///
    /// * `player` - The player whose advancements will be modified
    /// * `advancements` - A slice of advancements to apply the action to
    /// * `show_advancement` - A flag to control advancement notification display (currently unused)
    ///
    /// # Returns
    ///
    /// Returns the number of advancements successfully modified. An advancement is counted as
    /// successful if [`perform_single_inner`] returns `true`
    async fn perform(
        &self,
        player: &Arc<Player>,
        advancements: &[&'static Advancement],
        show_advancement: bool,
    ) -> i32 {
        let mut guard = player.advancements.lock().await;
        if !show_advancement {
            guard.flush_dirty(player, true);
        }
        let count = advancements
            .iter()
            .filter(|advancement| self.perform_single_inner(&mut guard, advancement))
            .count() as i32;
        if !show_advancement {
            guard.flush_dirty(player, false);
        }
        count
    }

    async fn perform_criterion(
        &self,
        player: &Arc<Player>,
        advancement: &'static Advancement,
        criterion: &str,
    ) -> bool {
        let mut guard = player.advancements.lock().await;
        match self {
            Self::Grant => guard.award(advancement, criterion),
            Self::Revoke => guard.revoke(advancement, criterion),
        }
    }

    /// return the corresponding key of the action
    const fn get_key(&self) -> &str {
        match self {
            Self::Grant => "commands.advancement.grant",
            Self::Revoke => "commands.advancement.revoke",
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

/// Retrieves a collection of advancements based on the target advancement and traversal mode.
///
/// This function builds a list of advancements by traversing the advancement tree according to
/// the specified mode. The traversal can include parent advancements, child advancements, or both,
/// depending on the mode selected.
///
/// # Arguments
///
/// * `target` - The advancement to use as the starting point for traversal
/// * `mode` - The traversal mode that determines which advancements to include:
///   - `Mode::Only` - Returns only the target advancement
///   - `Mode::From` - Returns the target and all its descendants
///   - `Mode::Until` - Returns the target and all its ancestors
///   - `Mode::Through` - Returns the target and both ancestors and descendants
///   - `Mode::Everything` - Returns the target and all ancestors and descendants (never used)
///
/// # Returns
///
/// A vector of references to advancements matching the specified mode. If the target advancement
/// is not found in the tree, a vector containing only the target is returned.
fn get_advancements(target: &Advancement, mode: Mode) -> Vec<&Advancement> {
    let tree = &ADVANCEMENT_TREE;
    let target_node = tree.get_node_from_id(&target.id);
    target_node.map_or_else(
        || vec![target],
        |target_node| {
            let mut advancements = Vec::new();
            if mode.parents() {
                let mut parent = target_node.parent;
                while let Some(parent_id) = parent {
                    let current_node = &tree.nodes_vector[parent_id];
                    advancements.push(current_node.value);
                    parent = current_node.parent;
                }
            }
            advancements.push(target);
            if mode.children() {
                add_children(target_node, &mut advancements);
            }
            advancements
        },
    )
}

fn add_children(parent: &AdvancementNode, output: &mut Vec<&Advancement>) {
    for child in &parent.children {
        let node = &ADVANCEMENT_TREE.nodes_vector[*child];
        output.push(node.value);
        add_children(node, output);
    }
}

#[inline]
async fn perform_and_show(
    context: Arc<CommandSource>,
    players: &[Arc<Player>],
    action: Action,
    advancements: &[&'static Advancement],
) -> Result<i32, CommandSyntaxError> {
    perform(context, players, action, advancements, true).await
}

/// Performs a batch action (grant or revoke) on multiple advancements for multiple players.
///
/// This function iterates through each player and applies the specified action to all provided
/// advancements. It automatically handles error messaging based on the number of players and
/// advancements involved, as well as the number of successful operations.
///
/// # Arguments
///
/// * `context` - The command source context used to send feedback messages to the command executor
/// * `targets` - Slice of players who will be affected by the action
/// * `action` - The action to perform on each advancement (Grant or Revoke)
/// * `advancements` - Slice of advancements to apply the action to
/// * `show_advancement` - Whether to show advancement notifications to players (currently unused)
///
/// # Returns
///
/// Returns `Ok(i)` with the total count of successful operations if at least one advancement
/// was successfully modified for at least one player.
///
/// Returns `Err` with an appropriate localized error message if no operations succeeded. The
/// error message varies based on:
/// - Whether one or many players were targeted
/// - Whether one or many advancements were involved
/// - The type of action (Grant or Revoke)
async fn perform(
    context: Arc<CommandSource>,
    targets: &[Arc<Player>],
    action: Action,
    advancements: &[&'static Advancement],
    show_advancement: bool,
) -> Result<i32, CommandSyntaxError> {
    let mut i = 0;
    for player in targets {
        i += action.perform(player, advancements, show_advancement).await;
    }
    if i == 0 {
        return if let [first_advancement] = advancements[..] {
            if let [first_player] = targets {
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
        } else if let [first_player] = targets {
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
    let translate = if let [first_advancement] = advancements[..] {
        if let [first_player] = targets {
            TextComponent::translate(
                format!("{}.one.to.one.success", action.get_key()),
                [
                    first_advancement.name(),
                    first_player.get_display_name().await,
                ],
            )
        } else {
            TextComponent::translate(
                format!("{}.one.to.many.success", action.get_key()),
                [
                    first_advancement.name(),
                    TextComponent::text(targets.len().to_string()),
                ],
            )
        }
    } else if let [first] = targets {
        TextComponent::translate(
            format!("{}.many.to.one.success", action.get_key()),
            [
                TextComponent::text(advancements.len().to_string()),
                first.get_display_name().await,
            ],
        )
    } else {
        TextComponent::translate(
            format!("{}.many.to.many.success", action.get_key()),
            [
                TextComponent::text(advancements.len().to_string()),
                TextComponent::text(targets.len().to_string()),
            ],
        )
    };
    context.send_feedback(translate, true).await;
    Ok(i)
}

/// Performs an action (grant or revoke) on a specific advancement criterion for multiple players.
///
/// This function attempts to apply the specified action to a criterion of an advancement for each
/// of the given players. It handles error reporting based on the number of successful operations
/// and provides feedback to the command source.
///
/// # Arguments
///
/// * `context` - The command source context for sending feedback
/// * `targets` - The players to apply the action to
/// * `action` - The action to perform (Grant or Revoke)
/// * `advancement` - The advancement containing the criterion
/// * `criterion` - The specific criterion name to operate on
///
/// # Returns
///
/// Returns `Ok(count)` with the number of successful operations if at least one succeeded.
/// Returns `Err` with an appropriate error message if:
/// - The criterion doesn't exist in the advancement
/// - No operations succeeded
pub async fn perform_criterion(
    context: Arc<CommandSource>,
    targets: &[Arc<Player>],
    action: Action,
    advancement: &'static Advancement,
    criterion: &str,
) -> Result<i32, CommandSyntaxError> {
    if advancement.criteria.contains(&criterion) {
        let count = join_all(
            targets
                .iter()
                .map(|player| action.perform_criterion(player, advancement, criterion)),
        )
        .await
        .into_iter()
        .filter(|&success| success)
        .count() as i32;
        if count == 0 {
            if let [first_player] = targets {
                Err(match action {
                    Action::Grant => &ERROR_GRANT_CRITERION_TO_ONE_FAILURE,
                    Action::Revoke => &ERROR_REVOKE_CRITERION_TO_ONE_FAILURE,
                }
                .create_without_context_args_slice(&[
                    TextComponent::text(criterion.to_owned()),
                    advancement.name(),
                    first_player.get_display_name().await,
                ]))
            } else {
                Err(match action {
                    Action::Grant => &ERROR_GRANT_CRITERION_TO_MANY_FAILURE,
                    Action::Revoke => &ERROR_REVOKE_CRITERION_TO_MANY_FAILURE,
                }
                .create_without_context_args_slice(&[
                    TextComponent::text(criterion.to_owned()),
                    advancement.name(),
                    TextComponent::text(targets.len().to_string()),
                ]))
            }
        } else {
            let translate = if let [first_player] = targets {
                TextComponent::translate(
                    format!("{}.criterion.to.one.success", action.get_key()),
                    [
                        TextComponent::text(criterion.to_owned()),
                        advancement.name(),
                        first_player.get_display_name().await,
                    ],
                )
            } else {
                TextComponent::translate(
                    format!("{}.criterion.to.many.success", action.get_key()),
                    [
                        TextComponent::text(criterion.to_owned()),
                        advancement.name(),
                        TextComponent::text(targets.len().to_string()),
                    ],
                )
            };
            context.send_feedback(translate, true).await;
            Ok(count)
        }
    } else {
        Err(ERROR_CRITERION_NOT_FOUND.create_without_context(
            advancement.name(),
            TextComponent::text(criterion.to_owned()),
        ))
    }
}

/// use for when a criterion is specified to on grant/revoke this criterion
struct OnlyAdvancementCriterionExecutor {
    action: Action,
}

impl CommandExecutor for OnlyAdvancementCriterionExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        let action = self.action;
        Box::pin(async move {
            perform_criterion(
                context.source.clone(),
                &EntityArgumentType::get_players(context, ARG_TARGETS).await?,
                action,
                ResourceKeyArgument::get_advancement(context, ARG_ADVANCEMENT)?,
                StringArgumentType::get(context, ARG_CRITERION)?,
            )
            .await
        })
    }
}

/// use to grant/revoke advancement depending on the selected `mode`
struct AdvancementExecutor {
    action: Action,
    mode: Mode,
}

impl CommandExecutor for AdvancementExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        let action = self.action;
        let mode = self.mode;
        Box::pin(async move {
            perform_and_show(
                context.source.clone(),
                &EntityArgumentType::get_players(context, ARG_TARGETS).await?,
                action,
                &get_advancements(
                    ResourceKeyArgument::get_advancement(context, ARG_ADVANCEMENT)?,
                    mode,
                ),
            )
            .await
        })
    }
}

/// suggest the corresponding criterion of the specified advancement
struct CriterionSuggestionProvider;

impl SuggestionProvider for CriterionSuggestionProvider {
    fn suggest<'a>(
        &'a self,
        context: &'a CommandContext,
        builder: SuggestionsBuilder,
    ) -> SuggestionProviderResult<'a> {
        let suggestion = ResourceKeyArgument::get_advancement(context, ARG_ADVANCEMENT)
            .ok()
            .map(|adv| adv.criteria)
            .into_iter()
            .flatten()
            .map(ToString::to_string);
        Box::pin(async move { builder.filter_and_suggest_iter(suggestion).build() })
    }
}

/// executor to grant/revoke every advancement to specified players
struct EveryAdvancementExecutor {
    action: Action,
}

impl CommandExecutor for EveryAdvancementExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        let action = self.action;
        Box::pin(async move {
            perform(
                context.source.clone(),
                &EntityArgumentType::get_players(context, ARG_TARGETS).await?,
                action,
                &Advancement::get_advancements_list(),
                false,
            )
            .await
        })
    }
}

/// register the advancement command
pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    macro_rules! build_action {
        ($name:expr, $action:expr) => {
            literal($name).then(
                argument(ARG_TARGETS, EntityArgumentType::Players)
                    .then(
                        literal("only").then(
                            argument(
                                ARG_ADVANCEMENT,
                                ResourceKeyArgument(ADVANCEMENT_REGISTRY.clone()),
                            )
                            .executes(AdvancementExecutor {
                                action: $action,
                                mode: Mode::Only,
                            })
                            .then(
                                argument(ARG_CRITERION, StringArgumentType::GreedyPhrase)
                                    .suggests(CriterionSuggestionProvider)
                                    .executes(OnlyAdvancementCriterionExecutor { action: $action }),
                            ),
                        ),
                    )
                    .then(
                        literal("from").then(
                            argument(
                                ARG_ADVANCEMENT,
                                ResourceKeyArgument(ADVANCEMENT_REGISTRY.clone()),
                            )
                            .executes(AdvancementExecutor {
                                action: $action,
                                mode: Mode::From,
                            }),
                        ),
                    )
                    .then(
                        literal("until").then(
                            argument(
                                ARG_ADVANCEMENT,
                                ResourceKeyArgument(ADVANCEMENT_REGISTRY.clone()),
                            )
                            .executes(AdvancementExecutor {
                                action: $action,
                                mode: Mode::Until,
                            }),
                        ),
                    )
                    .then(
                        literal("through").then(
                            argument(
                                ARG_ADVANCEMENT,
                                ResourceKeyArgument(ADVANCEMENT_REGISTRY.clone()),
                            )
                            .executes(AdvancementExecutor {
                                action: $action,
                                mode: Mode::Through,
                            }),
                        ),
                    )
                    .then(
                        literal("everything")
                            .executes(EveryAdvancementExecutor { action: $action }),
                    ),
            )
        };
    }

    dispatcher.register(
        command(NAME, DESCRIPTION)
            .requires(PERMISSION)
            .then(build_action!("grant", Action::Grant))
            .then(build_action!("revoke", Action::Revoke)),
    );
}
