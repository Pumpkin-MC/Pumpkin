pub enum Sign {
    Plus,
    Minus,
}

pub enum SignedPrefix {
    None,
    Unsigned,
    Signed,
}

pub enum TypeSuffix {
    None,
    Byte,
    Short,
    Int,
    Long,
}

pub struct IntegerSuffix(pub SignedPrefix, pub TypeSuffix);

impl IntegerSuffix {
    pub const EMPTY: Self = Self(SignedPrefix::None, TypeSuffix::None);
}
