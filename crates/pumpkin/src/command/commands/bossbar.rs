use crate::command::args::bool::BoolArgConsumer;
use crate::command::args::bossbar_color::BossbarColorArgumentConsumer;
use crate::command::args::bossbar_style::BossbarStyleArgumentConsumer;
use crate::command::args::bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::resource_location::ResourceLocationArgumentConsumer;

use crate::command::args::{ConsumedArgs, FindArg, FindArgDefaultName};

use crate::command::args::textcomponent::TextComponentArgConsumer;
use crate::command::dispatcher::CommandError;
use crate::command::suggestion::suggestions::SuggestionsBuilder;
use crate::command::tree::builder::{argument, argument_default_name, literal};
use crate::command::tree::{CommandSuggestionProvider, CommandSuggestionResult, CommandTree};
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use crate::server::Server;
use crate::world::bossbar::Bossbar;
use crate::world::custom_bossbar::BossbarUpdateError;
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::hover::HoverEvent;
use uuid::Uuid;

const NAMES: [&str; 1] = ["bossbar"];
const DESCRIPTION: &str = "Creates and modifies boss bars";

const ARG_NAME: &str = "name";

const ARG_VISIBLE: &str = "visible";

const fn autocomplete_consumer() -> ResourceLocationArgumentConsumer {
    ResourceLocationArgumentConsumer
}

struct BossbarSuggestionProvider;

impl CommandSuggestionProvider for BossbarSuggestionProvider {
    fn suggest<'a>(
        &'a self,
        _src: &'a CommandSender,
        server: &'a Server,
        input: &'a str,
        start: usize,
        _end: usize,
    ) -> CommandSuggestionResult<'a> {
        Box::pin(async move {
            let mut builder = SuggestionsBuilder::new(input, start);
            let bossbars = server.bossbars.lock().await;
            let remaining = builder.remaining_lowercase().to_string();
            for key in bossbars.custom_bossbars.keys() {
                if key.to_lowercase().starts_with(&remaining) {
                    builder = builder.suggest(key.clone());
                }
            }
            builder.build()
        })
    }
}

enum CommandValueGet {
    Max,
    Players,
    Value,
    Visible,
}

enum CommandValueSet {
    Color,
    Max,
    Name,
    Players(bool),
    Style,
    Value,
    Visible,
}

struct AddExecutor;

impl CommandExecutor for AddExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let namespace = autocomplete_consumer()
                .find_arg_default_name(args)?
                .to_string();

            let text_component = TextComponentArgConsumer::find_arg(args, ARG_NAME)?;

            if server.bossbars.lock().await.has_bossbar(&namespace) {
                return Result::Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    translation::java::COMMANDS_BOSSBAR_CREATE_FAILED,
                    translation::bedrock::COMMANDS_BOSSBAR_ADD_FAILURE_EXISTS,
                    [TextComponent::text(namespace.clone())],
                )));
            }

            let bossbar = Bossbar::new(text_component);
            let mut bossbars = server.bossbars.lock().await;

            bossbars.create_bossbar(namespace.clone(), bossbar.clone());
            let new_size = bossbars.get_bossbars_len();
            drop(bossbars);

            sender
                .send_message(TextComponent::translate_cross(
                    translation::java::COMMANDS_BOSSBAR_CREATE_SUCCESS,
                    translation::bedrock::COMMANDS_BOSSBAR_ADD_SUCCESS,
                    [bossbar_prefix(bossbar.title.clone(), namespace.clone())],
                ))
                .await;

            Ok(new_size as i32)
        })
    }
}

struct GetExecutor(CommandValueGet);

impl CommandExecutor for GetExecutor {
    #[expect(clippy::too_many_lines)]
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let namespace = autocomplete_consumer()
                .find_arg_default_name(args)?
                .to_string();

            let Some(bossbar) = server.bossbars.lock().await.get_bossbar(&namespace) else {
                return Err(handle_bossbar_error(
                    BossbarUpdateError::InvalidResourceLocation(namespace.clone()),
                ));
            };

            match self.0 {
                CommandValueGet::Max => {
                    sender
                        .send_message(TextComponent::translate_cross(
                            translation::java::COMMANDS_BOSSBAR_GET_MAX,
                            translation::bedrock::COMMANDS_BOSSBAR_GET_MAX,
                            [
                                bossbar_prefix(
                                    bossbar.bossbar_data.title.clone(),
                                    namespace.clone(),
                                ),
                                TextComponent::text(bossbar.max.to_string()),
                            ],
                        ))
                        .await;
                    Ok(bossbar.max)
                }
                CommandValueGet::Players => {
                    let online_players: Vec<String> = server
                        .get_all_players()
                        .iter()
                        .filter(|player| bossbar.players.contains(&player.gameprofile.id))
                        .map(|player| player.gameprofile.name.clone())
                        .collect();
                    let count = online_players.len() as i32;

                    if count == 0 {
                        sender
                            .send_message(TextComponent::translate_cross(
                                translation::java::COMMANDS_BOSSBAR_GET_PLAYERS_NONE,
                                translation::bedrock::COMMANDS_BOSSBAR_GET_PLAYERS_NONE,
                                [bossbar_prefix(
                                    bossbar.bossbar_data.title.clone(),
                                    namespace.clone(),
                                )],
                            ))
                            .await;
                    } else {
                        sender
                            .send_message(TextComponent::translate_cross(
                                translation::java::COMMANDS_BOSSBAR_GET_PLAYERS_SOME,
                                if count == 1 {
                                    translation::bedrock::COMMANDS_BOSSBAR_GET_PLAYERS_ONE
                                } else {
                                    translation::bedrock::COMMANDS_BOSSBAR_GET_PLAYERS
                                },
                                [
                                    bossbar_prefix(
                                        bossbar.bossbar_data.title.clone(),
                                        namespace.clone(),
                                    ),
                                    TextComponent::text(count.to_string()),
                                    TextComponent::text(online_players.join(", ")),
                                ],
                            ))
                            .await;
                    }
                    Ok(count)
                }
                CommandValueGet::Value => {
                    sender
                        .send_message(TextComponent::translate_cross(
                            translation::java::COMMANDS_BOSSBAR_GET_VALUE,
                            translation::bedrock::COMMANDS_BOSSBAR_GET_VALUE,
                            [
                                bossbar_prefix(
                                    bossbar.bossbar_data.title.clone(),
                                    namespace.clone(),
                                ),
                                TextComponent::text(bossbar.value.to_string()),
                            ],
                        ))
                        .await;
                    Ok(bossbar.value)
                }
                CommandValueGet::Visible => {
                    let (java_key, bedrock_key) = if bossbar.visible {
                        (
                            translation::java::COMMANDS_BOSSBAR_GET_VISIBLE_VISIBLE,
                            translation::bedrock::COMMANDS_BOSSBAR_GET_VISIBLE_TRUE,
                        )
                    } else {
                        (
                            translation::java::COMMANDS_BOSSBAR_GET_VISIBLE_HIDDEN,
                            translation::bedrock::COMMANDS_BOSSBAR_GET_VISIBLE_FALSE,
                        )
                    };
                    sender
                        .send_message(TextComponent::translate_cross(
                            java_key,
                            bedrock_key,
                            [bossbar_prefix(
                                bossbar.bossbar_data.title.clone(),
                                namespace.clone(),
                            )],
                        ))
                        .await;
                    Ok(bossbar.visible as i32)
                }
            }
        })
    }
}

struct ListExecutor;

impl CommandExecutor for ListExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let bossbars = server.bossbars.lock().await.get_all_bossbars();

            if bossbars.is_empty() {
                sender
                    .send_message(TextComponent::translate_cross(
                        translation::java::COMMANDS_BOSSBAR_LIST_BARS_NONE,
                        translation::bedrock::COMMANDS_BOSSBAR_LIST_NONE,
                        [],
                    ))
                    .await;
                return Ok(0);
            }

            let mut bossbars_text = TextComponent::empty();
            for (i, bossbar) in bossbars.iter().enumerate() {
                if i == 0 {
                    bossbars_text = bossbars_text.add_child(bossbar_prefix(
                        bossbar.bossbar_data.title.clone(),
                        bossbar.namespace.clone(),
                    ));
                } else {
                    bossbars_text = bossbars_text.add_child(TextComponent::text(", ").add_child(
                        bossbar_prefix(
                            bossbar.bossbar_data.title.clone(),
                            bossbar.namespace.clone(),
                        ),
                    ));
                }
            }

            sender
                .send_message(TextComponent::translate_cross(
                    translation::java::COMMANDS_BOSSBAR_LIST_BARS_SOME,
                    translation::bedrock::COMMANDS_BOSSBAR_LIST,
                    [
                        TextComponent::text(bossbars.len().to_string()),
                        bossbars_text,
                    ],
                ))
                .await;

            Ok(bossbars.len() as i32)
        })
    }
}

struct RemoveExecutor;

impl CommandExecutor for RemoveExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let namespace = autocomplete_consumer()
                .find_arg_default_name(args)?
                .to_string();

            let Some(bossbar) = server.bossbars.lock().await.get_bossbar(&namespace) else {
                return Err(handle_bossbar_error(
                    BossbarUpdateError::InvalidResourceLocation(namespace),
                ));
            };

            sender
                .send_message(TextComponent::translate_cross(
                    translation::java::COMMANDS_BOSSBAR_REMOVE_SUCCESS,
                    translation::bedrock::COMMANDS_BOSSBAR_REMOVE,
                    [bossbar_prefix(
                        bossbar.bossbar_data.title.clone(),
                        namespace.clone(),
                    )],
                ))
                .await;

            let error = {
                match server
                    .bossbars
                    .lock()
                    .await
                    .remove_bossbar(server, namespace)
                    .await
                {
                    Ok(()) => return Ok(server.bossbars.lock().await.get_bossbars_len() as i32),
                    Err(error) => error,
                }
            };

            Err(handle_bossbar_error(error))
        })
    }
}

struct SetExecutor(CommandValueSet);

impl CommandExecutor for SetExecutor {
    #[expect(clippy::too_many_lines)]
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let namespace = autocomplete_consumer()
                .find_arg_default_name(args)?
                .to_string();

            let Some(bossbar) = server.bossbars.lock().await.get_bossbar(&namespace) else {
                return Err(handle_bossbar_error(
                    BossbarUpdateError::InvalidResourceLocation(namespace),
                ));
            };

            match self.0 {
                CommandValueSet::Color => {
                    let color = BossbarColorArgumentConsumer.find_arg_default_name(args)?;

                    match server
                        .bossbars
                        .lock()
                        .await
                        .update_color(server, namespace.clone(), *color)
                        .await
                    {
                        Ok(()) => {}
                        Err(err) => {
                            return Err(handle_bossbar_error(err));
                        }
                    }

                    sender
                        .send_message(TextComponent::translate_cross(
                            translation::java::COMMANDS_BOSSBAR_SET_COLOR_SUCCESS,
                            translation::java::COMMANDS_BOSSBAR_SET_COLOR_SUCCESS,
                            [bossbar_prefix(
                                bossbar.bossbar_data.title.clone(),
                                namespace,
                            )],
                        ))
                        .await;

                    Ok(0)
                }
                CommandValueSet::Max => {
                    let Ok(max_value) = max_value_consumer().find_arg_default_name(args)? else {
                        return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                            "parsing.int.invalid",
                            "parsing.int.invalid",
                            [TextComponent::text(i32::MAX.to_string())],
                        )));
                    };

                    match server
                        .bossbars
                        .lock()
                        .await
                        .update_max(server, namespace.clone(), max_value)
                        .await
                    {
                        Ok(()) => {}
                        Err(err) => {
                            return Err(handle_bossbar_error(err));
                        }
                    }

                    sender
                        .send_message(TextComponent::translate_cross(
                            translation::java::COMMANDS_BOSSBAR_SET_MAX_SUCCESS,
                            translation::java::COMMANDS_BOSSBAR_SET_MAX_SUCCESS,
                            [
                                bossbar_prefix(bossbar.bossbar_data.title.clone(), namespace),
                                TextComponent::text(max_value.to_string()),
                            ],
                        ))
                        .await;

                    Ok(max_value)
                }
                CommandValueSet::Name => {
                    let text_component = TextComponentArgConsumer::find_arg(args, ARG_NAME)?;
                    match server
                        .bossbars
                        .lock()
                        .await
                        .update_name(server, &namespace, text_component.clone())
                        .await
                    {
                        Ok(()) => {}
                        Err(err) => {
                            return Err(handle_bossbar_error(err));
                        }
                    }

                    sender
                        .send_message(TextComponent::translate_cross(
                            translation::java::COMMANDS_BOSSBAR_SET_NAME_SUCCESS,
                            translation::java::COMMANDS_BOSSBAR_SET_NAME_SUCCESS,
                            [bossbar_prefix(text_component, namespace)],
                        ))
                        .await;

                    Ok(0)
                }
                CommandValueSet::Players(has_players) => {
                    if !has_players {
                        match server
                            .bossbars
                            .lock()
                            .await
                            .update_players(server, namespace.clone(), vec![])
                            .await
                        {
                            Ok(()) => {}
                            Err(err) => {
                                return Err(handle_bossbar_error(err));
                            }
                        }
                        sender
                            .send_message(TextComponent::translate_cross(
                                translation::java::COMMANDS_BOSSBAR_SET_PLAYERS_SUCCESS_NONE,
                                translation::java::COMMANDS_BOSSBAR_SET_PLAYERS_SUCCESS_NONE,
                                [bossbar_prefix(
                                    bossbar.bossbar_data.title.clone(),
                                    namespace,
                                )],
                            ))
                            .await;

                        return Ok(0);
                    }

                    let targets = PlayersArgumentConsumer.find_arg_default_name(args)?;
                    let players: Vec<Uuid> =
                        targets.iter().map(|player| player.gameprofile.id).collect();
                    let count = players.len();

                    match server
                        .bossbars
                        .lock()
                        .await
                        .update_players(server, namespace.clone(), players)
                        .await
                    {
                        Ok(()) => {}
                        Err(err) => {
                            return Err(handle_bossbar_error(err));
                        }
                    }

                    let player_names = targets
                        .iter()
                        .map(|p| p.gameprofile.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");

                    sender
                        .send_message(TextComponent::translate_cross(
                            translation::java::COMMANDS_BOSSBAR_SET_PLAYERS_SUCCESS_SOME,
                            translation::java::COMMANDS_BOSSBAR_SET_PLAYERS_SUCCESS_SOME,
                            [
                                bossbar_prefix(bossbar.bossbar_data.title.clone(), namespace),
                                TextComponent::text(count.to_string()),
                                TextComponent::text(player_names),
                            ],
                        ))
                        .await;

                    Ok(count as i32)
                }
                CommandValueSet::Style => {
                    let style = BossbarStyleArgumentConsumer.find_arg_default_name(args)?;
                    match server
                        .bossbars
                        .lock()
                        .await
                        .update_division(server, namespace.clone(), *style)
                        .await
                    {
                        Ok(()) => {}
                        Err(err) => {
                            return Err(handle_bossbar_error(err));
                        }
                    }
                    sender
                        .send_message(TextComponent::translate_cross(
                            translation::java::COMMANDS_BOSSBAR_SET_STYLE_SUCCESS,
                            translation::java::COMMANDS_BOSSBAR_SET_STYLE_SUCCESS,
                            [bossbar_prefix(
                                bossbar.bossbar_data.title.clone(),
                                namespace,
                            )],
                        ))
                        .await;
                    Ok(0)
                }
                CommandValueSet::Value => {
                    let Ok(value) = value_consumer().find_arg_default_name(args)? else {
                        return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                            "parsing.int.invalid",
                            "parsing.int.invalid",
                            [TextComponent::text(i32::MAX.to_string())],
                        )));
                    };

                    match server
                        .bossbars
                        .lock()
                        .await
                        .update_value(server, namespace.clone(), value)
                        .await
                    {
                        Ok(()) => {}
                        Err(err) => {
                            return Err(handle_bossbar_error(err));
                        }
                    }

                    sender
                        .send_message(TextComponent::translate_cross(
                            translation::java::COMMANDS_BOSSBAR_SET_VALUE_SUCCESS,
                            translation::java::COMMANDS_BOSSBAR_SET_VALUE_SUCCESS,
                            [
                                bossbar_prefix(bossbar.bossbar_data.title.clone(), namespace),
                                TextComponent::text(value.to_string()),
                            ],
                        ))
                        .await;

                    Ok(value)
                }
                CommandValueSet::Visible => {
                    let visibility = BoolArgConsumer::find_arg(args, ARG_VISIBLE)?;

                    match server
                        .bossbars
                        .lock()
                        .await
                        .update_visibility(server, namespace.clone(), visibility)
                        .await
                    {
                        Ok(()) => {}
                        Err(err) => {
                            return Err(handle_bossbar_error(err));
                        }
                    }

                    let state = if visibility {
                        translation::java::COMMANDS_BOSSBAR_SET_VISIBLE_SUCCESS_VISIBLE
                    } else {
                        translation::java::COMMANDS_BOSSBAR_SET_VISIBLE_SUCCESS_HIDDEN
                    };

                    sender
                        .send_message(TextComponent::translate_cross(
                            state,
                            state,
                            [bossbar_prefix(
                                bossbar.bossbar_data.title.clone(),
                                namespace,
                            )],
                        ))
                        .await;

                    Ok(0)
                }
            }
        })
    }
}

const fn max_value_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new().min(1).name("max")
}

const fn value_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new().min(0).name("value")
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("add").then(
                argument_default_name(autocomplete_consumer())
                    .then(argument(ARG_NAME, TextComponentArgConsumer).execute(AddExecutor)),
            ),
        )
        .then(
            literal("get").then(
                argument_default_name(autocomplete_consumer())
                    .suggests(BossbarSuggestionProvider)
                    .then(literal("max").execute(GetExecutor(CommandValueGet::Max)))
                    .then(literal("players").execute(GetExecutor(CommandValueGet::Players)))
                    .then(literal("value").execute(GetExecutor(CommandValueGet::Value)))
                    .then(literal("visible").execute(GetExecutor(CommandValueGet::Visible))),
            ),
        )
        .then(literal("list").execute(ListExecutor))
        .then(
            literal("remove").then(
                argument_default_name(autocomplete_consumer())
                    .suggests(BossbarSuggestionProvider)
                    .execute(RemoveExecutor),
            ),
        )
        .then(
            literal("set").then(
                argument_default_name(autocomplete_consumer())
                    .suggests(BossbarSuggestionProvider)
                    .then(
                        literal("color").then(
                            argument_default_name(BossbarColorArgumentConsumer)
                                .execute(SetExecutor(CommandValueSet::Color)),
                        ),
                    )
                    .then(
                        literal("max").then(
                            argument_default_name(max_value_consumer())
                                .execute(SetExecutor(CommandValueSet::Max)),
                        ),
                    )
                    .then(
                        literal("name").then(
                            argument(ARG_NAME, TextComponentArgConsumer)
                                .execute(SetExecutor(CommandValueSet::Name)),
                        ),
                    )
                    .then(
                        literal("players")
                            .then(
                                argument_default_name(PlayersArgumentConsumer)
                                    .execute(SetExecutor(CommandValueSet::Players(true))),
                            )
                            .execute(SetExecutor(CommandValueSet::Players(false))),
                    )
                    .then(
                        literal("style").then(
                            argument_default_name(BossbarStyleArgumentConsumer)
                                .execute(SetExecutor(CommandValueSet::Style)),
                        ),
                    )
                    .then(
                        literal("value").then(
                            argument_default_name(value_consumer())
                                .execute(SetExecutor(CommandValueSet::Value)),
                        ),
                    )
                    .then(
                        literal("visible").then(
                            argument(ARG_VISIBLE, BoolArgConsumer)
                                .execute(SetExecutor(CommandValueSet::Visible)),
                        ),
                    ),
            ),
        )
}

fn bossbar_prefix(title: TextComponent, namespace: String) -> TextComponent {
    TextComponent::text("[")
        .add_child(title)
        .add_child(TextComponent::text("]"))
        .hover_event(HoverEvent::show_text(TextComponent::text(namespace)))
}

fn handle_bossbar_error(error: BossbarUpdateError) -> CommandError {
    match error {
        BossbarUpdateError::InvalidResourceLocation(location) => {
            CommandError::CommandFailed(TextComponent::translate_cross(
                translation::java::COMMANDS_BOSSBAR_UNKNOWN,
                translation::bedrock::COMMANDS_BOSSBAR_NOTFOUND,
                [TextComponent::text(location)],
            ))
        }
        BossbarUpdateError::NoChanges(value, variation) => {
            let key = variation.map_or_else(
                || format!("commands.bossbar.set.{value}.unchanged"),
                |var| format!("commands.bossbar.set.{value}.unchanged.{var}"),
            );

            CommandError::CommandFailed(TextComponent::translate_cross(key.clone(), key, []))
        }
    }
}
