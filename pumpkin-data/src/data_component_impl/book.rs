use crate::data_component_impl::{DataComponentImpl, default_impl};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;

/// A value together with its optional server-side filtered variant, as used by
/// book contents in vanilla (`raw` plus an optional `filtered` override).
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FilterableString {
    pub raw: String,
    pub filtered: Option<String>,
}
impl FilterableString {
    #[must_use]
    pub const fn pass_through(raw: String) -> Self {
        Self {
            raw,
            filtered: None,
        }
    }
    /// Reads either the full `{raw, filtered?}` form or a bare string.
    fn read(tag: &NbtTag) -> Option<Self> {
        match tag {
            NbtTag::String(raw) => Some(Self::pass_through(raw.to_string())),
            NbtTag::Compound(compound) => Some(Self {
                raw: compound.get_string("raw")?.to_string(),
                filtered: compound.get_string("filtered").map(str::to_string),
            }),
            _ => None,
        }
    }
    /// Writes the canonical `{raw, filtered?}` compound.
    fn write(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_string("raw", self.raw.clone());
        if let Some(filtered) = &self.filtered {
            compound.put_string("filtered", filtered.clone());
        }
        NbtTag::Compound(compound)
    }
}

/// A written-book page: a text component kept as its raw NBT (a string or a
/// compound) plus its optional filtered variant. The component payload is
/// preserved verbatim so nothing is lost across a save/load cycle.
#[derive(Clone, Debug, PartialEq)]
pub struct FilterablePage {
    pub raw: NbtTag,
    pub filtered: Option<NbtTag>,
}
impl FilterablePage {
    /// Reads the `{raw, filtered?}` form, otherwise treats the whole tag as the
    /// bare page component.
    fn read(tag: &NbtTag) -> Self {
        if let NbtTag::Compound(compound) = tag
            && let Some(raw) = compound.get("raw")
        {
            return Self {
                raw: raw.clone(),
                filtered: compound.get("filtered").cloned(),
            };
        }
        Self {
            raw: tag.clone(),
            filtered: None,
        }
    }
    /// Writes the canonical `{raw, filtered?}` compound.
    fn write(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put("raw", self.raw.clone());
        if let Some(filtered) = &self.filtered {
            compound.put("filtered", filtered.clone());
        }
        NbtTag::Compound(compound)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct WritableBookContentImpl {
    pub pages: Vec<FilterableString>,
}
impl WritableBookContentImpl {
    pub fn read_data(tag: &NbtTag) -> Option<Self> {
        let mut pages = Vec::new();
        if let NbtTag::Compound(compound) = tag
            && let Some(list) = compound.get_list("pages")
        {
            for page in list {
                if let Some(page) = FilterableString::read(page) {
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
        let pages: Vec<NbtTag> = self.pages.iter().map(FilterableString::write).collect();
        compound.put_list("pages", pages);
        NbtTag::Compound(compound)
    }
    default_impl!(WritableBookContent);
}

#[derive(Clone, Debug, PartialEq)]
pub struct WrittenBookContentImpl {
    pub title: FilterableString,
    pub author: String,
    pub generation: i32,
    pub pages: Vec<FilterablePage>,
    pub resolved: bool,
}
impl WrittenBookContentImpl {
    pub fn read_data(tag: &NbtTag) -> Option<Self> {
        let compound = tag.extract_compound()?;
        let title = compound
            .get("title")
            .and_then(FilterableString::read)
            .unwrap_or_else(|| FilterableString::pass_through(String::new()));
        let author = compound
            .get_string("author")
            .unwrap_or_default()
            .to_string();
        let generation = compound.get_int("generation").unwrap_or(0);
        let resolved = compound.get_bool("resolved").unwrap_or(false);
        let mut pages = Vec::new();
        if let Some(list) = compound.get_list("pages") {
            for page in list {
                pages.push(FilterablePage::read(page));
            }
        }
        Some(Self {
            title,
            author,
            generation,
            pages,
            resolved,
        })
    }
}
impl DataComponentImpl for WrittenBookContentImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put("title", self.title.write());
        compound.put_string("author", self.author.clone());
        compound.put_int("generation", self.generation);
        let pages: Vec<NbtTag> = self.pages.iter().map(FilterablePage::write).collect();
        compound.put_list("pages", pages);
        compound.put_bool("resolved", self.resolved);
        NbtTag::Compound(compound)
    }
    default_impl!(WrittenBookContent);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DebugStickStateImpl;
impl DataComponentImpl for DebugStickStateImpl {
    default_impl!(DebugStickState);
}
