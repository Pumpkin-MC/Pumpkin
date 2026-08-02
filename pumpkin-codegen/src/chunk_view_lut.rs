use proc_macro2::TokenStream;
use quote::quote;

const MAX_VIEW_DISTANCE: u8 = 32;

pub fn build() -> TokenStream {
    let entries = (0..=MAX_VIEW_DISTANCE).map(|dist| {
        if dist < 2 {
            return quote! { &[] };
        }
        let mut positions = vec![];
        let d = i64::from(dist);

        for z in -(d + 2)..=(d + 2) {
            for x in -(d + 2)..=(d + 2) {
                let rel_x = (x.abs() - 2).max(0);
                let rel_z = (z.abs() - 2).max(0);
                if rel_x * rel_x + rel_z * rel_z < d * d {
                    positions.push((x as i8, z as i8));
                }
            }
        }

        positions.sort_by_key(|&(x, z)| i32::from(x).pow(2) + i32::from(z).pow(2));

        let array_elems = positions.into_iter().map(|(x, z)| quote!((#x, #z)));

        quote! {
            &[ #(#array_elems),* ]
        }
    });

    let array_len = MAX_VIEW_DISTANCE as usize + 1;

    quote! {
        /// The maximum supported view distance
        pub const MAX_VIEW_DISTANCE: u8 = #MAX_VIEW_DISTANCE;

        /// Static precomputed lookup table for relative chunk offsets by view distance (0..=32).
        pub static CHUNK_VIEW_LUT: [&[(i8, i8)]; #array_len] = [ #(#entries),* ];
    }
}
