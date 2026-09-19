use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;

use crate::version::JavaMinecraftVersion;

use super::TextComponent;

/// Rewrites sign's NBT data to JSON strings for 1.20–1.21.4.
pub fn remap_block_entity_sign_nbt(nbt: &mut NbtCompound, version: JavaMinecraftVersion) -> bool {
    if version >= JavaMinecraftVersion::V_1_21_5 {
        return false;
    }

    let mut changed = false;
    for key in ["front_text", "back_text"] {
        if let Some(NbtTag::Compound(face)) = nbt.child_tags.get_mut(key) {
            changed |= remap_sign_face(face, version);
        }
    }
    changed
}

/// Rewrites sign face's NBT data to JSON strings for 1.20–1.21.4.
fn remap_sign_face(face: &mut NbtCompound, version: JavaMinecraftVersion) -> bool {
    let mut changed = false;
    for key in ["messages", "filtered_messages"] {
        if let Some(NbtTag::List(lines)) = face.child_tags.get_mut(key) {
            for line in lines.iter_mut() {
                *line = line_to_legacy_json(line, version);
            }
            changed |= !lines.is_empty();
        }
    }
    changed
}

fn line_to_legacy_json(tag: &NbtTag, version: JavaMinecraftVersion) -> NbtTag {
    NbtTag::String(match tag {
        NbtTag::String(raw) => plain_to_json_string(raw),
        other => TextComponent::from_nbt(other)
            .to_json_for_version(&version)
            .into_boxed_str(),
    })
}

/// JSON string encoding (`Hello` -> `"Hello"`).
fn plain_to_json_string(text: &str) -> Box<str> {
    serde_json::to_string(text)
        .unwrap_or_else(|_| format!("{text:?}"))
        .into_boxed_str()
}

#[cfg(test)]
mod tests {
    use super::{plain_to_json_string, remap_block_entity_sign_nbt};
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_nbt::tag::NbtTag;

    use crate::version::JavaMinecraftVersion;

    fn sign_nbt_with_plain_line(line: &str) -> NbtCompound {
        let mut face = NbtCompound::new();
        face.put_list(
            "messages",
            vec![
                NbtTag::String(line.into()),
                NbtTag::String("".into()),
                NbtTag::String("".into()),
                NbtTag::String("".into()),
            ],
        );
        let mut nbt = NbtCompound::new();
        nbt.put_compound("front_text", face);
        nbt
    }

    fn first_message(nbt: &NbtCompound) -> &NbtTag {
        let face = nbt.get_compound("front_text").unwrap();
        match face.get("messages") {
            Some(NbtTag::List(list)) => &list[0],
            _ => panic!("expected messages list"),
        }
    }

    #[test]
    fn json_quotes_plain_and_escaped_lines() {
        assert_eq!(&*plain_to_json_string("Hello"), r#""Hello""#);
        assert_eq!(&*plain_to_json_string(""), r#""""#);
        assert_eq!(&*plain_to_json_string(r#"say "hi""#), r#""say \"hi\"""#);
    }

    #[test]
    fn json_wrap_on_legacy_clients_only() {
        for version in [
            JavaMinecraftVersion::V_1_21,
            JavaMinecraftVersion::V_1_21_2,
            JavaMinecraftVersion::V_1_21_4,
        ] {
            let mut nbt = sign_nbt_with_plain_line("Hello");
            assert!(remap_block_entity_sign_nbt(&mut nbt, version));
            assert_eq!(
                first_message(&nbt),
                &NbtTag::String(r#""Hello""#.to_string().into_boxed_str())
            );
        }

        for version in [JavaMinecraftVersion::V_1_21_5, JavaMinecraftVersion::V_26_2] {
            let mut nbt = sign_nbt_with_plain_line("Hello");
            assert!(!remap_block_entity_sign_nbt(&mut nbt, version));
            assert_eq!(
                first_message(&nbt),
                &NbtTag::String("Hello".to_string().into_boxed_str())
            );
        }
    }
}
