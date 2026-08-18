use pumpkin_data::translation;
use pumpkin_protocol::java::client::play::{CWaypoint, WaypointIcon};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;

use crate::command::args::{
    FindArg, entity::EntityArgumentConsumer, hex_color::HexColorArgumentConsumer,
    resource_location::ResourceLocationArgumentConsumer, team_color::TeamColorArgumentConsumer,
};
use crate::command::tree::builder::{argument, literal};
use crate::command::{
    CommandError, CommandExecutor, CommandResult, CommandSender, ConsumedArgs, tree::CommandTree,
};

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
            let world = worlds.first().ok_or(CommandError::InvalidRequirement)?;
            let dimension = world.dimension.minecraft_name.to_string();

            sender
                .send_message(pumpkin_macros::translate_cross!(
                    translation::java::COMMANDS_WAYPOINT_LIST_EMPTY,
                    translation::java::COMMANDS_WAYPOINT_LIST_EMPTY,
                    TextComponent::text(dimension)
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
            let waypoint_entity = EntityArgumentConsumer::find_arg(args, ARG_WAYPOINT)?;
            let entity = waypoint_entity.get_entity();
            let pos = entity.pos.load();
            let block_pos = BlockPos::new(
                pos.x.floor() as i32,
                pos.y.floor() as i32,
                pos.z.floor() as i32,
            );
            let uuid = entity.entity_uuid;

            let color_val = match self.0 {
                ColorAction::Named => {
                    let color = TeamColorArgumentConsumer::find_arg(args, ARG_COLOR)?;
                    let rgb = color.to_rgb();
                    i32::from_be_bytes([0, rgb.red, rgb.green, rgb.blue])
                }
                ColorAction::Hex => HexColorArgumentConsumer::find_arg(args, ARG_COLOR)? as i32,
                ColorAction::Reset => 0xFFFFFF,
            };

            let packet = CWaypoint::update_position(
                uuid,
                Some(WaypointIcon {
                    style: None,
                    color: color_val,
                }),
                block_pos,
            );

            if let Some(player) = sender.as_player() {
                player.client.enqueue_packet(&packet).await;
            }

            match self.0 {
                ColorAction::Named => {
                    let color = TeamColorArgumentConsumer::find_arg(args, ARG_COLOR)?;
                    sender
                        .send_message(pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_WAYPOINT_MODIFY_COLOR,
                            translation::java::COMMANDS_WAYPOINT_MODIFY_COLOR,
                            TextComponent::text(color.name()).color_named(color)
                        ))
                        .await;
                }
                ColorAction::Hex => {
                    let color_val = HexColorArgumentConsumer::find_arg(args, ARG_COLOR)?;
                    let hex_str = format!("{:06X}", color_val & 0xFFFFFF);
                    sender
                        .send_message(pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_WAYPOINT_MODIFY_COLOR,
                            translation::java::COMMANDS_WAYPOINT_MODIFY_COLOR,
                            TextComponent::text(hex_str)
                        ))
                        .await;
                }
                ColorAction::Reset => {
                    sender
                        .send_message(pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_WAYPOINT_MODIFY_COLOR_RESET,
                            translation::java::COMMANDS_WAYPOINT_MODIFY_COLOR_RESET
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
            let waypoint_entity = EntityArgumentConsumer::find_arg(args, ARG_WAYPOINT)?;
            let entity = waypoint_entity.get_entity();
            let pos = entity.pos.load();
            let block_pos = BlockPos::new(
                pos.x.floor() as i32,
                pos.y.floor() as i32,
                pos.z.floor() as i32,
            );
            let uuid = entity.entity_uuid;

            let style_str = match self.0 {
                StyleAction::Set => {
                    let style = ResourceLocationArgumentConsumer::find_arg(args, ARG_STYLE)?;
                    Some(style)
                }
                StyleAction::Reset => None,
            };

            let packet = CWaypoint::update_position(
                uuid,
                Some(WaypointIcon {
                    style: style_str,
                    color: 0xFFFFFF,
                }),
                block_pos,
            );

            if let Some(player) = sender.as_player() {
                player.client.enqueue_packet(&packet).await;
            }

            sender
                .send_message(pumpkin_macros::translate_cross!(
                    translation::java::COMMANDS_WAYPOINT_MODIFY_STYLE,
                    translation::java::COMMANDS_WAYPOINT_MODIFY_STYLE
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
