use async_trait::async_trait;

use crate::command::args::resource::effect::EffectTypeArgumentConsumer;

use crate::TextComponent;

use crate::command::args::players::PlayersArgumentConsumer;

use crate::command::args::{Arg, ConsumedArgs};
use crate::command::dispatcher::CommandError;
use crate::command::dispatcher::CommandError::InvalidConsumption;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandSender};
use crate::server::Server;

const NAMES: [&str; 1] = ["effect"];

const DESCRIPTION: &str = "Adds or removes the status effects of players and other entities.";

// const ARG_CLEAR: &str = "clear";
const ARG_GIVE: &str = "give";

const ARG_TARGET: &str = "target";
const ARG_EFFECT: &str = "effect";

struct GiveExecutor;

#[async_trait]
impl CommandExecutor for GiveExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Players(targets)) = args.get(ARG_TARGET) else {
            return Err(InvalidConsumption(Some(ARG_TARGET.into())));
        };
        let Some(Arg::Effect(effect)) = args.get(ARG_EFFECT) else {
            return Err(InvalidConsumption(Some(ARG_EFFECT.into())));
        };

        let target_count = targets.len();

        for target in targets {
            target
                .add_effect(
                    crate::entity::effect::Effect {
                        r#type: *effect,
                        duration: 30,
                        amplifier: 1,
                        ambient: true,
                        show_particles: true,
                        show_icon: true,
                    },
                    true,
                )
                .await;
        }

        let translation_name =
            TextComponent::translate(format!("effect.minecraft.{}", effect.to_name()), []);
        if target_count == 1 {
            // TODO: use entity name
            sender
                .send_message(TextComponent::translate(
                    "commands.effect.give.success.single",
                    [
                        translation_name,
                        TextComponent::text(targets[0].gameprofile.name.clone()),
                    ],
                ))
                .await;
        } else {
            sender
                .send_message(TextComponent::translate(
                    "commands.effect.give.success.multiple",
                    [
                        translation_name,
                        TextComponent::text(target_count.to_string()),
                    ],
                ))
                .await;
        }

        Ok(())
    }
}

#[allow(clippy::redundant_closure_for_method_calls)]
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        literal(ARG_GIVE).then(
            argument(ARG_TARGET, PlayersArgumentConsumer)
                .then(argument(ARG_EFFECT, EffectTypeArgumentConsumer).execute(GiveExecutor)),
        ),
    )
    // TODO: Add more things
}
