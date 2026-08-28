use pumpkin_data::translation;
use pumpkin_protocol::java::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};
use pumpkin_util::text::TextComponent;

use crate::command::CommandSender;
use crate::command::args::{
    Arg, ArgumentConsumer, ConsumeResult, ConsumeResultWithSyntax, DefaultNameArgConsumer, FindArg,
    GetClientSideArgParser, SuggestResult,
};
use crate::command::dispatcher::CommandError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::tree::RawArgs;
use crate::server::Server;

pub const INVALID_HEX_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_HEXCOLOR_INVALID,
    translation::java::ARGUMENT_HEXCOLOR_INVALID,
);

pub struct HexColorArgumentConsumer;

fn parse_hex_color(color_str: &str) -> Option<u32> {
    match color_str.len() {
        3 => {
            let r = u32::from_str_radix(&color_str[0..1], 16).ok()? * 17;
            let g = u32::from_str_radix(&color_str[1..2], 16).ok()? * 17;
            let b = u32::from_str_radix(&color_str[2..3], 16).ok()? * 17;
            Some((r << 16) | (g << 8) | b)
        }
        6 => {
            let r = u32::from_str_radix(&color_str[0..2], 16).ok()?;
            let g = u32::from_str_radix(&color_str[2..4], 16).ok()?;
            let b = u32::from_str_radix(&color_str[4..6], 16).ok()?;
            Some((r << 16) | (g << 8) | b)
        }
        _ => None,
    }
}

impl GetClientSideArgParser for HexColorArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        ArgumentType::HexColor
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for HexColorArgumentConsumer {
    fn consume<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let s_opt: Option<&'a str> = args.pop().map(|arg| arg.value);

        s_opt.and_then(|s| parse_hex_color(s).map(Arg::HexColor))
    }

    fn consume_with_syntax<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResultWithSyntax<'a> {
        let Some(raw_arg) = args.pop() else {
            return Ok(None);
        };

        parse_hex_color(raw_arg.value)
            .map(|color| Some(Arg::HexColor(color)))
            .ok_or_else(|| {
                INVALID_HEX_ERROR_TYPE.create_without_context_args_slice(&[TextComponent::text(
                    raw_arg.value.to_string(),
                )])
            })
    }

    fn suggest(&self, _sender: &CommandSender, _server: &Server, _input: &str) -> SuggestResult {
        let suggestions = vec![
            CommandSuggestion::new("F00".to_string(), None),
            CommandSuggestion::new("FF0000".to_string(), None),
        ];
        Ok(Some(suggestions))
    }
}

impl DefaultNameArgConsumer for HexColorArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "color"
    }
}

impl<'a> FindArg<'a> for HexColorArgumentConsumer {
    type Data = u32;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::HexColor(color)) => Ok(*color),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_color_works() {
        assert_eq!(parse_hex_color("F00"), Some(0xFF0000));
        assert_eq!(parse_hex_color("0F0"), Some(0x00FF00));
        assert_eq!(parse_hex_color("00F"), Some(0x0000FF));
        assert_eq!(parse_hex_color("FF0000"), Some(0xFF0000));
        assert_eq!(parse_hex_color("123456"), Some(0x123456));
        assert_eq!(parse_hex_color("FFFFF"), None);
        assert_eq!(parse_hex_color("F"), None);
        assert_eq!(parse_hex_color("GGGGGG"), None);
    }
}
