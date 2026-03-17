use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

/// A compiled bitset, ready to be embedded in generated code.
pub struct Bitset {
    /// The generated module containing the bitset constants and lookup function.
    pub items: TokenStream,
    /// The identifier of the private module wrapping the bitset.
    pub mod_ident: Ident,
    /// The identifier of the `contains` function exposed from the module.
    pub contains_ident: Ident,
}

/// Builds a compact `u64`-word bitset for a set of `u16` IDs and returns the generated code.
///
/// # Arguments
/// - `name` – Base name used to derive all generated identifier names.
/// - `ids` – Slice of numeric IDs to include in the bitset.
///
/// # Returns
/// A [`Bitset`] containing the generated module `TokenStream` and helper identifiers.
pub fn gen_u16_bitset(name: &str, ids: &[u16]) -> Bitset {
    let max_id = ids.iter().copied().max().unwrap_or(0);
    //let min_id = ids.iter().copied().min().unwrap_or(0);

    let words = ((max_id as usize) + 64) / 64;

    let mut bitset = vec![0u64; words];
    for &id in ids {
        let index = (id as usize) >> 6;
        let bit = u32::from(id) & 63;
        bitset[index] |= 1u64 << bit;
    }
    let name_uppercase = name.to_uppercase();

    let mod_ident = Ident::new(
        &format!("__{}_bitset", name.to_lowercase()),
        Span::call_site(),
    );
    let max_ident = Ident::new(&format!("{name_uppercase}_MAX_ID"), Span::call_site());
    let words_ident = Ident::new(&format!("{name_uppercase}_WORDS"), Span::call_site());
    let bitset_ident = Ident::new(&format!("{name_uppercase}_BITSET"), Span::call_site());
    let contains_ident = Ident::new(
        &format!("{}_contains", name.to_lowercase()),
        Span::call_site(),
    );

    let items = quote! {
        mod #mod_ident {
            pub const #max_ident: u16 = #max_id;
            pub const #words_ident: usize = #words;
            pub static #bitset_ident: [u64; #words_ident] = [ #(#bitset),* ];

            #[inline(always)]
            pub(super) const fn #contains_ident(id: u16) -> bool {
                if id > #max_ident {
                    return false;
                }
                let index: usize = (id as usize) >> 6;
                let bit: u32 = (id as u32) & 63;

                ((#bitset_ident[index] >> bit) & 1) != 0
            }
        }

    };
    Bitset {
        items,
        mod_ident,
        contains_ident,
    }
}
