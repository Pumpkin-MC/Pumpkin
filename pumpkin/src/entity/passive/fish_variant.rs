//! Shared helpers for Salmon/`TropicalFish` variant handling.
//!
//! Deliberately narrow: the two species' variant *algorithms* differ (Salmon is a weighted
//! 3-value enum, `TropicalFish` is a 90/10 common-or-random packed triple) and are not unified
//! here - only the mechanically identical bit-packing shape and a generic weighted-pick helper
//! are shared.

use crate::block::entities::sign::DyeColor;
use rand::RngExt;

/// `TropicalFish.packVariant`: `pattern.getPackedId() & 0xFFFF | (base & 0xFF) << 16 |
/// (patternColor & 0xFF) << 24`.
#[must_use]
pub const fn pack_variant(pattern_packed_id: u16, base_color: u8, pattern_color: u8) -> i32 {
    (pattern_packed_id as i32) & 0xFFFF
        | ((base_color as i32) & 0xFF) << 16
        | ((pattern_color as i32) & 0xFF) << 24
}

#[must_use]
pub const fn unpack_pattern_id(packed: i32) -> u16 {
    (packed & 0xFFFF) as u16
}

#[must_use]
pub const fn unpack_base_color_id(packed: i32) -> u8 {
    ((packed >> 16) & 0xFF) as u8
}

#[must_use]
pub const fn unpack_pattern_color_id(packed: i32) -> u8 {
    ((packed >> 24) & 0xFF) as u8
}

/// `DyeColor.byId`, clamped to `WHITE` for any id outside the 16 known dye colors (matching
/// vanilla's `ByIdMap` out-of-bounds fallback behavior for enums indexed this way).
#[must_use]
pub const fn dye_color_from_id(id: u8) -> DyeColor {
    match id {
        1 => DyeColor::Orange,
        2 => DyeColor::Magenta,
        3 => DyeColor::LightBlue,
        4 => DyeColor::Yellow,
        5 => DyeColor::Lime,
        6 => DyeColor::Pink,
        7 => DyeColor::Gray,
        8 => DyeColor::LightGray,
        9 => DyeColor::Cyan,
        10 => DyeColor::Purple,
        11 => DyeColor::Blue,
        12 => DyeColor::Brown,
        13 => DyeColor::Green,
        14 => DyeColor::Red,
        15 => DyeColor::Black,
        _ => DyeColor::White,
    }
}

/// Pure cumulative-weight walk.
///
/// Matches vanilla's `WeightedList` semantics (uniform pick over the sum of weights). `roll`
/// must be `< sum of weights`; split out from [`pick_weighted`] so the boundary logic is
/// testable without an RNG.
#[must_use]
pub fn pick_weighted_by_roll<T: Copy>(mut roll: u32, choices: &[(T, u32)]) -> T {
    for (value, weight) in choices {
        if roll < *weight {
            return *value;
        }
        roll -= *weight;
    }
    choices[choices.len() - 1].0
}

/// Rolls a value from a `(value, weight)` list. Returns the first entry if `choices` is empty
/// or all weights are zero.
pub fn pick_weighted<T: Copy>(rng: &mut impl RngExt, choices: &[(T, u32)]) -> T {
    let total: u32 = choices.iter().map(|(_, w)| *w).sum();
    if total == 0 {
        return choices[0].0;
    }
    pick_weighted_by_roll(rng.random_range(0..total), choices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_unpack_round_trips() {
        let pattern_id = 0x0503u16;
        let packed = pack_variant(pattern_id, 14, 7);
        assert_eq!(unpack_pattern_id(packed), pattern_id);
        assert_eq!(unpack_base_color_id(packed), 14);
        assert_eq!(unpack_pattern_color_id(packed), 7);
    }

    #[test]
    fn pack_matches_vanilla_bit_layout() {
        // pattern.getPackedId() & 0xFFFF | (base & 0xFF) << 16 | (patternColor & 0xFF) << 24
        let packed = pack_variant(0x0102, 0x03, 0x04);
        assert_eq!(packed, 0x0403_0102u32 as i32);
    }

    #[test]
    fn dye_color_ids_round_trip_through_from_id() {
        assert_eq!(dye_color_from_id(0), DyeColor::White);
        assert_eq!(dye_color_from_id(14), DyeColor::Red);
        assert_eq!(dye_color_from_id(15), DyeColor::Black);
        // Out-of-range clamps to White, matching vanilla's ByIdMap fallback.
        assert_eq!(dye_color_from_id(255), DyeColor::White);
    }

    #[test]
    fn pick_weighted_by_roll_respects_boundaries() {
        // Salmon's 30/50/15 split: small is [0,30), medium is [30,80), large is [80,95).
        let choices = [("small", 30u32), ("medium", 50u32), ("large", 15u32)];
        assert_eq!(pick_weighted_by_roll(0, &choices), "small");
        assert_eq!(pick_weighted_by_roll(29, &choices), "small");
        assert_eq!(pick_weighted_by_roll(30, &choices), "medium");
        assert_eq!(pick_weighted_by_roll(79, &choices), "medium");
        assert_eq!(pick_weighted_by_roll(80, &choices), "large");
        assert_eq!(pick_weighted_by_roll(94, &choices), "large");
    }

    #[test]
    fn pick_weighted_falls_back_to_first_when_all_weights_are_zero() {
        let choices = [("only", 0u32)];
        assert_eq!(pick_weighted(&mut rand::rng(), &choices), "only");
    }
}
