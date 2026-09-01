use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::hover::HoverEvent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::item::ItemStackArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;

const DESCRIPTION: &str = "Give items to player(s).";
const PERMISSION: &str = "minecraft:command.give";

struct GiveExecutor {
    has_count: bool,
}

impl CommandExecutor for GiveExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_players(context, "targets")?;
        let parsed_stack = ItemStackArgumentType::get(context, "item")?;
        let item = parsed_stack.item;

        let item_count = if self.has_count {
            IntegerArgumentType::get(context, "count")?
        } else {
            1
        };

        for target in &targets {
            let max_stack = i32::from(parsed_stack.get_max_stack_size());
            let mut remaining = item_count;

            while remaining > 0 {
                let take = remaining.min(max_stack);
                let mut stack = parsed_stack.clone();
                stack.item_count = take as u8;
                target.inventory().insert_stack_anywhere(&mut stack);
                if !stack.is_empty() {
                    target.drop_item(stack);
                }
                remaining -= take;
            }
        }

        let item_name = item.registry_key;
        let msg = if targets.len() == 1 {
            TextComponent::translate_cross(
                pumpkin_data::translation::java::COMMANDS_GIVE_SUCCESS_SINGLE,
                pumpkin_data::translation::bedrock::COMMANDS_GIVE_SUCCESS,
                [
                    TextComponent::text(item_count.to_string()),
                    TextComponent::text("[")
                        .add_child(item.translated_name())
                        .add_child(TextComponent::text("]"))
                        .hover_event(HoverEvent::ShowItem {
                            id: item_name.to_string().into(),
                            count: Some(item_count.min(99)),
                        }),
                    targets[0].as_ref().get_display_name(),
                ],
            )
        } else {
            TextComponent::translate_cross(
                pumpkin_data::translation::java::COMMANDS_GIVE_SUCCESS_MULTIPLE,
                pumpkin_data::translation::bedrock::COMMANDS_GIVE_SUCCESS,
                [
                    TextComponent::text(item_count.to_string()),
                    TextComponent::text("[")
                        .add_child(item.translated_name())
                        .add_child(TextComponent::text("]"))
                        .hover_event(HoverEvent::ShowItem {
                            id: item_name.to_string().into(),
                            count: Some(item_count.min(99)),
                        }),
                    TextComponent::text(targets.len().to_string()),
                ],
            )
        };
        context.source.send_feedback(msg, true);

        Ok(targets.len() as i32)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("give", DESCRIPTION).requires(PERMISSION).then(
            argument("targets", EntityArgumentType::Players).then(
                argument("item", ItemStackArgumentType)
                    .executes(GiveExecutor { has_count: false })
                    .then(
                        argument("count", IntegerArgumentType::with_min(1))
                            .executes(GiveExecutor { has_count: true }),
                    ),
            ),
        ),
    );
}
