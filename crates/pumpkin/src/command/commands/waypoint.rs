use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::CommandResult;
use crate::command::args::{
    FindArg, entity::EntityArgumentConsumer, hex_color::HexColorArgumentConsumer,
    resource_location::ResourceLocationArgumentConsumer, team_color::TeamColorArgumentConsumer,
};
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandSender, ConsumedArgs, tree::CommandTree};

const NAMES: [&str; 1] = ["waypoint"];
const DESCRIPTION: &str = "List or modify waypoints.";
const ARG_WAYPOINT: &str = "waypoint";
const ARG_COLOR: &str = "color";
const ARG_STYLE: &str = "style";

struct ListExecutor;

impl CommandExecutor for ListExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let worlds = server.worlds.load();
            let world = worlds
                .first()
                .expect("There should always be at least one world");
            let dimension = world.dimension.minecraft_name.to_string();

            // Currently no active waypoints are tracked in the level
            sender
                .send_message(TextComponent::translate(
                    translation::java::COMMANDS_WAYPOINT_LIST_EMPTY,
                    [TextComponent::text(dimension)],
                ))
                .await;
            Ok(0)
        })
    }
}

enum ColorAction {
    Named,
    Hex,
    Reset,
}

struct ColorExecutor(ColorAction);

impl CommandExecutor for ColorExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let _waypoint_entity = EntityArgumentConsumer::find_arg(args, ARG_WAYPOINT)?;

            match self.0 {
                ColorAction::Named => {
                    let color = TeamColorArgumentConsumer::find_arg(args, ARG_COLOR)?;
                    sender
                        .send_message(TextComponent::translate(
                            translation::java::COMMANDS_WAYPOINT_MODIFY_COLOR,
                            [TextComponent::text(color.name()).color_named(color)],
                        ))
                        .await;
                }
                ColorAction::Hex => {
                    let color_val = HexColorArgumentConsumer::find_arg(args, ARG_COLOR)?;
                    let hex_str = format!("{:06X}", color_val & 0xFFFFFF);
                    sender
                        .send_message(TextComponent::translate(
                            translation::java::COMMANDS_WAYPOINT_MODIFY_COLOR,
                            [TextComponent::text(hex_str)],
                        ))
                        .await;
                }
                ColorAction::Reset => {
                    sender
                        .send_message(TextComponent::translate(
                            translation::java::COMMANDS_WAYPOINT_MODIFY_COLOR_RESET,
                            [],
                        ))
                        .await;
                }
            }

            Ok(0)
        })
    }
}

enum StyleAction {
    Set,
    Reset,
}

struct StyleExecutor(StyleAction);

impl CommandExecutor for StyleExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let _waypoint_entity = EntityArgumentConsumer::find_arg(args, ARG_WAYPOINT)?;

            match self.0 {
                StyleAction::Set => {
                    let _style = ResourceLocationArgumentConsumer::find_arg(args, ARG_STYLE)?;
                }
                StyleAction::Reset => {}
            }

            sender
                .send_message(TextComponent::translate(
                    translation::java::COMMANDS_WAYPOINT_MODIFY_STYLE,
                    [],
                ))
                .await;

            Ok(0)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    let color_node = literal("color")
        .then(
            argument(ARG_COLOR, TeamColorArgumentConsumer)
                .execute(ColorExecutor(ColorAction::Named)),
        )
        .then(literal("hex").then(
            argument(ARG_COLOR, HexColorArgumentConsumer).execute(ColorExecutor(ColorAction::Hex)),
        ))
        .then(literal("reset").execute(ColorExecutor(ColorAction::Reset)));

    let style_node = literal("style")
        .then(literal("reset").execute(StyleExecutor(StyleAction::Reset)))
        .then(
            literal("set").then(
                argument(ARG_STYLE, ResourceLocationArgumentConsumer)
                    .execute(StyleExecutor(StyleAction::Set)),
            ),
        );

    let modify_node = literal("modify").then(
        argument(ARG_WAYPOINT, EntityArgumentConsumer)
            .then(color_node)
            .then(style_node),
    );

    CommandTree::new(NAMES, DESCRIPTION)
        .then(literal("list").execute(ListExecutor))
        .then(modify_node)
}
