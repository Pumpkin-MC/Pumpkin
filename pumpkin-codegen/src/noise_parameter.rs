use std::{collections::BTreeMap, fs};

use proc_macro2::TokenStream;
use pumpkin_util::DoublePerlinNoiseParametersCodec;
use quote::{format_ident, quote};

/// Generates the `TokenStream` for `DoublePerlinNoiseParameters` constants and its `id_to_parameters` lookup.
pub fn build() -> TokenStream {
    let json: BTreeMap<String, DoublePerlinNoiseParametersCodec> =
        serde_json::from_str(&fs::read_to_string("../assets/noise_parameters.json").unwrap())
            .expect("Failed to parse noise_parameters.json");
    let mut variants = TokenStream::new();
    let mut match_variants = TokenStream::new();

    for (i, (raw_name, parameter)) in json.iter().enumerate() {
        let name = raw_name
            .strip_prefix("minecraft:")
            .unwrap()
            .replace("/", "_");
        let simple_id = name.clone();
        let name_ident = format_ident!("{}", name.to_uppercase());
        let first_octave = parameter.first_octave;
        let amplitudes = &parameter.amplitudes;

        variants.extend([quote! {
            pub const #name_ident: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
                #i,
                #first_octave,
                &[#(#amplitudes),*],
                #raw_name
            );
        }]);

        match_variants.extend([quote! {
            #simple_id => &Self::#name_ident,
        }]);
    }
    let count = json.len();

    quote! {
        pub struct DoublePerlinNoiseParameters {
            pub id: usize,
            pub first_octave: i32,
            pub amplitudes: &'static [f64],
            name: &'static str,
        }

        impl DoublePerlinNoiseParameters {
            pub const COUNT: usize = #count;

            pub const fn new(id: usize, first_octave: i32, amplitudes: &'static [f64], name: &'static str) -> Self {
                Self {
                    id,
                    first_octave,
                    amplitudes,
                    name
                }
            }

            pub const fn name(&self) -> &'static str {
                self.name
            }

            pub fn id_to_parameters(id: &str) -> Option<&DoublePerlinNoiseParameters> {
                Some(match id {
                    #match_variants
                    _ => return None,
                })
            }

            #variants
        }

    }
}
