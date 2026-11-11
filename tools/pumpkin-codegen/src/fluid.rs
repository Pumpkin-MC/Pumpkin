use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::fs;

use crate::block::BlockStateId;

/// One state entry of `fluids.json`.
#[derive(Deserialize)]
struct FluidState {
    height: f32,
    level: i16,
    is_empty: bool,
    blast_resistance: f32,
    block_state_id: BlockStateId,
    is_still: bool,
}

/// One property of a fluid in `fluids.json`.
#[derive(Deserialize)]
struct Property {
    name: String,
    values: Vec<String>,
}

/// Spread tuning in one level kind.
#[derive(Deserialize)]
struct Flow {
    tick_delay: u8,
    drop_off: Option<u8>,
    slope_find_distance: Option<u8>,
}

impl Flow {
    fn to_tokens(&self, fluid: &str) -> TokenStream {
        assert_eq!(
            self.drop_off.is_some(),
            self.slope_find_distance.is_some(),
            "{fluid}"
        );
        // no FlowingFluid: no spread
        let (tick_delay, drop_off, slope_find_distance) = (
            self.tick_delay,
            self.drop_off.unwrap_or(0),
            self.slope_find_distance.unwrap_or(0),
        );
        quote! {
            FluidFlow {
                tick_delay: #tick_delay,
                drop_off: #drop_off,
                slope_find_distance: #slope_find_distance,
            }
        }
    }
}

/// One fluid entry of `fluids.json`.
#[derive(Deserialize)]
pub struct Fluid {
    pub name: String,
    pub id: u16,
    properties: Vec<Property>,
    default_state_index: u16,
    states: Vec<FluidState>,
    flow: Flow,
    fast_lava_flow: Flow,
}

fn state_tokens(state: &FluidState, falling: bool) -> TokenStream {
    let FluidState {
        height,
        level,
        is_empty,
        blast_resistance,
        block_state_id,
        is_still,
    } = state;
    // is_source = is_still
    quote! {
        FluidState {
            height: #height,
            level: #level,
            is_empty: #is_empty,
            blast_resistance: #blast_resistance,
            block_state_id: #block_state_id,
            is_still: #is_still,
            is_source: #is_still,
            falling: #falling,
        }
    }
}

/// Flowing fluid of `fluid`'s family, else `fluid`.
fn family_of<'a>(fluids: &'a [Fluid], fluid: &'a Fluid) -> &'a Fluid {
    let flowing = format!("flowing_{}", fluid.name);
    fluids
        .iter()
        .find(|other| other.name == flowing)
        .unwrap_or(fluid)
}

/// Generates the five fluid constants and their lookups.
pub fn build() -> TokenStream {
    let fluids: Vec<Fluid> =
        serde_json::from_str(&fs::read_to_string("../../assets/fluids.json").unwrap())
            .expect("Failed to parse fluids.json");

    let mut constants = Vec::new();
    let mut id_arms = Vec::new();
    let mut name_arms = Vec::new();
    let mut family_arms = Vec::new();
    let mut flowing_arms = Vec::new();
    let mut state_arms = Vec::new();
    for fluid in &fluids {
        let ident = format_ident!("{}", fluid.name.to_shouty_snake_case());
        let family = family_of(&fluids, fluid);
        if family.id != fluid.id {
            let (id, family_id) = (fluid.id, family.id);
            let family_ident = format_ident!("{}", family.name.to_shouty_snake_case());
            family_arms.push(quote! { #id => #family_id, });
            flowing_arms.push(quote! { #id => &Self::#family_ident, });
        }
        // row asserts: flowing rows are level + 8 * falling, falling true first
        if fluid.properties.len() == 2 {
            let names: Vec<&str> = fluid.properties.iter().map(|p| p.name.as_str()).collect();
            assert_eq!(names, ["falling", "level"], "{}", fluid.name);
            assert_eq!(
                fluid.properties[0].values,
                ["true", "false"],
                "{}",
                fluid.name
            );
            let first = fluid.states[0].block_state_id.0;
            for (row, state) in fluid.states.iter().enumerate() {
                assert_eq!(state.level as usize, row % 8 + 1, "{}", fluid.name);
                assert_eq!(state.block_state_id.0, first - row as u16, "{}", fluid.name);
            }
        }
        let flowing = fluid.properties.len() == 2;

        let (id, name, default_state_index) = (fluid.id, &fluid.name, fluid.default_state_index);
        let flow = fluid.flow.to_tokens(&fluid.name);
        let fast_lava_flow = fluid.fast_lava_flow.to_tokens(&fluid.name);
        let states = fluid
            .states
            .iter()
            .enumerate()
            .map(|(row, state)| state_tokens(state, flowing && row < 8));
        constants.push(quote! {
            pub const #ident: Fluid = Fluid {
                id: #id,
                name: #name,
                states: &[#(#states),*],
                default_state_index: #default_state_index,
                flow: #flow,
                fast_lava_flow: #fast_lava_flow,
            };
        });
        id_arms.push(quote! { #id => Some(&Self::#ident), });
        name_arms.push(quote! { #name => Some(&Self::#ident), });

        let ids = fluid.states.iter().map(|s| s.block_state_id.0);
        let (start, end) = (ids.clone().min().unwrap(), ids.max().unwrap());
        state_arms.push((
            end - start,
            quote! { #start..=#end => Some(&Self::#ident), },
        ));
    }
    let max_slope_find_distance = fluids
        .iter()
        .flat_map(|f| [&f.flow, &f.fast_lava_flow])
        .filter_map(|flow| flow.slope_find_distance)
        .max()
        .unwrap();

    // narrow ranges first: sources before their flowing family
    state_arms.sort_by_key(|(width, _)| *width);
    let state_arms = state_arms.into_iter().map(|(_, arm)| arm);

    quote! {
        use super::{Fluid, FluidFlow, FluidState};
        use crate::BlockStateId;

        impl Fluid {
            #(#constants)*

            /// Largest `slope_find_distance` of any fluid.
            pub const MAX_SLOPE_FIND_DISTANCE: u8 = #max_slope_find_distance;

            #[must_use]
            pub fn from_registry_key(name: &str) -> Option<&'static Self> {
                match name {
                    #(#name_arms)*
                    _ => None,
                }
            }

            #[must_use]
            pub const fn from_id(id: u16) -> Option<&'static Self> {
                match id {
                    #(#id_arms)*
                    _ => None,
                }
            }

            /// Id of the flowing fluid of `id`'s family.
            #[must_use]
            pub const fn family_id(id: u16) -> u16 {
                match id {
                    #(#family_arms)*
                    own => own,
                }
            }

            #[must_use]
            pub const fn to_flowing(&'static self) -> &'static Self {
                match self.id {
                    #(#flowing_arms)*
                    _ => self,
                }
            }

            #[must_use]
            #[allow(clippy::match_overlapping_arm)]
            pub const fn from_state_id(id: BlockStateId) -> Option<&'static Self> {
                match id.as_u16() {
                    #(#state_arms)*
                    _ => None,
                }
            }
        }
    }
}
