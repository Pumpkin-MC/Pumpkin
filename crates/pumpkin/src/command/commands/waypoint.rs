use pumpkin_data::translation;
use pumpkin_protocol::java::client::play::{CWaypoint, WaypointIcon};
use pumpkin_util::PermissionLvl;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::hex_color::HexColorArgumentType;
use crate::command::argument_types::identifier::IdentifierArgumentType;
use crate::command::argument_types::team_color::TeamColorArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "List or modify waypoints.";
const PERMISSION: &str = "minecraft:command.waypoint";

struct ListExecutor;

impl CommandExecutor for ListExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let world = context.source.world();
        let dimension = world.dimension.minecraft_name.to_string();

        context.source.send_feedback(
            pumpkin_macros::translate_cross!(
                translation::java::COMMANDS_WAYPOINT_LIST_EMPTY,
                translation::java::COMMANDS_WAYPOINT_LIST_EMPTY,
                TextComponent::text(dimension)
            ),
            false,
        );
        Ok(0)
    }
}

enum ColorAction {
    Named,
    Hex,
    Reset,
}

struct ColorExecutor(ColorAction);

impl CommandExecutor for ColorExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let waypoint_entity = EntityArgumentType::get_entity(context, "waypoint")?;
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
                let color = TeamColorArgumentType::get(context, "color")?;
                let rgb = color.to_rgb();
                i32::from_be_bytes([0, rgb.red, rgb.green, rgb.blue])
            }
            ColorAction::Hex => {
                let rgb = HexColorArgumentType::get(context, "color")?;
                i32::from_be_bytes([0, rgb.red, rgb.green, rgb.blue])
            }
            ColorAction::Reset => 0xFFFFFF,
        };

        if let Some(player) = context.source.as_player() {
            let packet = CWaypoint::update_position(
                uuid,
                Some(WaypointIcon {
                    style: None,
                    color: color_val,
                }),
                block_pos,
            );
            player.try_send_client_packet(&packet);
        }

        match self.0 {
            ColorAction::Named => {
                let color = TeamColorArgumentType::get(context, "color")?;
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_WAYPOINT_MODIFY_COLOR,
                        translation::java::COMMANDS_WAYPOINT_MODIFY_COLOR,
                        TextComponent::text(color.name()).color_named(color)
                    ),
                    true,
                );
            }
            ColorAction::Hex => {
                let rgb = HexColorArgumentType::get(context, "color")?;
                let hex_str = format!("{:02X}{:02X}{:02X}", rgb.red, rgb.green, rgb.blue);
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_WAYPOINT_MODIFY_COLOR,
                        translation::java::COMMANDS_WAYPOINT_MODIFY_COLOR,
                        TextComponent::text(hex_str)
                    ),
                    true,
                );
            }
            ColorAction::Reset => {
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_WAYPOINT_MODIFY_COLOR_RESET,
                        translation::java::COMMANDS_WAYPOINT_MODIFY_COLOR_RESET
                    ),
                    true,
                );
            }
        }

        Ok(0)
    }
}

enum StyleAction {
    Set,
    Reset,
}

struct StyleExecutor(StyleAction);

impl CommandExecutor for StyleExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let waypoint_entity = EntityArgumentType::get_entity(context, "waypoint")?;
        let entity = waypoint_entity.get_entity();
        let pos = entity.pos.load();
        let block_pos = BlockPos::new(
            pos.x.floor() as i32,
            pos.y.floor() as i32,
            pos.z.floor() as i32,
        );
        let uuid = entity.entity_uuid;

        let style_owned = match self.0 {
            StyleAction::Set => {
                let style = IdentifierArgumentType::get(context, "style")?;
                Some(style.to_string())
            }
            StyleAction::Reset => None,
        };

        if let Some(player) = context.source.as_player() {
            let packet = CWaypoint::update_position(
                uuid,
                Some(WaypointIcon {
                    style: style_owned.as_deref(),
                    color: 0xFFFFFF,
                }),
                block_pos,
            );
            player.try_send_client_packet(&packet);
        }

        context.source.send_feedback(
            pumpkin_macros::translate_cross!(
                translation::java::COMMANDS_WAYPOINT_MODIFY_STYLE,
                translation::java::COMMANDS_WAYPOINT_MODIFY_STYLE
            ),
            true,
        );

        Ok(0)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let color_node = literal("color")
        .then(argument("color", TeamColorArgumentType).executes(ColorExecutor(ColorAction::Named)))
        .then(literal("hex").then(
            argument("color", HexColorArgumentType).executes(ColorExecutor(ColorAction::Hex)),
        ))
        .then(literal("reset").executes(ColorExecutor(ColorAction::Reset)));

    let style_node = literal("style")
        .then(literal("reset").executes(StyleExecutor(StyleAction::Reset)))
        .then(literal("set").then(
            argument("style", IdentifierArgumentType).executes(StyleExecutor(StyleAction::Set)),
        ));

    let modify_node = literal("modify").then(
        argument("waypoint", EntityArgumentType::Entity)
            .then(color_node)
            .then(style_node),
    );

    dispatcher.register(
        command("waypoint", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("list").executes(ListExecutor))
            .then(modify_node),
    );
}
