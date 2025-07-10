use core::f32;
use std::collections::HashSet;

use pumpkin_data::{BlockDirection, BlockState};
use pumpkin_util::{
    math::{lerp, position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    block::BlockStateCodec,
    generation::{height_limit::HeightLimitView, rule::RuleTest},
    world::BlockRegistryExt,
};

#[derive(Deserialize)]
pub struct OreFeature {
    size: i32,
    discard_chance_on_air_exposure: f32,
    targets: Vec<OreTarget>,
}

#[derive(Deserialize)]
struct OreTarget {
    pub target: RuleTest,
    pub state: BlockStateCodec,
}

impl OreFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk,
        _block_registry: &dyn BlockRegistryExt,
        _min_y: i8,
        _height: u16,
        _feature: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let f = random.next_f32() * f32::consts::PI;
        let g = self.size as f32 / 8.0f32;
        let i = ((self.size as f32 / 16.0f32 * 2.0 + 1.0) / 2.0).ceil() as i32;

        let d = pos.0.x as f64 + f.sin() as f64 * g as f64;
        let e = pos.0.x as f64 - f.sin() as f64 * g as f64;
        let h = pos.0.z as f64 + f.cos() as f64 * g as f64; // Use f.cos() for Math.cos(f)
        let j = pos.0.z as f64 - f.cos() as f64 * g as f64;

        let _k = 2; // This variable 'k' from Java seems unused
        let l = pos.0.y as f64 + random.next_bounded_i32(3) as f64 - 2.0;
        let m = pos.0.y as f64 + random.next_bounded_i32(3) as f64 - 2.0;

        let n = pos.0.x - g.ceil() as i32 - i;
        let o = pos.0.y - 2 - i;
        let p = pos.0.z - g.ceil() as i32 - i;
        let q = 2 * (g.ceil() as i32 + i);
        let r = 2 * (2 + i);

        for _ in n..=(n + q) {
            for _ in p..=(p + q) {
                if o > chunk.ocean_floor_height_exclusive(&pos.0.to_vec2_i32()) as i32 {
                    continue;
                }
                return self.generate_vein_part(chunk, random, d, e, h, j, l, m, n, o, p, q, r);
            }
        }
        false
    }

    #[expect(clippy::too_many_arguments)]
    fn generate_vein_part(
        &self,
        chunk: &mut ProtoChunk,
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
        let mut bit_set = HashSet::new();
        let mut mutable_pos = BlockPos::ZERO;
        let j = self.size;
        let mut ds = vec![0.0; (j * 4) as usize];
        for k in 0..j {
            let f = k as f32 / j as f32;
            let d = lerp(f as f64, start_x, end_x);
            let e = lerp(f as f64, start_y, end_y);
            let g = lerp(f as f64, start_z, end_z);
            let h = random.next_f64() * j as f64 / 16.0;
            let l = (((f32::consts::PI * f).sin() + 1.0) * h as f32 + 1.0) / 2.0;

            ds[k as usize * 4] = d;
            ds[k as usize * 4 + 1] = e;
            ds[k as usize * 4 + 2] = g;
            ds[k as usize * 4 + 3] = l as f64;
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
                    for aa_val in p_bound..=s_bound {
                        let ab_val = (aa_val as f64 + 0.5 - h_val) / d_val;
                        if u_val * u_val + w_val * w_val + ab_val * ab_val >= 1.0 {
                            continue;
                        }
                        if chunk.out_of_height(v_val as i16) {
                            continue;
                        }

                        let ac = (t_val - x_bound)
                            + (v_val - y_bound) * horizontal_size
                            + (aa_val - z_bound) * horizontal_size * vertical_size;

                        if bit_set.contains(&ac) {
                            continue;
                        }
                        bit_set.insert(ac);

                        mutable_pos.0.x = t_val;
                        mutable_pos.0.y = v_val;
                        mutable_pos.0.z = aa_val;

                        // if !world.is_valid_for_set_block(&mutable_pos) {
                        //     continue;
                        // }

                        let ad = t_val;
                        let ae = v_val;
                        let af = aa_val;

                        let block_state = chunk.get_block_state(&Vector3::new(ad, ae, af));

                        for target in &self.targets {
                            if self.should_place(
                                chunk,
                                block_state.to_state(),
                                random,
                                target,
                                &mut mutable_pos,
                            ) {
                                chunk.set_block_state(
                                    &Vector3::new(ad, ae, af),
                                    target.state.get_state(),
                                );
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

    fn should_place(
        &self,
        chunk: &mut ProtoChunk,
        state: &'static BlockState,
        random: &mut RandomGenerator,
        target: &OreTarget,
        pos: &mut BlockPos,
    ) -> bool {
        if !target.target.test(state, random) {
            return false;
        }
        if Self::should_not_discard(random, self.discard_chance_on_air_exposure) {
            return true;
        }
        !Self::is_exposed_to_air(chunk, pos)
    }

    fn should_not_discard(random: &mut RandomGenerator, chance: f32) -> bool {
        if chance <= 0.0f32 {
            return true;
        }
        if chance >= 1.0f32 {
            return false;
        }
        random.next_f32() >= chance
    }

    fn is_exposed_to_air(chunk: &mut ProtoChunk, pos: &BlockPos) -> bool {
        for dir in BlockDirection::all() {
            if chunk
                .get_block_state(&pos.offset(dir.to_offset()).0)
                .to_state()
                .is_air()
            {
                return true;
            }
        }
        false
    }
}
