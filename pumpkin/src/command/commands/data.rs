use crate::command::CommandResult;
use crate::command::args::entity::EntityArgumentConsumer;
use crate::command::tree::builder::literal;
use crate::command::{
    CommandError, CommandExecutor, CommandSender,
    args::{Arg, ConsumedArgs},
    tree::{CommandTree, builder::argument},
};
use crate::entity::NBTStorage;
use CommandError::InvalidConsumption;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

const NAMES: [&str; 1] = ["data"];
const DESCRIPTION: &str = "Query and modify data of entities and blocks";

const ARG_ENTITY: &str = "entity";

struct GetEntityDataExecutor;

impl CommandExecutor for GetEntityDataExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Entity(entity)) = args.get(&ARG_ENTITY) else {
                return Err(InvalidConsumption(Some(ARG_ENTITY.into())));
            };
            display_data(entity.as_nbt_storage(), entity.get_display_name().await, sender).await
        })
    }
}

#[expect(clippy::too_many_lines)]
pub fn snbt_colorful_display(tag: &NbtTag, depth: usize) -> Result<TextComponent, String> {
    let folded = TextComponent::text("<...>").color_named(NamedColor::Gray);
    match tag {
        NbtTag::End => Err("Unexpected end tag".into()),
        NbtTag::Byte(value) => {
            let byte_format = TextComponent::text("b").color_named(NamedColor::Red);
            Ok(TextComponent::text(format!("{value}"))
                .color_named(NamedColor::Gold)
                .add_child(byte_format))
        }
        NbtTag::Short(value) => {
            let short_format = TextComponent::text("s").color_named(NamedColor::Red);
            Ok(TextComponent::text(format!("{value}"))
                .color_named(NamedColor::Gold)
                .add_child(short_format))
        }
        NbtTag::Int(value) => {
            Ok(TextComponent::text(format!("{value}")).color_named(NamedColor::Gold))
        }
        NbtTag::Long(value) => {
            let long_format = TextComponent::text("L").color_named(NamedColor::Red);
            Ok(TextComponent::text(format!("{value}"))
                .color_named(NamedColor::Gold)
                .add_child(long_format))
        }
        NbtTag::Float(value) => {
            let float_format = TextComponent::text("f").color_named(NamedColor::Red);
            Ok(TextComponent::text(format!("{value}"))
                .color_named(NamedColor::Gold)
                .add_child(float_format))
        }
        NbtTag::Double(value) => {
            let double_format = TextComponent::text("d").color_named(NamedColor::Red);
            Ok(TextComponent::text(format!("{value}"))
                .color_named(NamedColor::Gold)
                .add_child(double_format))
        }
        NbtTag::ByteArray(value) => {
            let byte_array_format = TextComponent::text("B").color_named(NamedColor::Red);
            let mut content = TextComponent::text("[")
                .add_child(byte_array_format.clone())
                .add_child(TextComponent::text("; "));

            for (index, byte) in value.iter().take(128).enumerate() {
                content = content
                    .add_child(TextComponent::text(format!("{byte}")))
                    .add_child(byte_array_format.clone());
                if index < value.len() - 1 {
                    content = content.add_child(TextComponent::text(", "));
                }
            }

            if value.len() > 128 {
                content = content.add_child(folded);
            }

            content = content.add_child(TextComponent::text("]"));
            Ok(content)
        }
        NbtTag::String(value) => {
            let escaped_value = value
                .replace('"', "\\\"")
                .replace('\\', "\\\\")
                .replace('\n', "\\n")
                .replace('\t', "\\t")
                .replace('\r', "\\r")
                .replace('\x0c', "\\f")
                .replace('\x08', "\\b");

            Ok(TextComponent::text(format!("\"{escaped_value}\"")).color_named(NamedColor::Green))
        }
        NbtTag::List(value) => {
            if value.is_empty() {
                Ok(TextComponent::text("[]"))
            } else if depth >= 64 {
                Ok(TextComponent::text("[")
                    .add_child(folded)
                    .add_child(TextComponent::text("]")))
            } else {
                let mut content = TextComponent::text("[");

                for (index, item) in value.iter().take(128).enumerate() {
                    let item_display = snbt_colorful_display(item, depth + 1)
                        .map_err(|string| format!("Error displaying item.[{index}]: {string}"))?;
                    content = content.add_child(item_display);

                    if index < value.len() - 1 {
                        content = content.add_child(TextComponent::text(", "));
                    }
                }

                if value.len() > 128 {
                    content = content.add_child(folded);
                }

                content = content.add_child(TextComponent::text("]"));
                Ok(content)
            }
        }
        NbtTag::Compound(value) => {
            if value.is_empty() {
                Ok(TextComponent::text("{}"))
            } else if depth >= 64 {
                Ok(TextComponent::text("{")
                    .add_child(folded)
                    .add_child(TextComponent::text("}")))
            } else {
                let mut content = TextComponent::text("{");

                for (index, (key, item)) in value.child_tags.iter().take(128).enumerate() {
                    let item_display = snbt_colorful_display(item, depth + 1)
                        .map_err(|string| format!("Error displaying item.{key}: {string}"))?;
                    content = content
                        .add_child(TextComponent::text(key.clone()).color_named(NamedColor::Aqua))
                        .add_child(TextComponent::text(": "))
                        .add_child(item_display);

                    if index < value.child_tags.len() - 1 {
                        content = content.add_child(TextComponent::text(", "));
                    }
                }

                if value.child_tags.len() > 128 {
                    content = content.add_child(folded);
                }

                content = content.add_child(TextComponent::text("}"));
                Ok(content)
            }
        }
        NbtTag::IntArray(value) => {
            let int_array_format = TextComponent::text("I").color_named(NamedColor::Red);
            let mut content = TextComponent::text("[")
                .add_child(int_array_format)
                .add_child(TextComponent::text("; "));

            for (index, int) in value.iter().take(128).enumerate() {
                content = content
                    .add_child(TextComponent::text(format!("{int}")).color_named(NamedColor::Gold));
                if index < value.len() - 1 {
                    content = content.add_child(TextComponent::text(", "));
                }
            }

            if value.len() > 128 {
                content = content.add_child(folded);
            }

            content = content.add_child(TextComponent::text("]"));
            Ok(content)
        }
        NbtTag::LongArray(value) => {
            let long_array_format = TextComponent::text("L").color_named(NamedColor::Red);
            let mut content = TextComponent::text("[")
                .add_child(long_array_format.clone())
                .add_child(TextComponent::text("; "));

            for (index, long) in value.iter().take(128).enumerate() {
                content = content
                    .add_child(TextComponent::text(format!("{long}")))
                    .add_child(long_array_format.clone());
                if index < value.len() - 1 {
                    content = content.add_child(TextComponent::text(", "));
                }
            }

            if value.len() > 128 {
                content = content.add_child(folded);
            }

            content = content.add_child(TextComponent::text("]"));
            Ok(content)
        }
    }
}

async fn display_data(
    storage: &dyn NBTStorage,
    target_name: TextComponent,
    sender: &CommandSender
) -> Result<i32, CommandError> {
    let mut nbt = NbtCompound::new();
    storage.write_nbt(&mut nbt).await;
    let tag = NbtTag::Compound(nbt);

    let result = get_i32_result(&tag)?;
    let display = snbt_colorful_display(&tag, 0)
        .map_err(|string| CommandError::CommandFailed(TextComponent::text(string)))?;
    sender.send_message(
        TextComponent::translate(
            "commands.data.entity.query",
            [target_name, display]
        )
    ).await;

    Ok(result)
}

fn get_i32_result(tag: &NbtTag) -> Result<i32, CommandError> {
    match tag {
        NbtTag::End => Err(
            CommandError::CommandFailed(
                TextComponent::translate("commands.data.get.unknown", [])
            )
        ),

        NbtTag::Byte(b) => Ok(*b as i32),
        NbtTag::Short(s) => Ok(*s as i32),
        NbtTag::Int(i) => Ok(*i),
        NbtTag::Long(l) => Ok(
            (*l).clamp(
                i32::MIN as i64,
                i32::MAX as i64
            ) as i32
        ),
        NbtTag::Float(f) => Ok({
            let i = *f as i32;
            if *f < i as f32 {
                i - 1
            } else {
                i
            }
        }),
        NbtTag::Double(d) => Ok({
            let i = *d as i32;
            if *d < i as f64 {
                i - 1
            } else {
                i
            }
        }),

        NbtTag::ByteArray(items) => Ok(items.len() as i32),
        NbtTag::IntArray(items) => Ok(items.len() as i32),
        NbtTag::LongArray(items) => Ok(items.len() as i32),

        NbtTag::String(string) => Ok(string.len() as i32),
        NbtTag::List(nbt_tags) => Ok(nbt_tags.len() as i32),
        NbtTag::Compound(nbt_compound) => Ok(nbt_compound.child_tags.len() as i32),
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        literal("get").then(
            literal("entity")
                .then(argument(ARG_ENTITY, EntityArgumentConsumer).execute(GetEntityDataExecutor)),
        ),
    )
}
