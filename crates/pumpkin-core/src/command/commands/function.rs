use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;

use crate::command::argument_builder::{
    ArgumentBuilder, LiteralArgumentBuilder, argument, command, literal,
};
use crate::command::argument_types::function::FunctionArgumentType;
use crate::command::argument_types::nbt::NbtCompoundArgumentType;
use crate::command::argument_types::nbt_path::{NbtPath, NbtPathArgumentType};
use crate::command::commands::data::{TargetKind, get_single_tag, make_target_arg};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::suggestion::provider::{SuggestionProvider, SuggestionProviderResult};
use crate::command::suggestion::suggestions::SuggestionsBuilder;

const DESCRIPTION: &str = "Runs commands found in the corresponding function files.";
const PERMISSION: &str = "minecraft:command.function";

static ERROR_UNKNOWN_FUNCTION: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENTS_FUNCTION_UNKNOWN,
    translation::java::ARGUMENTS_FUNCTION_UNKNOWN,
);

struct FunctionSuggestionProvider;

impl SuggestionProvider for FunctionSuggestionProvider {
    fn suggest(
        &self,
        context: &CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> SuggestionProviderResult {
        let server = context.server();
        let function_names = server.datapack_manager.get_function_names();
        for name in function_names {
            builder = builder.suggest(name);
        }
        builder.build()
    }
}

static ERROR_ARGUMENT_NOT_COMPOUND: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_FUNCTION_ERROR_ARGUMENT_NOT_COMPOUND,
    translation::java::COMMANDS_FUNCTION_ERROR_ARGUMENT_NOT_COMPOUND,
);

static ERROR_INSTANTIATION_FAILURE: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_FUNCTION_INSTANTIATIONFAILURE,
    translation::java::COMMANDS_FUNCTION_INSTANTIATIONFAILURE,
);

/// Mirrors vanilla `TagType.getName()`.
const fn tag_type_name(tag: &NbtTag) -> &'static str {
    match tag {
        NbtTag::End => "END",
        NbtTag::Byte(_) => "BYTE",
        NbtTag::Short(_) => "SHORT",
        NbtTag::Int(_) => "INT",
        NbtTag::Long(_) => "LONG",
        NbtTag::Float(_) => "FLOAT",
        NbtTag::Double(_) => "DOUBLE",
        NbtTag::ByteArray(_) => "BYTE[]",
        NbtTag::String(_) => "STRING",
        NbtTag::List(_) => "LIST",
        NbtTag::Compound(_) => "COMPOUND",
        NbtTag::IntArray(_) => "INT[]",
        NbtTag::LongArray(_) => "LONG[]",
    }
}

#[derive(Clone, Copy)]
enum FunctionArguments {
    None,
    Inline,
    Source { kind: TargetKind, has_path: bool },
}

struct FunctionExecutor(FunctionArguments);

impl FunctionExecutor {
    fn arguments(
        &self,
        context: &CommandContext,
    ) -> Result<Option<NbtCompound>, CommandSyntaxError> {
        match self.0 {
            FunctionArguments::None => Ok(None),
            FunctionArguments::Inline => Ok(Some(
                NbtCompoundArgumentType::get(context, "arguments")?.clone(),
            )),
            FunctionArguments::Source { kind, has_path } => {
                let accessor = kind.access(context, "source")?;
                if !has_path {
                    return Ok(Some(accessor.get_data()?));
                }
                let path = context.get_argument::<NbtPath>("path")?;
                match get_single_tag(path, accessor.as_ref())? {
                    NbtTag::Compound(compound) => Ok(Some(compound)),
                    other => Err(ERROR_ARGUMENT_NOT_COMPOUND
                        .create_without_context(TextComponent::text(tag_type_name(&other)))),
                }
            }
        }
    }
}

impl CommandExecutor for FunctionExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let name_str = FunctionArgumentType::get(context, "name")?;
        let server = context.server();
        let arguments = self.arguments(context)?;

        let result = match server.datapack_manager.execute_function_with(
            server,
            &context.source,
            name_str,
            arguments.as_ref(),
        ) {
            Ok(result) => result,
            Err(_)
                if !server
                    .datapack_manager
                    .get_function_names()
                    .iter()
                    .any(|n| n == name_str) =>
            {
                return Err(ERROR_UNKNOWN_FUNCTION
                    .create_without_context(TextComponent::text(name_str.to_string())));
            }
            Err(message) => {
                return Err(ERROR_INSTANTIATION_FAILURE.create_without_context(
                    TextComponent::text(name_str.to_string()),
                    TextComponent::text(message),
                ));
            }
        };
        if let Some(value) = result.returned {
            return Ok(value);
        }
        let executed_count = result.executed;

        if name_str.starts_with('#') {
            context.source.send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_FUNCTION_SUCCESS_MULTIPLE,
                    translation::java::COMMANDS_FUNCTION_SUCCESS_MULTIPLE,
                    [
                        TextComponent::text(executed_count.to_string()),
                        TextComponent::text(name_str.to_string()),
                    ],
                ),
                true,
            );
        } else {
            context.source.send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_FUNCTION_SUCCESS_SINGLE,
                    translation::java::COMMANDS_FUNCTION_SUCCESS_SINGLE,
                    [
                        TextComponent::text(executed_count.to_string()),
                        TextComponent::text(name_str.to_string()),
                    ],
                ),
                true,
            );
        }

        Ok(executed_count as i32)
    }
}

fn with_sources() -> LiteralArgumentBuilder {
    let mut with = literal("with");
    for (kind, name) in [
        (TargetKind::Block, "block"),
        (TargetKind::Entity, "entity"),
        (TargetKind::Storage, "storage"),
    ] {
        with = with.then(
            literal(name).then(
                make_target_arg(kind, "source_target")
                    .executes(FunctionExecutor(FunctionArguments::Source {
                        kind,
                        has_path: false,
                    }))
                    .then(
                        argument("path", NbtPathArgumentType).executes(FunctionExecutor(
                            FunctionArguments::Source {
                                kind,
                                has_path: true,
                            },
                        )),
                    ),
            ),
        );
    }
    with
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("function", DESCRIPTION).requires(PERMISSION).then(
            argument("name", FunctionArgumentType)
                .suggests(FunctionSuggestionProvider)
                .executes(FunctionExecutor(FunctionArguments::None))
                .then(
                    argument("arguments", NbtCompoundArgumentType)
                        .executes(FunctionExecutor(FunctionArguments::Inline)),
                )
                .then(with_sources()),
        ),
    );
}
