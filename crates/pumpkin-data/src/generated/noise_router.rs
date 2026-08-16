/* This file is generated. Do not edit manually. */
use crate::chunk::DoublePerlinNoiseParameters;
pub trait NoiseEvaluationContext {
    fn sample_noise(
        &mut self,
        noise_id: DoublePerlinNoiseParameters,
        x: f64,
        y: f64,
        z: f64,
    ) -> f64;
    fn sample_shift_a(
        &mut self,
        noise_id: DoublePerlinNoiseParameters,
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
    ) -> f64;
    fn sample_shift_b(
        &mut self,
        noise_id: DoublePerlinNoiseParameters,
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
    ) -> f64;
    fn sample_shifted_noise(
        &mut self,
        noise_id: DoublePerlinNoiseParameters,
        shift_x: f64,
        shift_y: f64,
        shift_z: f64,
        xz_scale: f64,
        y_scale: f64,
    ) -> f64;
    fn sample_interpolated_noise(&mut self, pos: &pumpkin_util::math::vector3::Vector3<i32>)
    -> f64;
    fn sample_beardifier(&mut self, pos: &pumpkin_util::math::vector3::Vector3<i32>) -> f64;
    fn sample_blend_alpha(&mut self, pos: &pumpkin_util::math::vector3::Vector3<i32>) -> f64;
    fn sample_blend_offset(&mut self, pos: &pumpkin_util::math::vector3::Vector3<i32>) -> f64;
    fn sample_blend_density(
        &mut self,
        input_val: f64,
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
    ) -> f64;
    fn sample_end_islands(&mut self, pos: &pumpkin_util::math::vector3::Vector3<i32>) -> f64;
    fn sample_wrapper(
        &mut self,
        wrapper_index: usize,
        wrapper_type: WrapperType,
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        eval_input: &dyn Fn(&pumpkin_util::math::vector3::Vector3<i32>, &mut Self) -> f64,
    ) -> f64;
    fn sample_spline(
        &mut self,
        spline_index: usize,
        location_value: f64,
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
    ) -> f64;
    fn sample_find_top_surface(
        &mut self,
        density_fn: &dyn Fn(&pumpkin_util::math::vector3::Vector3<i32>, &mut Self) -> f64,
        upper_bound_fn: &dyn Fn(&pumpkin_util::math::vector3::Vector3<i32>, &mut Self) -> f64,
        lower_bound: i32,
        cell_height: i32,
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
    ) -> f64;
}
pub mod overworld_noise_evaluator {
    use super::*;
    #[inline(always)]
    pub fn overworld_node_0<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let _ = ctx;
        let y = pos.y as f64;
        let clamped = y.clamp(-64f64, -40f64);
        let delta = (clamped - -64f64) / (-40f64 - -64f64);
        0f64 + delta * (1f64 - 0f64)
    }
    #[inline(always)]
    pub fn overworld_node_1<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let _ = ctx;
        let y = pos.y as f64;
        let clamped = y.clamp(240f64, 256f64);
        let delta = (clamped - 240f64) / (256f64 - 240f64);
        1f64 + delta * (0f64 - 1f64)
    }
    #[inline(always)]
    pub fn overworld_node_2<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let _ = ctx;
        let y = pos.y as f64;
        let clamped = y.clamp(-64f64, 320f64);
        let delta = (clamped - -64f64) / (320f64 - -64f64);
        1.5f64 + delta * (-1.5f64 - 1.5f64)
    }
    #[inline(always)]
    pub fn overworld_node_3<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_blend_offset(pos)
    }
    #[inline(always)]
    pub fn overworld_node_4<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_blend_alpha(pos)
    }
    #[inline(always)]
    pub fn overworld_node_5<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(5usize, WrapperType::CacheOnce, pos, &overworld_node_4)
    }
    #[inline(always)]
    pub fn overworld_node_6<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_5(pos, ctx) * -1f64
    }
    #[inline(always)]
    pub fn overworld_node_7<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_6(pos, ctx) + 1f64
    }
    #[inline(always)]
    pub fn overworld_node_8<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_3(pos, ctx) * overworld_node_7(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_9<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_shift_a(DoublePerlinNoiseParameters::OFFSET, pos)
    }
    #[inline(always)]
    pub fn overworld_node_10<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(10usize, WrapperType::Cache2D, pos, &overworld_node_9)
    }
    #[inline(always)]
    pub fn overworld_node_11<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(11usize, WrapperType::CacheFlat, pos, &overworld_node_10)
    }
    #[inline(always)]
    pub fn overworld_node_12<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let _ = (pos, ctx);
        0f64
    }
    #[inline(always)]
    pub fn overworld_node_13<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_shift_b(DoublePerlinNoiseParameters::OFFSET, pos)
    }
    #[inline(always)]
    pub fn overworld_node_14<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(14usize, WrapperType::Cache2D, pos, &overworld_node_13)
    }
    #[inline(always)]
    pub fn overworld_node_15<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(15usize, WrapperType::CacheFlat, pos, &overworld_node_14)
    }
    #[inline(always)]
    pub fn overworld_node_16<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let sx = overworld_node_11(pos, ctx);
        let sy = overworld_node_12(pos, ctx);
        let sz = overworld_node_15(pos, ctx);
        ctx.sample_shifted_noise(
            DoublePerlinNoiseParameters::CONTINENTALNESS,
            sx,
            sy,
            sz,
            0.25f64,
            0f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_17<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(17usize, WrapperType::CacheFlat, pos, &overworld_node_16)
    }
    #[inline(always)]
    pub fn overworld_node_18<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let sx = overworld_node_11(pos, ctx);
        let sy = overworld_node_12(pos, ctx);
        let sz = overworld_node_15(pos, ctx);
        ctx.sample_shifted_noise(
            DoublePerlinNoiseParameters::EROSION,
            sx,
            sy,
            sz,
            0.25f64,
            0f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_19<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(19usize, WrapperType::CacheFlat, pos, &overworld_node_18)
    }
    #[inline(always)]
    pub fn overworld_node_20<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let sx = overworld_node_11(pos, ctx);
        let sy = overworld_node_12(pos, ctx);
        let sz = overworld_node_15(pos, ctx);
        ctx.sample_shifted_noise(
            DoublePerlinNoiseParameters::RIDGE,
            sx,
            sy,
            sz,
            0.25f64,
            0f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_21<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(21usize, WrapperType::CacheFlat, pos, &overworld_node_20)
    }
    #[inline(always)]
    pub fn overworld_node_22<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_21(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn overworld_node_23<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_22(pos, ctx) + -0.6666666666666666f64
    }
    #[inline(always)]
    pub fn overworld_node_24<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_23(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn overworld_node_25<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_24(pos, ctx) + -0.3333333333333333f64
    }
    #[inline(always)]
    pub fn overworld_node_26<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_25(pos, ctx) * -3f64
    }
    #[inline(always)]
    pub fn overworld_node_27<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let location_val = overworld_node_17(pos, ctx);
        ctx.sample_spline(27usize, location_val, pos)
    }
    #[inline(always)]
    pub fn overworld_node_28<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_27(pos, ctx) + -0.5037500262260437f64
    }
    #[inline(always)]
    pub fn overworld_node_29<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_28(pos, ctx) * overworld_node_5(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_30<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_8(pos, ctx) + overworld_node_29(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_31<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(31usize, WrapperType::Cache2D, pos, &overworld_node_30)
    }
    #[inline(always)]
    pub fn overworld_node_32<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(32usize, WrapperType::CacheFlat, pos, &overworld_node_31)
    }
    #[inline(always)]
    pub fn overworld_node_33<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_2(pos, ctx) + overworld_node_32(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_34<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let location_val = overworld_node_17(pos, ctx);
        ctx.sample_spline(34usize, location_val, pos)
    }
    #[inline(always)]
    pub fn overworld_node_35<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_4(pos, ctx) * overworld_node_34(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_36<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(36usize, WrapperType::Cache2D, pos, &overworld_node_35)
    }
    #[inline(always)]
    pub fn overworld_node_37<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(37usize, WrapperType::CacheFlat, pos, &overworld_node_36)
    }
    #[inline(always)]
    pub fn overworld_node_38<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::JAGGED,
            pos.x as f64 * 1500f64,
            pos.y as f64 * 0f64,
            pos.z as f64 * 1500f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_39<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let v = overworld_node_38(pos, ctx);
        if v > 0.0 { v } else { v * 0.5 }
    }
    #[inline(always)]
    pub fn overworld_node_40<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_37(pos, ctx) * overworld_node_39(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_41<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(41usize, WrapperType::CacheFlat, pos, &overworld_node_40)
    }
    #[inline(always)]
    pub fn overworld_node_42<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_33(pos, ctx) + overworld_node_41(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_43<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let location_val = overworld_node_17(pos, ctx);
        ctx.sample_spline(43usize, location_val, pos)
    }
    #[inline(always)]
    pub fn overworld_node_44<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_43(pos, ctx) + -10f64
    }
    #[inline(always)]
    pub fn overworld_node_45<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_4(pos, ctx) * overworld_node_44(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_46<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_45(pos, ctx) + 10f64
    }
    #[inline(always)]
    pub fn overworld_node_47<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(47usize, WrapperType::Cache2D, pos, &overworld_node_46)
    }
    #[inline(always)]
    pub fn overworld_node_48<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(48usize, WrapperType::CacheFlat, pos, &overworld_node_47)
    }
    #[inline(always)]
    pub fn overworld_node_49<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_42(pos, ctx) * overworld_node_48(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_50<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let v = overworld_node_49(pos, ctx);
        if v > 0.0 { v } else { v * 0.25 }
    }
    #[inline(always)]
    pub fn overworld_node_51<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_50(pos, ctx) * 4f64
    }
    #[inline(always)]
    pub fn overworld_node_52<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_interpolated_noise(pos)
    }
    #[inline(always)]
    pub fn overworld_node_53<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_51(pos, ctx) + overworld_node_52(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_54<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(54usize, WrapperType::CacheOnce, pos, &overworld_node_53)
    }
    #[inline(always)]
    pub fn overworld_node_55<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::CAVE_ENTRANCE,
            pos.x as f64 * 0.75f64,
            pos.y as f64 * 0.5f64,
            pos.z as f64 * 0.75f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_56<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_55(pos, ctx) + 0.37f64
    }
    #[inline(always)]
    pub fn overworld_node_57<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let _ = ctx;
        let y = pos.y as f64;
        let clamped = y.clamp(-10f64, 30f64);
        let delta = (clamped - -10f64) / (30f64 - -10f64);
        0.3f64 + delta * (0f64 - 0.3f64)
    }
    #[inline(always)]
    pub fn overworld_node_58<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_56(pos, ctx) + overworld_node_57(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_59<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_ROUGHNESS_MODULATOR,
            pos.x as f64 * 1f64,
            pos.y as f64 * 1f64,
            pos.z as f64 * 1f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_60<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_59(pos, ctx) * -0.05f64
    }
    #[inline(always)]
    pub fn overworld_node_61<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_60(pos, ctx) + -0.05f64
    }
    #[inline(always)]
    pub fn overworld_node_62<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_ROUGHNESS,
            pos.x as f64 * 1f64,
            pos.y as f64 * 1f64,
            pos.z as f64 * 1f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_63<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_62(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn overworld_node_64<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_63(pos, ctx) + -0.4f64
    }
    #[inline(always)]
    pub fn overworld_node_65<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_61(pos, ctx) * overworld_node_64(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_66<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(66usize, WrapperType::CacheOnce, pos, &overworld_node_65)
    }
    #[inline(always)]
    pub fn overworld_node_67<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_RARITY,
            pos.x as f64 * 2f64,
            pos.y as f64 * 1f64,
            pos.z as f64 * 2f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_68<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(68usize, WrapperType::CacheOnce, pos, &overworld_node_67)
    }
    #[inline(always)]
    pub fn overworld_node_69<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
            pos.x as f64 * 1.3333333333333333f64,
            pos.y as f64 * 1.3333333333333333f64,
            pos.z as f64 * 1.3333333333333333f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_70<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_69(pos, ctx) * 0.75f64
    }
    #[inline(always)]
    pub fn overworld_node_71<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
            pos.x as f64 * 1f64,
            pos.y as f64 * 1f64,
            pos.z as f64 * 1f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_72<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
            pos.x as f64 * 0.6666666666666666f64,
            pos.y as f64 * 0.6666666666666666f64,
            pos.z as f64 * 0.6666666666666666f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_73<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_72(pos, ctx) * 1.5f64
    }
    #[inline(always)]
    pub fn overworld_node_74<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
            pos.x as f64 * 0.5f64,
            pos.y as f64 * 0.5f64,
            pos.z as f64 * 0.5f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_75<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_74(pos, ctx) * 2f64
    }
    #[inline(always)]
    pub fn overworld_node_76<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let input_val = overworld_node_68(pos, ctx);
        let thresholds = &[-0.5f64, 0f64, 0.5f64];
        let mut selected = thresholds.len();
        for (i, &t) in thresholds.iter().enumerate() {
            if input_val < t {
                selected = i;
                break;
            }
        }
        match selected {
            0usize => overworld_node_70(pos, ctx),
            1usize => overworld_node_71(pos, ctx),
            2usize => overworld_node_73(pos, ctx),
            _ => overworld_node_75(pos, ctx),
        }
    }
    #[inline(always)]
    pub fn overworld_node_77<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_76(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn overworld_node_78<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
            pos.x as f64 * 1.3333333333333333f64,
            pos.y as f64 * 1.3333333333333333f64,
            pos.z as f64 * 1.3333333333333333f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_79<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_78(pos, ctx) * 0.75f64
    }
    #[inline(always)]
    pub fn overworld_node_80<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
            pos.x as f64 * 1f64,
            pos.y as f64 * 1f64,
            pos.z as f64 * 1f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_81<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
            pos.x as f64 * 0.6666666666666666f64,
            pos.y as f64 * 0.6666666666666666f64,
            pos.z as f64 * 0.6666666666666666f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_82<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_81(pos, ctx) * 1.5f64
    }
    #[inline(always)]
    pub fn overworld_node_83<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
            pos.x as f64 * 0.5f64,
            pos.y as f64 * 0.5f64,
            pos.z as f64 * 0.5f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_84<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_83(pos, ctx) * 2f64
    }
    #[inline(always)]
    pub fn overworld_node_85<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let input_val = overworld_node_68(pos, ctx);
        let thresholds = &[-0.5f64, 0f64, 0.5f64];
        let mut selected = thresholds.len();
        for (i, &t) in thresholds.iter().enumerate() {
            if input_val < t {
                selected = i;
                break;
            }
        }
        match selected {
            0usize => overworld_node_79(pos, ctx),
            1usize => overworld_node_80(pos, ctx),
            2usize => overworld_node_82(pos, ctx),
            _ => overworld_node_84(pos, ctx),
        }
    }
    #[inline(always)]
    pub fn overworld_node_86<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_85(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn overworld_node_87<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_77(pos, ctx).max(overworld_node_86(pos, ctx))
    }
    #[inline(always)]
    pub fn overworld_node_88<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_THICKNESS,
            pos.x as f64 * 1f64,
            pos.y as f64 * 1f64,
            pos.z as f64 * 1f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_89<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_88(pos, ctx) * -0.011499999999999996f64
    }
    #[inline(always)]
    pub fn overworld_node_90<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_89(pos, ctx) + -0.0765f64
    }
    #[inline(always)]
    pub fn overworld_node_91<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_87(pos, ctx) + overworld_node_90(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_92<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_91(pos, ctx).clamp(-1f64, 1f64)
    }
    #[inline(always)]
    pub fn overworld_node_93<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_66(pos, ctx) + overworld_node_92(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_94<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_58(pos, ctx).min(overworld_node_93(pos, ctx))
    }
    #[inline(always)]
    pub fn overworld_node_95<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(95usize, WrapperType::CacheOnce, pos, &overworld_node_94)
    }
    #[inline(always)]
    pub fn overworld_node_96<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_95(pos, ctx) * 5f64
    }
    #[inline(always)]
    pub fn overworld_node_97<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_54(pos, ctx).min(overworld_node_96(pos, ctx))
    }
    #[inline(always)]
    pub fn overworld_node_98<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::CAVE_LAYER,
            pos.x as f64 * 1f64,
            pos.y as f64 * 8f64,
            pos.z as f64 * 1f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_99<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let v = overworld_node_98(pos, ctx);
        v * v
    }
    #[inline(always)]
    pub fn overworld_node_100<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_99(pos, ctx) * 4f64
    }
    #[inline(always)]
    pub fn overworld_node_101<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::CAVE_CHEESE,
            pos.x as f64 * 1f64,
            pos.y as f64 * 0.6666666666666666f64,
            pos.z as f64 * 1f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_102<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_101(pos, ctx) + 0.27f64
    }
    #[inline(always)]
    pub fn overworld_node_103<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_102(pos, ctx).clamp(-1f64, 1f64)
    }
    #[inline(always)]
    pub fn overworld_node_104<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_54(pos, ctx) * -0.64f64
    }
    #[inline(always)]
    pub fn overworld_node_105<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_104(pos, ctx) + 1.5f64
    }
    #[inline(always)]
    pub fn overworld_node_106<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_105(pos, ctx).clamp(0f64, 0.5f64)
    }
    #[inline(always)]
    pub fn overworld_node_107<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_103(pos, ctx) + overworld_node_106(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_108<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_100(pos, ctx) + overworld_node_107(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_109<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_108(pos, ctx).min(overworld_node_95(pos, ctx))
    }
    #[inline(always)]
    pub fn overworld_node_110<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D_MODULATOR,
            pos.x as f64 * 2f64,
            pos.y as f64 * 1f64,
            pos.z as f64 * 2f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_111<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D,
            pos.x as f64 * 2f64,
            pos.y as f64 * 2f64,
            pos.z as f64 * 2f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_112<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_111(pos, ctx) * 0.5f64
    }
    #[inline(always)]
    pub fn overworld_node_113<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D,
            pos.x as f64 * 1.3333333333333333f64,
            pos.y as f64 * 1.3333333333333333f64,
            pos.z as f64 * 1.3333333333333333f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_114<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_113(pos, ctx) * 0.75f64
    }
    #[inline(always)]
    pub fn overworld_node_115<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D,
            pos.x as f64 * 1f64,
            pos.y as f64 * 1f64,
            pos.z as f64 * 1f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_116<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D,
            pos.x as f64 * 0.5f64,
            pos.y as f64 * 0.5f64,
            pos.z as f64 * 0.5f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_117<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_116(pos, ctx) * 2f64
    }
    #[inline(always)]
    pub fn overworld_node_118<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D,
            pos.x as f64 * 0.3333333333333333f64,
            pos.y as f64 * 0.3333333333333333f64,
            pos.z as f64 * 0.3333333333333333f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_119<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_118(pos, ctx) * 3f64
    }
    #[inline(always)]
    pub fn overworld_node_120<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let input_val = overworld_node_110(pos, ctx);
        let thresholds = &[-0.75f64, -0.5f64, 0.5f64, 0.75f64];
        let mut selected = thresholds.len();
        for (i, &t) in thresholds.iter().enumerate() {
            if input_val < t {
                selected = i;
                break;
            }
        }
        match selected {
            0usize => overworld_node_112(pos, ctx),
            1usize => overworld_node_114(pos, ctx),
            2usize => overworld_node_115(pos, ctx),
            3usize => overworld_node_117(pos, ctx),
            _ => overworld_node_119(pos, ctx),
        }
    }
    #[inline(always)]
    pub fn overworld_node_121<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_120(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn overworld_node_122<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D_THICKNESS,
            pos.x as f64 * 2f64,
            pos.y as f64 * 1f64,
            pos.z as f64 * 2f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_123<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_122(pos, ctx) * -0.35000000000000003f64
    }
    #[inline(always)]
    pub fn overworld_node_124<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_123(pos, ctx) + -0.95f64
    }
    #[inline(always)]
    pub fn overworld_node_125<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(125usize, WrapperType::CacheOnce, pos, &overworld_node_124)
    }
    #[inline(always)]
    pub fn overworld_node_126<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_125(pos, ctx) * 0.083f64
    }
    #[inline(always)]
    pub fn overworld_node_127<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_121(pos, ctx) + overworld_node_126(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_128<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D_ELEVATION,
            pos.x as f64 * 1f64,
            pos.y as f64 * 0f64,
            pos.z as f64 * 1f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_129<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_128(pos, ctx) * 8f64
    }
    #[inline(always)]
    pub fn overworld_node_130<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(130usize, WrapperType::CacheFlat, pos, &overworld_node_129)
    }
    #[inline(always)]
    pub fn overworld_node_131<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let _ = ctx;
        let y = pos.y as f64;
        let clamped = y.clamp(-64f64, 320f64);
        let delta = (clamped - -64f64) / (320f64 - -64f64);
        8f64 + delta * (-40f64 - 8f64)
    }
    #[inline(always)]
    pub fn overworld_node_132<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_130(pos, ctx) + overworld_node_131(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_133<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_132(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn overworld_node_134<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_133(pos, ctx) + overworld_node_125(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_135<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let v = overworld_node_134(pos, ctx);
        v * v * v
    }
    #[inline(always)]
    pub fn overworld_node_136<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_127(pos, ctx).max(overworld_node_135(pos, ctx))
    }
    #[inline(always)]
    pub fn overworld_node_137<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_136(pos, ctx).clamp(-1f64, 1f64)
    }
    #[inline(always)]
    pub fn overworld_node_138<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_137(pos, ctx) + overworld_node_66(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_139<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_109(pos, ctx).min(overworld_node_138(pos, ctx))
    }
    #[inline(always)]
    pub fn overworld_node_140<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::PILLAR,
            pos.x as f64 * 25f64,
            pos.y as f64 * 0.3f64,
            pos.z as f64 * 25f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_141<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_140(pos, ctx) * 2f64
    }
    #[inline(always)]
    pub fn overworld_node_142<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::PILLAR_RARENESS,
            pos.x as f64 * 1f64,
            pos.y as f64 * 1f64,
            pos.z as f64 * 1f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_143<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_142(pos, ctx) * -1f64
    }
    #[inline(always)]
    pub fn overworld_node_144<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_143(pos, ctx) + -1f64
    }
    #[inline(always)]
    pub fn overworld_node_145<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_141(pos, ctx) + overworld_node_144(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_146<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::PILLAR_THICKNESS,
            pos.x as f64 * 1f64,
            pos.y as f64 * 1f64,
            pos.z as f64 * 1f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_147<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_146(pos, ctx) * 0.55f64
    }
    #[inline(always)]
    pub fn overworld_node_148<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_147(pos, ctx) + 0.55f64
    }
    #[inline(always)]
    pub fn overworld_node_149<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let v = overworld_node_148(pos, ctx);
        v * v * v
    }
    #[inline(always)]
    pub fn overworld_node_150<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_145(pos, ctx) * overworld_node_149(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_151<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(151usize, WrapperType::CacheOnce, pos, &overworld_node_150)
    }
    #[inline(always)]
    pub fn overworld_node_152<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let _ = (pos, ctx);
        -1000000f64
    }
    #[inline(always)]
    pub fn overworld_node_153<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let val = overworld_node_151(pos, ctx);
        if val >= -1000000f64 && val < 0.03f64 {
            overworld_node_152(pos, ctx)
        } else {
            overworld_node_151(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn overworld_node_154<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_139(pos, ctx).max(overworld_node_153(pos, ctx))
    }
    #[inline(always)]
    pub fn overworld_node_155<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let val = overworld_node_54(pos, ctx);
        if val >= -1000000f64 && val < 1.5625f64 {
            overworld_node_97(pos, ctx)
        } else {
            overworld_node_154(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn overworld_node_156<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_155(pos, ctx) + 0.078125f64
    }
    #[inline(always)]
    pub fn overworld_node_157<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_1(pos, ctx) * overworld_node_156(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_158<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_157(pos, ctx) + -0.078125f64
    }
    #[inline(always)]
    pub fn overworld_node_159<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_158(pos, ctx) + -0.1171875f64
    }
    #[inline(always)]
    pub fn overworld_node_160<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_0(pos, ctx) * overworld_node_159(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_161<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_160(pos, ctx) + 0.1171875f64
    }
    #[inline(always)]
    pub fn overworld_node_162<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let val = overworld_node_161(pos, ctx);
        ctx.sample_blend_density(val, pos)
    }
    #[inline(always)]
    pub fn overworld_node_163<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_162(pos, ctx) * 0.64f64
    }
    #[inline(always)]
    pub fn overworld_node_164<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(
            164usize,
            WrapperType::Interpolated,
            pos,
            &overworld_node_163,
        )
    }
    #[inline(always)]
    pub fn overworld_node_165<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let c = overworld_node_164(pos, ctx).clamp(-1.0, 1.0);
        c / 2.0 - c * c * c / 24.0
    }
    #[inline(always)]
    pub fn overworld_node_166<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let _ = ctx;
        let y = pos.y as f64;
        let clamped = y.clamp(-4064f64, 4062f64);
        let delta = (clamped - -4064f64) / (4062f64 - -4064f64);
        -4064f64 + delta * (4062f64 - -4064f64)
    }
    #[inline(always)]
    pub fn overworld_node_167<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::NOODLE,
            pos.x as f64 * 1f64,
            pos.y as f64 * 1f64,
            pos.z as f64 * 1f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_168<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let _ = (pos, ctx);
        -1f64
    }
    #[inline(always)]
    pub fn overworld_node_169<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let val = overworld_node_166(pos, ctx);
        if val >= -60f64 && val < 321f64 {
            overworld_node_167(pos, ctx)
        } else {
            overworld_node_168(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn overworld_node_170<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(
            170usize,
            WrapperType::Interpolated,
            pos,
            &overworld_node_169,
        )
    }
    #[inline(always)]
    pub fn overworld_node_171<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let _ = (pos, ctx);
        64f64
    }
    #[inline(always)]
    pub fn overworld_node_172<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::NOODLE_THICKNESS,
            pos.x as f64 * 1f64,
            pos.y as f64 * 1f64,
            pos.z as f64 * 1f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_173<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_172(pos, ctx) * -0.025f64
    }
    #[inline(always)]
    pub fn overworld_node_174<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_173(pos, ctx) + -0.07500000000000001f64
    }
    #[inline(always)]
    pub fn overworld_node_175<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let val = overworld_node_166(pos, ctx);
        if val >= -60f64 && val < 321f64 {
            overworld_node_174(pos, ctx)
        } else {
            overworld_node_12(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn overworld_node_176<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(
            176usize,
            WrapperType::Interpolated,
            pos,
            &overworld_node_175,
        )
    }
    #[inline(always)]
    pub fn overworld_node_177<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::NOODLE_RIDGE_A,
            pos.x as f64 * 2.6666666666666665f64,
            pos.y as f64 * 2.6666666666666665f64,
            pos.z as f64 * 2.6666666666666665f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_178<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let val = overworld_node_166(pos, ctx);
        if val >= -60f64 && val < 321f64 {
            overworld_node_177(pos, ctx)
        } else {
            overworld_node_12(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn overworld_node_179<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(
            179usize,
            WrapperType::Interpolated,
            pos,
            &overworld_node_178,
        )
    }
    #[inline(always)]
    pub fn overworld_node_180<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_179(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn overworld_node_181<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::NOODLE_RIDGE_B,
            pos.x as f64 * 2.6666666666666665f64,
            pos.y as f64 * 2.6666666666666665f64,
            pos.z as f64 * 2.6666666666666665f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_182<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let val = overworld_node_166(pos, ctx);
        if val >= -60f64 && val < 321f64 {
            overworld_node_181(pos, ctx)
        } else {
            overworld_node_12(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn overworld_node_183<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(
            183usize,
            WrapperType::Interpolated,
            pos,
            &overworld_node_182,
        )
    }
    #[inline(always)]
    pub fn overworld_node_184<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_183(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn overworld_node_185<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_180(pos, ctx).max(overworld_node_184(pos, ctx))
    }
    #[inline(always)]
    pub fn overworld_node_186<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_185(pos, ctx) * 1.5f64
    }
    #[inline(always)]
    pub fn overworld_node_187<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_176(pos, ctx) + overworld_node_186(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_188<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let val = overworld_node_170(pos, ctx);
        if val >= -1000000f64 && val < 0f64 {
            overworld_node_171(pos, ctx)
        } else {
            overworld_node_187(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn overworld_node_189<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_165(pos, ctx).min(overworld_node_188(pos, ctx))
    }
    #[inline(always)]
    pub fn overworld_node_190<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_beardifier(pos)
    }
    #[inline(always)]
    pub fn overworld_node_191<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_189(pos, ctx) + overworld_node_190(pos, ctx)
    }
    #[inline(always)]
    pub fn overworld_node_192<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(192usize, WrapperType::CellCache, pos, &overworld_node_191)
    }
    #[inline(always)]
    pub fn overworld_node_193<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::AQUIFER_BARRIER,
            pos.x as f64 * 1f64,
            pos.y as f64 * 0.5f64,
            pos.z as f64 * 1f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_194<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::AQUIFER_FLUID_LEVEL_FLOODEDNESS,
            pos.x as f64 * 1f64,
            pos.y as f64 * 0.67f64,
            pos.z as f64 * 1f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_195<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::AQUIFER_FLUID_LEVEL_SPREAD,
            pos.x as f64 * 1f64,
            pos.y as f64 * 0.7142857142857143f64,
            pos.z as f64 * 1f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_196<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::AQUIFER_LAVA,
            pos.x as f64 * 1f64,
            pos.y as f64 * 1f64,
            pos.z as f64 * 1f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_197<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::ORE_VEININESS,
            pos.x as f64 * 1.5f64,
            pos.y as f64 * 1.5f64,
            pos.z as f64 * 1.5f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_198<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let val = overworld_node_166(pos, ctx);
        if val >= -60f64 && val < 51f64 {
            overworld_node_197(pos, ctx)
        } else {
            overworld_node_12(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn overworld_node_199<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(
            199usize,
            WrapperType::Interpolated,
            pos,
            &overworld_node_198,
        )
    }
    #[inline(always)]
    pub fn overworld_node_200<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::ORE_VEIN_A,
            pos.x as f64 * 4f64,
            pos.y as f64 * 4f64,
            pos.z as f64 * 4f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_201<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let val = overworld_node_166(pos, ctx);
        if val >= -60f64 && val < 51f64 {
            overworld_node_200(pos, ctx)
        } else {
            overworld_node_12(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn overworld_node_202<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(
            202usize,
            WrapperType::Interpolated,
            pos,
            &overworld_node_201,
        )
    }
    #[inline(always)]
    pub fn overworld_node_203<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_202(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn overworld_node_204<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::ORE_VEIN_B,
            pos.x as f64 * 4f64,
            pos.y as f64 * 4f64,
            pos.z as f64 * 4f64,
        )
    }
    #[inline(always)]
    pub fn overworld_node_205<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let val = overworld_node_166(pos, ctx);
        if val >= -60f64 && val < 51f64 {
            overworld_node_204(pos, ctx)
        } else {
            overworld_node_12(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn overworld_node_206<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(
            206usize,
            WrapperType::Interpolated,
            pos,
            &overworld_node_205,
        )
    }
    #[inline(always)]
    pub fn overworld_node_207<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_206(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn overworld_node_208<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_203(pos, ctx).max(overworld_node_207(pos, ctx))
    }
    #[inline(always)]
    pub fn overworld_node_209<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_208(pos, ctx) + -0.07999999821186066f64
    }
    #[inline(always)]
    pub fn overworld_node_210<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::ORE_GAP,
            pos.x as f64 * 1f64,
            pos.y as f64 * 1f64,
            pos.z as f64 * 1f64,
        )
    }
    #[inline(always)]
    pub fn sample_final_density<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_192(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_barrier_noise<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_193(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_fluid_level_floodedness_noise<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_194(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_fluid_level_spread_noise<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_195(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_lava_noise<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_196(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_vein_toggle<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_199(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_vein_ridged<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_209(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_vein_gap<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_210(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_erosion<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_19(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_depth<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        overworld_node_33(pos, ctx)
    }
}
pub mod nether_noise_evaluator {
    use super::*;
    #[inline(always)]
    pub fn nether_node_0<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let _ = ctx;
        let y = pos.y as f64;
        let clamped = y.clamp(-8f64, 24f64);
        let delta = (clamped - -8f64) / (24f64 - -8f64);
        0f64 + delta * (1f64 - 0f64)
    }
    #[inline(always)]
    pub fn nether_node_1<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let _ = ctx;
        let y = pos.y as f64;
        let clamped = y.clamp(104f64, 128f64);
        let delta = (clamped - 104f64) / (128f64 - 104f64);
        1f64 + delta * (0f64 - 1f64)
    }
    #[inline(always)]
    pub fn nether_node_2<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_interpolated_noise(pos)
    }
    #[inline(always)]
    pub fn nether_node_3<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        nether_node_2(pos, ctx) + -0.9375f64
    }
    #[inline(always)]
    pub fn nether_node_4<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        nether_node_1(pos, ctx) * nether_node_3(pos, ctx)
    }
    #[inline(always)]
    pub fn nether_node_5<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        nether_node_4(pos, ctx) + 0.9375f64
    }
    #[inline(always)]
    pub fn nether_node_6<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        nether_node_5(pos, ctx) + -2.5f64
    }
    #[inline(always)]
    pub fn nether_node_7<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        nether_node_0(pos, ctx) * nether_node_6(pos, ctx)
    }
    #[inline(always)]
    pub fn nether_node_8<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        nether_node_7(pos, ctx) + 2.5f64
    }
    #[inline(always)]
    pub fn nether_node_9<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let val = nether_node_8(pos, ctx);
        ctx.sample_blend_density(val, pos)
    }
    #[inline(always)]
    pub fn nether_node_10<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        nether_node_9(pos, ctx) * 0.64f64
    }
    #[inline(always)]
    pub fn nether_node_11<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(11usize, WrapperType::Interpolated, pos, &nether_node_10)
    }
    #[inline(always)]
    pub fn nether_node_12<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let c = nether_node_11(pos, ctx).clamp(-1.0, 1.0);
        c / 2.0 - c * c * c / 24.0
    }
    #[inline(always)]
    pub fn nether_node_13<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_beardifier(pos)
    }
    #[inline(always)]
    pub fn nether_node_14<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        nether_node_12(pos, ctx) + nether_node_13(pos, ctx)
    }
    #[inline(always)]
    pub fn nether_node_15<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(15usize, WrapperType::CellCache, pos, &nether_node_14)
    }
    #[inline(always)]
    pub fn nether_node_16<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let _ = (pos, ctx);
        0f64
    }
    #[inline(always)]
    pub fn sample_final_density<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        nether_node_15(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_barrier_noise<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        nether_node_16(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_fluid_level_floodedness_noise<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        nether_node_16(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_fluid_level_spread_noise<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        nether_node_16(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_lava_noise<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        nether_node_16(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_vein_toggle<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        nether_node_16(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_vein_ridged<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        nether_node_16(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_vein_gap<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        nether_node_16(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_erosion<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        nether_node_16(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_depth<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        nether_node_16(pos, ctx)
    }
}
pub mod end_noise_evaluator {
    use super::*;
    #[inline(always)]
    pub fn end_node_0<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let _ = ctx;
        let y = pos.y as f64;
        let clamped = y.clamp(4f64, 32f64);
        let delta = (clamped - 4f64) / (32f64 - 4f64);
        0f64 + delta * (1f64 - 0f64)
    }
    #[inline(always)]
    pub fn end_node_1<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let _ = ctx;
        let y = pos.y as f64;
        let clamped = y.clamp(56f64, 312f64);
        let delta = (clamped - 56f64) / (312f64 - 56f64);
        1f64 + delta * (0f64 - 1f64)
    }
    #[inline(always)]
    pub fn end_node_2<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_end_islands(pos)
    }
    #[inline(always)]
    pub fn end_node_3<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_interpolated_noise(pos)
    }
    #[inline(always)]
    pub fn end_node_4<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        end_node_2(pos, ctx) + end_node_3(pos, ctx)
    }
    #[inline(always)]
    pub fn end_node_5<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        end_node_4(pos, ctx) + 23.4375f64
    }
    #[inline(always)]
    pub fn end_node_6<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        end_node_1(pos, ctx) * end_node_5(pos, ctx)
    }
    #[inline(always)]
    pub fn end_node_7<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        end_node_6(pos, ctx) + -23.4375f64
    }
    #[inline(always)]
    pub fn end_node_8<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        end_node_7(pos, ctx) + 0.234375f64
    }
    #[inline(always)]
    pub fn end_node_9<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        end_node_0(pos, ctx) * end_node_8(pos, ctx)
    }
    #[inline(always)]
    pub fn end_node_10<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        end_node_9(pos, ctx) + -0.234375f64
    }
    #[inline(always)]
    pub fn end_node_11<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let val = end_node_10(pos, ctx);
        ctx.sample_blend_density(val, pos)
    }
    #[inline(always)]
    pub fn end_node_12<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        end_node_11(pos, ctx) * 0.64f64
    }
    #[inline(always)]
    pub fn end_node_13<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(13usize, WrapperType::Interpolated, pos, &end_node_12)
    }
    #[inline(always)]
    pub fn end_node_14<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let c = end_node_13(pos, ctx).clamp(-1.0, 1.0);
        c / 2.0 - c * c * c / 24.0
    }
    #[inline(always)]
    pub fn end_node_15<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let _ = (pos, ctx);
        0f64
    }
    #[inline(always)]
    pub fn end_node_16<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        ctx.sample_wrapper(16usize, WrapperType::Cache2D, pos, &end_node_2)
    }
    #[inline(always)]
    pub fn sample_final_density<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        end_node_14(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_barrier_noise<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        end_node_15(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_fluid_level_floodedness_noise<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        end_node_15(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_fluid_level_spread_noise<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        end_node_15(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_lava_noise<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        end_node_15(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_vein_toggle<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        end_node_15(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_vein_ridged<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        end_node_15(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_vein_gap<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        end_node_15(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_erosion<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        end_node_16(pos, ctx)
    }
    #[inline(always)]
    pub fn sample_depth<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        end_node_15(pos, ctx)
    }
}
pub struct NoiseData {
    pub noise_id: DoublePerlinNoiseParameters,
    pub xz_scale: f64,
    pub y_scale: f64,
}
pub struct FindTopSurfaceData {
    pub lower_bound: i32,
    pub cell_height: i32,
}
pub struct ShiftedNoiseData {
    pub xz_scale: f64,
    pub y_scale: f64,
    pub noise_id: DoublePerlinNoiseParameters,
}
pub struct InterpolatedNoiseSamplerData {
    pub scaled_xz_scale: f64,
    pub scaled_y_scale: f64,
    pub xz_factor: f64,
    pub y_factor: f64,
    pub smear_scale_multiplier: f64,
}
pub struct ClampedYGradientData {
    pub from_y: f64,
    pub to_y: f64,
    pub from_value: f64,
    pub to_value: f64,
}
impl ClampedYGradientData {
    #[inline]
    #[must_use]
    pub fn apply_y(&self, y: f64) -> f64 {
        let clamped = y.clamp(self.from_y, self.to_y);
        let delta = (clamped - self.from_y) / (self.to_y - self.from_y);
        self.from_value + delta * (self.to_value - self.from_value)
    }
}
#[derive(Copy, Clone)]
pub enum BinaryOperation {
    Add,
    Mul,
    Min,
    Max,
}
pub struct BinaryData {
    pub operation: BinaryOperation,
}
impl BinaryData {
    #[inline]
    #[must_use]
    pub const fn apply_density(&self, a: f64, b: f64) -> f64 {
        match self.operation {
            BinaryOperation::Add => a + b,
            BinaryOperation::Mul => a * b,
            BinaryOperation::Min => a.min(b),
            BinaryOperation::Max => a.max(b),
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
    pub argument: f64,
}
impl LinearData {
    #[inline]
    #[must_use]
    pub const fn apply_density(&self, density: f64) -> f64 {
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
}
pub struct UnaryData {
    pub operation: UnaryOperation,
}
impl UnaryData {
    #[inline]
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub const fn apply_density(&self, density: f64) -> f64 {
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
                    f64::INFINITY
                } else {
                    1.0 / density
                }
            }
        }
    }
}
pub struct ClampData {
    pub min_value: f64,
    pub max_value: f64,
}
impl ClampData {
    #[inline]
    #[must_use]
    pub const fn apply_density(&self, density: f64) -> f64 {
        density.clamp(self.min_value, self.max_value)
    }
}
pub struct RangeChoiceData {
    pub min_inclusive: f64,
    pub max_exclusive: f64,
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
        thresholds: &'static [f64],
        functions_indices: &'static [usize],
    },
    Wrapper {
        input_index: usize,
        wrapper: WrapperType,
    },
    Constant {
        value: f64,
    },
    ClampedYGradient {
        data: &'static ClampedYGradientData,
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
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: -64f64,
                    to_y: -40f64,
                    from_value: 0f64,
                    to_value: 1f64,
                },
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: 240f64,
                    to_y: 256f64,
                    from_value: 1f64,
                    to_value: 0f64,
                },
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: -64f64,
                    to_y: 320f64,
                    from_value: 1.5f64,
                    to_value: -1.5f64,
                },
            },
            BaseNoiseFunctionComponent::BlendOffset,
            BaseNoiseFunctionComponent::BlendAlpha,
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 4usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 5usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 6usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 3usize,
                argument2_index: 7usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::ShiftA {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 9usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 10usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Constant { value: 0f64 },
            BaseNoiseFunctionComponent::ShiftB {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 13usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 14usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 11usize,
                shift_y_index: 12usize,
                shift_z_index: 15usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::CONTINENTALNESS,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 16usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 11usize,
                shift_y_index: 12usize,
                shift_z_index: 15usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::EROSION,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 18usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 11usize,
                shift_y_index: 12usize,
                shift_z_index: 15usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::RIDGE,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 20usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 21usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 22usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.6666666666666666f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 23usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 24usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.3333333333333333f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 25usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -3f64,
                },
            },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 17usize,
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
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                input_index: 27usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.5037500262260437f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 28usize,
                argument2_index: 5usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 8usize,
                argument2_index: 29usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 30usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 31usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 2usize,
                argument2_index: 32usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 17usize,
                    points: &[
                        SplinePoint {
                            location: -0.11f32,
                            value: &SplineRepr::Fixed { value: 0f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: 0.03f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
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
                                                        location_function_index: 21usize,
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
                                            location_function_index: 26usize,
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
                                                        location_function_index: 21usize,
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
                                            location_function_index: 26usize,
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
                                                        location_function_index: 21usize,
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
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -1f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.19999999f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.44999996f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
                                                        location_function_index: 21usize,
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
                                            location_function_index: 26usize,
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
                                                        location_function_index: 21usize,
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
                                            location_function_index: 26usize,
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
                                                        location_function_index: 21usize,
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
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 4usize,
                argument2_index: 34usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 35usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 36usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::JAGGED,
                    xz_scale: 1500f64,
                    y_scale: 0f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 38usize,
                data: &UnaryData {
                    operation: UnaryOperation::HalfNegative,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 37usize,
                argument2_index: 39usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 40usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 33usize,
                argument2_index: 41usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 17usize,
                    points: &[
                        SplinePoint {
                            location: -0.19f32,
                            value: &SplineRepr::Fixed { value: 3.95f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.15f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.45f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.45f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.7f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.7f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
            BaseNoiseFunctionComponent::Linear {
                input_index: 43usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -10f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 4usize,
                argument2_index: 44usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 45usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 10f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 46usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 47usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 42usize,
                argument2_index: 48usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 49usize,
                data: &UnaryData {
                    operation: UnaryOperation::QuarterNegative,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 50usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 4f64,
                },
            },
            BaseNoiseFunctionComponent::InterpolatedNoiseSampler {
                data: &InterpolatedNoiseSamplerData {
                    scaled_xz_scale: 0.25f64,
                    scaled_y_scale: 0.25f64,
                    xz_factor: 80f64,
                    y_factor: 160f64,
                    smear_scale_multiplier: 8f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 51usize,
                argument2_index: 52usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 53usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::CAVE_ENTRANCE,
                    xz_scale: 0.75f64,
                    y_scale: 0.5f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 55usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.37f64,
                },
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: -10f64,
                    to_y: 30f64,
                    from_value: 0.3f64,
                    to_value: 0f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 56usize,
                argument2_index: 57usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_ROUGHNESS_MODULATOR,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 59usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.05f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 60usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.05f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_ROUGHNESS,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 62usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 63usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.4f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 61usize,
                argument2_index: 64usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 65usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_RARITY,
                    xz_scale: 2f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 67usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
                    xz_scale: 1.3333333333333333f64,
                    y_scale: 1.3333333333333333f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 69usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.75f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
                    xz_scale: 0.6666666666666666f64,
                    y_scale: 0.6666666666666666f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 72usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1.5f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
                    xz_scale: 0.5f64,
                    y_scale: 0.5f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 74usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 2f64,
                },
            },
            BaseNoiseFunctionComponent::IntervalSelect {
                input_index: 68usize,
                thresholds: &[-0.5f64, 0f64, 0.5f64],
                functions_indices: &[70usize, 71usize, 73usize, 75usize],
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 76usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
                    xz_scale: 1.3333333333333333f64,
                    y_scale: 1.3333333333333333f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 78usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.75f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
                    xz_scale: 0.6666666666666666f64,
                    y_scale: 0.6666666666666666f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 81usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1.5f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
                    xz_scale: 0.5f64,
                    y_scale: 0.5f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 83usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 2f64,
                },
            },
            BaseNoiseFunctionComponent::IntervalSelect {
                input_index: 68usize,
                thresholds: &[-0.5f64, 0f64, 0.5f64],
                functions_indices: &[79usize, 80usize, 82usize, 84usize],
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 85usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 77usize,
                argument2_index: 86usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_THICKNESS,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 88usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.011499999999999996f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 89usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.0765f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 87usize,
                argument2_index: 90usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 91usize,
                data: &ClampData {
                    min_value: -1f64,
                    max_value: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 66usize,
                argument2_index: 92usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 58usize,
                argument2_index: 93usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 94usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 95usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 5f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 54usize,
                argument2_index: 96usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::CAVE_LAYER,
                    xz_scale: 1f64,
                    y_scale: 8f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 98usize,
                data: &UnaryData {
                    operation: UnaryOperation::Square,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 99usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 4f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::CAVE_CHEESE,
                    xz_scale: 1f64,
                    y_scale: 0.6666666666666666f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 101usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.27f64,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 102usize,
                data: &ClampData {
                    min_value: -1f64,
                    max_value: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 54usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.64f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 104usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 1.5f64,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 105usize,
                data: &ClampData {
                    min_value: 0f64,
                    max_value: 0.5f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 103usize,
                argument2_index: 106usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 100usize,
                argument2_index: 107usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 108usize,
                argument2_index: 95usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D_MODULATOR,
                    xz_scale: 2f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D,
                    xz_scale: 2f64,
                    y_scale: 2f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 111usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.5f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D,
                    xz_scale: 1.3333333333333333f64,
                    y_scale: 1.3333333333333333f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 113usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.75f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D,
                    xz_scale: 0.5f64,
                    y_scale: 0.5f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 116usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 2f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D,
                    xz_scale: 0.3333333333333333f64,
                    y_scale: 0.3333333333333333f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 118usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 3f64,
                },
            },
            BaseNoiseFunctionComponent::IntervalSelect {
                input_index: 110usize,
                thresholds: &[-0.75f64, -0.5f64, 0.5f64, 0.75f64],
                functions_indices: &[112usize, 114usize, 115usize, 117usize, 119usize],
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 120usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D_THICKNESS,
                    xz_scale: 2f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 122usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.35000000000000003f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 123usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.95f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 124usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 125usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.083f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 121usize,
                argument2_index: 126usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D_ELEVATION,
                    xz_scale: 1f64,
                    y_scale: 0f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 128usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 8f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 129usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: -64f64,
                    to_y: 320f64,
                    from_value: 8f64,
                    to_value: -40f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 130usize,
                argument2_index: 131usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 132usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 133usize,
                argument2_index: 125usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 134usize,
                data: &UnaryData {
                    operation: UnaryOperation::Cube,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 127usize,
                argument2_index: 135usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 136usize,
                data: &ClampData {
                    min_value: -1f64,
                    max_value: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 137usize,
                argument2_index: 66usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 109usize,
                argument2_index: 138usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::PILLAR,
                    xz_scale: 25f64,
                    y_scale: 0.3f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 140usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 2f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::PILLAR_RARENESS,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 142usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 143usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -1f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 141usize,
                argument2_index: 144usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::PILLAR_THICKNESS,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 146usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.55f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 147usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.55f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 148usize,
                data: &UnaryData {
                    operation: UnaryOperation::Cube,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 145usize,
                argument2_index: 149usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 150usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Constant { value: -1000000f64 },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 151usize,
                when_in_range_index: 152usize,
                when_out_range_index: 151usize,
                data: &RangeChoiceData {
                    min_inclusive: -1000000f64,
                    max_exclusive: 0.03f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 139usize,
                argument2_index: 153usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 54usize,
                when_in_range_index: 97usize,
                when_out_range_index: 154usize,
                data: &RangeChoiceData {
                    min_inclusive: -1000000f64,
                    max_exclusive: 1.5625f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 155usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.078125f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 1usize,
                argument2_index: 156usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 157usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.078125f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 158usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.1171875f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 0usize,
                argument2_index: 159usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 160usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.1171875f64,
                },
            },
            BaseNoiseFunctionComponent::BlendDensity {
                input_index: 161usize,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 162usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.64f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 163usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 164usize,
                data: &UnaryData {
                    operation: UnaryOperation::Squeeze,
                },
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: -4064f64,
                    to_y: 4062f64,
                    from_value: -4064f64,
                    to_value: 4062f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::NOODLE,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Constant { value: -1f64 },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 166usize,
                when_in_range_index: 167usize,
                when_out_range_index: 168usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 321f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 169usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Constant { value: 64f64 },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::NOODLE_THICKNESS,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 172usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.025f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 173usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.07500000000000001f64,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 166usize,
                when_in_range_index: 174usize,
                when_out_range_index: 12usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 321f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 175usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::NOODLE_RIDGE_A,
                    xz_scale: 2.6666666666666665f64,
                    y_scale: 2.6666666666666665f64,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 166usize,
                when_in_range_index: 177usize,
                when_out_range_index: 12usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 321f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 178usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 179usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::NOODLE_RIDGE_B,
                    xz_scale: 2.6666666666666665f64,
                    y_scale: 2.6666666666666665f64,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 166usize,
                when_in_range_index: 181usize,
                when_out_range_index: 12usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 321f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 182usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 183usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 180usize,
                argument2_index: 184usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 185usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1.5f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 176usize,
                argument2_index: 186usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 170usize,
                when_in_range_index: 171usize,
                when_out_range_index: 187usize,
                data: &RangeChoiceData {
                    min_inclusive: -1000000f64,
                    max_exclusive: 0f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 165usize,
                argument2_index: 188usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Beardifier,
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 189usize,
                argument2_index: 190usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 191usize,
                wrapper: WrapperType::CellCache,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::AQUIFER_BARRIER,
                    xz_scale: 1f64,
                    y_scale: 0.5f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::AQUIFER_FLUID_LEVEL_FLOODEDNESS,
                    xz_scale: 1f64,
                    y_scale: 0.67f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::AQUIFER_FLUID_LEVEL_SPREAD,
                    xz_scale: 1f64,
                    y_scale: 0.7142857142857143f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::AQUIFER_LAVA,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::ORE_VEININESS,
                    xz_scale: 1.5f64,
                    y_scale: 1.5f64,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 166usize,
                when_in_range_index: 197usize,
                when_out_range_index: 12usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 51f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 198usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::ORE_VEIN_A,
                    xz_scale: 4f64,
                    y_scale: 4f64,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 166usize,
                when_in_range_index: 200usize,
                when_out_range_index: 12usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 51f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 201usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 202usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::ORE_VEIN_B,
                    xz_scale: 4f64,
                    y_scale: 4f64,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 166usize,
                when_in_range_index: 204usize,
                when_out_range_index: 12usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f64,
                    max_exclusive: 51f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 205usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 206usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 203usize,
                argument2_index: 207usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 208usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.07999999821186066f64,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::ORE_GAP,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
        ],
        barrier_noise: 193usize,
        fluid_level_floodedness_noise: 194usize,
        fluid_level_spread_noise: 195usize,
        lava_noise: 196usize,
        erosion: 19usize,
        depth: 33usize,
        final_density: 192usize,
        vein_toggle: 199usize,
        vein_ridged: 209usize,
        vein_gap: 210usize,
    },
    surface_estimator: BaseSurfaceEstimator {
        full_component_stack: &[
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: -64f64,
                    to_y: -40f64,
                    from_value: 0f64,
                    to_value: 1f64,
                },
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: 240f64,
                    to_y: 256f64,
                    from_value: 1f64,
                    to_value: 0f64,
                },
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: -64f64,
                    to_y: 320f64,
                    from_value: 1.5f64,
                    to_value: -1.5f64,
                },
            },
            BaseNoiseFunctionComponent::BlendOffset,
            BaseNoiseFunctionComponent::BlendAlpha,
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 4usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 5usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 6usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 3usize,
                argument2_index: 7usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::ShiftA {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 9usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 10usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Constant { value: 0f64 },
            BaseNoiseFunctionComponent::ShiftB {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 13usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 14usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 11usize,
                shift_y_index: 12usize,
                shift_z_index: 15usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::CONTINENTALNESS,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 16usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 11usize,
                shift_y_index: 12usize,
                shift_z_index: 15usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::EROSION,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 18usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 11usize,
                shift_y_index: 12usize,
                shift_z_index: 15usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::RIDGE,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 20usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 21usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 22usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.6666666666666666f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 23usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 24usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.3333333333333333f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 25usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -3f64,
                },
            },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 17usize,
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
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                input_index: 27usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.5037500262260437f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 28usize,
                argument2_index: 5usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 8usize,
                argument2_index: 29usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 30usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 31usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 32usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 2usize,
                argument2_index: 33usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 17usize,
                    points: &[
                        SplinePoint {
                            location: -0.19f32,
                            value: &SplineRepr::Fixed { value: 3.95f32 },
                            derivative: 0f32,
                        },
                        SplinePoint {
                            location: -0.15f32,
                            value: &SplineRepr::Standard {
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
                                location_function_index: 19usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.6f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 21usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.45f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.45f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.7f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.7f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 21usize,
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
            BaseNoiseFunctionComponent::Linear {
                input_index: 35usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -10f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 4usize,
                argument2_index: 36usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 37usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 10f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 38usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 39usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 40usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 34usize,
                argument2_index: 41usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 42usize,
                data: &UnaryData {
                    operation: UnaryOperation::QuarterNegative,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 43usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 4f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 44usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.703125f64,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 45usize,
                data: &ClampData {
                    min_value: -64f64,
                    max_value: 64f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 46usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.078125f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 1usize,
                argument2_index: 47usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 48usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.078125f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 49usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.1171875f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 0usize,
                argument2_index: 50usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 51usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.1171875f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 52usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.390625f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 41usize,
                data: &UnaryData {
                    operation: UnaryOperation::Invert,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 54usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.2734375f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 33usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -1f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 55usize,
                argument2_index: 56usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 57usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -128f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 58usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 128f64,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 59usize,
                data: &ClampData {
                    min_value: -40f64,
                    max_value: 320f64,
                },
            },
            BaseNoiseFunctionComponent::FindTopSurface {
                density_index: 53usize,
                upper_bound_index: 60usize,
                data: &FindTopSurfaceData {
                    lower_bound: -64i32,
                    cell_height: 8i32,
                },
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
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 1usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Constant { value: 0f64 },
            BaseNoiseFunctionComponent::ShiftB {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 4usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 5usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 2usize,
                shift_y_index: 3usize,
                shift_z_index: 6usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::RIDGE,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 7usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 2usize,
                shift_y_index: 3usize,
                shift_z_index: 6usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::TEMPERATURE,
                },
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 2usize,
                shift_y_index: 3usize,
                shift_z_index: 6usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::VEGETATION,
                },
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 2usize,
                shift_y_index: 3usize,
                shift_z_index: 6usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::CONTINENTALNESS,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 11usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 2usize,
                shift_y_index: 3usize,
                shift_z_index: 6usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::EROSION,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 13usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: -64f64,
                    to_y: 320f64,
                    from_value: 1.5f64,
                    to_value: -1.5f64,
                },
            },
            BaseNoiseFunctionComponent::BlendOffset,
            BaseNoiseFunctionComponent::BlendAlpha,
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 17usize,
                wrapper: WrapperType::CacheOnce,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 18usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 19usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 16usize,
                argument2_index: 20usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 8usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 22usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.6666666666666666f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 23usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 24usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.3333333333333333f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 25usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -3f64,
                },
            },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 12usize,
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
                                location_function_index: 14usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                location_function_index: 14usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                location_function_index: 14usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                location_function_index: 14usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                location_function_index: 14usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
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
                                            location_function_index: 26usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                                            location_function_index: 26usize,
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
                input_index: 27usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.5037500262260437f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 28usize,
                argument2_index: 18usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 21usize,
                argument2_index: 29usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 30usize,
                wrapper: WrapperType::Cache2D,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 31usize,
                wrapper: WrapperType::CacheFlat,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 15usize,
                argument2_index: 32usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
        ],
        temperature: 9usize,
        vegetation: 10usize,
        continents: 12usize,
        erosion: 14usize,
        depth: 33usize,
        ridges: 8usize,
    },
};
pub const NETHER_BASE_NOISE_ROUTER: BaseNoiseRouters = BaseNoiseRouters {
    noise: BaseNoiseRouter {
        full_component_stack: &[
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: -8f64,
                    to_y: 24f64,
                    from_value: 0f64,
                    to_value: 1f64,
                },
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: 104f64,
                    to_y: 128f64,
                    from_value: 1f64,
                    to_value: 0f64,
                },
            },
            BaseNoiseFunctionComponent::InterpolatedNoiseSampler {
                data: &InterpolatedNoiseSamplerData {
                    scaled_xz_scale: 0.25f64,
                    scaled_y_scale: 0.28125f64,
                    xz_factor: 80f64,
                    y_factor: 60f64,
                    smear_scale_multiplier: 8f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 2usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.9375f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 1usize,
                argument2_index: 3usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 4usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.9375f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 5usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -2.5f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 0usize,
                argument2_index: 6usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 7usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 2.5f64,
                },
            },
            BaseNoiseFunctionComponent::BlendDensity {
                input_index: 8usize,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 9usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.64f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 10usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 11usize,
                data: &UnaryData {
                    operation: UnaryOperation::Squeeze,
                },
            },
            BaseNoiseFunctionComponent::Beardifier,
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 12usize,
                argument2_index: 13usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 14usize,
                wrapper: WrapperType::CellCache,
            },
            BaseNoiseFunctionComponent::Constant { value: 0f64 },
        ],
        barrier_noise: 16usize,
        fluid_level_floodedness_noise: 16usize,
        fluid_level_spread_noise: 16usize,
        lava_noise: 16usize,
        erosion: 16usize,
        depth: 16usize,
        final_density: 15usize,
        vein_toggle: 16usize,
        vein_ridged: 16usize,
        vein_gap: 16usize,
    },
    surface_estimator: BaseSurfaceEstimator {
        full_component_stack: &[BaseNoiseFunctionComponent::Constant { value: 0f64 }],
    },
    multi_noise: BaseMultiNoiseRouter {
        full_component_stack: &[
            BaseNoiseFunctionComponent::Constant { value: 0f64 },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 0usize,
                shift_y_index: 0usize,
                shift_z_index: 0usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::NETHER_TEMPERATURE,
                },
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 0usize,
                shift_y_index: 0usize,
                shift_z_index: 0usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::NETHER_VEGETATION,
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
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: 4f64,
                    to_y: 32f64,
                    from_value: 0f64,
                    to_value: 1f64,
                },
            },
            BaseNoiseFunctionComponent::ClampedYGradient {
                data: &ClampedYGradientData {
                    from_y: 56f64,
                    to_y: 312f64,
                    from_value: 1f64,
                    to_value: 0f64,
                },
            },
            BaseNoiseFunctionComponent::EndIslands,
            BaseNoiseFunctionComponent::InterpolatedNoiseSampler {
                data: &InterpolatedNoiseSamplerData {
                    scaled_xz_scale: 0.25f64,
                    scaled_y_scale: 0.5f64,
                    xz_factor: 80f64,
                    y_factor: 160f64,
                    smear_scale_multiplier: 4f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 2usize,
                argument2_index: 3usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 4usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 23.4375f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 1usize,
                argument2_index: 5usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 6usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -23.4375f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 7usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.234375f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 0usize,
                argument2_index: 8usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 9usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.234375f64,
                },
            },
            BaseNoiseFunctionComponent::BlendDensity {
                input_index: 10usize,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 11usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.64f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 12usize,
                wrapper: WrapperType::Interpolated,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 13usize,
                data: &UnaryData {
                    operation: UnaryOperation::Squeeze,
                },
            },
            BaseNoiseFunctionComponent::Constant { value: 0f64 },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 2usize,
                wrapper: WrapperType::Cache2D,
            },
        ],
        barrier_noise: 15usize,
        fluid_level_floodedness_noise: 15usize,
        fluid_level_spread_noise: 15usize,
        lava_noise: 15usize,
        erosion: 16usize,
        depth: 15usize,
        final_density: 14usize,
        vein_toggle: 15usize,
        vein_ridged: 15usize,
        vein_gap: 15usize,
    },
    surface_estimator: BaseSurfaceEstimator {
        full_component_stack: &[BaseNoiseFunctionComponent::Constant { value: 0f64 }],
    },
    multi_noise: BaseMultiNoiseRouter {
        full_component_stack: &[
            BaseNoiseFunctionComponent::Constant { value: 0f64 },
            BaseNoiseFunctionComponent::EndIslands,
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 1usize,
                wrapper: WrapperType::Cache2D,
            },
        ],
        temperature: 0usize,
        vegetation: 0usize,
        continents: 0usize,
        erosion: 2usize,
        depth: 0usize,
        ridges: 0usize,
    },
};
