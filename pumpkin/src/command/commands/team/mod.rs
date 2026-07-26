use crate::command::argument_builder::{ArgumentBuilder, command};
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::entity::EntityBase;
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};

mod members;
mod modify;

use members::{add_branch, empty_branch, join_branch, leave_branch, list_branch, remove_branch};
use modify::modify_branch;

const DESCRIPTION: &str = "Manages teams.";
const PERMISSION: &str = "minecraft:command.team";

const ARG_TEAM_NAME: &str = "name";
const ARG_DISPLAY_NAME: &str = "displayName";
const ARG_TEAM: &str = "team";
const ARG_MEMBERS: &str = "members";
const ARG_VALUE: &str = "value";

const DUPLICATE_TEAM_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_TEAM_ADD_DUPLICATE,
    translation::java::COMMANDS_TEAM_ADD_DUPLICATE,
);

const TEAM_NOT_FOUND_ERROR: CommandErrorType<1> = CommandErrorType::new(
    translation::java::TEAM_NOTFOUND,
    translation::java::TEAM_NOTFOUND,
);

const EMPTY_UNCHANGED_ERROR: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_TEAM_EMPTY_UNCHANGED,
    translation::java::COMMANDS_TEAM_EMPTY_UNCHANGED,
);

const COLOR_UNCHANGED_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_TEAM_OPTION_COLOR_UNCHANGED,
    translation::java::COMMANDS_TEAM_OPTION_COLOR_UNCHANGED,
);

const NAME_UNCHANGED_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_TEAM_OPTION_NAME_UNCHANGED,
    translation::java::COMMANDS_TEAM_OPTION_NAME_UNCHANGED,
);

const NAMETAG_VISIBILITY_UNCHANGED_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_TEAM_OPTION_NAMETAGVISIBILITY_UNCHANGED,
    translation::java::COMMANDS_TEAM_OPTION_NAMETAGVISIBILITY_UNCHANGED,
);

const COLLISION_RULE_UNCHANGED_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_TEAM_OPTION_COLLISIONRULE_UNCHANGED,
    translation::java::COMMANDS_TEAM_OPTION_COLLISIONRULE_UNCHANGED,
);

const FRIENDLY_FIRE_ALREADY_ENABLED_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_TEAM_OPTION_FRIENDLYFIRE_ALREADYENABLED,
    translation::java::COMMANDS_TEAM_OPTION_FRIENDLYFIRE_ALREADYENABLED,
);

const FRIENDLY_FIRE_ALREADY_DISABLED_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_TEAM_OPTION_FRIENDLYFIRE_ALREADYDISABLED,
    translation::java::COMMANDS_TEAM_OPTION_FRIENDLYFIRE_ALREADYDISABLED,
);

const SEE_FRIENDLY_INVISIBLES_ALREADY_ENABLED_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_TEAM_OPTION_SEEFRIENDLYINVISIBLES_ALREADYENABLED,
    translation::java::COMMANDS_TEAM_OPTION_SEEFRIENDLYINVISIBLES_ALREADYENABLED,
);

const SEE_FRIENDLY_INVISIBLES_ALREADY_DISABLED_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_TEAM_OPTION_SEEFRIENDLYINVISIBLES_ALREADYDISABLED,
    translation::java::COMMANDS_TEAM_OPTION_SEEFRIENDLYINVISIBLES_ALREADYDISABLED,
);

fn get_entity_scoreboard_name(entity: &dyn EntityBase) -> String {
    entity.get_player().map_or_else(
        || entity.get_entity().entity_uuid.to_string(),
        |player| player.gameprofile.name.clone(),
    )
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("team", DESCRIPTION)
            .requires(PERMISSION)
            .then(add_branch())
            .then(remove_branch())
            .then(empty_branch())
            .then(join_branch())
            .then(leave_branch())
            .then(list_branch())
            .then(modify_branch()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_team_command_with_description() {
        let mut dispatcher = CommandDispatcher::new();
        let mut registry = PermissionRegistry::new();
        register(&mut dispatcher, &mut registry);
        let commands = dispatcher.get_all_commands();
        assert_eq!(commands.get("team").copied(), Some(DESCRIPTION));
    }
}
