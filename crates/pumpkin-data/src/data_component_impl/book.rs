use crate::data_component_impl::{DataComponentImpl, default_impl};
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
            for _ in l {
                pages.push(String::new());
            }
        }
        Some(Self { pages })
    }
}
impl DataComponentImpl for WritableBookContentImpl {
    default_impl!(WritableBookContent);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WrittenBookContentImpl {
    pub pages: Vec<String>,
}
impl WrittenBookContentImpl {
    pub fn read_data(tag: &NbtTag) -> Option<Self> {
        let mut pages = Vec::new();
        if let NbtTag::Compound(c) = tag
            && let Some(NbtTag::List(l)) = c.get("pages")
        {
            for _ in l {
                pages.push(String::new());
            }
        }
        Some(Self { pages })
    }
}
impl DataComponentImpl for WrittenBookContentImpl {
    default_impl!(WrittenBookContent);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DebugStickStateImpl;
impl DataComponentImpl for DebugStickStateImpl {
    default_impl!(DebugStickState);
}
