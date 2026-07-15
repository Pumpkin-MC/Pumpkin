use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use pumpkin_data::translation;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

const DESCRIPTION: &str = "Sends a message to your team.";
const PERMISSION: &str = "minecraft:command.teammsg";
const ARG_MESSAGE: &str = "message";

const NO_TEAM_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_TEAMMSG_FAILED_NOTEAM,
    translation::java::COMMANDS_TEAMMSG_FAILED_NOTEAM,
);

struct TeamMsgCommandExecutor;

impl CommandExecutor for TeamMsgCommandExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let player = context.source.player_or_err()?;
            let sender_name = &player.gameprofile.name;
            let world = context.world();
            let scoreboard = world.scoreboard.lock().await;

            let mut sender_team = None;
            for team in scoreboard.get_teams().values() {
                if team.players.contains(sender_name) {
                    sender_team = Some(team.clone());
                    break;
                }
            }

            let Some(team) = sender_team else {
                return Err(NO_TEAM_ERROR.create_without_context());
            };

            let team_display_name = team.display_name.clone().color_named(team.color);
            let sender_display_name = context.source.display_name.clone();
            let message_text = StringArgumentType::get(context, ARG_MESSAGE)?;
            let message_component = TextComponent::text(message_text.to_string());

            let online_players = world.players.load();
            let mut sent_count = 0;

            for player in online_players.iter() {
                if team.players.contains(&player.gameprofile.name) {
                    let msg = if player.gameprofile.name == *sender_name {
                        TextComponent::translate_cross(
                            translation::java::CHAT_TYPE_TEAM_SENT,
                            translation::java::CHAT_TYPE_TEAM_SENT,
                            [
                                team_display_name.clone(),
                                sender_display_name.clone(),
                                message_component.clone(),
                            ],
                        )
                    } else {
                        TextComponent::translate_cross(
                            translation::java::CHAT_TYPE_TEAM_TEXT,
                            translation::java::CHAT_TYPE_TEAM_TEXT,
                            [
                                team_display_name.clone(),
                                sender_display_name.clone(),
                                message_component.clone(),
                            ],
                        )
                    };

                    player.send_system_message(&msg).await;
                    sent_count += 1;
                }
            }

            Ok(sent_count)
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Allow,
    ));

    // Register /teammsg <message>
    dispatcher.register(command("teammsg", DESCRIPTION).requires(PERMISSION).then(
        argument(ARG_MESSAGE, StringArgumentType::GreedyPhrase).executes(TeamMsgCommandExecutor),
    ));

    // Register alias /tm <message>
    dispatcher.register(command("tm", DESCRIPTION).requires(PERMISSION).then(
        argument(ARG_MESSAGE, StringArgumentType::GreedyPhrase).executes(TeamMsgCommandExecutor),
    ));
}
