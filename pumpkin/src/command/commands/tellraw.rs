use crate::command::{
    CommandError, CommandExecutor, CommandSender,
    args::{
        Arg, ConsumedArgs, FindArg, players::PlayersArgumentConsumer,
        textcomponent::TextComponentArgConsumer,
    },
    tree::CommandTree,
    tree::builder::argument,
};
use async_trait::async_trait;
use pumpkin_util::text::color::{Color, NamedColor};
use pumpkin_util::text::style::Style;
use pumpkin_util::text::{TextComponent, TextComponentBase, TextContent};
const NAMES: [&str; 1] = ["tellraw"];

const DESCRIPTION: &str = "Send raw message to players.";

const ARG_TARGETS: &str = "targets";

const ARG_MESSAGE: &str = "message";

struct TellRawExecutor;

#[async_trait]
impl CommandExecutor for TellRawExecutor {
    async fn execute<'a>(
        &self,
        _sender: &mut CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Players(targets)) = args.get(&ARG_TARGETS) else {
            return Err(CommandError::InvalidConsumption(Some(ARG_TARGETS.into())));
        };

        let text = TextComponentArgConsumer::find_arg(args, ARG_MESSAGE)?;
        for target in targets {
            target.send_system_message(&text).await;
        }
        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    // TEST
    let text = TextComponent(TextComponentBase {
        content: TextContent::Text {
            text: "Hello, World".into(),
        },
        style: Style::default().color(Color::Named(NamedColor::Green)),
        extra: vec![],
    });
    println!("text: {}", serde_json::to_string(&text).unwrap_or_default());

    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_TARGETS, PlayersArgumentConsumer)
            .then(argument(ARG_MESSAGE, TextComponentArgConsumer).execute(TellRawExecutor)),
    )
}
