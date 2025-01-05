use async_trait::async_trait;
use pumpkin_core::text::{TextComponent, TextContent};
use pumpkin_protocol::client::play::CSystemChatMessage;
use pumpkin_registry::SYNCED_REGISTRIES;

use crate::command::{
    args::{arg_message::MsgArgConsumer, Arg, ConsumedArgs},
    tree::CommandTree,
    tree_builder::argument,
    CommandError, CommandExecutor, CommandSender,
};
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["say"];

const DESCRIPTION: &str = "Broadcast a message to all Players.";

const ARG_MESSAGE: &str = "message";

struct SayExecutor;

#[async_trait]
impl CommandExecutor for SayExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Msg(msg)) = args.get(ARG_MESSAGE) else {
            return Err(InvalidConsumption(Some(ARG_MESSAGE.into())));
        };

        let chat_type = SYNCED_REGISTRIES.chat_type.get("say_command")
                                .expect("Incomplete chat type registry, missing say_command");

        server
            .broadcast_packet_all(&CSystemChatMessage::new(
                &TextComponent { 
                    content: TextContent::Translate {
                        translate: chat_type.chat.translation_key.clone().into(),
                        with: vec![
                            TextComponent::text(sender.to_string()),
                            TextComponent::text(msg.to_string()),
                        ]
                    },
                    style: chat_type.chat.style.clone().unwrap_or_default(),
                    extra: vec![],
                },
                false,
            ))
            .await;
        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(argument(ARG_MESSAGE, MsgArgConsumer).execute(SayExecutor))
}
