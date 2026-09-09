use crate::data_component_impl::DataComponentImpl;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;

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
                if let NbtTag::String(s) = item {
                    pages.push(s.to_string());
                } else if let NbtTag::Compound(comp) = item {
                    if let Some(s) = comp.get_string("raw") {
                        pages.push(s.to_string());
                    }
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
            .map(|p| NbtTag::String(p.clone().into_boxed_str()))
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
            if let Some(s) = c.get_string("title") {
                title = s.to_string();
            }
            if let Some(s) = c.get_string("author") {
                author = s.to_string();
            }
            if let Some(NbtTag::List(l)) = c.get("pages") {
                for item in l {
                    if let NbtTag::String(s) = item {
                        pages.push(s.to_string());
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
        compound.put_string("title", self.title.clone());
        compound.put_string("author", self.author.clone());
        let pages_tags: Vec<NbtTag> = self
            .pages
            .iter()
            .map(|p| NbtTag::String(p.clone().into_boxed_str()))
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
