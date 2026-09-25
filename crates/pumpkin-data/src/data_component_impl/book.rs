use crate::data_component_impl::DataComponentImpl;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;

fn read_filterable_text(tag: &NbtTag) -> Option<String> {
    match tag {
        NbtTag::String(text) => Some(text.to_string()),
        NbtTag::Compound(compound) => compound.get_string("raw").map(|text| text.to_string()),
        _ => None,
    }
}

fn write_filterable_text(text: &str) -> NbtTag {
    let mut compound = NbtCompound::new();
    compound.put_string("raw", text.to_string());
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
                if let Some(page) = read_filterable_text(item) {
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
        let pages_tags: Vec<NbtTag> = self
            .pages
            .iter()
            .map(|page| write_filterable_text(page))
            .collect();
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
            if let Some(title_tag) = c.get("title")
                && let Some(text) = read_filterable_text(title_tag)
            {
                title = text;
            }
            if let Some(s) = c.get_string("author") {
                author = s.to_string();
            }
            if let Some(NbtTag::List(l)) = c.get("pages") {
                for item in l {
                    if let Some(page) = read_filterable_text(item) {
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
        compound.put("title", write_filterable_text(&self.title));
        compound.put_string("author", self.author.clone());
        let pages_tags: Vec<NbtTag> = self
            .pages
            .iter()
            .map(|page| write_filterable_text(page))
            .collect();
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

    #[test]
    fn written_book_reads_filterable_and_legacy_text() {
        let mut title = NbtCompound::new();
        title.put_string("raw", "A Partner".to_string());

        let mut filterable_page = NbtCompound::new();
        filterable_page.put_string("raw", "Filterable page".to_string());

        let mut book = NbtCompound::new();
        book.put("title", NbtTag::Compound(title));
        book.put_string("author", "Dylan Collins".to_string());
        book.put(
            "pages",
            NbtTag::List(vec![
                NbtTag::String("Legacy page".to_string().into_boxed_str()),
                NbtTag::Compound(filterable_page),
            ]),
        );

        let parsed = WrittenBookContentImpl::read_data(&NbtTag::Compound(book)).unwrap();

        assert_eq!(parsed.title, "A Partner");
        assert_eq!(parsed.author, "Dylan Collins");
        assert_eq!(parsed.pages, vec!["Legacy page", "Filterable page"]);
    }

    #[test]
    fn book_writers_emit_filterable_text_compounds() {
        let writable = WritableBookContentImpl {
            pages: vec!["Writable page".to_string()],
        };
        let NbtTag::Compound(writable_nbt) = writable.write_data() else {
            panic!("writable book data should be a compound");
        };
        let Some(NbtTag::List(writable_pages)) = writable_nbt.get("pages") else {
            panic!("writable book pages should be a list");
        };
        let NbtTag::Compound(writable_page) = &writable_pages[0] else {
            panic!("writable book page should be filterable text");
        };
        assert_eq!(writable_page.get_string("raw"), Some("Writable page"));

        let written = WrittenBookContentImpl {
            title: "Title".to_string(),
            author: "Author".to_string(),
            pages: vec!["Written page".to_string()],
        };
        let NbtTag::Compound(written_nbt) = written.write_data() else {
            panic!("written book data should be a compound");
        };
        let Some(NbtTag::Compound(title)) = written_nbt.get("title") else {
            panic!("written book title should be filterable text");
        };
        assert_eq!(title.get_string("raw"), Some("Title"));

        let Some(NbtTag::List(written_pages)) = written_nbt.get("pages") else {
            panic!("written book pages should be a list");
        };
        let NbtTag::Compound(written_page) = &written_pages[0] else {
            panic!("written book page should be filterable text");
        };
        assert_eq!(written_page.get_string("raw"), Some("Written page"));
    }
}
