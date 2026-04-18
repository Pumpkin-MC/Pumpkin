use proc_macro::TokenStream;
use proc_macro_error2::__export::proc_macro2;
use quote::quote;
use syn::{Data, DeriveInput, Error, Field, Ident, LitStr, parse_macro_input};

/// Derives the `Encode` trait for a struct.
///
/// Each field in a struct **must** have a `field` attribute.
/// The `field` attribute can have the following metas:
///
/// - `skip`: Skips serializing the field entirely.
/// - `name = "x"`: Sets this field to be encoded with the key `"x"`.
#[proc_macro_derive(Encode, attributes(field))]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();

    match &input.data {
        Data::Struct(data) => {
            let mut builder_encodes = Vec::new();
            for field in &data.fields {
                match encode_field_tokens(field) {
                    Ok(Some(tokens)) => {
                        builder_encodes.push(tokens);
                    }
                    Err(e) => return e.to_compile_error().into(),
                    _ => {}
                }
            }
            quote! {
                impl crate::codec::Encode for #name {
                    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
                        let mut builder = ops.map_builder();
                        #(#builder_encodes)*
                        builder.build(prefix)
                    }
                }
            }.into()
        }
        _ => Error::new_spanned(&input, "Only structs are supported")
            .to_compile_error()
            .into(),
    }
}

enum FieldData {
    /// Serialization occurs with the given field name.
    Present { name: String },
    /// Serialization of the field is ignored.
    Skipped,
}

/// Parses a single field to get its [`FieldData`].
fn generate_field_data(field: &Field) -> Result<FieldData, Error> {
    fn duplicate_error(ident: &Ident) -> Error {
        Error::new_spanned(ident, format!("A `{ident}` path was already defined"))
    }

    let mut field_name = None;
    let mut skipped = false;

    for attr in &field.attrs {
        if attr.path().is_ident("field") {
            attr.parse_nested_meta(|meta| {
                let ident = meta.path.get_ident().expect("Ident should exist");
                // #[field(skip)]
                if meta.path.is_ident("skip") {
                    if skipped {
                        return Err(duplicate_error(ident));
                    }
                    skipped = true;
                }
                // #[field(name = "x")]
                if meta.path.is_ident("name") {
                    if field_name.is_some() {
                        return Err(duplicate_error(ident));
                    }
                    let value = meta.value()?;
                    let lit = value.parse::<LitStr>()?;
                    field_name = Some(lit.value());
                }
                Ok(())
            })?;
        }
    }

    if skipped {
        if field_name.is_some() {
            return Err(Error::new_spanned(
                &field.ident,
                "Cannot specify any meta other than `skip` for a skipped field",
            ));
        }
        Ok(FieldData::Skipped)
    } else if let Some(name) = field_name {
        Ok(FieldData::Present { name })
    } else {
        Err(Error::new_spanned(
            &field.ident,
            "No `field` name or `skip` path was specified",
        ))
    }
}

fn encode_field_tokens(field: &Field) -> Result<Option<proc_macro2::TokenStream>, Error> {
    match generate_field_data(field)? {
        FieldData::Present { name } => {
            let ident = field.ident.as_ref().unwrap();
            let encoded_name_lit = LitStr::new(&name, proc_macro2::Span::call_site());
            Ok(Some(quote! {
                builder = crate::codec::FieldEncode::encode_field(&self.#ident, #encoded_name_lit, ops, builder);
            }))
        }
        FieldData::Skipped => Ok(None),
    }
}
