use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::text::{TextComponent, color::NamedColor};

#[allow(clippy::too_many_lines)]
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

#[allow(clippy::too_many_lines)]
pub fn snbt_display(tag: &NbtTag, depth: usize) -> Result<TextComponent, String> {
    let folded = TextComponent::text("<...>");
    match tag {
        NbtTag::End => Err("Unexpected end tag".into()),
        NbtTag::Byte(value) => Ok(TextComponent::text(format!("{value}b"))),
        NbtTag::Short(value) => Ok(TextComponent::text(format!("{value}s"))),
        NbtTag::Int(value) => Ok(TextComponent::text(format!("{value}"))),
        NbtTag::Long(value) => Ok(TextComponent::text(format!("{value}L"))),
        NbtTag::Float(value) => Ok(TextComponent::text(format!("{value}f"))),
        NbtTag::Double(value) => Ok(TextComponent::text(format!("{value}d"))),
        NbtTag::ByteArray(value) => {
            let mut content = TextComponent::text("[B; ");

            for (index, byte) in value.iter().take(128).enumerate() {
                content = content.add_child(TextComponent::text(format!("{byte}B")));
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

            Ok(TextComponent::text(format!("\"{escaped_value}\"")))
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
                    let item_display = snbt_display(item, depth + 1)
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
                    let item_display = snbt_display(item, depth + 1)
                        .map_err(|string| format!("Error displaying item.{key}: {string}"))?;
                    content = content
                        .add_child(TextComponent::text(format!("{}: ", key.clone())))
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
            let mut content = TextComponent::text("[I; ");

            for (index, int) in value.iter().take(128).enumerate() {
                content = content.add_child(TextComponent::text(format!("{int}")));
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
            let mut content = TextComponent::text("[L; ");

            for (index, long) in value.iter().take(128).enumerate() {
                content = content.add_child(TextComponent::text(format!("{long}L")));
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
