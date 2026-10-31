use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use serde::Deserialize;
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Deserialize, Clone)]
pub struct RawTemplatePool {
    pub fallback: String,
    pub elements: Vec<RawWeightedPoolElement>,
}

#[derive(Deserialize, Clone)]
pub struct RawWeightedPoolElement {
    pub element: RawPoolElement,
    pub weight: u32,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "element_type")]
pub enum RawPoolElement {
    #[serde(rename = "minecraft:empty_pool_element")]
    Empty,
    #[serde(rename = "minecraft:single_pool_element")]
    Single {
        location: String,
        processors: RawProcessorList,
        projection: RawProjection,
    },
    #[serde(rename = "minecraft:legacy_single_pool_element")]
    LegacySingle {
        location: String,
        processors: RawProcessorList,
        projection: RawProjection,
    },
    #[serde(rename = "minecraft:list_pool_element")]
    List {
        elements: Vec<RawPoolElement>,
        projection: RawProjection,
    },
    #[serde(rename = "minecraft:feature_pool_element")]
    Feature {
        feature: String,
        projection: RawProjection,
    },
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum RawProcessorList {
    Named(String),
    Inline { processors: Vec<serde_json::Value> },
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawProjection {
    Rigid,
    TerrainMatching,
}

impl RawPoolElement {
    fn projection(&self) -> RawProjection {
        match self {
            Self::Empty => RawProjection::Rigid,
            Self::Single { projection, .. }
            | Self::LegacySingle { projection, .. }
            | Self::List { projection, .. }
            | Self::Feature { projection, .. } => *projection,
        }
    }
}

impl ToTokens for RawWeightedPoolElement {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let weight = self.weight;
        let proj = match self.element.projection() {
            RawProjection::Rigid => quote!(TemplatePoolProjection::Rigid),
            RawProjection::TerrainMatching => quote!(TemplatePoolProjection::TerrainMatching),
        };
        let kind = element_to_tokens(&self.element);
        tokens.extend(quote!(
            StaticPoolElement {
                weight: #weight,
                projection: #proj,
                kind: #kind,
            }
        ));
    }
}

fn element_to_tokens(element: &RawPoolElement) -> TokenStream {
    match element {
        RawPoolElement::Empty => quote!(StaticPoolElementKind::Empty),
        RawPoolElement::Single {
            location,
            processors,
            projection: _,
        } => {
            let proc_str = match processors {
                RawProcessorList::Named(s) => s.as_str(),
                RawProcessorList::Inline { .. } => "",
            };
            quote!(StaticPoolElementKind::Single {
                location: #location,
                processors: #proc_str,
                legacy: false,
            })
        }
        RawPoolElement::LegacySingle {
            location,
            processors,
            projection: _,
        } => {
            let proc_str = match processors {
                RawProcessorList::Named(s) => s.as_str(),
                RawProcessorList::Inline { .. } => "",
            };
            quote!(StaticPoolElementKind::Single {
                location: #location,
                processors: #proc_str,
                legacy: true,
            })
        }
        RawPoolElement::List {
            elements,
            projection: _,
        } => {
            let children: Vec<TokenStream> = elements.iter().map(element_to_tokens).collect();
            quote!(StaticPoolElementKind::List(&[#(#children),*]))
        }
        RawPoolElement::Feature {
            feature,
            projection: _,
        } => {
            quote!(StaticPoolElementKind::Feature(#feature))
        }
    }
}

pub fn build() -> TokenStream {
    let pool_dir = Path::new("../../assets/datapack/data/minecraft/worldgen/template_pool");
    let mut pools: BTreeMap<String, RawTemplatePool> = BTreeMap::new();

    fn visit_dir(dir: &Path, base: &Path, pools: &mut BTreeMap<String, RawTemplatePool>) {
        let mut entries: Vec<_> = fs::read_dir(dir)
            .expect("Failed to read template_pool directory")
            .flatten()
            .collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                visit_dir(&path, base, pools);
            } else if path.extension().is_some_and(|ext| ext == "json") {
                let rel = path.strip_prefix(base).unwrap();
                let stem = rel.with_extension("").to_string_lossy().into_owned();
                let content = fs::read_to_string(&path).expect("Failed to read template pool JSON");
                let pool: RawTemplatePool =
                    serde_json::from_str(&content).expect("Failed to parse template pool JSON");
                pools.insert(stem, pool);
            }
        }
    }

    visit_dir(pool_dir, pool_dir, &mut pools);

    let mut const_defs = TokenStream::new();
    let mut lookup_arms = TokenStream::new();
    let mut all_names = Vec::new();

    for (name, pool) in &pools {
        let ident_name = name.replace(['/', '-'], "_").to_uppercase();
        let const_name = format_ident!("{}", ident_name);
        let id_str = format!("minecraft:{name}");
        let fallback = &pool.fallback;
        let elements = &pool.elements;

        const_defs.extend(quote!(
            pub const #const_name: StaticTemplatePool = StaticTemplatePool {
                id: #id_str,
                fallback: #fallback,
                elements: &[#(#elements),*],
            };
        ));

        lookup_arms.extend(quote!(
            #name => Some(&Self::#const_name),
        ));

        all_names.push(name.clone());
    }

    let all_names_tokens: Vec<TokenStream> = all_names
        .iter()
        .map(|name| {
            let full = format!("minecraft:{name}");
            quote!(#full)
        })
        .collect();

    quote!(
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum TemplatePoolProjection {
            Rigid,
            TerrainMatching,
        }

        #[derive(Clone, Debug, PartialEq, Eq)]
        pub enum StaticPoolElementKind {
            Empty,
            Single {
                location: &'static str,
                processors: &'static str,
                legacy: bool,
            },
            List(&'static [StaticPoolElementKind]),
            Feature(&'static str),
        }

        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct StaticPoolElement {
            pub weight: u32,
            pub projection: TemplatePoolProjection,
            pub kind: StaticPoolElementKind,
        }

        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct StaticTemplatePool {
            pub id: &'static str,
            pub fallback: &'static str,
            pub elements: &'static [StaticPoolElement],
        }

        impl StaticTemplatePool {
            #const_defs

            #[must_use]
            pub fn get(name: &str) -> Option<&'static Self> {
                let stripped = name.strip_prefix("minecraft:").unwrap_or(name);
                match stripped {
                    #lookup_arms
                    _ => None,
                }
            }

            #[must_use]
            pub const fn all_names() -> &'static [&'static str] {
                &[#(#all_names_tokens),*]
            }
        }

        #[must_use]
        pub fn get_template_pool(name: &str) -> Option<&'static StaticTemplatePool> {
            StaticTemplatePool::get(name)
        }
    )
}
