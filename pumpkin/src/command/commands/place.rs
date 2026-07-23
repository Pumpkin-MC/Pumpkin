use pumpkin_data::Rotation;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::coordinates::block_pos::BlockPosArgumentType;
use crate::command::argument_types::template::TemplateNameArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::world::block_placer::WorldBlockPlacer;

use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};

const DESCRIPTION: &str = "Places a structure template in the world.";
const PERMISSION: &str = "minecraft:command.place";

static TEMPLATE_NOT_FOUND: CommandErrorType<1> = CommandErrorType::new(
    "commands.place.template.invalid",
    "commands.place.template.invalid",
);

struct PlaceTemplateExecutor;

impl CommandExecutor for PlaceTemplateExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let template_name = context.get_argument::<String>("template")?.clone();

            let block_pos =
                BlockPosArgumentType::get_block_pos(context, "pos").unwrap_or_else(|_| {
                    let p = context.source.position;
                    BlockPos::new(p.x as i32, p.y as i32, p.z as i32)
                });

            let template_name = template_name
                .strip_prefix("minecraft:")
                .unwrap_or(&template_name)
                .to_string();
            let Some(template) =
                pumpkin_world::generation::structure::template::get_template(&template_name)
            else {
                return Err(TEMPLATE_NOT_FOUND
                    .create_without_context(TextComponent::text(template_name.to_string())));
            };

            let mut placer = WorldBlockPlacer::new(context.world());
            pumpkin_world::generation::structure::template::place_template(
                &mut placer,
                &template,
                block_pos.0,
                (0, 0),
                Rotation::None,
                false,
                false,
                &[],
                None,
            );

            context
                .world()
                .queue_block_updates(&placer.changed_positions)
                .await;
            context.world().flush_block_updates().await;
            placer.finalize().await;

            context
                .source
                .send_feedback(
                    TextComponent::translate(
                        "commands.place.template.success",
                        [
                            TextComponent::text(template_name.to_string()),
                            TextComponent::text(block_pos.0.x.to_string()),
                            TextComponent::text(block_pos.0.y.to_string()),
                            TextComponent::text(block_pos.0.z.to_string()),
                        ],
                    ),
                    true,
                )
                .await;

            Ok(1)
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("place", DESCRIPTION).requires(PERMISSION).then(
            literal("template").then(
                argument("template", TemplateNameArgumentType)
                    .executes(PlaceTemplateExecutor)
                    .then(argument("pos", BlockPosArgumentType).executes(PlaceTemplateExecutor)),
            ),
        ),
    );
}
