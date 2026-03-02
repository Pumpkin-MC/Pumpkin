use crate::command::args::bool::BoolArgConsumer;
use crate::command::args::bossbar_color::BossbarColorArgumentConsumer;
use crate::command::args::bossbar_style::BossbarStyleArgumentConsumer;
use crate::command::args::bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::resource_location::IdentifierArgumentConsumer;

use crate::command::args::{CommandErrorMappable, ConsumedArgs, FindArg, FindArgDefaultName};

use crate::command::args::textcomponent::TextComponentArgConsumer;
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, argument_default_name, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use crate::entity::EntityBase;
use crate::server::Server;
use crate::world::bossbar::Bossbar;
use crate::world::custom_bossbar::{BossbarUpdateError, CustomBossbar};
use pumpkin_data::translation::{self, COMMANDS_BOSSBAR_CREATE_FAILED};
use pumpkin_util::identifier::Identifier;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::hover::HoverEvent;
use std::fmt::Write as _;
use uuid::Uuid;

const NAMES: [&str; 1] = ["bossbar"];
const DESCRIPTION: &str = "Display bossbar";

const ARG_NAME: &str = "name";

const ARG_VISIBLE: &str = "visible";

const fn autocomplete_consumer() -> IdentifierArgumentConsumer {
    // TODO: Add autocompletion when implemented properly
    IdentifierArgumentConsumer
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
            let identifier = autocomplete_consumer()
                .find_arg_default_name(args)?
                .map_to_command_error()?;

            let text_component = TextComponentArgConsumer::find_arg(args, ARG_NAME)?;

            if server.bossbars.lock().await.has_bossbar(&identifier) {
                return Result::Err(CommandError::CommandFailed(TextComponent::translate(
                    COMMANDS_BOSSBAR_CREATE_FAILED,
                    [TextComponent::text(identifier.to_string())],
                )));
            }

            let bossbar = Bossbar::new(text_component);
            let mut bossbars = server.bossbars.lock().await;

            bossbars.create_bossbar(identifier.clone(), bossbar.clone());
            let new_size = bossbars.get_bossbars_len();
            drop(bossbars);

            sender
                .send_message(TextComponent::translate(
                    "commands.bossbar.create.success",
                    [bossbar_prefix(bossbar.title.clone(), &identifier)],
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
            let identifier = autocomplete_consumer()
                .find_arg_default_name(args)?
                .map_to_command_error()?;

            let bossbars_lock = server.bossbars.lock().await;

            let bossbar = bossbars_lock
                .get_bossbar_or_err(&identifier)
                .map_err(handle_bossbar_error)?;

            match self.0 {
                CommandValueGet::Max => {
                    sender
                        .send_message(TextComponent::translate(
                            "commands.bossbar.get.max",
                            [
                                bossbar_prefix(bossbar.bossbar_data.title.clone(), &identifier),
                                TextComponent::text(bossbar.max.to_string()),
                            ],
                        ))
                        .await;
                    Ok(bossbar.max)
                }
                CommandValueGet::Players => {
                    let players = get_bossbar_players(bossbar, server).await;
                    let len = players.len();

                    if len == 0 {
                        sender
                            .send_message(TextComponent::translate(
                                "commands.bossbar.get.players.none",
                                [bossbar_prefix(
                                    bossbar.bossbar_data.title.clone(),
                                    &identifier,
                                )],
                            ))
                            .await;
                    } else {
                        sender
                            .send_message(TextComponent::translate(
                                "commands.bossbar.get.players.some",
                                [
                                    bossbar_prefix(bossbar.bossbar_data.title.clone(), &identifier),
                                    TextComponent::text(players.len().to_string()),
                                    join_components(players),
                                ],
                            ))
                            .await;
                    }
                    Ok(len as i32)
                }
                CommandValueGet::Value => {
                    sender
                        .send_message(TextComponent::translate(
                            "commands.bossbar.get.value",
                            [
                                bossbar_prefix(bossbar.bossbar_data.title.clone(), &identifier),
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
                                &identifier,
                            )],
                        ))
                        .await;
                    Ok(bossbar.visible as i32)
                }
            }
        })
    }
}

async fn get_bossbar_players(bossbar: &CustomBossbar, server: &Server) -> Vec<TextComponent> {
    let futures = bossbar
        .players
        .iter()
        .filter_map(|uuid| server.get_player_by_uuid(*uuid))
        .map(|player| Box::pin(async move { player.get_display_name().await }))
        .collect::<Vec<_>>();

    futures::future::join_all(futures).await
}

fn join_components(components: Vec<TextComponent>) -> TextComponent {
    let mut result = TextComponent::text("");
    let mut iter = components.into_iter().peekable();

    while let Some(next) = iter.next() {
        result = result.add_child(next);
        if iter.peek().is_some() {
            result = result.add_child(TextComponent::text(", "));
        }
    }

    result
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
            let bossbars_lock = server.bossbars.lock().await;
            let bossbars = bossbars_lock.get_all_bossbars();

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
            for (i, (identifier, bossbar)) in bossbars.iter().enumerate() {
                if i == 0 {
                    bossbars_text = bossbars_text.add_child(bossbar_prefix(
                        bossbar.bossbar_data.title.clone(),
                        identifier,
                    ));
                } else {
                    bossbars_text = bossbars_text.add_child(TextComponent::text(", ").add_child(
                        bossbar_prefix(bossbar.bossbar_data.title.clone(), identifier),
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
            let identifier = autocomplete_consumer()
                .find_arg_default_name(args)?
                .map_to_command_error()?;

            let mut bossbars_lock = server.bossbars.lock().await;
            match bossbars_lock.remove_bossbar(server, &identifier).await {
                Ok(bossbar_removed) => {
                    sender
                        .send_message(TextComponent::translate(
                            "commands.bossbar.remove.success",
                            [bossbar_prefix(
                                bossbar_removed.bossbar_data.title.clone(),
                                &identifier,
                            )],
                        ))
                        .await;
                    Ok(bossbars_lock.get_bossbars_len() as i32)
                }
                Err(error) => Err(handle_bossbar_error(error)),
            }
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
            let identifier = autocomplete_consumer()
                .find_arg_default_name(args)?
                .map_to_command_error()?;

            let mut bossbars_lock = server.bossbars.lock().await;

            let bossbar = bossbars_lock
                .get_bossbar_mut_or_err(&identifier)
                .map_err(handle_bossbar_error)?;

            match self.0 {
                CommandValueSet::Color => {
                    let color = BossbarColorArgumentConsumer.find_arg_default_name(args)?;

                    match bossbar.update_color(server, *color).await {
                        Ok(()) => {
                            sender
                                .send_message(TextComponent::translate(
                                    "commands.bossbar.set.color.success",
                                    [bossbar_prefix(
                                        bossbar.bossbar_data.title.clone(),
                                        &identifier,
                                    )],
                                ))
                                .await;

                            Ok(0)
                        }
                        Err(error) => Err(handle_bossbar_error(error)),
                    }
                }
                CommandValueSet::Max => {
                    let Ok(max_value) = max_value_consumer().find_arg_default_name(args)? else {
                        return Err(CommandError::CommandFailed(TextComponent::translate(
                            "parsing.int.invalid",
                            [TextComponent::text(i32::MAX.to_string())],
                        )));
                    };

                    match bossbar
                        .update_health(server, max_value, bossbar.value)
                        .await
                    {
                        Ok(()) => {
                            sender
                                .send_message(TextComponent::translate(
                                    "commands.bossbar.set.max.success",
                                    [
                                        bossbar_prefix(
                                            bossbar.bossbar_data.title.clone(),
                                            &identifier,
                                        ),
                                        TextComponent::text(max_value.to_string()),
                                    ],
                                ))
                                .await;

                            Ok(max_value)
                        }
                        Err(error) => Err(handle_bossbar_error(error)),
                    }
                }
                CommandValueSet::Name => {
                    let text_component = TextComponentArgConsumer::find_arg(args, ARG_NAME)?;

                    match bossbar.update_name(server, text_component.clone()).await {
                        Ok(()) => {
                            sender
                                .send_message(TextComponent::translate(
                                    "commands.bossbar.set.name.success",
                                    [bossbar_prefix(text_component, &identifier)],
                                ))
                                .await;

                            Ok(0)
                        }
                        Err(error) => Err(handle_bossbar_error(error)),
                    }
                }
                CommandValueSet::Players(has_players) => {
                    if has_players {
                        let targets = PlayersArgumentConsumer.find_arg_default_name(args)?;
                        let players: Vec<Uuid> =
                            targets.iter().map(|player| player.gameprofile.id).collect();

                        match bossbar.update_players(server, players).await {
                            Ok(()) => {
                                let players = get_bossbar_players(bossbar, server).await;
                                let len = players.len();

                                if len == 0 {
                                    sender
                                        .send_message(TextComponent::translate(
                                            "commands.bossbar.set.players.success.none",
                                            [bossbar_prefix(
                                                bossbar.bossbar_data.title.clone(),
                                                &identifier,
                                            )],
                                        ))
                                        .await;
                                } else {
                                    sender
                                        .send_message(TextComponent::translate(
                                            "commands.bossbar.set.players.success.some",
                                            [
                                                bossbar_prefix(
                                                    bossbar.bossbar_data.title.clone(),
                                                    &identifier,
                                                ),
                                                TextComponent::text(players.len().to_string()),
                                                join_components(players),
                                            ],
                                        ))
                                        .await;
                                }
                                Ok(len as i32)
                            }
                            Err(err) => Err(handle_bossbar_error(err)),
                        }
                    } else {
                        match bossbar.update_players(server, vec![]).await {
                            Ok(()) => {
                                sender
                                    .send_message(TextComponent::translate(
                                        "commands.bossbar.set.players.success.none",
                                        [bossbar_prefix(
                                            bossbar.bossbar_data.title.clone(),
                                            &identifier,
                                        )],
                                    ))
                                    .await;

                                Ok(0)
                            }
                            Err(error) => Err(handle_bossbar_error(error)),
                        }
                    }
                }
                CommandValueSet::Style => {
                    let style = BossbarStyleArgumentConsumer.find_arg_default_name(args)?;
                    match bossbar.update_division(server, *style).await {
                        Ok(()) => {
                            sender
                                .send_message(TextComponent::translate(
                                    "commands.bossbar.set.style.success",
                                    [bossbar_prefix(
                                        bossbar.bossbar_data.title.clone(),
                                        &identifier,
                                    )],
                                ))
                                .await;
                            Ok(0)
                        }
                        Err(err) => Err(handle_bossbar_error(err)),
                    }
                }
                CommandValueSet::Value => {
                    let Ok(value) = value_consumer().find_arg_default_name(args)? else {
                        return Err(CommandError::CommandFailed(TextComponent::translate(
                            "parsing.int.invalid",
                            [TextComponent::text(i32::MAX.to_string())],
                        )));
                    };

                    match bossbar.update_health(server, bossbar.max, value).await {
                        Ok(()) => {
                            sender
                                .send_message(TextComponent::translate(
                                    "commands.bossbar.set.value.success",
                                    [
                                        bossbar_prefix(
                                            bossbar.bossbar_data.title.clone(),
                                            &identifier,
                                        ),
                                        TextComponent::text(value.to_string()),
                                    ],
                                ))
                                .await;

                            Ok(value)
                        }
                        Err(err) => Err(handle_bossbar_error(err)),
                    }
                }
                CommandValueSet::Visible => {
                    let visibility = BoolArgConsumer::find_arg(args, ARG_VISIBLE)?;

                    match bossbar.update_visibility(server, visibility).await {
                        Ok(()) => {
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
                                        &identifier,
                                    )],
                                ))
                                .await;

                            Ok(visibility as i32)
                        }
                        Err(err) => Err(handle_bossbar_error(err)),
                    }
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

fn bossbar_prefix(title: TextComponent, identifier: &Identifier) -> TextComponent {
    TextComponent::text("[")
        .add_child(title)
        .add_child(TextComponent::text("]"))
        .hover_event(HoverEvent::show_text(TextComponent::text(
            identifier.to_string(),
        )))
}

fn handle_bossbar_error(error: BossbarUpdateError) -> CommandError {
    match error {
        BossbarUpdateError::UnknownBossbar(location) => {
            CommandError::CommandFailed(TextComponent::translate(
                translation::COMMANDS_BOSSBAR_UNKNOWN,
                [TextComponent::text(location.to_string())],
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
