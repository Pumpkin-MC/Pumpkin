use crate::data_component_impl::DataComponentImpl;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;

/// Reads a filterable text, which vanilla stores either as a plain string or as
/// a compound with a `raw` field, e.g. `{raw: "A Partner"}`.
fn read_text(tag: &NbtTag) -> Option<String> {
    match tag {
        NbtTag::String(value) => Some(value.to_string()),
        NbtTag::Compound(compound) => compound.get_string("raw").map(ToString::to_string),
        _ => None,
    }
}

fn text_tag(value: &str) -> NbtTag {
    let mut compound = NbtCompound::new();
    compound.put_string("raw", value.to_string());
    NbtTag::Compound(compound)
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WritableBookContentImpl {
    pub pages: Vec<String>,
}
impl WritableBookContentImpl {
    pub fn read_data(tag: &NbtTag) -> Option<Self> {
        let mut pages = Vec::new();
        if let NbtTag::Compound(c) = tag
            && let Some(NbtTag::List(l)) = c.get("pages")
        {
            for item in l {
                if let Some(page) = read_text(item) {
                    pages.push(page);
                }
            }
        }
        Some(Self { pages })
    }
}
impl DataComponentImpl for WritableBookContentImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        let pages_tags: Vec<NbtTag> = self.pages.iter().map(|p| text_tag(p)).collect();
        compound.put("pages", NbtTag::List(pages_tags));
        NbtTag::Compound(compound)
    }
    default_impl!(WritableBookContent);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WrittenBookContentImpl {
    pub title: String,
    pub author: String,
    pub pages: Vec<String>,
}
impl WrittenBookContentImpl {
    pub fn read_data(tag: &NbtTag) -> Option<Self> {
        let mut pages = Vec::new();
        let mut title = String::new();
        let mut author = String::new();
        if let NbtTag::Compound(c) = tag {
            if let Some(value) = c.get("title").and_then(read_text) {
                title = value;
            }
            if let Some(s) = c.get_string("author") {
                author = s.to_string();
            }
            if let Some(NbtTag::List(l)) = c.get("pages") {
                for item in l {
                    if let Some(page) = read_text(item) {
                        pages.push(page);
                    }
                }
            }
        }
        Some(Self {
            title,
            author,
            pages,
        })
    }
}
impl DataComponentImpl for WrittenBookContentImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put("title", text_tag(&self.title));
        compound.put_string("author", self.author.clone());
        let pages_tags: Vec<NbtTag> = self.pages.iter().map(|p| text_tag(p)).collect();
        compound.put("pages", NbtTag::List(pages_tags));
        NbtTag::Compound(compound)
    }
    default_impl!(WrittenBookContent);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DebugStickStateImpl;
impl DebugStickStateImpl {
    pub const fn read_data(_data: &NbtTag) -> Option<Self> {
        Some(Self)
    }
}
impl DataComponentImpl for DebugStickStateImpl {
    default_impl!(DebugStickState);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compound(values: &[(&str, NbtTag)]) -> NbtTag {
        let mut compound = NbtCompound::new();
        for (key, value) in values {
            compound.put(key, value.clone());
        }
        NbtTag::Compound(compound)
    }

    fn raw_list(values: &[&str]) -> NbtTag {
        NbtTag::List(values.iter().map(|v| text_tag(v)).collect())
    }

    #[test]
    fn written_book_reads_raw_pages_and_title() {
        let tag = compound(&[
            ("title", text_tag("A Partner")),
            ("author", NbtTag::String("Dylan Collins".into())),
            ("pages", raw_list(&["first page", "second page"])),
        ]);
        let content = WrittenBookContentImpl::read_data(&tag).unwrap();
        assert_eq!(content.title, "A Partner");
        assert_eq!(content.author, "Dylan Collins");
        assert_eq!(content.pages, ["first page", "second page"]);
    }

    #[test]
    fn written_book_still_reads_plain_strings() {
        let tag = compound(&[
            ("title", NbtTag::String("Plain".into())),
            (
                "pages",
                NbtTag::List(vec![NbtTag::String("plain page".into())]),
            ),
        ]);
        let content = WrittenBookContentImpl::read_data(&tag).unwrap();
        assert_eq!(content.title, "Plain");
        assert_eq!(content.pages, ["plain page"]);
    }

    #[test]
    fn writable_book_reads_raw_pages() {
        let tag = compound(&[("pages", raw_list(&["hello"]))]);
        let content = WritableBookContentImpl::read_data(&tag).unwrap();
        assert_eq!(content.pages, ["hello"]);
    }

    #[test]
    fn written_book_round_trip_keeps_pages() {
        let content = WrittenBookContentImpl {
            title: "Title".to_string(),
            author: "Author".to_string(),
            pages: vec!["one".to_string(), "two".to_string()],
        };
        let tag = content.write_data();
        let written_compound = match &tag {
            NbtTag::Compound(c) => c,
            _ => panic!("expected a compound"),
        };
        assert!(matches!(
            written_compound.get("title"),
            Some(NbtTag::Compound(_))
        ));
        let read_back = WrittenBookContentImpl::read_data(&tag).unwrap();
        assert_eq!(read_back, content);
    }
}
