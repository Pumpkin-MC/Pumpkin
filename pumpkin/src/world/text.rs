use async_trait::async_trait;
use colored::Colorize;
use pumpkin_nbt::{compound::NbtCompound, path::get_tag_by_path, tag::NbtTag};
use pumpkin_util::text::{
    TextComponent, TextComponentBase,
    color::NamedColor,
    content::{NbtSource, NbtValue, TextContent},
    hover::HoverEvent,
    style::Style,
    translation::{Locale, get_translation, reorder_substitutions},
};
use std::borrow::Cow;
use uuid::Uuid;

use crate::{
    command::{CommandSender, args::entities::TargetSelector},
    entity::player::Player,
};
/// TODO List
/// - Add server locale support
/// - Use translations in the logs
/// - - `Client_kick_reason` messages
/// - Document all about `TextComponents`
/// - Add support for translations on commands descriptions
/// - Open a public translation system, maybe a Crowdin like Minecraft?
/// - Integrate custom translations with the plugins API
/// - Solve command discrepances (unquoted keys, type value)

#[async_trait]
pub trait ComponentResolution: Send + Sync {
    async fn resolve(self, player: Option<&Player>) -> Self;
}
#[async_trait]
pub trait TextResolution: Send + Sync {
    async fn to_string(self, player: Option<&Player>, stylized: bool) -> String;
    async fn to_send(self, player: &Player) -> Self;
}
#[async_trait]
impl TextResolution for TextComponent {
    async fn to_string(self, player: Option<&Player>, stylized: bool) -> String {
        self.0.to_string(player, stylized).await
    }
    async fn to_send(self, player: &Player) -> Self {
        Self(self.0.to_send(player).await)
    }
}
#[async_trait]
impl TextResolution for TextComponentBase {
    async fn to_string(self, player: Option<&Player>, stylized: bool) -> String {
        let resolved = self.resolve(player).await;
        let mut text = match resolved.content {
            TextContent::Text { text } => text.into_owned(),
            TextContent::Translate {
                translate,
                fallback,
                with,
            } => {
                let fallback = fallback
                    .unwrap_or_else(|| format!("minecraft:{translate}").into())
                    .to_string();
                translated(
                    format!("minecraft:{translate}"),
                    player,
                    fallback,
                    with,
                    stylized,
                )
                .await
            }
            TextContent::Keybind { keybind } => keybind.into_owned(),
            TextContent::Custom { key, with, .. } => {
                let fallback = key.clone();
                translated(key, player, fallback, with, stylized).await
            }
            _ => String::new(),
        };
        for child in resolved.extra {
            text += &child.to_string(player, stylized).await;
        }
        if !stylized {
            return text;
        }

        let style = resolved.style;
        let color = style.color;
        if let Some(color) = color {
            text = color.console_color(&text).to_string();
        }
        if style.bold.is_some() {
            text = text.bold().to_string();
        }
        if style.italic.is_some() {
            text = text.italic().to_string();
        }
        if style.underlined.is_some() {
            text = text.underline().to_string();
        }
        if style.strikethrough.is_some() {
            text = text.strikethrough().to_string();
        }
        text
    }
    async fn to_send(self, player: &Player) -> Self {
        let resolved = self.resolve(Some(player)).await;
        let locale = player.locale().await;
        // Divide the translation into slices and inserts the substitutions
        let mut component = match resolved.content {
            TextContent::Custom { key, with, .. } => {
                let fallback = key.clone();
                let translation = get_translation(key, locale, fallback);
                if with.is_empty() || !translation.contains('%') {
                    Self {
                        content: TextContent::Text {
                            text: Cow::Owned(translation),
                        },
                        style: resolved.style,
                        extra: resolved.extra,
                    }
                } else {
                    let mut translation_parent: String = String::new();
                    let mut translation_slices = vec![];

                    let (substitutions, ranges) = reorder_substitutions(&translation, with);
                    let mut idx = 0;
                    for substitute in substitutions {
                        let range = ranges[idx];
                        if idx == 0 {
                            translation_parent = translation[..range.start].to_string();
                        }
                        translation_slices.push(substitute);
                        if range.end >= translation.len() - 1 {
                            continue;
                        }

                        translation_slices.push(Self {
                            content: TextContent::Text {
                                text: if idx == ranges.len() - 1 {
                                    // Last substitution, append the rest of the translation
                                    Cow::Owned(translation[range.end + 1..].to_string())
                                } else {
                                    Cow::Owned(
                                        translation[range.end + 1..ranges[idx + 1].start]
                                            .to_string(),
                                    )
                                },
                            },
                            style: Style::default(),
                            extra: vec![],
                        });
                        idx += 1;
                    }
                    for i in resolved.extra {
                        translation_slices.push(i);
                    }
                    Self {
                        content: TextContent::Text {
                            text: translation_parent.into(),
                        },
                        style: resolved.style,
                        extra: translation_slices,
                    }
                }
            }
            _ => resolved, // If not a translation, return as is
        };
        // Ensure that the extra components are translated
        let mut extra = vec![];
        for extra_component in component.extra {
            extra.push(extra_component.to_send(player).await);
        }
        component.extra = extra;
        // If the hover event is present, it will also be processed
        match component.style.hover_event {
            None => return component,
            Some(hover) => {
                component.style.hover_event = match hover {
                    HoverEvent::ShowText { value } => {
                        let mut hover_components = vec![];
                        for hover_component in value {
                            hover_components.push(hover_component.to_send(player).await);
                        }
                        Some(HoverEvent::ShowText {
                            value: hover_components,
                        })
                    }
                    HoverEvent::ShowEntity { name, id, uuid } => match name {
                        None => Some(HoverEvent::ShowEntity {
                            name: None,
                            id,
                            uuid,
                        }),
                        Some(name) => {
                            let mut translated_names = Vec::new();
                            for part in name {
                                translated_names.push(part.to_send(player).await);
                            }
                            Some(HoverEvent::ShowEntity {
                                name: Some(translated_names),
                                id,
                                uuid,
                            })
                        }
                    },
                    HoverEvent::ShowItem { id, count } => Some(HoverEvent::ShowItem { id, count }),
                };
            }
        }

        component
    }
}

#[async_trait]
impl ComponentResolution for TextComponentBase {
    async fn resolve(self, _player: Option<&Player>) -> Self {
        match self.content {
            TextContent::Scoreboard { .. } => resolve_scoreboard(),
            TextContent::EntityNames {
                selector,
                separator,
                sender,
            } => resolve_entities(selector, separator, sender, self.style, self.extra).await,
            TextContent::Nbt { value } => resolve_nbt(value, self.style, self.extra).await,
            _ => self,
        }
    }
}
fn resolve_scoreboard() -> TextComponentBase {
    TextComponent::custom("pumpkin", "text.error.no_scoreboard", &[])
        .color_named(NamedColor::Red)
        .0
}
async fn resolve_entities(
    selector: Cow<'static, str>,
    separator: Option<Vec<TextComponentBase>>,
    sender: Option<Uuid>,
    style: Style,
    extra: Vec<TextComponentBase>,
) -> TextComponentBase {
    match crate::server::get_server() {
        Some(server) => {
            let Ok(selector) = selector.parse::<TargetSelector>() else {
                return TextComponentBase::default();
            };
            let sender = match sender {
                None => None,
                Some(sender) => server
                    .get_player_by_uuid(sender)
                    .await
                    .map(CommandSender::Player),
            };
            let entities = server.select_entities(&selector, sender.as_ref()).await;
            let separator = separator.map_or_else(
                || TextComponent::text(", ").color_named(NamedColor::Gray).0,
                |separator| {
                    let mut sep = TextComponentBase::default();
                    let mut first = true;
                    for part in separator {
                        if first {
                            sep = part;
                            first = false;
                            continue;
                        }
                        sep.extra.push(part);
                    }
                    sep
                },
            );
            let mut parent = TextComponentBase::default();
            let mut names = vec![];
            for (i, entity) in entities.iter().enumerate() {
                if i == 0 {
                    names.push(entity.get_display_name().await.0);
                    continue;
                }
                names.push(separator.clone());
                names.push(entity.get_display_name().await.0);
            }
            let mut extra = extra;
            names.append(&mut extra);
            parent.style = style;
            parent.extra = names;
            parent
        }
        None => {
            TextComponent::custom("pumpkin", "text.error.no_data", &[])
                .color_named(NamedColor::Red)
                .0
        }
    }
}
async fn resolve_nbt(
    value: NbtValue,
    style: Style,
    extra: Vec<TextComponentBase>,
) -> TextComponentBase {
    match crate::server::get_server() {
        Some(server) => {
            let source = match value.source {
                Some(source) => source,
                None => {
                    if value.entity.is_some() {
                        NbtSource::Entity
                    } else if value.block.is_some() {
                        NbtSource::Block
                    } else if value.storage.is_some() {
                        NbtSource::Storage
                    } else {
                        return TextComponentBase::default();
                    }
                }
            };
            // I wonder for what is value.interpret
            match source {
                NbtSource::Entity => {
                    if let Some(entity) = value.entity {
                        let Ok(selector) = entity.parse::<TargetSelector>() else {
                            return TextComponentBase::default();
                        };
                        let sender = match value.sender {
                            None => None,
                            Some(sender) => server
                                .get_player_by_uuid(sender)
                                .await
                                .map(CommandSender::Player),
                        };
                        let entities = server.select_entities(&selector, sender.as_ref()).await;
                        if entities.is_empty() {
                            return TextComponentBase::default();
                        }
                        let separator = value.separator.map_or_else(
                            || TextComponent::text(", ").0,
                            |separator| {
                                let mut sep = TextComponentBase::default();
                                let mut first = true;
                                for part in separator {
                                    if first {
                                        sep = part;
                                        first = false;
                                        continue;
                                    }
                                    sep.extra.push(part);
                                }
                                sep
                            },
                        );
                        let mut text = TextComponentBase::default();
                        let mut components = vec![];
                        for (i, entity) in entities.iter().enumerate() {
                            if i != 0 {
                                components.push(separator.clone());
                            }
                            let mut nbt = NbtCompound::new();
                            entity.write_nbt(&mut nbt).await;
                            let tags = get_tag_by_path(&NbtTag::Compound(nbt), &value.nbt);
                            for (j, tag) in tags.iter().enumerate() {
                                if j != 0 {
                                    components.push(separator.clone());
                                }
                                if let Ok(display) = snbt_display(tag, 0) {
                                    components.push(display.0);
                                }
                            }
                        }
                        let mut extra = extra;
                        components.append(&mut extra);
                        text.style = style;
                        text.extra = components;
                        text
                    } else {
                        TextComponentBase::default()
                    }
                }
                NbtSource::Block => {
                    TextComponent::custom("pumpkin", "text.error.no_nbt.block", &[])
                        .color_named(NamedColor::Red)
                        .0
                }
                NbtSource::Storage => {
                    TextComponent::custom("pumpkin", "text.error.no_nbt.storage", &[])
                        .color_named(NamedColor::Red)
                        .0
                }
            }
        }
        None => {
            TextComponent::custom("pumpkin", "text.error.no_data", &[])
                .color_named(NamedColor::Red)
                .0
        }
    }
}

async fn translated<P: Into<Cow<'static, str>>>(
    namespaced_key: P,
    player: Option<&Player>,
    fallback: P,
    with: Vec<TextComponentBase>,
    stylized: bool,
) -> String {
    let locale = match player {
        Some(player) => player.locale().await,
        None => Locale::EnUs,
    };
    let mut translation = get_translation(namespaced_key.into(), locale, fallback.into());
    if with.is_empty() || !translation.contains('%') {
        return translation;
    }

    let (substitutions, indices) = reorder_substitutions(&translation, with);
    let mut translated_substitutions = Vec::new();
    for substitution in substitutions {
        translated_substitutions.push(substitution.to_string(player, stylized).await);
    }
    let mut displacement = 0i32;
    for (idx, &range) in indices.iter().enumerate() {
        let sub_idx = idx.clamp(0, translated_substitutions.len() - 1);
        let substitution = &translated_substitutions[sub_idx];
        translation.replace_range(
            range.start + displacement as usize..=range.end + displacement as usize,
            substitution,
        );
        displacement += substitution.len() as i32 - range.len() as i32;
    }
    translation
}

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
