use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use uuid::Uuid;

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::coordinates::vec3::Vec3ArgumentType;
use crate::command::argument_types::resource::{ENTITY_TYPE_ARGUMENT, ResourceArgument};
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::r#type::from_type;

const DESCRIPTION: &str = "Spawns an entity at position.";
const PERMISSION: &str = "minecraft:command.summon";

struct SummonExecutor {
    has_pos: bool,
}

impl CommandExecutor for SummonExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let entity_type = ResourceArgument::get_summonable_entity_type(context, "entity")?;
        let pos = if self.has_pos {
            Vec3ArgumentType::get_coordinates(context, "pos")?.resolve(&context.source)
        } else {
            context.source.position
        };

        let world = context.source.world();
        let entity = from_type(entity_type, pos, world, Uuid::new_v4());
        let name = entity.get_display_name();
        world.spawn_entity(entity);

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_SUMMON_SUCCESS,
                translation::bedrock::COMMANDS_SUMMON_SUCCESS,
                [name],
            ),
            true,
        );

        Ok(1)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("summon", DESCRIPTION).requires(PERMISSION).then(
            argument("entity", ENTITY_TYPE_ARGUMENT.clone())
                .executes(SummonExecutor { has_pos: false })
                .then(
                    argument("pos", Vec3ArgumentType::Default)
                        .executes(SummonExecutor { has_pos: true }),
                ),
        ),
    );
}
