use async_trait::async_trait;
use pumpkin_data::entity;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::hover::HoverEvent;

use crate::command::args::entities::EntitiesArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, require};
use crate::command::{CommandError, CommandExecutor, CommandSender};
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["kill"];
const DESCRIPTION: &str = "Kills all target entities.";

const ARG_TARGET: &str = "target";

struct Executor;

#[async_trait]
impl CommandExecutor for Executor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Entities(targets)) = args.get(&ARG_TARGET) else {
            return Err(InvalidConsumption(Some(ARG_TARGET.into())));
        };

        let target_count = targets.len();
        let mut name = String::new();
        for target in targets {
            target.living_entity.kill().await;
            name.clone_from(&target.gameprofile.name);
        }

        let msg = if target_count == 1 {
            let entity = &targets[0].living_entity.entity;
            let mut entity_display =
                TextComponent::text(name.clone()).hover_event(HoverEvent::show_entity(
                    entity.entity_uuid.to_string(),
                    entity.entity_type.resource_name.into(),
                    Some(TextComponent::text(name.clone())),
                ));

            if entity.entity_type == entity::EntityType::PLAYER {
                entity_display = entity_display.click_event(ClickEvent::SuggestCommand(
                    format!("/tell {} ", name.clone()).into(),
                ));
            }

            TextComponent::translate("commands.kill.success.single", [entity_display])
        } else {
            TextComponent::translate(
                "commands.kill.success.multiple",
                [TextComponent::text(target_count.to_string())],
            )
        };

        sender.send_message(msg).await;

        Ok(())
    }
}

struct SelfExecutor;

#[async_trait]
impl CommandExecutor for SelfExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let target = sender.as_player().ok_or(CommandError::InvalidRequirement)?;
        let name = target.gameprofile.name.clone();
        let entity = &target.living_entity.entity;

        target.living_entity.kill().await;

        sender
            .send_message(TextComponent::translate(
                "commands.kill.success.single",
                [TextComponent::text(name.clone())
                    .hover_event(HoverEvent::show_entity(
                        entity.entity_uuid.to_string(),
                        entity.entity_type.resource_name.into(),
                        Some(TextComponent::text(name.clone())),
                    ))
                    .click_event(ClickEvent::SuggestCommand(
                        format!("/tell {} ", name.clone()).into(),
                    ))],
            ))
            .await;

        Ok(())
    }
}

#[allow(clippy::redundant_closure_for_method_calls)] // causes lifetime issues
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_TARGET, EntitiesArgumentConsumer).execute(Executor))
        .then(require(|sender| sender.is_player()).execute(SelfExecutor))
}
