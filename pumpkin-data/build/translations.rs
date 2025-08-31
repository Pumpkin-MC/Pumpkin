use proc_macro2::TokenStream;
use pumpkin_util::global_path;
use quote::format_ident;
use quote::quote;
use serde::Deserialize;
use std::{collections::BTreeMap, fs::File, io::BufReader};

const VERSION: &str = "1.21.8";

#[derive(Deserialize, Clone)]
struct MinecraftResource {
    hash: String,
    size: u32,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/en_us.json");
    println!("cargo:rerun-if-changed=../assets/translations/en_us.json");
    // Hardcode en_us translations (as static slice chunks)
    let file = File::open(global_path!("../../assets/en_us.json")).unwrap();
    let reader = BufReader::new(file);
    let vanilla: BTreeMap<String, String> = serde_json::from_reader(reader).unwrap();
    let file = File::open(global_path!("../../assets/translations/en_us.json")).unwrap();
    let reader = BufReader::new(file);
    let custom: BTreeMap<String, String> = serde_json::from_reader(reader).unwrap();

    let mut entries: Vec<(String, String)> = Vec::with_capacity(vanilla.len() + custom.len());
    for (key, text) in vanilla {
        entries.push((format!("minecraft:{key}"), text));
    }
    for (key, text) in custom {
        entries.push((format!("pumpkin:{key}"), text));
    }

    // Chunk into static slices of ~500 items to avoid stack overflow
    let chunk_size: usize = 514;
    let mut slice_consts = TokenStream::new();
    let mut extend_stmts = TokenStream::new();
    for (i, chunk) in entries.chunks(chunk_size).enumerate() {
        let ident = format_ident!("EN_US_SLICE_{}", i);
        let mut slice_items = TokenStream::new();
        for (k, v) in chunk {
            let k = k.as_str();
            let v = v.as_str();
            slice_items.extend([quote! { (#k, #v), }]);
        }
        slice_consts.extend([quote! {
            static #ident: &[(&str, &str)] = &[
                #slice_items
            ];
        }]);

        extend_stmts.extend([quote! {
            en_us.extend(#ident.iter().map(|(k, v)| (String::from(*k), String::from(*v))));
        }]);
    }

    // Get locales hashes
    let response =
        reqwest::blocking::get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
            .unwrap();
    let versions_list: serde_json::Value = response.json().unwrap();
    let mut version_info_link = "";
    for version in versions_list.get("versions").unwrap().as_array().unwrap() {
        let version = version.as_object().unwrap();
        if version.get("id").unwrap().as_str().unwrap() == VERSION {
            version_info_link = version.get("url").unwrap().as_str().unwrap();
            break;
        }
    }
    let response = reqwest::blocking::get(version_info_link).unwrap();
    let version_info: serde_json::Value = response.json().unwrap();
    let resources_link = version_info
        .get("assetIndex")
        .unwrap()
        .as_object()
        .unwrap()
        .get("url")
        .unwrap()
        .as_str()
        .unwrap();
    let response = reqwest::blocking::get(resources_link).unwrap();
    let mut resources: BTreeMap<String, BTreeMap<String, MinecraftResource>> =
        response.json().unwrap();
    let langs: BTreeMap<String, MinecraftResource> = resources
        .get_mut("objects")
        .unwrap()
        .iter()
        .filter_map(|(a, b)| {
            if a.starts_with("minecraft/lang") {
                return Some((a.clone(), b.clone()));
            }
            None
        })
        .collect();
    let hashes = langs
        .iter()
        .map(|(a, b)| {
            let locale = a
                .strip_prefix("minecraft/lang/")
                .unwrap()
                .strip_suffix(".json")
                .unwrap();
            let hash = &b.hash;
            let size = b.size;
            quote! {
                #locale => Some((#hash, #size)),
            }
        })
        .collect::<TokenStream>();

    quote!(
        use std::borrow::Cow;
        use std::collections::BTreeMap;
        use std::error::Error;
        use std::fs::File;
        use std::io::BufReader;
        use std::path::Path;

        // Generated static chunks for en_us translations
        #slice_consts

        pub struct TranslationManager(BTreeMap<String, BTreeMap<String, String>>);

        impl TranslationManager {
            pub fn new() -> Self {
                let mut en_us = BTreeMap::new();
                #extend_stmts

                let mut map = BTreeMap::new();
                map.insert(String::from("en_us"), en_us);
                Self(map)
            }

            pub fn get<P: Into<String>>(
                &self,
                locale: P,
                namespaced_key: P,
                fallback: Option<Cow<'static, str>>,
            ) -> String {
                let namespaced_key = namespaced_key.into();
                let translation = self
                    .0
                    .get(&locale.into())
                    .or_else(|| self.0.get("en_us"))
                    .unwrap();
                if let Some(text) = translation.get(&namespaced_key) {
                    return text.clone();
                }
                if let Some(fallback) = fallback {
                    return fallback.to_string();
                }
                namespaced_key
            }

            /// Adds a new translated text to any locale.
            ///
            /// Returns `Some(String)` if a previous translation was replaced,
            /// containing the replaced text.
            pub fn add<P: Into<String>>(
                &mut self,
                locale: P,
                namespaced_key: P,
                text: P,
            ) -> Option<String> {
                let locale = locale.into();
                if let Some(translation) = self.0.get_mut(&locale) {
                    return translation.insert(namespaced_key.into(), text.into());
                }
                self.0.insert(
                    locale,
                    BTreeMap::from([(namespaced_key.into(), text.into())]),
                );
                None
            }

            pub fn add_file<P: Into<String>, F: AsRef<Path>>(
                &mut self,
                locale: P,
                namespace: P,
                file_path: F,
            ) -> Result<Vec<String>, Box<dyn Error>> {
                let locale = locale.into();
                let namespace = namespace.into();
                let file = File::open(file_path)?;
                let reader = BufReader::new(file);
                let texts: BTreeMap<String, String> = serde_json::from_reader(reader)?;

                if let Some(translation) = self.0.get_mut(&locale) {
                    let mut replaced_texts = Vec::new();
                    for text in texts {
                        if let Some(replaced) =
                            translation.insert(format!("{}:{}", namespace, text.0), text.1)
                        {
                            replaced_texts.push(replaced);
                        }
                    }
                    return Ok(replaced_texts);
                }
                let mut translation = BTreeMap::new();
                for text in texts {
                    translation.insert(format!("{}:{}", namespace, text.0), text.1);
                }
                self.0.insert(locale, translation);
                Ok(Vec::new())
            }

            pub fn locale_hash(locale: &str) -> Option<(&'static str, u32)> {
                match locale {
                    #hashes
                    _ => None,
                }
            }
        }

        impl Default for TranslationManager {
            fn default() -> Self {
                Self::new()
            }
        }
    )
}
