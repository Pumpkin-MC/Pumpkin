use syn::meta::ParseNestedMeta;
use syn::{Attribute, Error, Ident, Path};

macro_rules! add_attribute_branch {
    ($path:ident, $ident:literal, $var:ident) => {
        if $path.is_ident($ident) {
            return Some(Self::$var);
        }
    };
}

pub(crate) use add_attribute_branch;

/// An abstract parsed attribute.
pub trait ParsedAttribute: Sized {
    /// Tries to get an attribute from a `Path`.
    fn from_path(path: &Path) -> Option<Self>;

    /// Parses a list of [`Attribute`]s and uses a function taking a parsed attribute of this type.
    /// This function can then do something with the parsed attributes.
    ///
    /// All attributes will try to parse a `#[codec(...)]` attribute.
    fn parse_attributes(
        attributes: &[Attribute],
        mut meta_function: impl FnMut(Self, &ParseNestedMeta, &Ident) -> Result<(), Error>,
    ) -> Result<(), Error> {
        for attr in attributes {
            if attr.path().is_ident("codec") {
                attr.parse_nested_meta(|meta| {
                    let ident = meta.path.get_ident().expect("Ident should exist");
                    Self::from_path(&meta.path).map_or_else(
                        || Err(Error::new_spanned(ident, "Invalid attribute")),
                        |attribute| meta_function(attribute, &meta, ident),
                    )
                })?;
            }
        }
        Ok(())
    }
}
