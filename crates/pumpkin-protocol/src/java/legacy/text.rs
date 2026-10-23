//! Text components before 26.3. Core writes the current NBT, like `ViaBackwards` component
//! rewriters

use std::fmt::Write as _;

use pumpkin_nbt::{compound::NbtCompound, serializer::NbtWriteHelperJava, tag::NbtTag};
use pumpkin_util::{
    text::{TextComponent, TextComponentBase, color::Color},
    translation::Locale,
    version::JavaMinecraftVersion,
};

/// NBT on the wire since 1.20.3
#[must_use]
pub fn encode(component: &TextComponent, version: &JavaMinecraftVersion) -> Box<[u8]> {
    let mut bytes = Vec::new();
    let _ = to_nbt(component, version).serialize(&mut NbtWriteHelperJava::new(&mut bytes));
    bytes.into_boxed_slice()
}

/// JSON before 1.20.3: also the sign text layout up to 1.21.4
#[must_use]
pub fn to_json(component: &TextComponent, version: &JavaMinecraftVersion) -> String {
    tag_to_json(&to_nbt(component, version)).to_string()
}

#[must_use]
pub fn to_nbt(component: &TextComponent, version: &JavaMinecraftVersion) -> NbtTag {
    let mut tag = component.to_nbt_tag();
    downgrade(&mut tag, *version);
    tag
}

/// `§` codes, RGB colours before 1.16 become the nearest named colour.
#[must_use]
pub fn to_legacy_string(
    component: &TextComponent,
    version: &JavaMinecraftVersion,
    locale: Locale,
) -> String {
    if *version >= JavaMinecraftVersion::V_1_16 {
        return component.to_legacy_string(locale);
    }
    let mut base = component.0.clone();
    name_rgb_colors(&mut base);
    TextComponent(base).to_legacy_string(locale)
}

fn name_rgb_colors(base: &mut TextComponentBase) {
    base.style.color = base.style.color.map(|color| color.downsample());
    base.extra.iter_mut().for_each(name_rgb_colors);
}

fn downgrade(tag: &mut NbtTag, version: JavaMinecraftVersion) {
    match tag {
        NbtTag::Compound(compound) => downgrade_compound(compound, version),
        NbtTag::List(list) => list.iter_mut().for_each(|tag| downgrade(tag, version)),
        _ => {}
    }
}

fn downgrade_compound(tag: &mut NbtCompound, version: JavaMinecraftVersion) {
    for key in ["extra", "with", "fallback"] {
        if let Some(child) = tag.child_tags.get_mut(key) {
            downgrade(child, version);
        }
    }
    for key in ["hover_event", "hoverEvent"] {
        if let Some(NbtTag::Compound(hover)) = tag.child_tags.get_mut(key) {
            for inner in ["value", "name", "contents"] {
                if let Some(child) = hover.child_tags.get_mut(inner) {
                    downgrade(child, version);
                }
            }
        }
    }

    // 1.21.9 object contents (ViaBackwards 1.21.9 -> 1.21.7)
    if version < JavaMinecraftVersion::V_1_21_9 {
        object_to_text(tag);
    }
    // Snake case events, flat hover fields (ViaBackwards 1.21.5 -> 1.21.4)
    if version < JavaMinecraftVersion::V_1_21_5 {
        if let Some(NbtTag::Compound(mut click)) = tag.child_tags.remove("click_event") {
            click_to_1_21_4(&mut click);
            tag.put_compound("clickEvent", click);
        }
        if let Some(NbtTag::Compound(mut hover)) = tag.child_tags.remove("hover_event") {
            hover_to_1_21_4(&mut hover);
            tag.put_compound("hoverEvent", hover);
        }
    }
    if version < JavaMinecraftVersion::V_1_21_4 {
        tag.child_tags.remove("shadow_color");
    }
    // Hex colours, hover contents and fonts (ViaBackwards 1.16 -> 1.15.2)
    if version < JavaMinecraftVersion::V_1_16 {
        if let Some(NbtTag::String(color)) = tag.child_tags.get_mut("color")
            && let Some(Color::Rgb(rgb)) = color.strip_prefix('#').and_then(Color::from_hex_str)
        {
            *color = rgb.to_nearest_named().name().into();
        }
        tag.child_tags.remove("font");
        if let Some(NbtTag::Compound(hover)) = tag.child_tags.get_mut("hoverEvent") {
            hover_to_1_15(hover, version);
        }
    }
    // copy_to_clipboard was added in 1.15
    if version < JavaMinecraftVersion::V_1_15
        && let Some(NbtTag::Compound(click)) = tag.child_tags.get_mut("clickEvent")
        && click.get_string("action") == Some("copy_to_clipboard")
    {
        click.put_string("action", "suggest_command".to_string());
    }
}

/// Keeps the text vanilla shows when the object can't be drawn.
fn object_to_text(tag: &mut NbtCompound) {
    let is_object = tag.get_string("type") == Some("object")
        || ["player", "sprite", "atlas", "object"]
            .iter()
            .any(|key| tag.has(key));
    if !is_object {
        return;
    }
    let default = tag
        .get_compound("player")
        .map_or_else(String::new, |player| {
            player.get_string("name").map_or_else(
                || "[unknown player head]".to_string(),
                |name| format!("[{name} head]"),
            )
        });
    let fallback = tag.child_tags.remove("fallback");
    for key in ["type", "object", "player", "hat", "sprite", "atlas"] {
        tag.child_tags.remove(key);
    }
    match fallback {
        Some(fallback) => {
            tag.put_string("text", String::new());
            let mut extra = vec![fallback];
            if let Some(NbtTag::List(rest)) = tag.child_tags.remove("extra") {
                extra.extend(rest);
            }
            tag.put_list("extra", extra);
        }
        None => tag.put_string("text", default),
    }
}

fn click_to_1_21_4(click: &mut NbtCompound) {
    let value = match click.get_string("action") {
        Some("open_url") => click.child_tags.remove("url"),
        Some("open_file") => click.child_tags.remove("path"),
        Some("suggest_command") => click.child_tags.remove("command"),
        // Commands needed their slash until 1.21.5
        Some("run_command") => match click.child_tags.remove("command") {
            Some(NbtTag::String(command)) if !command.starts_with('/') => {
                Some(NbtTag::String(format!("/{command}").into()))
            }
            command => command,
        },
        Some("change_page") => click.child_tags.remove("page").and_then(|page| match page {
            NbtTag::Int(page) => Some(NbtTag::String(page.to_string().into())),
            _ => None,
        }),
        _ => None,
    };
    if let Some(value) = value {
        click.put("value", value);
    }
}

fn hover_to_1_21_4(hover: &mut NbtCompound) {
    match hover.get_string("action") {
        Some("show_text") => {
            if let Some(value) = hover.child_tags.remove("value") {
                hover.put("contents", value);
            }
        }
        Some("show_item") => {
            let mut contents = NbtCompound::new();
            for key in ["id", "count", "components"] {
                if let Some(value) = hover.child_tags.remove(key) {
                    contents.put(key, value);
                }
            }
            hover.put_compound("contents", contents);
        }
        Some("show_entity") => {
            let mut contents = NbtCompound::new();
            for (from, to) in [("id", "type"), ("uuid", "id"), ("name", "name")] {
                if let Some(value) = hover.child_tags.remove(from) {
                    contents.put(to, value);
                }
            }
            hover.put_compound("contents", contents);
        }
        _ => {}
    }
}

/// Before 1.16 items and entities were SNBT in a text `value`.
fn hover_to_1_15(hover: &mut NbtCompound, version: JavaMinecraftVersion) {
    let Some(contents) = hover.child_tags.remove("contents") else {
        return;
    };
    let value = match (hover.get_string("action"), contents) {
        (Some("show_item"), NbtTag::Compound(item)) => {
            let id = item.get_string("id").unwrap_or("minecraft:air");
            let count = item.get_int("count").unwrap_or(1);
            NbtTag::String(format!("{{id:{},Count:{count}b}}", snbt_string(id)).into())
        }
        (Some("show_entity"), NbtTag::Compound(entity)) => {
            let mut snbt = String::from("{");
            if let Some(name) = entity.get("name") {
                // A JSON component since 1.13, plain text before
                let name = if version >= JavaMinecraftVersion::V_1_13 {
                    tag_to_json(name).to_string()
                } else {
                    TextComponent::from_nbt(name).get_text()
                };
                let _ = write!(snbt, "name:{},", snbt_string(&name));
            }
            let kind = entity.get_string("type").unwrap_or_default();
            let id = entity.get_string("id").unwrap_or_default();
            let _ = write!(snbt, "type:{},id:{}}}", snbt_string(kind), snbt_string(id));
            NbtTag::String(snbt.into())
        }
        (_, text) => text,
    };
    hover.put("value", value);
}

fn snbt_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

const BOOLEAN_KEYS: [&str; 7] = [
    "bold",
    "italic",
    "underlined",
    "strikethrough",
    "obfuscated",
    "interpret",
    "hat",
];

/// Component NBT to JSON: style flags are booleans and `{"": x}` list wrappers unwrap.
fn tag_to_json(tag: &NbtTag) -> serde_json::Value {
    match tag {
        NbtTag::Compound(compound) => {
            if compound.child_tags.len() == 1
                && let Some(inner) = compound.get("")
            {
                return tag_to_json(inner);
            }
            compound
                .child_tags
                .iter()
                .map(|(key, value)| {
                    let json = match value {
                        NbtTag::Byte(flag) if BOOLEAN_KEYS.contains(&&**key) => {
                            serde_json::Value::Bool(*flag != 0)
                        }
                        other => tag_to_json(other),
                    };
                    (key.to_string(), json)
                })
                .collect::<serde_json::Map<_, _>>()
                .into()
        }
        NbtTag::List(list) => list.iter().map(tag_to_json).collect(),
        NbtTag::String(text) => serde_json::Value::String(text.to_string()),
        NbtTag::Byte(v) => (*v).into(),
        NbtTag::Short(v) => (*v).into(),
        NbtTag::Int(v) => (*v).into(),
        NbtTag::Long(v) => (*v).into(),
        NbtTag::Float(v) => (*v).into(),
        NbtTag::Double(v) => (*v).into(),
        NbtTag::ByteArray(v) => v.iter().copied().collect(),
        NbtTag::IntArray(v) => v.iter().copied().collect(),
        NbtTag::LongArray(v) => v.iter().copied().collect(),
        NbtTag::End => serde_json::Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use pumpkin_util::text::{click::ClickEvent, color::RGBColor, hover::HoverEvent};

    use super::*;

    fn json(component: &TextComponent, version: JavaMinecraftVersion) -> serde_json::Value {
        serde_json::from_str(&to_json(component, &version)).unwrap()
    }

    #[test]
    fn click_event_uses_value_before_1_21_5() {
        let link = TextComponent::text("link").click_event(ClickEvent::OpenUrl {
            url: Cow::Borrowed("https://example.com"),
        });
        let tag = to_nbt(&link, &JavaMinecraftVersion::V_1_21_4);
        let NbtTag::Compound(tag) = tag else { panic!() };
        let click = tag.get_compound("clickEvent").unwrap();
        assert_eq!(click.get_string("value"), Some("https://example.com"));
        assert!(click.get_string("url").is_none());

        let run = TextComponent::text("go").click_event(ClickEvent::RunCommand {
            command: Cow::Borrowed("spawn"),
        });
        assert_eq!(
            json(&run, JavaMinecraftVersion::V_1_20_2)["clickEvent"]["value"],
            "/spawn"
        );

        let page = TextComponent::text("next").click_event(ClickEvent::ChangePage { page: 3 });
        assert_eq!(
            json(&page, JavaMinecraftVersion::V_1_20_2)["clickEvent"]["value"],
            "3"
        );
    }

    #[test]
    fn hover_uses_contents_from_1_16_and_snbt_before() {
        let entity = TextComponent::text("pig").hover_event(HoverEvent::ShowEntity {
            id: Cow::Borrowed("minecraft:pig"),
            uuid: Cow::Borrowed("00000000-0000-0000-0000-000000000001"),
            name: Some(vec![TextComponent::text("Bob \"B\"").0]),
        });
        let hover = &json(&entity, JavaMinecraftVersion::V_1_16)["hoverEvent"];
        assert_eq!(hover["contents"]["type"], "minecraft:pig");
        assert_eq!(hover["contents"]["name"], "Bob \"B\"");

        let hover = &json(&entity, JavaMinecraftVersion::V_1_12_2)["hoverEvent"];
        assert_eq!(
            hover["value"],
            r#"{name:"Bob \"B\"",type:"minecraft:pig",id:"00000000-0000-0000-0000-000000000001"}"#
        );
    }

    #[test]
    fn rgb_becomes_nearest_named_before_1_16() {
        let text = TextComponent::text("hi").color_rgb(RGBColor::new(0xff, 0x50, 0x50));
        assert_eq!(json(&text, JavaMinecraftVersion::V_1_15_2)["color"], "red");
        assert_eq!(
            json(&text, JavaMinecraftVersion::V_1_16)["color"],
            "#FF5050"
        );
        assert_eq!(
            to_legacy_string(&text, &JavaMinecraftVersion::V_1_12_2, Locale::EnUs),
            "§chi"
        );
    }

    #[test]
    fn plain_text_stays_a_string_and_flags_are_booleans() {
        let plain = TextComponent::text("hello");
        assert_eq!(json(&plain, JavaMinecraftVersion::V_1_8), "hello");
        let bold = TextComponent::text("hey").bold();
        assert_eq!(json(&bold, JavaMinecraftVersion::V_1_8)["bold"], true);
    }
}
