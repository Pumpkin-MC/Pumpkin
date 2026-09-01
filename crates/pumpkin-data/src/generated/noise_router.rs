/* This file is generated. Do not edit manually. */
use crate::chunk::DoublePerlinNoiseParameters;
pub trait NoiseEvaluationContext {
    fn sample_noise(
        &mut self,
        noise_id: DoublePerlinNoiseParameters,
        x: f32,
        y: f32,
        z: f32,
    ) -> f32;
    fn sample_shift_a(
        &mut self,
        noise_id: DoublePerlinNoiseParameters,
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
    ) -> f32;
    fn sample_shift_b(
        &mut self,
        noise_id: DoublePerlinNoiseParameters,
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
    ) -> f32;
    fn sample_shifted_noise(
        &mut self,
        noise_id: DoublePerlinNoiseParameters,
        shift_x: f32,
        shift_y: f32,
        shift_z: f32,
        xz_scale: f32,
        y_scale: f32,
    ) -> f32;
    fn sample_interpolated_noise(&mut self, pos: &pumpkin_util::math::vector3::Vector3<i32>)
    -> f32;
    fn sample_beardifier(&mut self, pos: &pumpkin_util::math::vector3::Vector3<i32>) -> f32;
    fn sample_blend_alpha(&mut self, pos: &pumpkin_util::math::vector3::Vector3<i32>) -> f32;
    fn sample_blend_offset(&mut self, pos: &pumpkin_util::math::vector3::Vector3<i32>) -> f32;
    fn sample_blend_density(
        &mut self,
        input_val: f32,
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
    ) -> f32;
    fn sample_end_islands(&mut self, pos: &pumpkin_util::math::vector3::Vector3<i32>) -> f32;
    fn sample_wrapper(
        &mut self,
        wrapper_index: usize,
        wrapper_type: WrapperType,
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        eval_input: &dyn Fn(&pumpkin_util::math::vector3::Vector3<i32>, &mut Self) -> f32,
    ) -> f32;
    fn sample_spline(
        &mut self,
        spline_index: usize,
        location_value: f32,
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
    ) -> f32;
    fn sample_find_top_surface(
        &mut self,
        density_fn: &dyn Fn(&pumpkin_util::math::vector3::Vector3<i32>, &mut Self) -> f32,
        upper_bound_fn: &dyn Fn(&pumpkin_util::math::vector3::Vector3<i32>, &mut Self) -> f32,
        lower_bound: i32,
        cell_height: i32,
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
    ) -> f32;
}
pub mod overworld_compiled {
    use super::*;
    #[inline(always)]
    pub fn eval_overworld_0<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = ctx;
        let coord = pos.y;
        let rel = coord.clamp(-64i32, -40i32) - -64i32;
        0f32 + rel as f32 * 0.041666668f32
    }
    #[inline(always)]
    pub fn eval_overworld_1<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        0.1171875f32
    }
    #[inline(always)]
    pub fn eval_overworld_2<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = ctx;
        let coord = pos.y;
        let rel = coord.clamp(240i32, 256i32) - 240i32;
        1f32 + rel as f32 * -0.0625f32
    }
    #[inline(always)]
    pub fn eval_overworld_3<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        -0.078125f32
    }
    #[inline(always)]
    pub fn eval_overworld_4<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = ctx;
        let coord = pos.y;
        let rel = coord.clamp(-64i32, 320i32) - -64i32;
        1.5f32 + rel as f32 * -0.0078125f32
    }
    #[inline(always)]
    pub fn eval_overworld_5<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_blend_alpha(pos)
    }
    #[inline(always)]
    pub fn eval_overworld_6<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_blend_offset(pos)
    }
    #[inline(always)]
    pub fn eval_overworld_7<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_shift_a(DoublePerlinNoiseParameters::OFFSET, pos)
    }
    #[inline(always)]
    pub fn eval_overworld_8<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(8usize, WrapperType::CacheOnce, pos, &eval_overworld_7)
    }
    #[inline(always)]
    pub fn eval_overworld_9<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        0f32
    }
    #[inline(always)]
    pub fn eval_overworld_10<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_shift_b(DoublePerlinNoiseParameters::OFFSET, pos)
    }
    #[inline(always)]
    pub fn eval_overworld_11<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(11usize, WrapperType::CacheOnce, pos, &eval_overworld_10)
    }
    #[inline(always)]
    pub fn eval_overworld_12<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let sx = eval_overworld_8(pos, ctx);
        let sy = eval_overworld_9(pos, ctx);
        let sz = eval_overworld_11(pos, ctx);
        ctx.sample_shifted_noise(
            DoublePerlinNoiseParameters::CONTINENTALNESS,
            sx,
            sy,
            sz,
            0.25f32,
            0f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_13<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(13usize, WrapperType::CacheOnce, pos, &eval_overworld_12)
    }
    #[inline(always)]
    pub fn eval_overworld_14<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let sx = eval_overworld_8(pos, ctx);
        let sy = eval_overworld_9(pos, ctx);
        let sz = eval_overworld_11(pos, ctx);
        ctx.sample_shifted_noise(
            DoublePerlinNoiseParameters::EROSION,
            sx,
            sy,
            sz,
            0.25f32,
            0f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_15<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(15usize, WrapperType::CacheOnce, pos, &eval_overworld_14)
    }
    #[inline(always)]
    pub fn eval_overworld_16<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let sx = eval_overworld_8(pos, ctx);
        let sy = eval_overworld_9(pos, ctx);
        let sz = eval_overworld_11(pos, ctx);
        ctx.sample_shifted_noise(
            DoublePerlinNoiseParameters::RIDGE,
            sx,
            sy,
            sz,
            0.25f32,
            0f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_17<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(17usize, WrapperType::CacheOnce, pos, &eval_overworld_16)
    }
    #[inline(always)]
    pub fn eval_overworld_18<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_17(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_19<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_18(pos, ctx) + -0.6666667f32
    }
    #[inline(always)]
    pub fn eval_overworld_20<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_19(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_21<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_20(pos, ctx) + -0.33333334f32
    }
    #[inline(always)]
    pub fn eval_overworld_22<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_21(pos, ctx) * -3f32
    }
    #[inline(always)]
    pub fn eval_overworld_23<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let location_val = eval_overworld_13(pos, ctx);
        ctx.sample_spline(23usize, location_val, pos)
    }
    #[inline(always)]
    pub fn eval_overworld_24<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_23(pos, ctx) + -0.50375f32
    }
    #[inline(always)]
    pub fn eval_overworld_25<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let a = eval_overworld_5(pos, ctx);
        let f = eval_overworld_6(pos, ctx);
        let s = eval_overworld_24(pos, ctx);
        f + a * (s - f)
    }
    #[inline(always)]
    pub fn eval_overworld_26<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(26usize, WrapperType::CacheOnce, pos, &eval_overworld_25)
    }
    #[inline(always)]
    pub fn eval_overworld_27<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_4(pos, ctx) + eval_overworld_26(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_28<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let location_val = eval_overworld_13(pos, ctx);
        ctx.sample_spline(28usize, location_val, pos)
    }
    #[inline(always)]
    pub fn eval_overworld_29<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let a = eval_overworld_5(pos, ctx);
        let f = eval_overworld_9(pos, ctx);
        let s = eval_overworld_28(pos, ctx);
        f + a * (s - f)
    }
    #[inline(always)]
    pub fn eval_overworld_30<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(30usize, WrapperType::CacheOnce, pos, &eval_overworld_29)
    }
    #[inline(always)]
    pub fn eval_overworld_31<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::JAGGED,
            pos.x as f32 * 1500f32,
            pos.y as f32 * 0f32,
            pos.z as f32 * 1500f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_32<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let v = eval_overworld_31(pos, ctx);
        if v > 0.0 { v } else { v * 0.5 }
    }
    #[inline(always)]
    pub fn eval_overworld_33<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_30(pos, ctx) * eval_overworld_32(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_34<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(34usize, WrapperType::CacheOnce, pos, &eval_overworld_33)
    }
    #[inline(always)]
    pub fn eval_overworld_35<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_27(pos, ctx) + eval_overworld_34(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_36<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        10f32
    }
    #[inline(always)]
    pub fn eval_overworld_37<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let location_val = eval_overworld_13(pos, ctx);
        ctx.sample_spline(37usize, location_val, pos)
    }
    #[inline(always)]
    pub fn eval_overworld_38<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let a = eval_overworld_5(pos, ctx);
        let f = eval_overworld_36(pos, ctx);
        let s = eval_overworld_37(pos, ctx);
        f + a * (s - f)
    }
    #[inline(always)]
    pub fn eval_overworld_39<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(39usize, WrapperType::CacheOnce, pos, &eval_overworld_38)
    }
    #[inline(always)]
    pub fn eval_overworld_40<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_35(pos, ctx) * eval_overworld_39(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_41<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let v = eval_overworld_40(pos, ctx);
        if v > 0.0 { v } else { v * 0.25 }
    }
    #[inline(always)]
    pub fn eval_overworld_42<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_41(pos, ctx) * 4f32
    }
    #[inline(always)]
    pub fn eval_overworld_43<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_interpolated_noise(pos)
    }
    #[inline(always)]
    pub fn eval_overworld_44<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_42(pos, ctx) + eval_overworld_43(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_45<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(45usize, WrapperType::CacheOnce, pos, &eval_overworld_44)
    }
    #[inline(always)]
    pub fn eval_overworld_46<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::CAVE_ENTRANCE,
            pos.x as f32 * 0.75f32,
            pos.y as f32 * 0.5f32,
            pos.z as f32 * 0.75f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_47<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_46(pos, ctx) + 0.37f32
    }
    #[inline(always)]
    pub fn eval_overworld_48<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = ctx;
        let coord = pos.y;
        let rel = coord.clamp(-10i32, 30i32) - -10i32;
        0.3f32 + rel as f32 * -0.0075000003f32
    }
    #[inline(always)]
    pub fn eval_overworld_49<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_47(pos, ctx) + eval_overworld_48(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_50<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_ROUGHNESS_MODULATOR,
            pos.x as f32 * 1f32,
            pos.y as f32 * 1f32,
            pos.z as f32 * 1f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_51<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_50(pos, ctx) * -0.05f32
    }
    #[inline(always)]
    pub fn eval_overworld_52<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_51(pos, ctx) + -0.05f32
    }
    #[inline(always)]
    pub fn eval_overworld_53<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_ROUGHNESS,
            pos.x as f32 * 1f32,
            pos.y as f32 * 1f32,
            pos.z as f32 * 1f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_54<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_53(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_55<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_54(pos, ctx) + -0.4f32
    }
    #[inline(always)]
    pub fn eval_overworld_56<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_52(pos, ctx) * eval_overworld_55(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_57<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(57usize, WrapperType::CacheOnce, pos, &eval_overworld_56)
    }
    #[inline(always)]
    pub fn eval_overworld_58<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_RARITY,
            pos.x as f32 * 2f32,
            pos.y as f32 * 1f32,
            pos.z as f32 * 2f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_59<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(59usize, WrapperType::CacheOnce, pos, &eval_overworld_58)
    }
    #[inline(always)]
    pub fn eval_overworld_60<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
            pos.x as f32 * 1.3333334f32,
            pos.y as f32 * 1.3333334f32,
            pos.z as f32 * 1.3333334f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_61<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_60(pos, ctx) * 0.75f32
    }
    #[inline(always)]
    pub fn eval_overworld_62<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
            pos.x as f32 * 1f32,
            pos.y as f32 * 1f32,
            pos.z as f32 * 1f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_63<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_62(pos, ctx) * 1f32
    }
    #[inline(always)]
    pub fn eval_overworld_64<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
            pos.x as f32 * 0.6666667f32,
            pos.y as f32 * 0.6666667f32,
            pos.z as f32 * 0.6666667f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_65<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_64(pos, ctx) * 1.5f32
    }
    #[inline(always)]
    pub fn eval_overworld_66<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
            pos.x as f32 * 0.5f32,
            pos.y as f32 * 0.5f32,
            pos.z as f32 * 0.5f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_67<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_66(pos, ctx) * 2f32
    }
    #[inline(always)]
    pub fn eval_overworld_68<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let input_val = eval_overworld_59(pos, ctx);
        let thresholds = &[-0.5f32, 0f32, 0.5f32];
        let mut selected = thresholds.len();
        for (i, &t) in thresholds.iter().enumerate() {
            if input_val < t {
                selected = i;
                break;
            }
        }
        match selected {
            0usize => eval_overworld_61(pos, ctx),
            1usize => eval_overworld_63(pos, ctx),
            2usize => eval_overworld_65(pos, ctx),
            _ => eval_overworld_67(pos, ctx),
        }
    }
    #[inline(always)]
    pub fn eval_overworld_69<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_68(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_70<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
            pos.x as f32 * 1.3333334f32,
            pos.y as f32 * 1.3333334f32,
            pos.z as f32 * 1.3333334f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_71<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_70(pos, ctx) * 0.75f32
    }
    #[inline(always)]
    pub fn eval_overworld_72<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
            pos.x as f32 * 1f32,
            pos.y as f32 * 1f32,
            pos.z as f32 * 1f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_73<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_72(pos, ctx) * 1f32
    }
    #[inline(always)]
    pub fn eval_overworld_74<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
            pos.x as f32 * 0.6666667f32,
            pos.y as f32 * 0.6666667f32,
            pos.z as f32 * 0.6666667f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_75<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_74(pos, ctx) * 1.5f32
    }
    #[inline(always)]
    pub fn eval_overworld_76<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
            pos.x as f32 * 0.5f32,
            pos.y as f32 * 0.5f32,
            pos.z as f32 * 0.5f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_77<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_76(pos, ctx) * 2f32
    }
    #[inline(always)]
    pub fn eval_overworld_78<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let input_val = eval_overworld_59(pos, ctx);
        let thresholds = &[-0.5f32, 0f32, 0.5f32];
        let mut selected = thresholds.len();
        for (i, &t) in thresholds.iter().enumerate() {
            if input_val < t {
                selected = i;
                break;
            }
        }
        match selected {
            0usize => eval_overworld_71(pos, ctx),
            1usize => eval_overworld_73(pos, ctx),
            2usize => eval_overworld_75(pos, ctx),
            _ => eval_overworld_77(pos, ctx),
        }
    }
    #[inline(always)]
    pub fn eval_overworld_79<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_78(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_80<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_69(pos, ctx).max(eval_overworld_79(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_overworld_81<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_THICKNESS,
            pos.x as f32 * 1f32,
            pos.y as f32 * 1f32,
            pos.z as f32 * 1f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_82<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_81(pos, ctx) * -0.011500001f32
    }
    #[inline(always)]
    pub fn eval_overworld_83<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_82(pos, ctx) + -0.0765f32
    }
    #[inline(always)]
    pub fn eval_overworld_84<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_80(pos, ctx) + eval_overworld_83(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_85<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_84(pos, ctx).clamp(-1f32, 1f32)
    }
    #[inline(always)]
    pub fn eval_overworld_86<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_57(pos, ctx) + eval_overworld_85(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_87<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_49(pos, ctx).min(eval_overworld_86(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_overworld_88<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(88usize, WrapperType::CacheOnce, pos, &eval_overworld_87)
    }
    #[inline(always)]
    pub fn eval_overworld_89<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_88(pos, ctx) * 5f32
    }
    #[inline(always)]
    pub fn eval_overworld_90<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_45(pos, ctx).min(eval_overworld_89(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_overworld_91<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::CAVE_LAYER,
            pos.x as f32 * 1f32,
            pos.y as f32 * 8f32,
            pos.z as f32 * 1f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_92<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let v = eval_overworld_91(pos, ctx);
        v * v
    }
    #[inline(always)]
    pub fn eval_overworld_93<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_92(pos, ctx) * 4f32
    }
    #[inline(always)]
    pub fn eval_overworld_94<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::CAVE_CHEESE,
            pos.x as f32 * 1f32,
            pos.y as f32 * 0.6666667f32,
            pos.z as f32 * 1f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_95<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_94(pos, ctx) + 0.27f32
    }
    #[inline(always)]
    pub fn eval_overworld_96<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_95(pos, ctx).clamp(-1f32, 1f32)
    }
    #[inline(always)]
    pub fn eval_overworld_97<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_45(pos, ctx) * -0.64f32
    }
    #[inline(always)]
    pub fn eval_overworld_98<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_97(pos, ctx) + 1.5f32
    }
    #[inline(always)]
    pub fn eval_overworld_99<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_98(pos, ctx).clamp(0f32, 0.5f32)
    }
    #[inline(always)]
    pub fn eval_overworld_100<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_96(pos, ctx) + eval_overworld_99(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_101<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_93(pos, ctx) + eval_overworld_100(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_102<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_101(pos, ctx).min(eval_overworld_88(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_overworld_103<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D_MODULATOR,
            pos.x as f32 * 2f32,
            pos.y as f32 * 1f32,
            pos.z as f32 * 2f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_104<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D,
            pos.x as f32 * 2f32,
            pos.y as f32 * 2f32,
            pos.z as f32 * 2f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_105<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_104(pos, ctx) * 0.5f32
    }
    #[inline(always)]
    pub fn eval_overworld_106<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D,
            pos.x as f32 * 1.3333334f32,
            pos.y as f32 * 1.3333334f32,
            pos.z as f32 * 1.3333334f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_107<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_106(pos, ctx) * 0.75f32
    }
    #[inline(always)]
    pub fn eval_overworld_108<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D,
            pos.x as f32 * 1f32,
            pos.y as f32 * 1f32,
            pos.z as f32 * 1f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_109<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_108(pos, ctx) * 1f32
    }
    #[inline(always)]
    pub fn eval_overworld_110<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D,
            pos.x as f32 * 0.5f32,
            pos.y as f32 * 0.5f32,
            pos.z as f32 * 0.5f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_111<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_110(pos, ctx) * 2f32
    }
    #[inline(always)]
    pub fn eval_overworld_112<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D,
            pos.x as f32 * 0.33333334f32,
            pos.y as f32 * 0.33333334f32,
            pos.z as f32 * 0.33333334f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_113<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_112(pos, ctx) * 3f32
    }
    #[inline(always)]
    pub fn eval_overworld_114<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let input_val = eval_overworld_103(pos, ctx);
        let thresholds = &[-0.75f32, -0.5f32, 0.5f32, 0.75f32];
        let mut selected = thresholds.len();
        for (i, &t) in thresholds.iter().enumerate() {
            if input_val < t {
                selected = i;
                break;
            }
        }
        match selected {
            0usize => eval_overworld_105(pos, ctx),
            1usize => eval_overworld_107(pos, ctx),
            2usize => eval_overworld_109(pos, ctx),
            3usize => eval_overworld_111(pos, ctx),
            _ => eval_overworld_113(pos, ctx),
        }
    }
    #[inline(always)]
    pub fn eval_overworld_115<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_114(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_116<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D_THICKNESS,
            pos.x as f32 * 2f32,
            pos.y as f32 * 1f32,
            pos.z as f32 * 2f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_117<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_116(pos, ctx) * -0.34999996f32
    }
    #[inline(always)]
    pub fn eval_overworld_118<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_117(pos, ctx) + -0.95f32
    }
    #[inline(always)]
    pub fn eval_overworld_119<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(119usize, WrapperType::CacheOnce, pos, &eval_overworld_118)
    }
    #[inline(always)]
    pub fn eval_overworld_120<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_119(pos, ctx) * 0.083f32
    }
    #[inline(always)]
    pub fn eval_overworld_121<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_115(pos, ctx) + eval_overworld_120(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_122<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D_ELEVATION,
            pos.x as f32 * 1f32,
            pos.y as f32 * 0f32,
            pos.z as f32 * 1f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_123<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_122(pos, ctx) * 8f32
    }
    #[inline(always)]
    pub fn eval_overworld_124<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(124usize, WrapperType::CacheOnce, pos, &eval_overworld_123)
    }
    #[inline(always)]
    pub fn eval_overworld_125<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = ctx;
        let coord = pos.y;
        let rel = coord.clamp(-64i32, 320i32) - -64i32;
        8f32 + rel as f32 * -0.125f32
    }
    #[inline(always)]
    pub fn eval_overworld_126<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_124(pos, ctx) + eval_overworld_125(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_127<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_126(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_128<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_127(pos, ctx) + eval_overworld_119(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_129<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let v = eval_overworld_128(pos, ctx);
        v * v * v
    }
    #[inline(always)]
    pub fn eval_overworld_130<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_121(pos, ctx).max(eval_overworld_129(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_overworld_131<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_130(pos, ctx).clamp(-1f32, 1f32)
    }
    #[inline(always)]
    pub fn eval_overworld_132<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_131(pos, ctx) + eval_overworld_57(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_133<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_102(pos, ctx).min(eval_overworld_132(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_overworld_134<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::PILLAR,
            pos.x as f32 * 25f32,
            pos.y as f32 * 0.3f32,
            pos.z as f32 * 25f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_135<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_134(pos, ctx) * 2f32
    }
    #[inline(always)]
    pub fn eval_overworld_136<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::PILLAR_RARENESS,
            pos.x as f32 * 1f32,
            pos.y as f32 * 1f32,
            pos.z as f32 * 1f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_137<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_136(pos, ctx) * -1f32
    }
    #[inline(always)]
    pub fn eval_overworld_138<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_137(pos, ctx) + -1f32
    }
    #[inline(always)]
    pub fn eval_overworld_139<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_135(pos, ctx) + eval_overworld_138(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_140<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::PILLAR_THICKNESS,
            pos.x as f32 * 1f32,
            pos.y as f32 * 1f32,
            pos.z as f32 * 1f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_141<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_140(pos, ctx) * 0.55f32
    }
    #[inline(always)]
    pub fn eval_overworld_142<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_141(pos, ctx) + 0.55f32
    }
    #[inline(always)]
    pub fn eval_overworld_143<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let v = eval_overworld_142(pos, ctx);
        v * v * v
    }
    #[inline(always)]
    pub fn eval_overworld_144<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_139(pos, ctx) * eval_overworld_143(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_145<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(145usize, WrapperType::CacheOnce, pos, &eval_overworld_144)
    }
    #[inline(always)]
    pub fn eval_overworld_146<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        -1000000f32
    }
    #[inline(always)]
    pub fn eval_overworld_147<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_145(pos, ctx);
        if val >= -1000000f32 && val < 0.03f32 {
            eval_overworld_146(pos, ctx)
        } else {
            eval_overworld_145(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_148<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_133(pos, ctx).max(eval_overworld_147(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_overworld_149<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_45(pos, ctx);
        if val >= -1000000f32 && val < 1.5625f32 {
            eval_overworld_90(pos, ctx)
        } else {
            eval_overworld_148(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_150<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let a = eval_overworld_2(pos, ctx);
        let f = eval_overworld_3(pos, ctx);
        let s = eval_overworld_149(pos, ctx);
        f + a * (s - f)
    }
    #[inline(always)]
    pub fn eval_overworld_151<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let a = eval_overworld_0(pos, ctx);
        let f = eval_overworld_1(pos, ctx);
        let s = eval_overworld_150(pos, ctx);
        f + a * (s - f)
    }
    #[inline(always)]
    pub fn eval_overworld_152<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_151(pos, ctx);
        ctx.sample_blend_density(val, pos)
    }
    #[inline(always)]
    pub fn eval_overworld_153<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_152(pos, ctx) * 0.64f32
    }
    #[inline(always)]
    pub fn eval_overworld_154<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(
            154usize,
            WrapperType::Interpolated,
            pos,
            &eval_overworld_153,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_155<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let c = eval_overworld_154(pos, ctx).clamp(-1.0, 1.0);
        c / 2.0 - c * c * c / 24.0
    }
    #[inline(always)]
    pub fn eval_overworld_156<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = ctx;
        let coord = pos.y;
        let rel = coord.clamp(-4064i32, 4062i32) - -4064i32;
        -4064f32 + rel as f32 * 1f32
    }
    #[inline(always)]
    pub fn eval_overworld_157<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::NOODLE,
            pos.x as f32 * 1f32,
            pos.y as f32 * 1f32,
            pos.z as f32 * 1f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_158<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        -1f32
    }
    #[inline(always)]
    pub fn eval_overworld_159<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_156(pos, ctx);
        if val >= -60f32 && val < 321f32 {
            eval_overworld_157(pos, ctx)
        } else {
            eval_overworld_158(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_160<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(
            160usize,
            WrapperType::Interpolated,
            pos,
            &eval_overworld_159,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_161<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        64f32
    }
    #[inline(always)]
    pub fn eval_overworld_162<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::NOODLE_THICKNESS,
            pos.x as f32 * 1f32,
            pos.y as f32 * 1f32,
            pos.z as f32 * 1f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_163<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_162(pos, ctx) * -0.025f32
    }
    #[inline(always)]
    pub fn eval_overworld_164<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_163(pos, ctx) + -0.075f32
    }
    #[inline(always)]
    pub fn eval_overworld_165<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_156(pos, ctx);
        if val >= -60f32 && val < 321f32 {
            eval_overworld_164(pos, ctx)
        } else {
            eval_overworld_9(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_166<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(
            166usize,
            WrapperType::Interpolated,
            pos,
            &eval_overworld_165,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_167<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::NOODLE_RIDGE_A,
            pos.x as f32 * 2.6666667f32,
            pos.y as f32 * 2.6666667f32,
            pos.z as f32 * 2.6666667f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_168<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_156(pos, ctx);
        if val >= -60f32 && val < 321f32 {
            eval_overworld_167(pos, ctx)
        } else {
            eval_overworld_9(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_169<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(
            169usize,
            WrapperType::Interpolated,
            pos,
            &eval_overworld_168,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_170<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_169(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_171<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::NOODLE_RIDGE_B,
            pos.x as f32 * 2.6666667f32,
            pos.y as f32 * 2.6666667f32,
            pos.z as f32 * 2.6666667f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_172<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_156(pos, ctx);
        if val >= -60f32 && val < 321f32 {
            eval_overworld_171(pos, ctx)
        } else {
            eval_overworld_9(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_173<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(
            173usize,
            WrapperType::Interpolated,
            pos,
            &eval_overworld_172,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_174<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_173(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_175<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_170(pos, ctx).max(eval_overworld_174(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_overworld_176<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_175(pos, ctx) * 1.5f32
    }
    #[inline(always)]
    pub fn eval_overworld_177<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_166(pos, ctx) + eval_overworld_176(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_178<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_160(pos, ctx);
        if val >= -1000000f32 && val < 0f32 {
            eval_overworld_161(pos, ctx)
        } else {
            eval_overworld_177(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_179<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_155(pos, ctx).min(eval_overworld_178(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_overworld_180<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_beardifier(pos)
    }
    #[inline(always)]
    pub fn eval_overworld_181<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_179(pos, ctx) + eval_overworld_180(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_182<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(182usize, WrapperType::CellCache, pos, &eval_overworld_181)
    }
    #[inline(always)]
    pub fn eval_overworld_183<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::AQUIFER_BARRIER,
            pos.x as f32 * 1f32,
            pos.y as f32 * 0.5f32,
            pos.z as f32 * 1f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_184<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::AQUIFER_FLUID_LEVEL_FLOODEDNESS,
            pos.x as f32 * 1f32,
            pos.y as f32 * 0.67f32,
            pos.z as f32 * 1f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_185<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::AQUIFER_FLUID_LEVEL_SPREAD,
            pos.x as f32 * 1f32,
            pos.y as f32 * 0.71428573f32,
            pos.z as f32 * 1f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_186<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::AQUIFER_LAVA,
            pos.x as f32 * 1f32,
            pos.y as f32 * 1f32,
            pos.z as f32 * 1f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_187<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::ORE_VEININESS,
            pos.x as f32 * 1.5f32,
            pos.y as f32 * 1.5f32,
            pos.z as f32 * 1.5f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_188<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_156(pos, ctx);
        if val >= -64f32 && val < 57f32 {
            eval_overworld_187(pos, ctx)
        } else {
            eval_overworld_9(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_189<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(
            189usize,
            WrapperType::Interpolated,
            pos,
            &eval_overworld_188,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_190<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(190usize, WrapperType::CacheOnce, pos, &eval_overworld_189)
    }
    #[inline(always)]
    pub fn eval_overworld_191<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        0.08f32
    }
    #[inline(always)]
    pub fn eval_overworld_192<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::ORE_VEIN_A,
            pos.x as f32 * 4f32,
            pos.y as f32 * 4f32,
            pos.z as f32 * 4f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_193<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        1f32
    }
    #[inline(always)]
    pub fn eval_overworld_194<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_156(pos, ctx);
        if val >= -64f32 && val < 57f32 {
            eval_overworld_192(pos, ctx)
        } else {
            eval_overworld_193(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_195<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(
            195usize,
            WrapperType::Interpolated,
            pos,
            &eval_overworld_194,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_196<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_195(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_197<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::ORE_VEIN_B,
            pos.x as f32 * 4f32,
            pos.y as f32 * 4f32,
            pos.z as f32 * 4f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_198<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_156(pos, ctx);
        if val >= -64f32 && val < 57f32 {
            eval_overworld_197(pos, ctx)
        } else {
            eval_overworld_193(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_199<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(
            199usize,
            WrapperType::Interpolated,
            pos,
            &eval_overworld_198,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_200<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_199(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_201<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_196(pos, ctx).max(eval_overworld_200(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_overworld_202<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_191(pos, ctx) - eval_overworld_201(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_203<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_190(pos, ctx);
        if val >= -0.4f32 && val < 0.4f32 {
            eval_overworld_158(pos, ctx)
        } else {
            eval_overworld_202(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_204<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(204usize, WrapperType::CacheOnce, pos, &eval_overworld_203)
    }
    #[inline(always)]
    pub fn eval_overworld_205<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        -0.3f32
    }
    #[inline(always)]
    pub fn eval_overworld_206<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::ORE_GAP,
            pos.x as f32 * 1f32,
            pos.y as f32 * 1f32,
            pos.z as f32 * 1f32,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_207<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_205(pos, ctx) - eval_overworld_206(pos, ctx)
    }
}
pub mod nether_compiled {
    use super::*;
    #[inline(always)]
    pub fn eval_nether_0<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = ctx;
        let coord = pos.y;
        let rel = coord.clamp(-8i32, 24i32) - -8i32;
        0f32 + rel as f32 * 0.03125f32
    }
    #[inline(always)]
    pub fn eval_nether_1<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        2.5f32
    }
    #[inline(always)]
    pub fn eval_nether_2<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = ctx;
        let coord = pos.y;
        let rel = coord.clamp(104i32, 128i32) - 104i32;
        1f32 + rel as f32 * -0.041666668f32
    }
    #[inline(always)]
    pub fn eval_nether_3<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        0.9375f32
    }
    #[inline(always)]
    pub fn eval_nether_4<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_interpolated_noise(pos)
    }
    #[inline(always)]
    pub fn eval_nether_5<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let a = eval_nether_2(pos, ctx);
        let f = eval_nether_3(pos, ctx);
        let s = eval_nether_4(pos, ctx);
        f + a * (s - f)
    }
    #[inline(always)]
    pub fn eval_nether_6<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let a = eval_nether_0(pos, ctx);
        let f = eval_nether_1(pos, ctx);
        let s = eval_nether_5(pos, ctx);
        f + a * (s - f)
    }
    #[inline(always)]
    pub fn eval_nether_7<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_nether_6(pos, ctx);
        ctx.sample_blend_density(val, pos)
    }
    #[inline(always)]
    pub fn eval_nether_8<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_nether_7(pos, ctx) * 0.64f32
    }
    #[inline(always)]
    pub fn eval_nether_9<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(9usize, WrapperType::Interpolated, pos, &eval_nether_8)
    }
    #[inline(always)]
    pub fn eval_nether_10<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let c = eval_nether_9(pos, ctx).clamp(-1.0, 1.0);
        c / 2.0 - c * c * c / 24.0
    }
    #[inline(always)]
    pub fn eval_nether_11<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_beardifier(pos)
    }
    #[inline(always)]
    pub fn eval_nether_12<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_nether_10(pos, ctx) + eval_nether_11(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_nether_13<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(13usize, WrapperType::CellCache, pos, &eval_nether_12)
    }
    #[inline(always)]
    pub fn eval_nether_14<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        0f32
    }
}
pub mod end_compiled {
    use super::*;
    #[inline(always)]
    pub fn eval_end_0<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = ctx;
        let coord = pos.y;
        let rel = coord.clamp(4i32, 32i32) - 4i32;
        0f32 + rel as f32 * 0.035714287f32
    }
    #[inline(always)]
    pub fn eval_end_1<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        -0.234375f32
    }
    #[inline(always)]
    pub fn eval_end_2<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = ctx;
        let coord = pos.y;
        let rel = coord.clamp(56i32, 312i32) - 56i32;
        1f32 + rel as f32 * -0.00390625f32
    }
    #[inline(always)]
    pub fn eval_end_3<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        -23.4375f32
    }
    #[inline(always)]
    pub fn eval_end_4<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        100f32
    }
    #[inline(always)]
    pub fn eval_end_5<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = ctx;
        let dx = (pos.x - 0i32) as f32;
        let dy = (pos.y - 0i32) as f32;
        let dz = (pos.z - 0i32) as f32;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
    #[inline(always)]
    pub fn eval_end_6<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_end_4(pos, ctx) - eval_end_5(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_end_7<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_end_6(pos, ctx).clamp(-100f32, 80f32)
    }
    #[inline(always)]
    pub fn eval_end_8<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        8f32
    }
    #[inline(always)]
    pub fn eval_end_9<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_end_7(pos, ctx) - eval_end_8(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_end_10<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_end_9(pos, ctx) * 0.0078125f32
    }
    #[inline(always)]
    pub fn eval_end_11<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let slice_pos = pumpkin_util::math::vector3::Vector3::new(pos.x, 0i32, pos.z);
        eval_end_10(&slice_pos, ctx)
    }
    #[inline(always)]
    pub fn eval_end_12<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_end_islands(pos)
    }
    #[inline(always)]
    pub fn eval_end_13<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_end_11(pos, ctx).max(eval_end_12(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_end_14<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(14usize, WrapperType::CacheOnce, pos, &eval_end_13)
    }
    #[inline(always)]
    pub fn eval_end_15<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_interpolated_noise(pos)
    }
    #[inline(always)]
    pub fn eval_end_16<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_end_14(pos, ctx) + eval_end_15(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_end_17<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let a = eval_end_2(pos, ctx);
        let f = eval_end_3(pos, ctx);
        let s = eval_end_16(pos, ctx);
        f + a * (s - f)
    }
    #[inline(always)]
    pub fn eval_end_18<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let a = eval_end_0(pos, ctx);
        let f = eval_end_1(pos, ctx);
        let s = eval_end_17(pos, ctx);
        f + a * (s - f)
    }
    #[inline(always)]
    pub fn eval_end_19<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_end_18(pos, ctx);
        ctx.sample_blend_density(val, pos)
    }
    #[inline(always)]
    pub fn eval_end_20<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_end_19(pos, ctx) * 0.64f32
    }
    #[inline(always)]
    pub fn eval_end_21<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(21usize, WrapperType::Interpolated, pos, &eval_end_20)
    }
    #[inline(always)]
    pub fn eval_end_22<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let c = eval_end_21(pos, ctx).clamp(-1.0, 1.0);
        c / 2.0 - c * c * c / 24.0
    }
    #[inline(always)]
    pub fn eval_end_23<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_beardifier(pos)
    }
    #[inline(always)]
    pub fn eval_end_24<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_end_22(pos, ctx) + eval_end_23(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_end_25<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(25usize, WrapperType::CellCache, pos, &eval_end_24)
    }
    #[inline(always)]
    pub fn eval_end_26<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        0f32
    }
}
pub struct NoiseData {
    pub noise_id: DoublePerlinNoiseParameters,
    pub xz_scale: f32,
    pub y_scale: f32,
}
pub struct FindTopSurfaceData {
    pub lower_bound: i32,
    pub cell_height: i32,
}
pub struct ShiftedNoiseData {
    pub xz_scale: f32,
    pub y_scale: f32,
    pub noise_id: DoublePerlinNoiseParameters,
}
pub struct InterpolatedNoiseSamplerData {
    pub scaled_xz_scale: f32,
    pub scaled_y_scale: f32,
    pub xz_factor: f32,
    pub y_factor: f32,
    pub smear_scale_multiplier: f32,
}
pub struct ClampedYGradientData {
    pub from_y: f32,
    pub to_y: f32,
    pub from_value: f32,
    pub to_value: f32,
}
#[derive(Copy, Clone)]
pub enum Axis {
    X,
    Y,
    Z,
}
#[derive(Copy, Clone)]
pub enum Tiling {
    ClampToEdge,
    Repeat,
    MirroredRepeat,
}
pub struct GradientData {
    pub axis: Axis,
    pub tiling: Tiling,
    pub from_coordinate: i32,
    pub to_coordinate: i32,
    pub from_value: f32,
    pub to_value: f32,
}
#[derive(Copy, Clone)]
pub enum DistanceMetric {
    Euclidean,
    EuclideanSquared,
    Manhattan,
    Chebyshev,
}
pub struct DistanceToPointData {
    pub point: [i32; 3],
    pub metric: DistanceMetric,
}
#[derive(Copy, Clone)]
pub enum RoundingOperation {
    Floor,
    Round,
    Ceil,
    Truncate,
}
pub struct RoundingData {
    pub operation: RoundingOperation,
}
#[derive(Copy, Clone)]
pub enum BinaryOperation {
    Add,
    Mul,
    Min,
    Max,
    Sub,
    Div,
    Pow,
}
pub struct BinaryData {
    pub operation: BinaryOperation,
}
impl BinaryData {
    #[inline]
    #[must_use]
    pub const fn apply_density(&self, a: f32, b: f32) -> f32 {
        match self.operation {
            BinaryOperation::Add => a + b,
            BinaryOperation::Mul => a * b,
            BinaryOperation::Min => a.min(b),
            BinaryOperation::Max => a.max(b),
            BinaryOperation::Sub => a - b,
            BinaryOperation::Div => {
                if b == 0.0 {
                    0.0
                } else {
                    a / b
                }
            }
            BinaryOperation::Pow => a,
        }
    }
}
#[derive(Copy, Clone)]
pub enum LinearOperation {
    Add,
    Mul,
}
pub struct LinearData {
    pub operation: LinearOperation,
    pub argument: f32,
}
impl LinearData {
    #[inline]
    #[must_use]
    pub const fn apply_density(&self, density: f32) -> f32 {
        match self.operation {
            LinearOperation::Add => density + self.argument,
            LinearOperation::Mul => density * self.argument,
        }
    }
}
#[derive(Copy, Clone)]
pub enum UnaryOperation {
    Abs,
    Square,
    Cube,
    HalfNegative,
    QuarterNegative,
    Squeeze,
    Invert,
    Negate,
    Sqrt,
    Log,
    Sign,
}
pub struct UnaryData {
    pub operation: UnaryOperation,
}
impl UnaryData {
    #[inline]
    #[must_use]
    pub fn apply_density(&self, density: f32) -> f32 {
        match self.operation {
            UnaryOperation::Abs => density.abs(),
            UnaryOperation::Square => density * density,
            UnaryOperation::Cube => density * density * density,
            UnaryOperation::HalfNegative => {
                if density > 0.0 {
                    density
                } else {
                    density * 0.5
                }
            }
            UnaryOperation::QuarterNegative => {
                if density > 0.0 {
                    density
                } else {
                    density * 0.25
                }
            }
            UnaryOperation::Squeeze => {
                let clamped = density.clamp(-1.0, 1.0);
                clamped / 2.0 - clamped * clamped * clamped / 24.0
            }
            UnaryOperation::Invert => {
                if density == 0.0 {
                    f32::INFINITY
                } else {
                    1.0 / density
                }
            }
            UnaryOperation::Negate => -density,
            UnaryOperation::Sqrt => density.sqrt(),
            UnaryOperation::Log => density.ln(),
            UnaryOperation::Sign => {
                if density > 0.0 {
                    1.0
                } else if density < 0.0 {
                    -1.0
                } else {
                    0.0
                }
            }
        }
    }
}
pub struct ClampData {
    pub min_value: f32,
    pub max_value: f32,
}
impl ClampData {
    #[inline]
    #[must_use]
    pub const fn apply_density(&self, density: f32) -> f32 {
        density.clamp(self.min_value, self.max_value)
    }
}
pub struct RangeChoiceData {
    pub min_inclusive: f32,
    pub max_exclusive: f32,
}
pub struct SplinePoint {
    pub location: f32,
    pub value: &'static SplineRepr,
    pub derivative: f32,
}
pub enum SplineRepr {
    Standard {
        location_function_index: usize,
        points: &'static [SplinePoint],
    },
    Fixed {
        value: f32,
    },
}
#[derive(Copy, Clone)]
pub enum WrapperType {
    Interpolated,
    CacheFlat,
    Cache2D,
    CacheOnce,
    CellCache,
}
pub enum BaseNoiseFunctionComponent {
    Beardifier,
    BlendAlpha,
    BlendOffset,
    BlendDensity {
        input_index: usize,
    },
    FindTopSurface {
        density_index: usize,
        upper_bound_index: usize,
        data: &'static FindTopSurfaceData,
    },
    EndIslands,
    Noise {
        data: &'static NoiseData,
    },
    ShiftA {
        noise_id: DoublePerlinNoiseParameters,
    },
    ShiftB {
        noise_id: DoublePerlinNoiseParameters,
    },
    ShiftedNoise {
        shift_x_index: usize,
        shift_y_index: usize,
        shift_z_index: usize,
        data: &'static ShiftedNoiseData,
    },
    InterpolatedNoiseSampler {
        data: &'static InterpolatedNoiseSamplerData,
    },
    IntervalSelect {
        input_index: usize,
        thresholds: &'static [f32],
        functions_indices: &'static [usize],
    },
    Wrapper {
        input_index: usize,
        wrapper: WrapperType,
    },
    Constant {
        value: f32,
    },
    ClampedYGradient {
        data: &'static ClampedYGradientData,
    },
    Gradient {
        data: &'static GradientData,
    },
    DistanceToPoint {
        data: &'static DistanceToPointData,
    },
    Lerp {
        alpha_index: usize,
        first_index: usize,
        second_index: usize,
    },
    Rounding {
        input_index: usize,
        multiple_index: usize,
        data: &'static RoundingData,
    },
    Slice {
        input_index: usize,
        axis: Axis,
        coordinate: i32,
    },
    Binary {
        argument1_index: usize,
        argument2_index: usize,
        data: &'static BinaryData,
    },
    Linear {
        input_index: usize,
        data: &'static LinearData,
    },
    Unary {
        input_index: usize,
        data: &'static UnaryData,
    },
    Clamp {
        input_index: usize,
        data: &'static ClampData,
    },
    RangeChoice {
        input_index: usize,
        when_in_range_index: usize,
        when_out_range_index: usize,
        data: &'static RangeChoiceData,
    },
    Spline {
        spline: &'static SplineRepr,
    },
}
pub struct BaseNoiseRouter {
    pub full_component_stack: &'static [BaseNoiseFunctionComponent],
    pub barrier_noise: usize,
    pub fluid_level_floodedness_noise: usize,
    pub fluid_level_spread_noise: usize,
    pub lava_noise: usize,
    pub erosion: usize,
    pub depth: usize,
    pub final_density: usize,
    pub vein_toggle: usize,
    pub vein_ridged: usize,
    pub vein_gap: usize,
}
pub struct BaseSurfaceEstimator {
    pub full_component_stack: &'static [BaseNoiseFunctionComponent],
}
pub struct BaseMultiNoiseRouter {
    pub full_component_stack: &'static [BaseNoiseFunctionComponent],
    pub temperature: usize,
    pub vegetation: usize,
    pub continents: usize,
    pub erosion: usize,
    pub depth: usize,
    pub ridges: usize,
}
pub struct BaseNoiseRouters {
    pub noise: BaseNoiseRouter,
    pub surface_estimator: BaseSurfaceEstimator,
    pub multi_noise: BaseMultiNoiseRouter,
}
pub const OVERWORLD_BASE_NOISE_ROUTER: BaseNoiseRouters = BaseNoiseRouters {
    noise: BaseNoiseRouter {
        full_component_stack: &[
            BaseNoiseFunctionComponent::Gradient {
                data: &GradientData {
                    axis: Axis::Y,
                    tiling: Tiling::ClampToEdge,
                    from_coordinate: -64i32,
                    to_coordinate: -40i32,
                    from_value: 0f32,
                    to_value: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Constant {
                value: 0.1171875f32,
            },
            BaseNoiseFunctionComponent::Gradient {
                data: &GradientData {
                    axis: Axis::Y,
                    tiling: Tiling::ClampToEdge,
                    from_coordinate: 240i32,
                    to_coordinate: 256i32,
                    from_value: 1f32,
                    to_value: 0f32,
                },
            },
            BaseNoiseFunctionComponent::Constant {
                value: -0.078125f32,
            },
            BaseNoiseFunctionComponent::Gradient {
                data: &GradientData {
                    axis: Axis::Y,
                    tiling: Tiling::ClampToEdge,
                    from_coordinate: -64i32,
                    to_coordinate: 320i32,
                    from_value: 1.5f32,
                    to_value: -1.5f32,
                },
            },
            BaseNoiseFunctionComponent::BlendAlpha,
            BaseNoiseFunctionComponent::BlendOffset,
            BaseNoiseFunctionComponent::ShiftA {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 7usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Constant { value: 0f32 },
            BaseNoiseFunctionComponent::ShiftB {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 10usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 8usize,
                shift_y_index: 9usize,
                shift_z_index: 11usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f32,
                    y_scale: 0f32,
                    noise_id: DoublePerlinNoiseParameters::CONTINENTALNESS,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 12usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 8usize,
                shift_y_index: 9usize,
                shift_z_index: 11usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f32,
                    y_scale: 0f32,
                    noise_id: DoublePerlinNoiseParameters::EROSION,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 14usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 8usize,
                shift_y_index: 9usize,
                shift_z_index: 11usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f32,
                    y_scale: 0f32,
                    noise_id: DoublePerlinNoiseParameters::RIDGE,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 16usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 17usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 18usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.6666667f32,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 19usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 20usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.33333334f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 21usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -3f32,
                },
            },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 13usize,
                    points: &[
                        SplinePoint {
                            location: -1.1f32,
                            value: &SplineRepr::Fixed { value: 0.044f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -1.02f32,
                            value: &SplineRepr::Fixed { value: -0.2222f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.51f32,
                            value: &SplineRepr::Fixed { value: -0.2222f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.44f32,
                            value: &SplineRepr::Fixed { value: -0.12f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.18f32,
                            value: &SplineRepr::Fixed { value: -0.12f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.16f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 15usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.08880186f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.69000006f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.115760356f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.6400001f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.75f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.65f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.5954547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.6054547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.100000024f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.3f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0.1f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0.06f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.15f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 15usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.08880186f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.69000006f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.115760356f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.6400001f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.75f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.65f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.5954547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.6054547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.100000024f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.3f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0.1f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0.06f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.1f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 15usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.08880186f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.69000006f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.115760356f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.6400001f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.75f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.65f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.5954547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.6054547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.100000024f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.25f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.001f32 },
                                                    derivative: 0.01f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.003f32 },
                                                    derivative: 0.01f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0.094000004f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.12f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 0.25f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 15usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.20235021f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.7161751f32,
                                                    },
                                                    derivative: 0.5138249f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1.23f32 },
                                                    derivative: 0.5138249f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.44682026f32,
                                                    },
                                                    derivative: 0.43317974f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.88f32 },
                                                    derivative: 0.43317974f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.30829495f32,
                                                    },
                                                    derivative: 0.3917051f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.70000005f32,
                                                    },
                                                    derivative: 0.3917051f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.25f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.42000002f32,
                                                    },
                                                    derivative: 0.049000014f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.0069999998f32,
                                                    },
                                                    derivative: 0.07f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.021f32 },
                                                    derivative: 0.07f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0.658f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.42000002f32,
                                                    },
                                                    derivative: 0.049000014f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.1f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.1f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.58f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.12f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 1f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 15usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.34792626f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.9239631f32,
                                                    },
                                                    derivative: 0.5760369f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1.5f32 },
                                                    derivative: 0.5760369f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.5391705f32,
                                                    },
                                                    derivative: 0.4608295f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1f32 },
                                                    derivative: 0.4608295f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.5391705f32,
                                                    },
                                                    derivative: 0.4608295f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1f32 },
                                                    derivative: 0.4608295f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.6f32 },
                                                    derivative: 0.070000015f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0.099999994f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.099999994f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0.94f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.6f32 },
                                                    derivative: 0.070000015f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.05f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.05f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.58f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0.015f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                    ],
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 23usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.50375f32,
                },
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 5usize,
                first_index: 6usize,
                second_index: 24usize,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 25usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 4usize,
                argument2_index: 26usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 13usize,
                    points: &[
                        SplinePoint {
                            location: -0.11f32,
                            value: &SplineRepr::Fixed { value: 0f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 0.03f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 15usize,
                                points: &[
                                    SplinePoint {
                                        location: -1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.19999999f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.44999996f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.63f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.78f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.19999999f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.44999996f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.315f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.15f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.5775f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.19999999f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.44999996f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.315f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.15f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.375f32,
                                        value: &SplineRepr::Fixed { value: 0f32 },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 0.65f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 15usize,
                                points: &[
                                    SplinePoint {
                                        location: -1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.19999999f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.44999996f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.63f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.63f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.78f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.19999999f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.44999996f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.63f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.5775f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.19999999f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.44999996f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.63f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.01f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.375f32,
                                        value: &SplineRepr::Fixed { value: 0f32 },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                    ],
                },
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 5usize,
                first_index: 9usize,
                second_index: 28usize,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 29usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::JAGGED,
                    xz_scale: 1500f32,
                    y_scale: 0f32,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 31usize,
                data: &UnaryData {
                    operation: UnaryOperation::HalfNegative,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 30usize,
                argument2_index: 32usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 33usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 27usize,
                argument2_index: 34usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Constant { value: 10f32 },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 13usize,
                    points: &[
                        SplinePoint {
                            location: -0.19f32,
                            value: &SplineRepr::Fixed { value: 3.95f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.15f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 15usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.5f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.25f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.03f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.35f32,
                                        value: &SplineRepr::Fixed { value: 6.25f32 },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.25f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.25f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.62f32,
                                        value: &SplineRepr::Fixed { value: 6.25f32 },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.1f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 15usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.5f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.25f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.03f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.35f32,
                                        value: &SplineRepr::Fixed { value: 5.47f32 },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 5.47f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 5.47f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.62f32,
                                        value: &SplineRepr::Fixed { value: 5.47f32 },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 0.03f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 15usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.5f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.25f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.03f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.35f32,
                                        value: &SplineRepr::Fixed { value: 5.08f32 },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 5.08f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 5.08f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.62f32,
                                        value: &SplineRepr::Fixed { value: 5.08f32 },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 0.06f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 15usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 4.69f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.5f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 4.69f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.25f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 4.69f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.03f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 4.69f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.05f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.45f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 4.69f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.7f32,
                                                    value: &SplineRepr::Fixed { value: 1.56f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.45f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 4.69f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.7f32,
                                                    value: &SplineRepr::Fixed { value: 1.56f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.7f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 4.69f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.15f32,
                                                    value: &SplineRepr::Fixed { value: 1.37f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.7f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 4.69f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.15f32,
                                                    value: &SplineRepr::Fixed { value: 1.37f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.58f32,
                                        value: &SplineRepr::Fixed { value: 4.69f32 },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                    ],
                },
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 5usize,
                first_index: 36usize,
                second_index: 37usize,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 38usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 35usize,
                argument2_index: 39usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 40usize,
                data: &UnaryData {
                    operation: UnaryOperation::QuarterNegative,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 41usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 4f32,
                },
            },
            BaseNoiseFunctionComponent::InterpolatedNoiseSampler {
                data: &InterpolatedNoiseSamplerData {
                    scaled_xz_scale: 0.25f32,
                    scaled_y_scale: 0.25f32,
                    xz_factor: 80f32,
                    y_factor: 160f32,
                    smear_scale_multiplier: 8f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 42usize,
                argument2_index: 43usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 44usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::CAVE_ENTRANCE,
                    xz_scale: 0.75f32,
                    y_scale: 0.5f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 46usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.37f32,
                },
            },
            BaseNoiseFunctionComponent::Gradient {
                data: &GradientData {
                    axis: Axis::Y,
                    tiling: Tiling::ClampToEdge,
                    from_coordinate: -10i32,
                    to_coordinate: 30i32,
                    from_value: 0.3f32,
                    to_value: 0f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 47usize,
                argument2_index: 48usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_ROUGHNESS_MODULATOR,
                    xz_scale: 1f32,
                    y_scale: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 50usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.05f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 51usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.05f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_ROUGHNESS,
                    xz_scale: 1f32,
                    y_scale: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 53usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 54usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.4f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 52usize,
                argument2_index: 55usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 56usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_RARITY,
                    xz_scale: 2f32,
                    y_scale: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 58usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
                    xz_scale: 1.3333334f32,
                    y_scale: 1.3333334f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 60usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.75f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
                    xz_scale: 1f32,
                    y_scale: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 62usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
                    xz_scale: 0.6666667f32,
                    y_scale: 0.6666667f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 64usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1.5f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
                    xz_scale: 0.5f32,
                    y_scale: 0.5f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 66usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 2f32,
                },
            },
            BaseNoiseFunctionComponent::IntervalSelect {
                input_index: 59usize,
                thresholds: &[-0.5f32, 0f32, 0.5f32],
                functions_indices: &[61usize, 63usize, 65usize, 67usize],
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 68usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
                    xz_scale: 1.3333334f32,
                    y_scale: 1.3333334f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 70usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.75f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
                    xz_scale: 1f32,
                    y_scale: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 72usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
                    xz_scale: 0.6666667f32,
                    y_scale: 0.6666667f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 74usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1.5f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
                    xz_scale: 0.5f32,
                    y_scale: 0.5f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 76usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 2f32,
                },
            },
            BaseNoiseFunctionComponent::IntervalSelect {
                input_index: 59usize,
                thresholds: &[-0.5f32, 0f32, 0.5f32],
                functions_indices: &[71usize, 73usize, 75usize, 77usize],
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 78usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 69usize,
                argument2_index: 79usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_THICKNESS,
                    xz_scale: 1f32,
                    y_scale: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 81usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.011500001f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 82usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.0765f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 80usize,
                argument2_index: 83usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 84usize,
                data: &ClampData {
                    min_value: -1f32,
                    max_value: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 57usize,
                argument2_index: 85usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 49usize,
                argument2_index: 86usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 87usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 88usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 5f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 45usize,
                argument2_index: 89usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::CAVE_LAYER,
                    xz_scale: 1f32,
                    y_scale: 8f32,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 91usize,
                data: &UnaryData {
                    operation: UnaryOperation::Square,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 92usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 4f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::CAVE_CHEESE,
                    xz_scale: 1f32,
                    y_scale: 0.6666667f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 94usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.27f32,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 95usize,
                data: &ClampData {
                    min_value: -1f32,
                    max_value: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 45usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.64f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 97usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 1.5f32,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 98usize,
                data: &ClampData {
                    min_value: 0f32,
                    max_value: 0.5f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 96usize,
                argument2_index: 99usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 93usize,
                argument2_index: 100usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 101usize,
                argument2_index: 88usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D_MODULATOR,
                    xz_scale: 2f32,
                    y_scale: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D,
                    xz_scale: 2f32,
                    y_scale: 2f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 104usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.5f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D,
                    xz_scale: 1.3333334f32,
                    y_scale: 1.3333334f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 106usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.75f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D,
                    xz_scale: 1f32,
                    y_scale: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 108usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D,
                    xz_scale: 0.5f32,
                    y_scale: 0.5f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 110usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 2f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D,
                    xz_scale: 0.33333334f32,
                    y_scale: 0.33333334f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 112usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 3f32,
                },
            },
            BaseNoiseFunctionComponent::IntervalSelect {
                input_index: 103usize,
                thresholds: &[-0.75f32, -0.5f32, 0.5f32, 0.75f32],
                functions_indices: &[105usize, 107usize, 109usize, 111usize, 113usize],
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 114usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D_THICKNESS,
                    xz_scale: 2f32,
                    y_scale: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 116usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.34999996f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 117usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.95f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 118usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 119usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.083f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 115usize,
                argument2_index: 120usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D_ELEVATION,
                    xz_scale: 1f32,
                    y_scale: 0f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 122usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 8f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 123usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Gradient {
                data: &GradientData {
                    axis: Axis::Y,
                    tiling: Tiling::ClampToEdge,
                    from_coordinate: -64i32,
                    to_coordinate: 320i32,
                    from_value: 8f32,
                    to_value: -40f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 124usize,
                argument2_index: 125usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 126usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 127usize,
                argument2_index: 119usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 128usize,
                data: &UnaryData {
                    operation: UnaryOperation::Cube,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 121usize,
                argument2_index: 129usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 130usize,
                data: &ClampData {
                    min_value: -1f32,
                    max_value: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 131usize,
                argument2_index: 57usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 102usize,
                argument2_index: 132usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::PILLAR,
                    xz_scale: 25f32,
                    y_scale: 0.3f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 134usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 2f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::PILLAR_RARENESS,
                    xz_scale: 1f32,
                    y_scale: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 136usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -1f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 137usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -1f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 135usize,
                argument2_index: 138usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::PILLAR_THICKNESS,
                    xz_scale: 1f32,
                    y_scale: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 140usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.55f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 141usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.55f32,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 142usize,
                data: &UnaryData {
                    operation: UnaryOperation::Cube,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 139usize,
                argument2_index: 143usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 144usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Constant { value: -1000000f32 },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 145usize,
                when_in_range_index: 146usize,
                when_out_range_index: 145usize,
                data: &RangeChoiceData {
                    min_inclusive: -1000000f32,
                    max_exclusive: 0.03f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 133usize,
                argument2_index: 147usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 45usize,
                when_in_range_index: 90usize,
                when_out_range_index: 148usize,
                data: &RangeChoiceData {
                    min_inclusive: -1000000f32,
                    max_exclusive: 1.5625f32,
                },
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 2usize,
                first_index: 3usize,
                second_index: 149usize,
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 0usize,
                first_index: 1usize,
                second_index: 150usize,
            },
            BaseNoiseFunctionComponent::BlendDensity {
                input_index: 151usize,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 152usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.64f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 153usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 154usize,
                data: &UnaryData {
                    operation: UnaryOperation::Squeeze,
                },
            },
            BaseNoiseFunctionComponent::Gradient {
                data: &GradientData {
                    axis: Axis::Y,
                    tiling: Tiling::ClampToEdge,
                    from_coordinate: -4064i32,
                    to_coordinate: 4062i32,
                    from_value: -4064f32,
                    to_value: 4062f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::NOODLE,
                    xz_scale: 1f32,
                    y_scale: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Constant { value: -1f32 },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 156usize,
                when_in_range_index: 157usize,
                when_out_range_index: 158usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f32,
                    max_exclusive: 321f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 159usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Constant { value: 64f32 },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::NOODLE_THICKNESS,
                    xz_scale: 1f32,
                    y_scale: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 162usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.025f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 163usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.075f32,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 156usize,
                when_in_range_index: 164usize,
                when_out_range_index: 9usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f32,
                    max_exclusive: 321f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 165usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::NOODLE_RIDGE_A,
                    xz_scale: 2.6666667f32,
                    y_scale: 2.6666667f32,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 156usize,
                when_in_range_index: 167usize,
                when_out_range_index: 9usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f32,
                    max_exclusive: 321f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 168usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 169usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::NOODLE_RIDGE_B,
                    xz_scale: 2.6666667f32,
                    y_scale: 2.6666667f32,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 156usize,
                when_in_range_index: 171usize,
                when_out_range_index: 9usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f32,
                    max_exclusive: 321f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 172usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 173usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 170usize,
                argument2_index: 174usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 175usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1.5f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 166usize,
                argument2_index: 176usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 160usize,
                when_in_range_index: 161usize,
                when_out_range_index: 177usize,
                data: &RangeChoiceData {
                    min_inclusive: -1000000f32,
                    max_exclusive: 0f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 155usize,
                argument2_index: 178usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Beardifier,
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 179usize,
                argument2_index: 180usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 181usize,
                wrapper: WrapperType::CellCache,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::AQUIFER_BARRIER,
                    xz_scale: 1f32,
                    y_scale: 0.5f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::AQUIFER_FLUID_LEVEL_FLOODEDNESS,
                    xz_scale: 1f32,
                    y_scale: 0.67f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::AQUIFER_FLUID_LEVEL_SPREAD,
                    xz_scale: 1f32,
                    y_scale: 0.71428573f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::AQUIFER_LAVA,
                    xz_scale: 1f32,
                    y_scale: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::ORE_VEININESS,
                    xz_scale: 1.5f32,
                    y_scale: 1.5f32,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 156usize,
                when_in_range_index: 187usize,
                when_out_range_index: 9usize,
                data: &RangeChoiceData {
                    min_inclusive: -64f32,
                    max_exclusive: 57f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 188usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 189usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Constant { value: 0.08f32 },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::ORE_VEIN_A,
                    xz_scale: 4f32,
                    y_scale: 4f32,
                },
            },
            BaseNoiseFunctionComponent::Constant { value: 1f32 },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 156usize,
                when_in_range_index: 192usize,
                when_out_range_index: 193usize,
                data: &RangeChoiceData {
                    min_inclusive: -64f32,
                    max_exclusive: 57f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 194usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 195usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::ORE_VEIN_B,
                    xz_scale: 4f32,
                    y_scale: 4f32,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 156usize,
                when_in_range_index: 197usize,
                when_out_range_index: 193usize,
                data: &RangeChoiceData {
                    min_inclusive: -64f32,
                    max_exclusive: 57f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 198usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 199usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 196usize,
                argument2_index: 200usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 191usize,
                argument2_index: 201usize,
                data: &BinaryData {
                    operation: BinaryOperation::Sub,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 190usize,
                when_in_range_index: 158usize,
                when_out_range_index: 202usize,
                data: &RangeChoiceData {
                    min_inclusive: -0.4f32,
                    max_exclusive: 0.4f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 203usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Constant { value: -0.3f32 },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::ORE_GAP,
                    xz_scale: 1f32,
                    y_scale: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 205usize,
                argument2_index: 206usize,
                data: &BinaryData {
                    operation: BinaryOperation::Sub,
                },
            },
        ],
        barrier_noise: 183usize,
        fluid_level_floodedness_noise: 184usize,
        fluid_level_spread_noise: 185usize,
        lava_noise: 186usize,
        erosion: 15usize,
        depth: 27usize,
        final_density: 182usize,
        vein_toggle: 190usize,
        vein_ridged: 204usize,
        vein_gap: 207usize,
    },
    surface_estimator: BaseSurfaceEstimator {
        full_component_stack: &[
            BaseNoiseFunctionComponent::Gradient {
                data: &GradientData {
                    axis: Axis::Y,
                    tiling: Tiling::ClampToEdge,
                    from_coordinate: -64i32,
                    to_coordinate: -40i32,
                    from_value: 0f32,
                    to_value: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Constant {
                value: 0.1171875f32,
            },
            BaseNoiseFunctionComponent::Gradient {
                data: &GradientData {
                    axis: Axis::Y,
                    tiling: Tiling::ClampToEdge,
                    from_coordinate: 240i32,
                    to_coordinate: 256i32,
                    from_value: 1f32,
                    to_value: 0f32,
                },
            },
            BaseNoiseFunctionComponent::Constant {
                value: -0.078125f32,
            },
            BaseNoiseFunctionComponent::Gradient {
                data: &GradientData {
                    axis: Axis::Y,
                    tiling: Tiling::ClampToEdge,
                    from_coordinate: -64i32,
                    to_coordinate: 320i32,
                    from_value: 1.5f32,
                    to_value: -1.5f32,
                },
            },
            BaseNoiseFunctionComponent::BlendAlpha,
            BaseNoiseFunctionComponent::BlendOffset,
            BaseNoiseFunctionComponent::ShiftA {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 7usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Constant { value: 0f32 },
            BaseNoiseFunctionComponent::ShiftB {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 10usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 8usize,
                shift_y_index: 9usize,
                shift_z_index: 11usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f32,
                    y_scale: 0f32,
                    noise_id: DoublePerlinNoiseParameters::CONTINENTALNESS,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 12usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 8usize,
                shift_y_index: 9usize,
                shift_z_index: 11usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f32,
                    y_scale: 0f32,
                    noise_id: DoublePerlinNoiseParameters::EROSION,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 14usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 8usize,
                shift_y_index: 9usize,
                shift_z_index: 11usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f32,
                    y_scale: 0f32,
                    noise_id: DoublePerlinNoiseParameters::RIDGE,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 16usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 17usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 18usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.6666667f32,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 19usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 20usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.33333334f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 21usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -3f32,
                },
            },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 13usize,
                    points: &[
                        SplinePoint {
                            location: -1.1f32,
                            value: &SplineRepr::Fixed { value: 0.044f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -1.02f32,
                            value: &SplineRepr::Fixed { value: -0.2222f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.51f32,
                            value: &SplineRepr::Fixed { value: -0.2222f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.44f32,
                            value: &SplineRepr::Fixed { value: -0.12f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.18f32,
                            value: &SplineRepr::Fixed { value: -0.12f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.16f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 15usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.08880186f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.69000006f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.115760356f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.6400001f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.75f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.65f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.5954547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.6054547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.100000024f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.3f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0.1f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0.06f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.15f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 15usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.08880186f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.69000006f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.115760356f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.6400001f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.75f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.65f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.5954547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.6054547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.100000024f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.3f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0.1f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0.06f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.1f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 15usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.08880186f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.69000006f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.115760356f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.6400001f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.75f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.65f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.5954547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.6054547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.100000024f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.25f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.001f32 },
                                                    derivative: 0.01f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.003f32 },
                                                    derivative: 0.01f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0.094000004f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.12f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 0.25f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 15usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.20235021f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.7161751f32,
                                                    },
                                                    derivative: 0.5138249f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1.23f32 },
                                                    derivative: 0.5138249f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.44682026f32,
                                                    },
                                                    derivative: 0.43317974f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.88f32 },
                                                    derivative: 0.43317974f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.30829495f32,
                                                    },
                                                    derivative: 0.3917051f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.70000005f32,
                                                    },
                                                    derivative: 0.3917051f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.25f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.42000002f32,
                                                    },
                                                    derivative: 0.049000014f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.0069999998f32,
                                                    },
                                                    derivative: 0.07f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.021f32 },
                                                    derivative: 0.07f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0.658f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.42000002f32,
                                                    },
                                                    derivative: 0.049000014f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.1f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.1f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.58f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.12f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 1f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 15usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.34792626f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.9239631f32,
                                                    },
                                                    derivative: 0.5760369f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1.5f32 },
                                                    derivative: 0.5760369f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.5391705f32,
                                                    },
                                                    derivative: 0.4608295f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1f32 },
                                                    derivative: 0.4608295f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.5391705f32,
                                                    },
                                                    derivative: 0.4608295f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1f32 },
                                                    derivative: 0.4608295f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.6f32 },
                                                    derivative: 0.070000015f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0.099999994f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.099999994f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0.94f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.6f32 },
                                                    derivative: 0.070000015f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.05f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.05f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.58f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0.015f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                    ],
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 23usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.50375f32,
                },
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 5usize,
                first_index: 6usize,
                second_index: 24usize,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 25usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 4usize,
                argument2_index: 26usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Constant { value: 10f32 },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 13usize,
                    points: &[
                        SplinePoint {
                            location: -0.19f32,
                            value: &SplineRepr::Fixed { value: 3.95f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.15f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 15usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.5f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.25f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.03f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.35f32,
                                        value: &SplineRepr::Fixed { value: 6.25f32 },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.25f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.25f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.62f32,
                                        value: &SplineRepr::Fixed { value: 6.25f32 },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.1f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 15usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.5f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.25f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.03f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.35f32,
                                        value: &SplineRepr::Fixed { value: 5.47f32 },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 5.47f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 5.47f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.62f32,
                                        value: &SplineRepr::Fixed { value: 5.47f32 },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 0.03f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 15usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.5f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.25f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.03f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.35f32,
                                        value: &SplineRepr::Fixed { value: 5.08f32 },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 5.08f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 5.08f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.625f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.62f32,
                                        value: &SplineRepr::Fixed { value: 5.08f32 },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 0.06f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 15usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 4.69f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.5f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 4.69f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.25f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 4.69f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.05f32,
                                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.05f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.03f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 17usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.2f32,
                                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.2f32,
                                                    value: &SplineRepr::Fixed { value: 4.69f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.05f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.45f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 4.69f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.7f32,
                                                    value: &SplineRepr::Fixed { value: 1.56f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.45f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 4.69f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.7f32,
                                                    value: &SplineRepr::Fixed { value: 1.56f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.7f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 4.69f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.15f32,
                                                    value: &SplineRepr::Fixed { value: 1.37f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 22usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.7f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 17usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 6.3f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.2f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 4.69f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.15f32,
                                                    value: &SplineRepr::Fixed { value: 1.37f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.58f32,
                                        value: &SplineRepr::Fixed { value: 4.69f32 },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                    ],
                },
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 5usize,
                first_index: 28usize,
                second_index: 29usize,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 30usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 27usize,
                argument2_index: 31usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 32usize,
                data: &UnaryData {
                    operation: UnaryOperation::QuarterNegative,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 33usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 4f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 34usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.703125f32,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 35usize,
                data: &ClampData {
                    min_value: -64f32,
                    max_value: 64f32,
                },
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 2usize,
                first_index: 3usize,
                second_index: 36usize,
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 0usize,
                first_index: 1usize,
                second_index: 37usize,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 38usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.390625f32,
                },
            },
            BaseNoiseFunctionComponent::Constant {
                value: 0.2734375f32,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 40usize,
                argument2_index: 31usize,
                data: &BinaryData {
                    operation: BinaryOperation::Div,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 41usize,
                argument2_index: 26usize,
                data: &BinaryData {
                    operation: BinaryOperation::Sub,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 42usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -128f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 43usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 128f32,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 44usize,
                data: &ClampData {
                    min_value: -40f32,
                    max_value: 320f32,
                },
            },
            BaseNoiseFunctionComponent::FindTopSurface {
                density_index: 39usize,
                upper_bound_index: 45usize,
                data: &FindTopSurfaceData {
                    lower_bound: -64i32,
                    cell_height: 8i32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 46usize,
                wrapper: WrapperType::Interpolated,
            },
        ],
    },
    multi_noise: BaseMultiNoiseRouter {
        full_component_stack: &[
            BaseNoiseFunctionComponent::ShiftA {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 0usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Constant { value: 0f32 },
            BaseNoiseFunctionComponent::ShiftB {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 3usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 1usize,
                shift_y_index: 2usize,
                shift_z_index: 4usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f32,
                    y_scale: 0f32,
                    noise_id: DoublePerlinNoiseParameters::RIDGE,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 5usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 1usize,
                shift_y_index: 2usize,
                shift_z_index: 4usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f32,
                    y_scale: 0f32,
                    noise_id: DoublePerlinNoiseParameters::TEMPERATURE,
                },
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 1usize,
                shift_y_index: 2usize,
                shift_z_index: 4usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f32,
                    y_scale: 0f32,
                    noise_id: DoublePerlinNoiseParameters::VEGETATION,
                },
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 1usize,
                shift_y_index: 2usize,
                shift_z_index: 4usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f32,
                    y_scale: 0f32,
                    noise_id: DoublePerlinNoiseParameters::CONTINENTALNESS,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 9usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 1usize,
                shift_y_index: 2usize,
                shift_z_index: 4usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f32,
                    y_scale: 0f32,
                    noise_id: DoublePerlinNoiseParameters::EROSION,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 11usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Gradient {
                data: &GradientData {
                    axis: Axis::Y,
                    tiling: Tiling::ClampToEdge,
                    from_coordinate: -64i32,
                    to_coordinate: 320i32,
                    from_value: 1.5f32,
                    to_value: -1.5f32,
                },
            },
            BaseNoiseFunctionComponent::BlendAlpha,
            BaseNoiseFunctionComponent::BlendOffset,
            BaseNoiseFunctionComponent::Unary {
                input_index: 6usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 16usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.6666667f32,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 17usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 18usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.33333334f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 19usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -3f32,
                },
            },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 10usize,
                    points: &[
                        SplinePoint {
                            location: -1.1f32,
                            value: &SplineRepr::Fixed { value: 0.044f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -1.02f32,
                            value: &SplineRepr::Fixed { value: -0.2222f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.51f32,
                            value: &SplineRepr::Fixed { value: -0.2222f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.44f32,
                            value: &SplineRepr::Fixed { value: -0.12f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.18f32,
                            value: &SplineRepr::Fixed { value: -0.12f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.16f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 12usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.08880186f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.69000006f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.115760356f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.6400001f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.75f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.65f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.5954547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.6054547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.100000024f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.3f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0.1f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0.06f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.15f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 12usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.08880186f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.69000006f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.115760356f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.6400001f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.75f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.65f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.5954547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.6054547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.100000024f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.3f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0.1f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0.06f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.1f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 12usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.08880186f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.69000006f32,
                                                    },
                                                    derivative: 0.38940096f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: -0.115760356f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.6400001f32,
                                                    },
                                                    derivative: 0.37788022f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.75f32,
                                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.65f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.5954547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.6054547f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.000000029802322f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.100000024f32,
                                                    },
                                                    derivative: 0.2534563f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.25f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.001f32 },
                                                    derivative: 0.01f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.003f32 },
                                                    derivative: 0.01f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                                    derivative: 0.094000004f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.060000002f32,
                                                    },
                                                    derivative: 0.007000001f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.12f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 0.25f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 12usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.20235021f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.7161751f32,
                                                    },
                                                    derivative: 0.5138249f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1.23f32 },
                                                    derivative: 0.5138249f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.44682026f32,
                                                    },
                                                    derivative: 0.43317974f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.88f32 },
                                                    derivative: 0.43317974f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.30829495f32,
                                                    },
                                                    derivative: 0.3917051f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.70000005f32,
                                                    },
                                                    derivative: 0.3917051f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.25f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.42000002f32,
                                                    },
                                                    derivative: 0.049000014f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.0069999998f32,
                                                    },
                                                    derivative: 0.07f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.021f32 },
                                                    derivative: 0.07f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                                    derivative: 0.658f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.42000002f32,
                                                    },
                                                    derivative: 0.049000014f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 20usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.1f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 20usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.1f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.58f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.12f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 1f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 12usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.34792626f32,
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.9239631f32,
                                                    },
                                                    derivative: 0.5760369f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1.5f32 },
                                                    derivative: 0.5760369f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.5391705f32,
                                                    },
                                                    derivative: 0.4608295f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1f32 },
                                                    derivative: 0.4608295f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed {
                                                        value: 0.5391705f32,
                                                    },
                                                    derivative: 0.4608295f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 1f32 },
                                                    derivative: 0.4608295f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.35f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.2f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.6f32 },
                                                    derivative: 0.070000015f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: -0.1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0.099999994f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.099999994f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                                    derivative: 0.94f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.6f32 },
                                                    derivative: 0.070000015f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.2f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.4f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.45f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 20usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.05f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.55f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 20usize,
                                                        points: &[
                                                            SplinePoint {
                                                                location: -1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: -0.05f32,
                                                                },
                                                                derivative: 0.5f32,
                                                            },
                                                            SplinePoint {
                                                                location: -0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.01f32,
                                                                },
                                                                derivative: 0f32,
                                                            },
                                                            SplinePoint {
                                                                location: 0.4f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.03f32,
                                                                },
                                                                derivative: 0.04f32,
                                                            },
                                                            SplinePoint {
                                                                location: 1f32,
                                                                value: &SplineRepr::Fixed {
                                                                    value: 0.1f32,
                                                                },
                                                                derivative: 0.049f32,
                                                            },
                                                        ],
                                                    },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                                    derivative: 0f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.58f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0.5f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                    SplinePoint {
                                        location: 0.7f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 20usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                                    derivative: 0.015f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0f32,
                                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.4f32,
                                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                                    derivative: 0.04f32,
                                                },
                                                SplinePoint {
                                                    location: 1f32,
                                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                                    derivative: 0.049f32,
                                                },
                                            ],
                                        },
                                        derivative: 0f32,
                                    },
                                ],
                            },
                            derivative: 0f32,
                        },
                    ],
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 21usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.50375f32,
                },
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 14usize,
                first_index: 15usize,
                second_index: 22usize,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 23usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 13usize,
                argument2_index: 24usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
        ],
        temperature: 7usize,
        vegetation: 8usize,
        continents: 10usize,
        erosion: 12usize,
        depth: 25usize,
        ridges: 6usize,
    },
};
pub const NETHER_BASE_NOISE_ROUTER: BaseNoiseRouters = BaseNoiseRouters {
    noise: BaseNoiseRouter {
        full_component_stack: &[
            BaseNoiseFunctionComponent::Gradient {
                data: &GradientData {
                    axis: Axis::Y,
                    tiling: Tiling::ClampToEdge,
                    from_coordinate: -8i32,
                    to_coordinate: 24i32,
                    from_value: 0f32,
                    to_value: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Constant { value: 2.5f32 },
            BaseNoiseFunctionComponent::Gradient {
                data: &GradientData {
                    axis: Axis::Y,
                    tiling: Tiling::ClampToEdge,
                    from_coordinate: 104i32,
                    to_coordinate: 128i32,
                    from_value: 1f32,
                    to_value: 0f32,
                },
            },
            BaseNoiseFunctionComponent::Constant { value: 0.9375f32 },
            BaseNoiseFunctionComponent::InterpolatedNoiseSampler {
                data: &InterpolatedNoiseSamplerData {
                    scaled_xz_scale: 0.25f32,
                    scaled_y_scale: 0.28125f32,
                    xz_factor: 80f32,
                    y_factor: 60f32,
                    smear_scale_multiplier: 8f32,
                },
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 2usize,
                first_index: 3usize,
                second_index: 4usize,
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 0usize,
                first_index: 1usize,
                second_index: 5usize,
            },
            BaseNoiseFunctionComponent::BlendDensity {
                input_index: 6usize,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 7usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.64f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 8usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 9usize,
                data: &UnaryData {
                    operation: UnaryOperation::Squeeze,
                },
            },
            BaseNoiseFunctionComponent::Beardifier,
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 10usize,
                argument2_index: 11usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 12usize,
                wrapper: WrapperType::CellCache,
            },
            BaseNoiseFunctionComponent::Constant { value: 0f32 },
        ],
        barrier_noise: 14usize,
        fluid_level_floodedness_noise: 14usize,
        fluid_level_spread_noise: 14usize,
        lava_noise: 14usize,
        erosion: 14usize,
        depth: 14usize,
        final_density: 13usize,
        vein_toggle: 14usize,
        vein_ridged: 14usize,
        vein_gap: 14usize,
    },
    surface_estimator: BaseSurfaceEstimator {
        full_component_stack: &[BaseNoiseFunctionComponent::Constant { value: 0f32 }],
    },
    multi_noise: BaseMultiNoiseRouter {
        full_component_stack: &[
            BaseNoiseFunctionComponent::Constant { value: 0f32 },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::NETHER_TEMPERATURE,
                    xz_scale: 0.25f32,
                    y_scale: 0f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::NETHER_VEGETATION,
                    xz_scale: 0.25f32,
                    y_scale: 0f32,
                },
            },
        ],
        temperature: 1usize,
        vegetation: 2usize,
        continents: 0usize,
        erosion: 0usize,
        depth: 0usize,
        ridges: 0usize,
    },
};
pub const END_BASE_NOISE_ROUTER: BaseNoiseRouters = BaseNoiseRouters {
    noise: BaseNoiseRouter {
        full_component_stack: &[
            BaseNoiseFunctionComponent::Gradient {
                data: &GradientData {
                    axis: Axis::Y,
                    tiling: Tiling::ClampToEdge,
                    from_coordinate: 4i32,
                    to_coordinate: 32i32,
                    from_value: 0f32,
                    to_value: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Constant {
                value: -0.234375f32,
            },
            BaseNoiseFunctionComponent::Gradient {
                data: &GradientData {
                    axis: Axis::Y,
                    tiling: Tiling::ClampToEdge,
                    from_coordinate: 56i32,
                    to_coordinate: 312i32,
                    from_value: 1f32,
                    to_value: 0f32,
                },
            },
            BaseNoiseFunctionComponent::Constant { value: -23.4375f32 },
            BaseNoiseFunctionComponent::Constant { value: 100f32 },
            BaseNoiseFunctionComponent::DistanceToPoint {
                data: &DistanceToPointData {
                    point: [0i32, 0i32, 0i32],
                    metric: DistanceMetric::Euclidean,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 4usize,
                argument2_index: 5usize,
                data: &BinaryData {
                    operation: BinaryOperation::Sub,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 6usize,
                data: &ClampData {
                    min_value: -100f32,
                    max_value: 80f32,
                },
            },
            BaseNoiseFunctionComponent::Constant { value: 8f32 },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 7usize,
                argument2_index: 8usize,
                data: &BinaryData {
                    operation: BinaryOperation::Sub,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 9usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.0078125f32,
                },
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 10usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::EndIslands,
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 11usize,
                argument2_index: 12usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 13usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::InterpolatedNoiseSampler {
                data: &InterpolatedNoiseSamplerData {
                    scaled_xz_scale: 0.25f32,
                    scaled_y_scale: 0.5f32,
                    xz_factor: 80f32,
                    y_factor: 160f32,
                    smear_scale_multiplier: 4f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 14usize,
                argument2_index: 15usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 2usize,
                first_index: 3usize,
                second_index: 16usize,
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 0usize,
                first_index: 1usize,
                second_index: 17usize,
            },
            BaseNoiseFunctionComponent::BlendDensity {
                input_index: 18usize,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 19usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.64f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 20usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 21usize,
                data: &UnaryData {
                    operation: UnaryOperation::Squeeze,
                },
            },
            BaseNoiseFunctionComponent::Beardifier,
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 22usize,
                argument2_index: 23usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 24usize,
                wrapper: WrapperType::CellCache,
            },
            BaseNoiseFunctionComponent::Constant { value: 0f32 },
        ],
        barrier_noise: 26usize,
        fluid_level_floodedness_noise: 26usize,
        fluid_level_spread_noise: 26usize,
        lava_noise: 26usize,
        erosion: 14usize,
        depth: 26usize,
        final_density: 25usize,
        vein_toggle: 26usize,
        vein_ridged: 26usize,
        vein_gap: 26usize,
    },
    surface_estimator: BaseSurfaceEstimator {
        full_component_stack: &[BaseNoiseFunctionComponent::Constant { value: 0f32 }],
    },
    multi_noise: BaseMultiNoiseRouter {
        full_component_stack: &[
            BaseNoiseFunctionComponent::Constant { value: 0f32 },
            BaseNoiseFunctionComponent::Constant { value: 100f32 },
            BaseNoiseFunctionComponent::DistanceToPoint {
                data: &DistanceToPointData {
                    point: [0i32, 0i32, 0i32],
                    metric: DistanceMetric::Euclidean,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 1usize,
                argument2_index: 2usize,
                data: &BinaryData {
                    operation: BinaryOperation::Sub,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 3usize,
                data: &ClampData {
                    min_value: -100f32,
                    max_value: 80f32,
                },
            },
            BaseNoiseFunctionComponent::Constant { value: 8f32 },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 4usize,
                argument2_index: 5usize,
                data: &BinaryData {
                    operation: BinaryOperation::Sub,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 6usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.0078125f32,
                },
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 7usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::EndIslands,
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 8usize,
                argument2_index: 9usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 10usize,
                wrapper: WrapperType::CacheOnce,
            },
        ],
        temperature: 0usize,
        vegetation: 0usize,
        continents: 0usize,
        erosion: 11usize,
        depth: 0usize,
        ridges: 0usize,
    },
};
