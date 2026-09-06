use core::f32;

use pumpkin_data::{BlockDirection, BlockState, BlockStateId};
use pumpkin_util::{
    math::{self, lerp, position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;
use crate::{generation::rule::RuleTest, world::WorldPortalExt};

pub struct OreFeature {
    pub size: i32,
    pub discard_chance_on_air_exposure: f32,
    pub targets: Vec<OreTarget>,
}

pub struct OreTarget {
    pub target: RuleTest,
    pub state: &'static BlockState,
}

impl OreFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _block_registry: &dyn WorldPortalExt,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let f = random.next_f32() * f32::consts::PI;
        let g = self.size as f32 / 8.0f32;
        let i = f32::midpoint(self.size as f32 / 16.0f32 * 2.0, 1.0).ceil() as i32;

        let (sin_spread, cos_spread) = Self::vein_spread(f, g);

        let d = pos.0.x as f64 + sin_spread;
        let e = pos.0.x as f64 - sin_spread;
        let h = pos.0.z as f64 + cos_spread;
        let j = pos.0.z as f64 - cos_spread;

        let l = pos.0.y as f64 + random.next_bounded_i32(3) as f64 - 2.0;
        let m = pos.0.y as f64 + random.next_bounded_i32(3) as f64 - 2.0;

        let n = pos.0.x - g.ceil() as i32 - i;
        let o = pos.0.y - 2 - i;
        let p = pos.0.z - g.ceil() as i32 - i;
        let q = 2 * (g.ceil() as i32 + i);
        let r = 2 * (2 + i);

        for xprobe in n..=(n + q) {
            for zprobe in p..=(p + q) {
                if o <= chunk.ocean_floor_height_exclusive(xprobe, zprobe) {
                    return self.generate_vein_part(chunk, random, d, e, h, j, l, m, n, o, p, q, r);
                }
            }
        }
        false
    }

    #[expect(clippy::too_many_arguments)]
    #[expect(clippy::too_many_lines)]
    fn generate_vein_part<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        start_x: f64,
        end_x: f64,
        start_z: f64,
        end_z: f64,
        start_y: f64,
        end_y: f64,
        x_bound: i32,
        y_bound: i32,
        z_bound: i32,
        horizontal_size: i32,
        vertical_size: i32,
    ) -> bool {
        let mut placed_blocks_count = 0;
        let bitset_size = (horizontal_size * vertical_size * horizontal_size) as usize;
        let mut bit_set = vec![false; bitset_size];
        let mut mutable_pos = BlockPos::ZERO;
        let j = self.size;
        let mut ds = vec![0.0; (j * 4) as usize];
        for k in 0..j {
            let f = k as f32 / j as f32;
            let d = lerp(f as f64, start_x, end_x);
            let e = lerp(f as f64, start_y, end_y);
            let g = lerp(f as f64, start_z, end_z);
            let h = random.next_f64() * j as f64 / 16.0;
            let l = Self::node_radius(f, h);

            ds[k as usize * 4] = d;
            ds[k as usize * 4 + 1] = e;
            ds[k as usize * 4 + 2] = g;
            ds[k as usize * 4 + 3] = l;
        }

        for k in 0..(j - 1) {
            if ds[k as usize * 4 + 3] <= 0.0 {
                continue;
            }
            for m in (k + 1)..j {
                if ds[m as usize * 4 + 3] <= 0.0 {
                    continue;
                }
                let h_val = ds[k as usize * 4 + 3] - ds[m as usize * 4 + 3];
                let d_val = ds[k as usize * 4] - ds[m as usize * 4];
                let e_val = ds[k as usize * 4 + 1] - ds[m as usize * 4 + 1];
                let g_val = ds[k as usize * 4 + 2] - ds[m as usize * 4 + 2];

                if h_val * h_val > d_val * d_val + e_val * e_val + g_val * g_val {
                    if h_val > 0.0 {
                        ds[m as usize * 4 + 3] = -1.0;
                        continue;
                    }
                    ds[k as usize * 4 + 3] = -1.0;
                }
            }
        }

        for m_idx in 0..j {
            let d_val = ds[m_idx as usize * 4 + 3];
            if d_val < 0.0 {
                continue;
            }
            let e_val = ds[m_idx as usize * 4];
            let g_val = ds[m_idx as usize * 4 + 1];
            let h_val = ds[m_idx as usize * 4 + 2];

            let n_bound = ((e_val - d_val).floor() as i32).max(x_bound);
            let o_bound = ((g_val - d_val).floor() as i32).max(y_bound);
            let p_bound = ((h_val - d_val).floor() as i32).max(z_bound);
            let q_bound = ((e_val + d_val).floor() as i32).max(n_bound);
            let r_bound = ((g_val + d_val).floor() as i32).max(o_bound);
            let s_bound = ((h_val + d_val).floor() as i32).max(p_bound);

            for t_val in n_bound..=q_bound {
                let u_val = (t_val as f64 + 0.5 - e_val) / d_val;
                if u_val * u_val >= 1.0 {
                    continue;
                }
                for v_val in o_bound..=r_bound {
                    let w_val = (v_val as f64 + 0.5 - g_val) / d_val;
                    if u_val * u_val + w_val * w_val >= 1.0 {
                        continue;
                    }
                    let base_idx = (t_val - x_bound) + (v_val - y_bound) * horizontal_size;

                    for aa_val in p_bound..=s_bound {
                        let ab_val = (aa_val as f64 + 0.5 - h_val) / d_val;
                        if u_val * u_val + w_val * w_val + ab_val * ab_val >= 1.0 {
                            continue;
                        }
                        if chunk.out_of_height(v_val as i16) {
                            continue;
                        }

                        let ac = (base_idx + (aa_val - z_bound) * horizontal_size * vertical_size)
                            as usize;

                        if bit_set[ac] {
                            continue;
                        }
                        bit_set[ac] = true;

                        mutable_pos.0.x = t_val;
                        mutable_pos.0.y = v_val;
                        mutable_pos.0.z = aa_val;

                        // if !world.is_valid_for_set_block(&mutable_pos) {
                        //     continue;
                        // }

                        // let ad = t_val;
                        // let ae = v_val;
                        // let af = aa_val;
                        // TODO: using a section would be faster

                        let pos_vec = Vector3::new(t_val, v_val, aa_val);
                        let block_state = GenerationCache::get_block_state(chunk, &pos_vec);

                        for target in &self.targets {
                            if Self::should_place(
                                self.discard_chance_on_air_exposure,
                                chunk,
                                block_state,
                                random,
                                target,
                                &mutable_pos,
                            ) {
                                chunk.set_block_state(&pos_vec, target.state);
                                placed_blocks_count += 1;
                                break; // Equivalent to 'continue block11;'
                            }
                        }
                    }
                }
            }
        }
        placed_blocks_count > 0
    }

    /// The two ends of the vein axis, as vanilla `OreFeature.place` offsets them:
    ///
    /// ```text
    /// double x0 = origin.getX() + Math.sin(dir) * spreadXY;
    /// double z0 = origin.getZ() + Math.cos(dir) * spreadXY;
    /// ```
    ///
    /// `Math.sin`/`Math.cos` take a `double`, so the `float` angle is widened *before* the sine.
    /// That is a different function from the `Mth.sin` lookup table `doPlace` uses a few lines
    /// later — the same method really does call both.
    #[must_use]
    pub fn vein_spread(dir: f32, spread_xy: f32) -> (f64, f64) {
        let dir = f64::from(dir);
        let spread_xy = f64::from(spread_xy);
        (dir.sin() * spread_xy, dir.cos() * spread_xy)
    }

    /// Radius of one node of the vein, exactly as vanilla `OreFeature.doPlace` computes it:
    ///
    /// ```text
    /// double r = ((Mth.sin((float) Math.PI * step) + 1.0F) * ss + 1.0) / 2.0;
    /// ```
    ///
    /// Two details matter for bit-for-bit parity:
    /// * `Mth.sin` is the 65536-entry lookup table (`pumpkin_util::math::sin`), not an accurate
    ///   sine — its error is around 1e-4, which is enough to gain or lose a shell of blocks.
    /// * only `Mth.sin(..) + 1.0F` is `float`; `* ss` widens to `double`, and the `+ 1.0` and
    ///   `/ 2.0` are `double` too. Java rounds the multiply and the add separately, so this must
    ///   not be contracted into a `mul_add`.
    #[must_use]
    #[expect(
        clippy::manual_midpoint,
        reason = "kept in vanilla's grouping: only `sin(..) + 1.0F` is float, the rest is double"
    )]
    pub fn node_radius(step: f32, ss: f64) -> f64 {
        (f64::from(math::sin(f32::consts::PI * step) + 1.0) * ss + 1.0) / 2.0
    }

    pub fn should_place<T: GenerationCache>(
        discard_chance: f32,
        chunk: &T,
        state: BlockStateId,
        random: &mut RandomGenerator,
        target: &OreTarget,
        pos: &BlockPos,
    ) -> bool {
        if !target.target.test(state, random) {
            return false;
        }
        if Self::should_not_discard(random, discard_chance) {
            return true;
        }
        !Self::is_exposed_to_air(chunk, pos)
    }

    pub fn should_not_discard(random: &mut RandomGenerator, chance: f32) -> bool {
        if chance <= 0.0f32 {
            return true;
        }
        if chance >= 1.0f32 {
            return false;
        }
        random.next_f32() >= chance
    }

    pub fn is_exposed_to_air<T: GenerationCache>(chunk: &T, pos: &BlockPos) -> bool {
        for dir in BlockDirection::all() {
            if GenerationCache::get_block_state(chunk, &pos.offset(dir.to_offset()).0)
                .to_state()
                .is_air()
            {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::OreFeature;

    /// Reference values produced by running vanilla's own expression
    /// `((Mth.sin((float) Math.PI * step) + 1.0F) * ss + 1.0) / 2.0` on a JVM
    /// (`step = (float) i / size`), printed as raw `double` bits.
    #[test]
    fn node_radius_matches_vanilla_do_place() {
        // (size, i, ss, expected raw f64 bits)
        let cases: [(i32, i32, f64, u64); 9] = [
            (33, 0, 1.9375, 0x3ff7_8000_0000_0000),
            (33, 1, 0.5, 0x3fe8_c27c_4000_0000),
            (33, 7, 1.9375, 0x4000_8a4f_4e00_0000),
            (33, 16, 0.314_159_265_358_979_3, 0x3fea_0c21_d7c6_5b3e),
            (33, 32, 1.0, 0x3ff0_c2ae_4000_0000),
            (64, 7, 0.5, 0x3fea_b1f3_5000_0000),
            (64, 16, 1.9375, 0x4001_3ae6_6300_0000),
            (64, 32, 0.314_159_265_358_979_3, 0x3fea_0d97_bb4e_7870),
            (9, 7, 1.9375, 0x4000_bb52_ad00_0000),
        ];
        for (size, i, ss, expected) in cases {
            let step = i as f32 / size as f32;
            let actual = OreFeature::node_radius(step, ss);
            assert_eq!(
                actual.to_bits(),
                expected,
                "size={size} i={i} ss={ss}: got {actual} ({:#018x}), want {:#018x}",
                actual.to_bits(),
                expected
            );
        }
    }

    /// Reference values produced by running vanilla's `Math.sin(dir) * spreadXY` /
    /// `Math.cos(dir) * spreadXY` on a JVM (`spreadXY = size / 8.0F`), as raw `double` bits.
    #[test]
    fn vein_spread_matches_vanilla_place() {
        // (dir, size, sin*spread bits, cos*spread bits)
        let cases: [(f32, i32, u64, u64); 8] = [
            (0.0, 33, 0x0000_0000_0000_0000, 0x4010_8000_0000_0000),
            (0.5, 33, 0x3fff_a45f_b7ed_5e20, 0x400c_f5d1_468e_593a),
            (0.5, 8, 0x3fde_aee8_744b_05f0, 0x3fec_1528_065b_7d50),
            (1.2345, 33, 0x400f_26c5_7166_4dc2, 0x3ff5_c790_472b_94a0),
            (1.2345, 64, 0x401e_351c_8cfe_5aeb, 0x4005_1e9b_6bcd_2b46),
            (2.718_281_7, 9, 0x3fdd_9385_aa8f_3df0, 0xbff0_6945_0c92_7123),
            (
                2.718_281_7,
                64,
                0x400a_4a3d_ecf1_1a9c,
                0xc01d_2cec_8820_c922,
            ),
            (
                core::f32::consts::PI,
                33,
                0xbe98_3362_fdee_653c,
                0xc010_7fff_ffff_ffee,
            ),
        ];
        for (dir, size, want_sin, want_cos) in cases {
            let spread_xy = size as f32 / 8.0f32;
            let (sin_spread, cos_spread) = OreFeature::vein_spread(dir, spread_xy);
            assert_eq!(
                sin_spread.to_bits(),
                want_sin,
                "dir={dir} size={size}: sin got {sin_spread}"
            );
            assert_eq!(
                cos_spread.to_bits(),
                want_cos,
                "dir={dir} size={size}: cos got {cos_spread}"
            );
        }
    }

    /// The old form took the sine in `f32` and widened afterwards, which is a different number:
    /// vanilla `3.893_931_280_073_702_3` vs `3.893_931_180_238_723_8` for dir=1.2345, size=33.
    #[test]
    fn vein_spread_differs_from_f32_sine() {
        let (sin_spread, _) = OreFeature::vein_spread(1.2345, 33.0 / 8.0);
        let old = f64::from(1.2345f32.sin()) * f64::from(33.0f32 / 8.0f32);
        assert!((sin_spread - old).abs() > 1e-9);
    }

    /// The naive all-`f32` form with an accurate sine, which Pumpkin used before, disagrees with
    /// vanilla by more than a ULP — enough to move the ellipsoid boundary by a whole block.
    #[test]
    fn node_radius_differs_from_naive_f32_form() {
        let step = 7.0f32 / 33.0f32;
        let ss = 1.9375f64;
        let naive = f32::midpoint(
            ((core::f32::consts::PI * step).sin() + 1.0) * ss as f32,
            1.0,
        );
        let vanilla = OreFeature::node_radius(step, ss);
        assert!((f64::from(naive) - vanilla).abs() > 1e-6);
    }
}
