use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;

use crate::ident;

#[derive(Deserialize)]
pub struct DoublePerlinNoiseParameters {
    first_octave: i32,
    amplitudes: Vec<f64>,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/noise_parameters.json");

    let json: HashMap<String, DoublePerlinNoiseParameters> =
        serde_json::from_str(include_str!("../../assets/noise_parameters.json"))
            .expect("Failed to parse noise_parameters.json");
    let mut variants = TokenStream::new();

    for (name, paremter) in json.iter() {
        let raw_name = format!("minecraft:{name}");
        let name = ident(name.to_uppercase());
        let first_octave = paremter.first_octave;
        let amplitudes = &paremter.amplitudes;
        variants.extend([quote! {
            pub const #name: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(#first_octave, &[#(#amplitudes),*], #raw_name);
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
        }

        #variants
    }
}
