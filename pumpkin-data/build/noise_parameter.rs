use std::collections::HashMap;

use proc_macro2::TokenStream;
use pumpkin_util::DoublePerlinNoiseParametersCodec;
use quote::{format_ident, quote};

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/noise_parameters.json");

    let json: HashMap<String, DoublePerlinNoiseParametersCodec> =
        serde_json::from_str(include_str!("../../assets/noise_parameters.json"))
            .expect("Failed to parse noise_parameters.json");
    let mut variants = TokenStream::new();
    let mut match_variants = TokenStream::new();

    for (name, parameter) in json.iter() {
        let raw_name = format!("minecraft:{name}");
        let simple_id = name;
        let name = format_ident!("{}", name.to_uppercase());
        let first_octave = parameter.first_octave;
        let amplitudes = &parameter.amplitudes;
        variants.extend([quote! {
            pub const #name: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(#first_octave, &[#(#amplitudes),*], #raw_name);
        }]);
        match_variants.extend([quote! {
            #simple_id => &#name,
        }]);
    }

    quote! {
        pub struct DoublePerlinNoiseParameters {
            pub first_octave: i32,
            pub amplitudes: &'static [f64],
            id: &'static str,
        }

        impl DoublePerlinNoiseParameters {
            pub const fn new(first_octave: i32, amplitudes: &'static [f64], id: &'static str) -> Self {
                Self {
                    first_octave,
                    amplitudes,
                    id
                }
            }

            pub const fn id(&self) -> &'static str {
                self.id
            }

            pub fn id_to_parameters(id: &str) -> Option<&DoublePerlinNoiseParameters> {
                Some(match id {
                    #match_variants
                    _ => return None,
                })
            }
        }

        #variants
    }
}
