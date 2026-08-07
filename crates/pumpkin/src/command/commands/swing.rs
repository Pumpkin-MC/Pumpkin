use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use pumpkin_data::translation;
use pumpkin_protocol::bedrock::server::animate::{AnimateAction, SAnimate};
use pumpkin_protocol::codec::var_ulong::VarULong;
use pumpkin_protocol::java::client::play::{Animation, CEntityAnimation};
use pumpkin_util::Hand;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

// Broadcasts the swing animation directly (mirrors LivingEntity::swing_hand in
// entity/living.rs, which only ever swings the main hand) instead of adding a hand
// parameter there -- that module has unrelated concurrent edits in flight this session.
async fn broadcast_swing(target: &dyn crate::entity::EntityBase, hand: Hand) {
    let entity = target.get_entity();
    let world = entity.world.load();
    let entity_id = entity.entity_id;

    let animation = match hand {
        Hand::Right => Animation::SwingMainArm,
        Hand::Left => Animation::SwingOffhand,
    };
    let je_packet = CEntityAnimation::new(entity_id.into(), animation);
    let be_packet = SAnimate {
        action: AnimateAction::SwingArm,
        runtime_entity_id: VarULong(entity_id as u64),
        data: 0.0,
        swing_source: None,
    };

    world.broadcast_editioned(&je_packet, &be_packet).await;
}

const DESCRIPTION: &str = "Makes entities swing their hand.";

const PERMISSION: &str = "minecraft:command.swing";

const ARG_TARGETS: &str = "targets";

// vanilla SwingCommand.ERROR_NO_LIVING_ENTITY (commands.swing.failed.notliving), no
// translation arguments.
const ERROR_NO_LIVING_ENTITY: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_SWING_FAILED_NOTLIVING,
    translation::java::COMMANDS_SWING_FAILED_NOTLIVING,
);

async fn swing_targets(
    context: &CommandContext<'_>,
    targets: &[std::sync::Arc<dyn crate::entity::EntityBase>],
    hand: Hand,
) -> Result<i32, crate::command::errors::command_syntax_error::CommandSyntaxError> {
    let mut swung = 0;
    for target in targets {
        if target.get_living_entity().is_some() {
            broadcast_swing(target.as_ref(), hand).await;
            swung += 1;
        }
    }

    if swung == 0 {
        return Err(ERROR_NO_LIVING_ENTITY.create_without_context());
    }

    let msg = if swung == 1 {
        TextComponent::translate_cross(
            translation::java::COMMANDS_SWING_SUCCESS_SINGLE,
            translation::java::COMMANDS_SWING_SUCCESS_SINGLE,
            [targets[0].get_display_name().await],
        )
    } else {
        TextComponent::translate_cross(
            translation::java::COMMANDS_SWING_SUCCESS_MULTIPLE,
            translation::java::COMMANDS_SWING_SUCCESS_MULTIPLE,
            [TextComponent::text(swung.to_string())],
        )
    };
    context.source.send_feedback(msg, true).await;

    Ok(swung)
}

struct SelfExecutor;

impl CommandExecutor for SelfExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let target = context.source.entity_or_err()?;
            swing_targets(context, &[target], Hand::Right).await
        })
    }
}

struct TargetsMainHandExecutor;

impl CommandExecutor for TargetsMainHandExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_entities(context, ARG_TARGETS).await?;
            swing_targets(context, &targets, Hand::Right).await
        })
    }
}

struct TargetsOffHandExecutor;

impl CommandExecutor for TargetsOffHandExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_entities(context, ARG_TARGETS).await?;
            swing_targets(context, &targets, Hand::Left).await
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
        command("swing", DESCRIPTION)
            .requires(PERMISSION)
            .executes(SelfExecutor)
            .then(
                argument(ARG_TARGETS, EntityArgumentType::Entities)
                    .executes(TargetsMainHandExecutor)
                    .then(literal("mainhand").executes(TargetsMainHandExecutor))
                    .then(literal("offhand").executes(TargetsOffHandExecutor)),
            ),
    );
}
