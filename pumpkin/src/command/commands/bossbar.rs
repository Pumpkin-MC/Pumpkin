use crate::command::args::bool::BoolArgConsumer;
use crate::command::args::bossbar_color::BossbarColorArgumentConsumer;
use crate::command::args::bossbar_style::BossbarStyleArgumentConsumer;
use crate::command::args::bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::resource_location::ResourceLocationArgumentConsumer;

use crate::command::args::{ConsumedArgs, FindArg, FindArgDefaultName};

use crate::command::args::textcomponent::TextComponentArgConsumer;
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, argument_default_name, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use crate::world::bossbar::Bossbar;
use crate::world::custom_bossbar::BossbarUpdateError;
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::hover::HoverEvent;
use std::fmt::Write as _;
use uuid::Uuid;

const NAMES: [&str; 1] = ["bossbar"];
const DESCRIPTION: &str = "Display bossbar";

const ARG_NAME: &str = "name";

const ARG_VISIBLE: &str = "visible";

const fn autocomplete_consumer() -> ResourceLocationArgumentConsumer {
    // TODO: Add autocompletion when implemented properly
    ResourceLocationArgumentConsumer
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
            let mut namespace = autocomplete_consumer()
                .find_arg_default_name(args)?
                .to_string();
            if !namespace.contains(':') {
                namespace = format!("minecraft:{namespace}");
            }

            let text_component = TextComponentArgConsumer::find_arg(args, ARG_NAME)?;

            if server.bossbars.lock().await.has_bossbar(&namespace) {
                return Result::Err(CommandError::CommandFailed(TextComponent::translate(
                    "commands.bossbar.create.failed",
                    [TextComponent::text(namespace.clone())],
                )));
            }

            let bossbar = Bossbar::new(text_component);
            let mut bossbars = server.bossbars.lock().await;

            bossbars.create_bossbar(namespace.clone(), bossbar.clone());
            let new_size = bossbars.get_bossbars_len();
            drop(bossbars);

            sender
                .send_message(TextComponent::translate(
                    "commands.bossbar.create.success",
                    [bossbar_prefix(bossbar.title.clone(), namespace.clone())],
                ))
                .await;

            Ok(new_size as i32)
        })
    }
}

struct GetExecutor(CommandValueGet);

impl CommandExecutor for GetExecutor {
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
                        .send_message(TextComponent::translate(
                            "commands.bossbar.get.max",
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
                CommandValueGet::Players => Ok(bossbar.players.len() as i32),
                CommandValueGet::Value => {
                    sender
                        .send_message(TextComponent::translate(
                            "commands.bossbar.get.value",
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
                    let state = if bossbar.visible {
                        "commands.bossbar.get.visible.visible"
                    } else {
                        "commands.bossbar.get.visible.hidden"
                    };
                    sender
                        .send_message(TextComponent::translate(
                            state,
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
                    .send_message(TextComponent::translate(
                        "commands.bossbar.list.bars.none",
                        [],
                    ))
                    .await;
                return Ok(0);
            }

            let mut bossbars_text = TextComponent::text("");
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
                .send_message(TextComponent::translate(
                    "commands.bossbar.list.bars.some",
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
                .send_message(TextComponent::translate(
                    "commands.bossbar.remove.success",
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
                    .remove_bossbar(server, namespace.clone())
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
            let namespace = autocomplete_consumer().find_arg_default_name(args)?;

            let Some(bossbar) = server.bossbars.lock().await.get_bossbar(namespace) else {
                return Err(handle_bossbar_error(
                    BossbarUpdateError::InvalidResourceLocation(namespace.to_string()),
                ));
            };

            match self.0 {
                CommandValueSet::Color => {
                    let color = BossbarColorArgumentConsumer.find_arg_default_name(args)?;

                    match server
                        .bossbars
                        .lock()
                        .await
                        .update_color(server, namespace.to_string(), color.clone())
                        .await
                    {
                        Ok(()) => {}
                        Err(err) => {
                            return Err(handle_bossbar_error(err));
                        }
                    }

                    sender
                        .send_message(TextComponent::translate(
                            "commands.bossbar.set.color.success",
                            [bossbar_prefix(
                                bossbar.bossbar_data.title.clone(),
                                namespace.to_string(),
                            )],
                        ))
                        .await;

                    Ok(0)
                }
                CommandValueSet::Max => {
                    let Ok(max_value) = max_value_consumer().find_arg_default_name(args)? else {
                        return Err(CommandError::CommandFailed(TextComponent::translate(
                            "parsing.int.invalid",
                            [TextComponent::text(i32::MAX.to_string())],
                        )));
                    };

                    match server
                        .bossbars
                        .lock()
                        .await
                        .update_health(server, namespace.to_string(), max_value, bossbar.value)
                        .await
                    {
                        Ok(()) => {}
                        Err(err) => {
                            return Err(handle_bossbar_error(err));
                        }
                    }

                    sender
                        .send_message(TextComponent::translate(
                            "commands.bossbar.set.max.success",
                            [
                                bossbar_prefix(
                                    bossbar.bossbar_data.title.clone(),
                                    namespace.to_string(),
                                ),
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
                        .update_name(server, namespace, text_component.clone())
                        .await
                    {
                        Ok(()) => {}
                        Err(err) => {
                            return Err(handle_bossbar_error(err));
                        }
                    }

                    sender
                        .send_message(TextComponent::translate(
                            "commands.bossbar.set.name.success",
                            [bossbar_prefix(text_component, namespace.to_string())],
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
                            .update_players(server, namespace.to_string(), vec![])
                            .await
                        {
                            Ok(()) => {}
                            Err(err) => {
                                return Err(handle_bossbar_error(err));
                            }
                        }
                        sender
                            .send_message(TextComponent::translate(
                                "commands.bossbar.set.players.success.none",
                                [bossbar_prefix(
                                    bossbar.bossbar_data.title.clone(),
                                    namespace.to_string(),
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
                        .update_players(server, namespace.to_string(), players)
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
                        .send_message(TextComponent::translate(
                            "commands.bossbar.set.players.success.some",
                            [
                                bossbar_prefix(
                                    bossbar.bossbar_data.title.clone(),
                                    namespace.to_string(),
                                ),
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
                        .update_division(server, namespace.to_string(), style.clone())
                        .await
                    {
                        Ok(()) => {}
                        Err(err) => {
                            return Err(handle_bossbar_error(err));
                        }
                    }
                    sender
                        .send_message(TextComponent::translate(
                            "commands.bossbar.set.style.success",
                            [bossbar_prefix(
                                bossbar.bossbar_data.title.clone(),
                                namespace.to_string(),
                            )],
                        ))
                        .await;
                    Ok(0)
                }
                CommandValueSet::Value => {
                    let Ok(value) = value_consumer().find_arg_default_name(args)? else {
                        return Err(CommandError::CommandFailed(TextComponent::translate(
                            "parsing.int.invalid",
                            [TextComponent::text(i32::MAX.to_string())],
                        )));
                    };

                    match server
                        .bossbars
                        .lock()
                        .await
                        .update_health(server, namespace.to_string(), bossbar.max, value)
                        .await
                    {
                        Ok(()) => {}
                        Err(err) => {
                            return Err(handle_bossbar_error(err));
                        }
                    }

                    sender
                        .send_message(TextComponent::translate(
                            "commands.bossbar.set.value.success",
                            [
                                bossbar_prefix(
                                    bossbar.bossbar_data.title.clone(),
                                    namespace.to_string(),
                                ),
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
                        .update_visibility(server, namespace.to_string(), visibility)
                        .await
                    {
                        Ok(()) => {}
                        Err(err) => {
                            return Err(handle_bossbar_error(err));
                        }
                    }

                    let state = if visibility {
                        "commands.bossbar.set.visible.success.visible"
                    } else {
                        "commands.bossbar.set.visible.success.hidden"
                    };

                    sender
                        .send_message(TextComponent::translate(
                            state,
                            [bossbar_prefix(
                                bossbar.bossbar_data.title.clone(),
                                namespace.to_string(),
                            )],
                        ))
                        .await;

                    Ok(visibility as i32)
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
                    .then(literal("max").execute(GetExecutor(CommandValueGet::Max)))
                    .then(literal("players").execute(GetExecutor(CommandValueGet::Players)))
                    .then(literal("value").execute(GetExecutor(CommandValueGet::Value)))
                    .then(literal("visible").execute(GetExecutor(CommandValueGet::Visible))),
            ),
        )
        .then(literal("list").execute(ListExecutor))
        .then(
            literal("remove")
                .then(argument_default_name(autocomplete_consumer()).execute(RemoveExecutor)),
        )
        .then(
            literal("set").then(
                argument_default_name(autocomplete_consumer())
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
            CommandError::CommandFailed(TextComponent::translate(
                translation::COMMANDS_BOSSBAR_UNKNOWN,
                [TextComponent::text(location)],
            ))
        }
        BossbarUpdateError::NoChanges(value, variation) => {
            let mut key = "commands.bossbar.set.".to_string();
            key.push_str(value);
            key.push_str(".unchanged");
            if let Some(variation) = variation {
                write!(key, ".{variation}").unwrap();
            }

            CommandError::CommandFailed(TextComponent::translate(key, []))
        }
    }
}
