use std::{collections::BTreeMap, fs};

use proc_macro2::TokenStream;
use pumpkin_util::DoublePerlinNoiseParametersCodec;
use quote::{format_ident, quote};

fn collect_noise_files(
    base: &std::path::Path,
    dir: &std::path::Path,
    result: &mut BTreeMap<String, DoublePerlinNoiseParametersCodec>,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_noise_files(base, &path, result);
        } else if path.extension().is_some_and(|ext| ext == "json") {
            let rel = path.strip_prefix(base).unwrap();
            let rel_str = rel
                .with_extension("")
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            let key = format!("minecraft:{rel_str}");
            let content = fs::read_to_string(&path).expect("Failed to read noise file");
            let param: DoublePerlinNoiseParametersCodec =
                serde_json::from_str(&content).expect("Failed to parse noise parameter JSON");
            result.insert(key, param);
        }
    }
}

pub fn build() -> TokenStream {
    let dir = std::path::Path::new("../../assets/datapacks/26_2/data/minecraft/worldgen/noise");
    let mut json: BTreeMap<String, DoublePerlinNoiseParametersCodec> = BTreeMap::new();
    collect_noise_files(dir, dir, &mut json);

    let mut variants = TokenStream::new();
    let mut match_variants = TokenStream::new();

    for (i, (raw_name, parameter)) in json.iter().enumerate() {
        let name = raw_name
            .strip_prefix("minecraft:")
            .unwrap()
            .replace("/", "_");
        let name_ident = format_ident!("{}", name.to_uppercase());

        // 1. MD5 Pre-calculation
        let hash = md5::compute(raw_name.as_bytes());
        let lo = u64::from_be_bytes(hash[0..8].try_into().unwrap());
        let hi = u64::from_be_bytes(hash[8..16].try_into().unwrap());

        // 2. Amplitude Pre-calculation
        let amplitudes = &parameter.amplitudes;
        let mut min_octave = i32::MAX;
        let mut max_octave = i32::MIN;

        for (index, amp) in amplitudes.iter().enumerate() {
            if *amp != 0.0 {
                min_octave = i32::min(min_octave, index as i32);
                max_octave = i32::max(max_octave, index as i32);
            }
        }

        // Equivalent to your create_amplitude logic
        let octaves = max_octave - min_octave;
        let create_amp_val = 0.1f64 * (1.0f64 + 1.0f64 / (octaves + 1) as f64);
        let final_amplitude = 0.16666666666666666f64 / create_amp_val;

        let first_octave = parameter.first_octave;

        variants.extend([quote! {
            pub const #name_ident: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
                #i,
                #first_octave,
                &[#(#amplitudes),*],
                #lo,
                #hi,
                #final_amplitude
            );
        }]);

        match_variants.extend([quote! {
            #name => &Self::#name_ident,
        }]);
    }
    let count = json.len();

    quote! {
        pub struct DoublePerlinNoiseParameters {
            pub id: usize,
            pub first_octave: i32,
            pub amplitudes: &'static [f64],
            pub lo: u64,
            pub hi: u64,
            pub amplitude: f64,
        }

        impl DoublePerlinNoiseParameters {
            pub const COUNT: usize = #count;

            pub const fn new(
                id: usize,
                first_octave: i32,
                amplitudes: &'static [f64],
                lo: u64,
                hi: u64,
                amplitude: f64,
            ) -> Self {
                Self { id, first_octave, amplitudes, lo, hi, amplitude }
            }

            pub fn id_to_parameters(id: &str) -> Option<&'static DoublePerlinNoiseParameters> {
                let id = id.strip_prefix("minecraft:").unwrap_or(id).replace("/", "_");
                Some(match id.as_str() {
                    #match_variants
                    _ => return None,
                })
            }

            #variants
        }
    }
}
