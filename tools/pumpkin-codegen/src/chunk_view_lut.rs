use proc_macro2::TokenStream;
use quote::quote;

const MAX_VIEW_DISTANCE: u8 = 32;
const MAX_CHEBYSHEV_RADIUS: u8 = 48;

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

    // Generate Chebyshev / square concentric ring offsets
    let mut all_chebyshev_offsets = vec![];
    let mut ring_ranges = vec![];
    let mut square_ranges = vec![];

    let mut current_idx = 0usize;
    for r in 0..=MAX_CHEBYSHEV_RADIUS {
        let r_i = r as i8;
        let start = current_idx;
        if r == 0 {
            all_chebyshev_offsets.push((0i8, 0i8));
            current_idx += 1;
        } else {
            // Top and bottom rows
            for x in -r_i..=r_i {
                all_chebyshev_offsets.push((x, -r_i));
                all_chebyshev_offsets.push((x, r_i));
                current_idx += 2;
            }
            // Left and right columns (excluding corners already added above)
            for z in (-r_i + 1)..=(r_i - 1) {
                all_chebyshev_offsets.push((-r_i, z));
                all_chebyshev_offsets.push((r_i, z));
                current_idx += 2;
            }
        }
        let end = current_idx;
        ring_ranges.push((start, end));
        square_ranges.push((0usize, end));
    }

    let chebyshev_total = all_chebyshev_offsets.len();
    let chebyshev_elems = all_chebyshev_offsets
        .into_iter()
        .map(|(x, z)| quote!((#x, #z)));

    let ring_bounds = ring_ranges.iter().map(|(s, e)| quote!((#s, #e)));
    let square_bounds = square_ranges.iter().map(|(_, e)| quote!(#e));
    let chebyshev_len = MAX_CHEBYSHEV_RADIUS as usize + 1;

    quote! {
        /// The maximum supported view distance
        pub const MAX_VIEW_DISTANCE: u8 = #MAX_VIEW_DISTANCE;

        /// Static precomputed lookup table for relative chunk offsets by view distance (0..=32).
        pub static CHUNK_VIEW_LUT: [&[(i8, i8)]; #array_len] = [ #(#entries),* ];

        /// The maximum supported Chebyshev / square radius (used for ticket levels and simulation distance)
        pub const MAX_CHEBYSHEV_RADIUS: u8 = #MAX_CHEBYSHEV_RADIUS;

        /// Flat backing storage for all concentric Chebyshev offsets up to `MAX_CHEBYSHEV_RADIUS`.
        pub static CHEBYSHEV_OFFSETS: [(i8, i8); #chebyshev_total] = [ #(#chebyshev_elems),* ];

        /// (start, end) index bounds in `CHEBYSHEV_OFFSETS` for each Chebyshev radius (0..=48).
        pub static CHEBYSHEV_RING_BOUNDS: [(usize, usize); #chebyshev_len] = [ #(#ring_bounds),* ];

        /// End index in `CHEBYSHEV_OFFSETS` for all chunks within Chebyshev radius (0..=48).
        pub static CHEBYSHEV_SQUARE_BOUNDS: [usize; #chebyshev_len] = [ #(#square_bounds),* ];

        /// Returns a precomputed slice of chunk offsets at exact Chebyshev radius `radius` (0..=48).
        #[inline]
        #[must_use]
        pub fn get_chebyshev_ring(radius: u8) -> &'static [(i8, i8)] {
            let r = (radius as usize).min(MAX_CHEBYSHEV_RADIUS as usize);
            let (start, end) = CHEBYSHEV_RING_BOUNDS[r];
            &CHEBYSHEV_OFFSETS[start..end]
        }

        /// Returns a precomputed slice of all chunk offsets within Chebyshev radius `radius` (0..=48), concentric-sorted.
        #[inline]
        #[must_use]
        pub fn get_chebyshev_square(radius: u8) -> &'static [(i8, i8)] {
            let r = (radius as usize).min(MAX_CHEBYSHEV_RADIUS as usize);
            let end = CHEBYSHEV_SQUARE_BOUNDS[r];
            &CHEBYSHEV_OFFSETS[..end]
        }
    }
}
