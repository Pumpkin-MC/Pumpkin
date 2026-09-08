/* This file is generated. Do not edit manually. */
use crate::chunk::DoublePerlinNoiseParameters;
pub trait NoiseEvaluationContext {
    fn sample_noise(
        &mut self,
        noise_id: DoublePerlinNoiseParameters,
        x: f64,
        y: f64,
        z: f64,
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
        xz_scale: f64,
        y_scale: f64,
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
        let slice_pos = pumpkin_util::math::vector3::Vector3::new(pos.x, 0i32, pos.z);
        eval_overworld_7(&slice_pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_9<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(9usize, WrapperType::Cache, pos, &eval_overworld_8)
    }
    #[inline(always)]
    pub fn eval_overworld_10<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        0f32
    }
    #[inline(always)]
    pub fn eval_overworld_11<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_shift_b(DoublePerlinNoiseParameters::OFFSET, pos)
    }
    #[inline(always)]
    pub fn eval_overworld_12<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let slice_pos = pumpkin_util::math::vector3::Vector3::new(pos.x, 0i32, pos.z);
        eval_overworld_11(&slice_pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_13<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(13usize, WrapperType::Cache, pos, &eval_overworld_12)
    }
    #[inline(always)]
    pub fn eval_overworld_14<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let sx = eval_overworld_9(pos, ctx);
        let sy = eval_overworld_10(pos, ctx);
        let sz = eval_overworld_13(pos, ctx);
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
    pub fn eval_overworld_15<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let slice_pos = pumpkin_util::math::vector3::Vector3::new(pos.x, 0i32, pos.z);
        eval_overworld_14(&slice_pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_16<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(16usize, WrapperType::Cache, pos, &eval_overworld_15)
    }
    #[inline(always)]
    pub fn eval_overworld_17<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let sx = eval_overworld_9(pos, ctx);
        let sy = eval_overworld_10(pos, ctx);
        let sz = eval_overworld_13(pos, ctx);
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
    pub fn eval_overworld_18<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let slice_pos = pumpkin_util::math::vector3::Vector3::new(pos.x, 0i32, pos.z);
        eval_overworld_17(&slice_pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_19<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(19usize, WrapperType::Cache, pos, &eval_overworld_18)
    }
    #[inline(always)]
    pub fn eval_overworld_20<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let sx = eval_overworld_9(pos, ctx);
        let sy = eval_overworld_10(pos, ctx);
        let sz = eval_overworld_13(pos, ctx);
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
    pub fn eval_overworld_21<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let slice_pos = pumpkin_util::math::vector3::Vector3::new(pos.x, 0i32, pos.z);
        eval_overworld_20(&slice_pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_22<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(22usize, WrapperType::Cache, pos, &eval_overworld_21)
    }
    #[inline(always)]
    pub fn eval_overworld_23<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_22(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_24<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_23(pos, ctx) + -0.6666667f32
    }
    #[inline(always)]
    pub fn eval_overworld_25<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_24(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_26<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_25(pos, ctx) + -0.33333334f32
    }
    #[inline(always)]
    pub fn eval_overworld_27<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_26(pos, ctx) * -3f32
    }
    #[inline(always)]
    pub fn eval_overworld_28<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let location_val = eval_overworld_16(pos, ctx);
        ctx.sample_spline(28usize, location_val, pos)
    }
    #[inline(always)]
    pub fn eval_overworld_29<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_28(pos, ctx) + -0.50375f32
    }
    #[inline(always)]
    pub fn eval_overworld_30<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let a = eval_overworld_5(pos, ctx);
        let f = eval_overworld_6(pos, ctx);
        let s = eval_overworld_29(pos, ctx);
        f + a * (s - f)
    }
    #[inline(always)]
    pub fn eval_overworld_31<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let slice_pos = pumpkin_util::math::vector3::Vector3::new(pos.x, 0i32, pos.z);
        eval_overworld_30(&slice_pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_32<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(32usize, WrapperType::Cache, pos, &eval_overworld_31)
    }
    #[inline(always)]
    pub fn eval_overworld_33<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let slice_pos = pumpkin_util::math::vector3::Vector3::new(pos.x, 0i32, pos.z);
        eval_overworld_32(&slice_pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_34<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_4(pos, ctx) + eval_overworld_33(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_35<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let location_val = eval_overworld_16(pos, ctx);
        ctx.sample_spline(35usize, location_val, pos)
    }
    #[inline(always)]
    pub fn eval_overworld_36<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let a = eval_overworld_5(pos, ctx);
        let f = eval_overworld_10(pos, ctx);
        let s = eval_overworld_35(pos, ctx);
        f + a * (s - f)
    }
    #[inline(always)]
    pub fn eval_overworld_37<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let slice_pos = pumpkin_util::math::vector3::Vector3::new(pos.x, 0i32, pos.z);
        eval_overworld_36(&slice_pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_38<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(38usize, WrapperType::Cache, pos, &eval_overworld_37)
    }
    #[inline(always)]
    pub fn eval_overworld_39<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::JAGGED,
            f64::from(pos.x) * 1500f64,
            f64::from(pos.y) * 0f64,
            f64::from(pos.z) * 1500f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_40<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let v = eval_overworld_39(pos, ctx);
        if v > 0.0 { v } else { v * 0.5 }
    }
    #[inline(always)]
    pub fn eval_overworld_41<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_38(pos, ctx) * eval_overworld_40(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_42<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let slice_pos = pumpkin_util::math::vector3::Vector3::new(pos.x, 0i32, pos.z);
        eval_overworld_41(&slice_pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_43<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(43usize, WrapperType::Cache, pos, &eval_overworld_42)
    }
    #[inline(always)]
    pub fn eval_overworld_44<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let slice_pos = pumpkin_util::math::vector3::Vector3::new(pos.x, 0i32, pos.z);
        eval_overworld_43(&slice_pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_45<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_34(pos, ctx) + eval_overworld_44(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_46<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        10f32
    }
    #[inline(always)]
    pub fn eval_overworld_47<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let location_val = eval_overworld_16(pos, ctx);
        ctx.sample_spline(47usize, location_val, pos)
    }
    #[inline(always)]
    pub fn eval_overworld_48<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let a = eval_overworld_5(pos, ctx);
        let f = eval_overworld_46(pos, ctx);
        let s = eval_overworld_47(pos, ctx);
        f + a * (s - f)
    }
    #[inline(always)]
    pub fn eval_overworld_49<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let slice_pos = pumpkin_util::math::vector3::Vector3::new(pos.x, 0i32, pos.z);
        eval_overworld_48(&slice_pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_50<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(50usize, WrapperType::Cache, pos, &eval_overworld_49)
    }
    #[inline(always)]
    pub fn eval_overworld_51<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let slice_pos = pumpkin_util::math::vector3::Vector3::new(pos.x, 0i32, pos.z);
        eval_overworld_50(&slice_pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_52<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_45(pos, ctx) * eval_overworld_51(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_53<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let v = eval_overworld_52(pos, ctx);
        if v > 0.0 { v } else { v * 0.25 }
    }
    #[inline(always)]
    pub fn eval_overworld_54<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_53(pos, ctx) * 4f32
    }
    #[inline(always)]
    pub fn eval_overworld_55<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_interpolated_noise(pos)
    }
    #[inline(always)]
    pub fn eval_overworld_56<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_54(pos, ctx) + eval_overworld_55(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_57<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(57usize, WrapperType::Cache, pos, &eval_overworld_56)
    }
    #[inline(always)]
    pub fn eval_overworld_58<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::CAVE_ENTRANCE,
            f64::from(pos.x) * 0.75f64,
            f64::from(pos.y) * 0.5f64,
            f64::from(pos.z) * 0.75f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_59<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_58(pos, ctx) + 0.37f32
    }
    #[inline(always)]
    pub fn eval_overworld_60<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = ctx;
        let coord = pos.y;
        let rel = coord.clamp(-10i32, 30i32) - -10i32;
        0.3f32 + rel as f32 * -0.0075000003f32
    }
    #[inline(always)]
    pub fn eval_overworld_61<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_59(pos, ctx) + eval_overworld_60(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_62<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_ROUGHNESS_MODULATOR,
            f64::from(pos.x) * 1f64,
            f64::from(pos.y) * 1f64,
            f64::from(pos.z) * 1f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_63<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_62(pos, ctx) * -0.05f32
    }
    #[inline(always)]
    pub fn eval_overworld_64<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_63(pos, ctx) + -0.05f32
    }
    #[inline(always)]
    pub fn eval_overworld_65<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_ROUGHNESS,
            f64::from(pos.x) * 1f64,
            f64::from(pos.y) * 1f64,
            f64::from(pos.z) * 1f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_66<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_65(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_67<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_66(pos, ctx) + -0.4f32
    }
    #[inline(always)]
    pub fn eval_overworld_68<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_64(pos, ctx) * eval_overworld_67(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_69<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(69usize, WrapperType::Cache, pos, &eval_overworld_68)
    }
    #[inline(always)]
    pub fn eval_overworld_70<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_RARITY,
            f64::from(pos.x) * 2f64,
            f64::from(pos.y) * 1f64,
            f64::from(pos.z) * 2f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_71<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(71usize, WrapperType::Cache, pos, &eval_overworld_70)
    }
    #[inline(always)]
    pub fn eval_overworld_72<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
            f64::from(pos.x) * 1.3333333333333333f64,
            f64::from(pos.y) * 1.3333333333333333f64,
            f64::from(pos.z) * 1.3333333333333333f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_73<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_72(pos, ctx) * 0.75f32
    }
    #[inline(always)]
    pub fn eval_overworld_74<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
            f64::from(pos.x) * 1f64,
            f64::from(pos.y) * 1f64,
            f64::from(pos.z) * 1f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_75<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_74(pos, ctx) * 1f32
    }
    #[inline(always)]
    pub fn eval_overworld_76<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
            f64::from(pos.x) * 0.6666666666666666f64,
            f64::from(pos.y) * 0.6666666666666666f64,
            f64::from(pos.z) * 0.6666666666666666f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_77<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_76(pos, ctx) * 1.5f32
    }
    #[inline(always)]
    pub fn eval_overworld_78<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
            f64::from(pos.x) * 0.5f64,
            f64::from(pos.y) * 0.5f64,
            f64::from(pos.z) * 0.5f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_79<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_78(pos, ctx) * 2f32
    }
    #[inline(always)]
    pub fn eval_overworld_80<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let input_val = eval_overworld_71(pos, ctx);
        let thresholds = &[-0.5f32, 0f32, 0.5f32];
        let mut selected = thresholds.len();
        for (i, &t) in thresholds.iter().enumerate() {
            if input_val < t {
                selected = i;
                break;
            }
        }
        match selected {
            0usize => eval_overworld_73(pos, ctx),
            1usize => eval_overworld_75(pos, ctx),
            2usize => eval_overworld_77(pos, ctx),
            _ => eval_overworld_79(pos, ctx),
        }
    }
    #[inline(always)]
    pub fn eval_overworld_81<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_80(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_82<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
            f64::from(pos.x) * 1.3333333333333333f64,
            f64::from(pos.y) * 1.3333333333333333f64,
            f64::from(pos.z) * 1.3333333333333333f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_83<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_82(pos, ctx) * 0.75f32
    }
    #[inline(always)]
    pub fn eval_overworld_84<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
            f64::from(pos.x) * 1f64,
            f64::from(pos.y) * 1f64,
            f64::from(pos.z) * 1f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_85<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_84(pos, ctx) * 1f32
    }
    #[inline(always)]
    pub fn eval_overworld_86<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
            f64::from(pos.x) * 0.6666666666666666f64,
            f64::from(pos.y) * 0.6666666666666666f64,
            f64::from(pos.z) * 0.6666666666666666f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_87<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_86(pos, ctx) * 1.5f32
    }
    #[inline(always)]
    pub fn eval_overworld_88<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
            f64::from(pos.x) * 0.5f64,
            f64::from(pos.y) * 0.5f64,
            f64::from(pos.z) * 0.5f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_89<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_88(pos, ctx) * 2f32
    }
    #[inline(always)]
    pub fn eval_overworld_90<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let input_val = eval_overworld_71(pos, ctx);
        let thresholds = &[-0.5f32, 0f32, 0.5f32];
        let mut selected = thresholds.len();
        for (i, &t) in thresholds.iter().enumerate() {
            if input_val < t {
                selected = i;
                break;
            }
        }
        match selected {
            0usize => eval_overworld_83(pos, ctx),
            1usize => eval_overworld_85(pos, ctx),
            2usize => eval_overworld_87(pos, ctx),
            _ => eval_overworld_89(pos, ctx),
        }
    }
    #[inline(always)]
    pub fn eval_overworld_91<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_90(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_92<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_81(pos, ctx).max(eval_overworld_91(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_overworld_93<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_3D_THICKNESS,
            f64::from(pos.x) * 1f64,
            f64::from(pos.y) * 1f64,
            f64::from(pos.z) * 1f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_94<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_93(pos, ctx) * -0.011500001f32
    }
    #[inline(always)]
    pub fn eval_overworld_95<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_94(pos, ctx) + -0.0765f32
    }
    #[inline(always)]
    pub fn eval_overworld_96<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_92(pos, ctx) + eval_overworld_95(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_97<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_96(pos, ctx).clamp(-1f32, 1f32)
    }
    #[inline(always)]
    pub fn eval_overworld_98<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_69(pos, ctx) + eval_overworld_97(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_99<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_61(pos, ctx).min(eval_overworld_98(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_overworld_100<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(100usize, WrapperType::Cache, pos, &eval_overworld_99)
    }
    #[inline(always)]
    pub fn eval_overworld_101<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_100(pos, ctx) * 5f32
    }
    #[inline(always)]
    pub fn eval_overworld_102<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_57(pos, ctx).min(eval_overworld_101(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_overworld_103<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::CAVE_LAYER,
            f64::from(pos.x) * 1f64,
            f64::from(pos.y) * 8f64,
            f64::from(pos.z) * 1f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_104<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let v = eval_overworld_103(pos, ctx);
        v * v
    }
    #[inline(always)]
    pub fn eval_overworld_105<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_104(pos, ctx) * 4f32
    }
    #[inline(always)]
    pub fn eval_overworld_106<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::CAVE_CHEESE,
            f64::from(pos.x) * 1f64,
            f64::from(pos.y) * 0.6666666666666666f64,
            f64::from(pos.z) * 1f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_107<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_106(pos, ctx) + 0.27f32
    }
    #[inline(always)]
    pub fn eval_overworld_108<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_107(pos, ctx).clamp(-1f32, 1f32)
    }
    #[inline(always)]
    pub fn eval_overworld_109<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_57(pos, ctx) * -0.64f32
    }
    #[inline(always)]
    pub fn eval_overworld_110<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_109(pos, ctx) + 1.5f32
    }
    #[inline(always)]
    pub fn eval_overworld_111<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_110(pos, ctx).clamp(0f32, 0.5f32)
    }
    #[inline(always)]
    pub fn eval_overworld_112<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_108(pos, ctx) + eval_overworld_111(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_113<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_105(pos, ctx) + eval_overworld_112(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_114<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_113(pos, ctx).min(eval_overworld_100(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_overworld_115<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D_MODULATOR,
            f64::from(pos.x) * 2f64,
            f64::from(pos.y) * 1f64,
            f64::from(pos.z) * 2f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_116<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D,
            f64::from(pos.x) * 2f64,
            f64::from(pos.y) * 2f64,
            f64::from(pos.z) * 2f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_117<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_116(pos, ctx) * 0.5f32
    }
    #[inline(always)]
    pub fn eval_overworld_118<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D,
            f64::from(pos.x) * 1.3333333333333333f64,
            f64::from(pos.y) * 1.3333333333333333f64,
            f64::from(pos.z) * 1.3333333333333333f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_119<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_118(pos, ctx) * 0.75f32
    }
    #[inline(always)]
    pub fn eval_overworld_120<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D,
            f64::from(pos.x) * 1f64,
            f64::from(pos.y) * 1f64,
            f64::from(pos.z) * 1f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_121<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_120(pos, ctx) * 1f32
    }
    #[inline(always)]
    pub fn eval_overworld_122<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D,
            f64::from(pos.x) * 0.5f64,
            f64::from(pos.y) * 0.5f64,
            f64::from(pos.z) * 0.5f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_123<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_122(pos, ctx) * 2f32
    }
    #[inline(always)]
    pub fn eval_overworld_124<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D,
            f64::from(pos.x) * 0.3333333333333333f64,
            f64::from(pos.y) * 0.3333333333333333f64,
            f64::from(pos.z) * 0.3333333333333333f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_125<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_124(pos, ctx) * 3f32
    }
    #[inline(always)]
    pub fn eval_overworld_126<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let input_val = eval_overworld_115(pos, ctx);
        let thresholds = &[-0.75f32, -0.5f32, 0.5f32, 0.75f32];
        let mut selected = thresholds.len();
        for (i, &t) in thresholds.iter().enumerate() {
            if input_val < t {
                selected = i;
                break;
            }
        }
        match selected {
            0usize => eval_overworld_117(pos, ctx),
            1usize => eval_overworld_119(pos, ctx),
            2usize => eval_overworld_121(pos, ctx),
            3usize => eval_overworld_123(pos, ctx),
            _ => eval_overworld_125(pos, ctx),
        }
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
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D_THICKNESS,
            f64::from(pos.x) * 2f64,
            f64::from(pos.y) * 1f64,
            f64::from(pos.z) * 2f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_129<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_128(pos, ctx) * -0.34999996f32
    }
    #[inline(always)]
    pub fn eval_overworld_130<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_129(pos, ctx) + -0.95f32
    }
    #[inline(always)]
    pub fn eval_overworld_131<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(131usize, WrapperType::Cache, pos, &eval_overworld_130)
    }
    #[inline(always)]
    pub fn eval_overworld_132<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_131(pos, ctx) * 0.083f32
    }
    #[inline(always)]
    pub fn eval_overworld_133<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_127(pos, ctx) + eval_overworld_132(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_134<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::SPAGHETTI_2D_ELEVATION,
            f64::from(pos.x) * 1f64,
            f64::from(pos.y) * 0f64,
            f64::from(pos.z) * 1f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_135<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_134(pos, ctx) * 8f32
    }
    #[inline(always)]
    pub fn eval_overworld_136<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let slice_pos = pumpkin_util::math::vector3::Vector3::new(pos.x, 0i32, pos.z);
        eval_overworld_135(&slice_pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_137<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(137usize, WrapperType::Cache, pos, &eval_overworld_136)
    }
    #[inline(always)]
    pub fn eval_overworld_138<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let slice_pos = pumpkin_util::math::vector3::Vector3::new(pos.x, 0i32, pos.z);
        eval_overworld_137(&slice_pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_139<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = ctx;
        let coord = pos.y;
        let rel = coord.clamp(-64i32, 320i32) - -64i32;
        8f32 + rel as f32 * -0.125f32
    }
    #[inline(always)]
    pub fn eval_overworld_140<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_138(pos, ctx) + eval_overworld_139(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_141<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_140(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_142<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_141(pos, ctx) + eval_overworld_131(pos, ctx)
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
        eval_overworld_133(pos, ctx).max(eval_overworld_143(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_overworld_145<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_144(pos, ctx).clamp(-1f32, 1f32)
    }
    #[inline(always)]
    pub fn eval_overworld_146<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_145(pos, ctx) + eval_overworld_69(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_147<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_114(pos, ctx).min(eval_overworld_146(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_overworld_148<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::PILLAR,
            f64::from(pos.x) * 25f64,
            f64::from(pos.y) * 0.3f64,
            f64::from(pos.z) * 25f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_149<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_148(pos, ctx) * 2f32
    }
    #[inline(always)]
    pub fn eval_overworld_150<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::PILLAR_RARENESS,
            f64::from(pos.x) * 1f64,
            f64::from(pos.y) * 1f64,
            f64::from(pos.z) * 1f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_151<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_150(pos, ctx) * -1f32
    }
    #[inline(always)]
    pub fn eval_overworld_152<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_151(pos, ctx) + -1f32
    }
    #[inline(always)]
    pub fn eval_overworld_153<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_149(pos, ctx) + eval_overworld_152(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_154<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::PILLAR_THICKNESS,
            f64::from(pos.x) * 1f64,
            f64::from(pos.y) * 1f64,
            f64::from(pos.z) * 1f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_155<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_154(pos, ctx) * 0.55f32
    }
    #[inline(always)]
    pub fn eval_overworld_156<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_155(pos, ctx) + 0.55f32
    }
    #[inline(always)]
    pub fn eval_overworld_157<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let v = eval_overworld_156(pos, ctx);
        v * v * v
    }
    #[inline(always)]
    pub fn eval_overworld_158<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_153(pos, ctx) * eval_overworld_157(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_159<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(159usize, WrapperType::Cache, pos, &eval_overworld_158)
    }
    #[inline(always)]
    pub fn eval_overworld_160<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        -1000000f32
    }
    #[inline(always)]
    pub fn eval_overworld_161<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_159(pos, ctx);
        if val >= -1000000f32 && val < 0.03f32 {
            eval_overworld_160(pos, ctx)
        } else {
            eval_overworld_159(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_162<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_147(pos, ctx).max(eval_overworld_161(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_overworld_163<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_57(pos, ctx);
        if val >= -1000000f32 && val < 1.5625f32 {
            eval_overworld_102(pos, ctx)
        } else {
            eval_overworld_162(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_164<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let a = eval_overworld_2(pos, ctx);
        let f = eval_overworld_3(pos, ctx);
        let s = eval_overworld_163(pos, ctx);
        f + a * (s - f)
    }
    #[inline(always)]
    pub fn eval_overworld_165<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let a = eval_overworld_0(pos, ctx);
        let f = eval_overworld_1(pos, ctx);
        let s = eval_overworld_164(pos, ctx);
        f + a * (s - f)
    }
    #[inline(always)]
    pub fn eval_overworld_166<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_165(pos, ctx);
        ctx.sample_blend_density(val, pos)
    }
    #[inline(always)]
    pub fn eval_overworld_167<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_166(pos, ctx) * 0.64f32
    }
    #[inline(always)]
    pub fn eval_overworld_168<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(
            168usize,
            WrapperType::Interpolated {
                cell_size_xz: 4i32,
                cell_size_y: 8i32,
            },
            pos,
            &eval_overworld_167,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_169<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let c = eval_overworld_168(pos, ctx).clamp(-1.0, 1.0);
        c / 2.0 - c * c * c / 24.0
    }
    #[inline(always)]
    pub fn eval_overworld_170<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = ctx;
        let coord = pos.y;
        let rel = coord.clamp(-4064i32, 4062i32) - -4064i32;
        -4064f32 + rel as f32 * 1f32
    }
    #[inline(always)]
    pub fn eval_overworld_171<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::NOODLE,
            f64::from(pos.x) * 1f64,
            f64::from(pos.y) * 1f64,
            f64::from(pos.z) * 1f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_172<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        -1f32
    }
    #[inline(always)]
    pub fn eval_overworld_173<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_170(pos, ctx);
        if val >= -60f32 && val < 321f32 {
            eval_overworld_171(pos, ctx)
        } else {
            eval_overworld_172(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_174<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(
            174usize,
            WrapperType::Interpolated {
                cell_size_xz: 4i32,
                cell_size_y: 8i32,
            },
            pos,
            &eval_overworld_173,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_175<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        64f32
    }
    #[inline(always)]
    pub fn eval_overworld_176<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::NOODLE_THICKNESS,
            f64::from(pos.x) * 1f64,
            f64::from(pos.y) * 1f64,
            f64::from(pos.z) * 1f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_177<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_176(pos, ctx) * -0.025f32
    }
    #[inline(always)]
    pub fn eval_overworld_178<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_177(pos, ctx) + -0.075f32
    }
    #[inline(always)]
    pub fn eval_overworld_179<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_170(pos, ctx);
        if val >= -60f32 && val < 321f32 {
            eval_overworld_178(pos, ctx)
        } else {
            eval_overworld_10(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_180<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(
            180usize,
            WrapperType::Interpolated {
                cell_size_xz: 4i32,
                cell_size_y: 8i32,
            },
            pos,
            &eval_overworld_179,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_181<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::NOODLE_RIDGE_A,
            f64::from(pos.x) * 2.6666666666666665f64,
            f64::from(pos.y) * 2.6666666666666665f64,
            f64::from(pos.z) * 2.6666666666666665f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_182<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_170(pos, ctx);
        if val >= -60f32 && val < 321f32 {
            eval_overworld_181(pos, ctx)
        } else {
            eval_overworld_10(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_183<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(
            183usize,
            WrapperType::Interpolated {
                cell_size_xz: 4i32,
                cell_size_y: 8i32,
            },
            pos,
            &eval_overworld_182,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_184<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_183(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_185<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::NOODLE_RIDGE_B,
            f64::from(pos.x) * 2.6666666666666665f64,
            f64::from(pos.y) * 2.6666666666666665f64,
            f64::from(pos.z) * 2.6666666666666665f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_186<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_170(pos, ctx);
        if val >= -60f32 && val < 321f32 {
            eval_overworld_185(pos, ctx)
        } else {
            eval_overworld_10(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_187<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(
            187usize,
            WrapperType::Interpolated {
                cell_size_xz: 4i32,
                cell_size_y: 8i32,
            },
            pos,
            &eval_overworld_186,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_188<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_187(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_189<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_184(pos, ctx).max(eval_overworld_188(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_overworld_190<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_189(pos, ctx) * 1.5f32
    }
    #[inline(always)]
    pub fn eval_overworld_191<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_180(pos, ctx) + eval_overworld_190(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_192<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_174(pos, ctx);
        if val >= -1000000f32 && val < 0f32 {
            eval_overworld_175(pos, ctx)
        } else {
            eval_overworld_191(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_193<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_169(pos, ctx).min(eval_overworld_192(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_overworld_194<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_beardifier(pos)
    }
    #[inline(always)]
    pub fn eval_overworld_195<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_193(pos, ctx) + eval_overworld_194(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_196<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::AQUIFER_BARRIER,
            f64::from(pos.x) * 1f64,
            f64::from(pos.y) * 0.5f64,
            f64::from(pos.z) * 1f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_197<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::AQUIFER_FLUID_LEVEL_FLOODEDNESS,
            f64::from(pos.x) * 1f64,
            f64::from(pos.y) * 0.67f64,
            f64::from(pos.z) * 1f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_198<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::AQUIFER_FLUID_LEVEL_SPREAD,
            f64::from(pos.x) * 1f64,
            f64::from(pos.y) * 0.7142857142857143f64,
            f64::from(pos.z) * 1f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_199<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::AQUIFER_LAVA,
            f64::from(pos.x) * 1f64,
            f64::from(pos.y) * 1f64,
            f64::from(pos.z) * 1f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_200<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::ORE_VEININESS,
            f64::from(pos.x) * 1.5f64,
            f64::from(pos.y) * 1.5f64,
            f64::from(pos.z) * 1.5f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_201<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_170(pos, ctx);
        if val >= -64f32 && val < 57f32 {
            eval_overworld_200(pos, ctx)
        } else {
            eval_overworld_10(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_202<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(
            202usize,
            WrapperType::Interpolated {
                cell_size_xz: 4i32,
                cell_size_y: 8i32,
            },
            pos,
            &eval_overworld_201,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_203<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(203usize, WrapperType::Cache, pos, &eval_overworld_202)
    }
    #[inline(always)]
    pub fn eval_overworld_204<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        0.08f32
    }
    #[inline(always)]
    pub fn eval_overworld_205<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::ORE_VEIN_A,
            f64::from(pos.x) * 4f64,
            f64::from(pos.y) * 4f64,
            f64::from(pos.z) * 4f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_206<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        1f32
    }
    #[inline(always)]
    pub fn eval_overworld_207<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_170(pos, ctx);
        if val >= -64f32 && val < 57f32 {
            eval_overworld_205(pos, ctx)
        } else {
            eval_overworld_206(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_208<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(
            208usize,
            WrapperType::Interpolated {
                cell_size_xz: 4i32,
                cell_size_y: 8i32,
            },
            pos,
            &eval_overworld_207,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_209<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_208(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_210<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::ORE_VEIN_B,
            f64::from(pos.x) * 4f64,
            f64::from(pos.y) * 4f64,
            f64::from(pos.z) * 4f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_211<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_170(pos, ctx);
        if val >= -64f32 && val < 57f32 {
            eval_overworld_210(pos, ctx)
        } else {
            eval_overworld_206(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_212<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(
            212usize,
            WrapperType::Interpolated {
                cell_size_xz: 4i32,
                cell_size_y: 8i32,
            },
            pos,
            &eval_overworld_211,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_213<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_212(pos, ctx).abs()
    }
    #[inline(always)]
    pub fn eval_overworld_214<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_209(pos, ctx).max(eval_overworld_213(pos, ctx))
    }
    #[inline(always)]
    pub fn eval_overworld_215<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_204(pos, ctx) - eval_overworld_214(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_216<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_overworld_203(pos, ctx);
        if val >= -0.4f32 && val < 0.4f32 {
            eval_overworld_172(pos, ctx)
        } else {
            eval_overworld_215(pos, ctx)
        }
    }
    #[inline(always)]
    pub fn eval_overworld_217<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(217usize, WrapperType::Cache, pos, &eval_overworld_216)
    }
    #[inline(always)]
    pub fn eval_overworld_218<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        -0.3f32
    }
    #[inline(always)]
    pub fn eval_overworld_219<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_noise(
            DoublePerlinNoiseParameters::ORE_GAP,
            f64::from(pos.x) * 1f64,
            f64::from(pos.y) * 1f64,
            f64::from(pos.z) * 1f64,
        )
    }
    #[inline(always)]
    pub fn eval_overworld_220<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_overworld_218(pos, ctx) - eval_overworld_219(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_overworld_221<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let slice_pos = pumpkin_util::math::vector3::Vector3::new(pos.x, 0i32, pos.z);
        eval_overworld_19(&slice_pos, ctx)
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
        ctx.sample_wrapper(
            9usize,
            WrapperType::Interpolated {
                cell_size_xz: 4i32,
                cell_size_y: 8i32,
            },
            pos,
            &eval_nether_8,
        )
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
        let slice_pos = pumpkin_util::math::vector3::Vector3::new(pos.x, 0i32, pos.z);
        eval_end_13(&slice_pos, ctx)
    }
    #[inline(always)]
    pub fn eval_end_15<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(15usize, WrapperType::Cache, pos, &eval_end_14)
    }
    #[inline(always)]
    pub fn eval_end_16<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let slice_pos = pumpkin_util::math::vector3::Vector3::new(pos.x, 0i32, pos.z);
        eval_end_15(&slice_pos, ctx)
    }
    #[inline(always)]
    pub fn eval_end_17<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_interpolated_noise(pos)
    }
    #[inline(always)]
    pub fn eval_end_18<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_end_16(pos, ctx) + eval_end_17(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_end_19<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let a = eval_end_2(pos, ctx);
        let f = eval_end_3(pos, ctx);
        let s = eval_end_18(pos, ctx);
        f + a * (s - f)
    }
    #[inline(always)]
    pub fn eval_end_20<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let a = eval_end_0(pos, ctx);
        let f = eval_end_1(pos, ctx);
        let s = eval_end_19(pos, ctx);
        f + a * (s - f)
    }
    #[inline(always)]
    pub fn eval_end_21<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let val = eval_end_20(pos, ctx);
        ctx.sample_blend_density(val, pos)
    }
    #[inline(always)]
    pub fn eval_end_22<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_end_21(pos, ctx) * 0.64f32
    }
    #[inline(always)]
    pub fn eval_end_23<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_wrapper(
            23usize,
            WrapperType::Interpolated {
                cell_size_xz: 8i32,
                cell_size_y: 4i32,
            },
            pos,
            &eval_end_22,
        )
    }
    #[inline(always)]
    pub fn eval_end_24<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let c = eval_end_23(pos, ctx).clamp(-1.0, 1.0);
        c / 2.0 - c * c * c / 24.0
    }
    #[inline(always)]
    pub fn eval_end_25<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        ctx.sample_beardifier(pos)
    }
    #[inline(always)]
    pub fn eval_end_26<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        eval_end_24(pos, ctx) + eval_end_25(pos, ctx)
    }
    #[inline(always)]
    pub fn eval_end_27<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f32 {
        let _ = (pos, ctx);
        0f32
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
    pub xz_scale: f64,
    pub y_scale: f64,
    pub xz_factor: f64,
    pub y_factor: f64,
    pub smear_scale_multiplier: f64,
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
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum WrapperType {
    Interpolated { cell_size_xz: i32, cell_size_y: i32 },
    Cache,
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
            BaseNoiseFunctionComponent::Slice {
                input_index: 7usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 8usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Constant { value: 0f32 },
            BaseNoiseFunctionComponent::ShiftB {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 11usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 12usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 9usize,
                shift_y_index: 10usize,
                shift_z_index: 13usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::CONTINENTALNESS,
                },
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 14usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 15usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 9usize,
                shift_y_index: 10usize,
                shift_z_index: 13usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::EROSION,
                },
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 17usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 18usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 9usize,
                shift_y_index: 10usize,
                shift_z_index: 13usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::RIDGE,
                },
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 20usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 21usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 22usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 23usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.6666667f32,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 24usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 25usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.33333334f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 26usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -3f32,
                },
            },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 16usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 27usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 27usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                input_index: 28usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.50375f32,
                },
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 5usize,
                first_index: 6usize,
                second_index: 29usize,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 30usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 31usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 32usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 4usize,
                argument2_index: 33usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 16usize,
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
                                            location_function_index: 27usize,
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
                                                        location_function_index: 22usize,
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
                                            location_function_index: 27usize,
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
                                                        location_function_index: 22usize,
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
                                            location_function_index: 27usize,
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
                                                        location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.19999999f32,
                                                    value: &SplineRepr::Fixed { value: 0f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: 0.44999996f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                                                        location_function_index: 22usize,
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
                                            location_function_index: 27usize,
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
                                                        location_function_index: 22usize,
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
                                            location_function_index: 27usize,
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
                                                        location_function_index: 22usize,
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
                first_index: 10usize,
                second_index: 35usize,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 36usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 37usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::JAGGED,
                    xz_scale: 1500f64,
                    y_scale: 0f64,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 39usize,
                data: &UnaryData {
                    operation: UnaryOperation::HalfNegative,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 38usize,
                argument2_index: 40usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 41usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 42usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 43usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 34usize,
                argument2_index: 44usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Constant { value: 10f32 },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 16usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.45f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.45f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.7f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.7f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                first_index: 46usize,
                second_index: 47usize,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 48usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 49usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 50usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 45usize,
                argument2_index: 51usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 52usize,
                data: &UnaryData {
                    operation: UnaryOperation::QuarterNegative,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 53usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 4f32,
                },
            },
            BaseNoiseFunctionComponent::InterpolatedNoiseSampler {
                data: &InterpolatedNoiseSamplerData {
                    xz_scale: 0.25f64,
                    y_scale: 0.125f64,
                    xz_factor: 80f64,
                    y_factor: 160f64,
                    smear_scale_multiplier: 8f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 54usize,
                argument2_index: 55usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 56usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::CAVE_ENTRANCE,
                    xz_scale: 0.75f64,
                    y_scale: 0.5f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 58usize,
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
                argument1_index: 59usize,
                argument2_index: 60usize,
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
                input_index: 62usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.05f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 63usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.05f32,
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
                input_index: 65usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 66usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.4f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 64usize,
                argument2_index: 67usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 68usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_RARITY,
                    xz_scale: 2f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 70usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
                    xz_scale: 1.3333333333333333f64,
                    y_scale: 1.3333333333333333f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 72usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.75f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_1,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 74usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1f32,
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
                input_index: 76usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1.5f32,
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
                input_index: 78usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 2f32,
                },
            },
            BaseNoiseFunctionComponent::IntervalSelect {
                input_index: 71usize,
                thresholds: &[-0.5f32, 0f32, 0.5f32],
                functions_indices: &[73usize, 75usize, 77usize, 79usize],
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 80usize,
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
                input_index: 82usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.75f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_3D_2,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 84usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1f32,
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
                input_index: 86usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1.5f32,
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
                input_index: 88usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 2f32,
                },
            },
            BaseNoiseFunctionComponent::IntervalSelect {
                input_index: 71usize,
                thresholds: &[-0.5f32, 0f32, 0.5f32],
                functions_indices: &[83usize, 85usize, 87usize, 89usize],
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 90usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 81usize,
                argument2_index: 91usize,
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
                input_index: 93usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.011500001f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 94usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.0765f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 92usize,
                argument2_index: 95usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 96usize,
                data: &ClampData {
                    min_value: -1f32,
                    max_value: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 69usize,
                argument2_index: 97usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 61usize,
                argument2_index: 98usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 99usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 100usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 5f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 57usize,
                argument2_index: 101usize,
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
                input_index: 103usize,
                data: &UnaryData {
                    operation: UnaryOperation::Square,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 104usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 4f32,
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
                input_index: 106usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.27f32,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 107usize,
                data: &ClampData {
                    min_value: -1f32,
                    max_value: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 57usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.64f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 109usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 1.5f32,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 110usize,
                data: &ClampData {
                    min_value: 0f32,
                    max_value: 0.5f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 108usize,
                argument2_index: 111usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 105usize,
                argument2_index: 112usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 113usize,
                argument2_index: 100usize,
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
                input_index: 116usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.5f32,
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
                input_index: 118usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.75f32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::SPAGHETTI_2D,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 120usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1f32,
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
                input_index: 122usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 2f32,
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
                input_index: 124usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 3f32,
                },
            },
            BaseNoiseFunctionComponent::IntervalSelect {
                input_index: 115usize,
                thresholds: &[-0.75f32, -0.5f32, 0.5f32, 0.75f32],
                functions_indices: &[117usize, 119usize, 121usize, 123usize, 125usize],
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 126usize,
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
                input_index: 128usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.34999996f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 129usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.95f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 130usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 131usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.083f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 127usize,
                argument2_index: 132usize,
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
                input_index: 134usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 8f32,
                },
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 135usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 136usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 137usize,
                axis: Axis::Y,
                coordinate: 0i32,
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
                argument1_index: 138usize,
                argument2_index: 139usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 140usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 141usize,
                argument2_index: 131usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 142usize,
                data: &UnaryData {
                    operation: UnaryOperation::Cube,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 133usize,
                argument2_index: 143usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 144usize,
                data: &ClampData {
                    min_value: -1f32,
                    max_value: 1f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 145usize,
                argument2_index: 69usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 114usize,
                argument2_index: 146usize,
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
                input_index: 148usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 2f32,
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
                input_index: 150usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -1f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 151usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -1f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 149usize,
                argument2_index: 152usize,
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
                input_index: 154usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.55f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 155usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 0.55f32,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 156usize,
                data: &UnaryData {
                    operation: UnaryOperation::Cube,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 153usize,
                argument2_index: 157usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 158usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Constant { value: -1000000f32 },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 159usize,
                when_in_range_index: 160usize,
                when_out_range_index: 159usize,
                data: &RangeChoiceData {
                    min_inclusive: -1000000f32,
                    max_exclusive: 0.03f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 147usize,
                argument2_index: 161usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 57usize,
                when_in_range_index: 102usize,
                when_out_range_index: 162usize,
                data: &RangeChoiceData {
                    min_inclusive: -1000000f32,
                    max_exclusive: 1.5625f32,
                },
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 2usize,
                first_index: 3usize,
                second_index: 163usize,
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 0usize,
                first_index: 1usize,
                second_index: 164usize,
            },
            BaseNoiseFunctionComponent::BlendDensity {
                input_index: 165usize,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 166usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.64f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 167usize,
                wrapper: WrapperType::Interpolated {
                    cell_size_xz: 4i32,
                    cell_size_y: 8i32,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 168usize,
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
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Constant { value: -1f32 },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 170usize,
                when_in_range_index: 171usize,
                when_out_range_index: 172usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f32,
                    max_exclusive: 321f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 173usize,
                wrapper: WrapperType::Interpolated {
                    cell_size_xz: 4i32,
                    cell_size_y: 8i32,
                },
            },
            BaseNoiseFunctionComponent::Constant { value: 64f32 },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::NOODLE_THICKNESS,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 176usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -0.025f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 177usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.075f32,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 170usize,
                when_in_range_index: 178usize,
                when_out_range_index: 10usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f32,
                    max_exclusive: 321f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 179usize,
                wrapper: WrapperType::Interpolated {
                    cell_size_xz: 4i32,
                    cell_size_y: 8i32,
                },
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::NOODLE_RIDGE_A,
                    xz_scale: 2.6666666666666665f64,
                    y_scale: 2.6666666666666665f64,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 170usize,
                when_in_range_index: 181usize,
                when_out_range_index: 10usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f32,
                    max_exclusive: 321f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 182usize,
                wrapper: WrapperType::Interpolated {
                    cell_size_xz: 4i32,
                    cell_size_y: 8i32,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 183usize,
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
                input_index: 170usize,
                when_in_range_index: 185usize,
                when_out_range_index: 10usize,
                data: &RangeChoiceData {
                    min_inclusive: -60f32,
                    max_exclusive: 321f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 186usize,
                wrapper: WrapperType::Interpolated {
                    cell_size_xz: 4i32,
                    cell_size_y: 8i32,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 187usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 184usize,
                argument2_index: 188usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 189usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 1.5f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 180usize,
                argument2_index: 190usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 174usize,
                when_in_range_index: 175usize,
                when_out_range_index: 191usize,
                data: &RangeChoiceData {
                    min_inclusive: -1000000f32,
                    max_exclusive: 0f32,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 169usize,
                argument2_index: 192usize,
                data: &BinaryData {
                    operation: BinaryOperation::Min,
                },
            },
            BaseNoiseFunctionComponent::Beardifier,
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 193usize,
                argument2_index: 194usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
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
                input_index: 170usize,
                when_in_range_index: 200usize,
                when_out_range_index: 10usize,
                data: &RangeChoiceData {
                    min_inclusive: -64f32,
                    max_exclusive: 57f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 201usize,
                wrapper: WrapperType::Interpolated {
                    cell_size_xz: 4i32,
                    cell_size_y: 8i32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 202usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Constant { value: 0.08f32 },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::ORE_VEIN_A,
                    xz_scale: 4f64,
                    y_scale: 4f64,
                },
            },
            BaseNoiseFunctionComponent::Constant { value: 1f32 },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 170usize,
                when_in_range_index: 205usize,
                when_out_range_index: 206usize,
                data: &RangeChoiceData {
                    min_inclusive: -64f32,
                    max_exclusive: 57f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 207usize,
                wrapper: WrapperType::Interpolated {
                    cell_size_xz: 4i32,
                    cell_size_y: 8i32,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 208usize,
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
                input_index: 170usize,
                when_in_range_index: 210usize,
                when_out_range_index: 206usize,
                data: &RangeChoiceData {
                    min_inclusive: -64f32,
                    max_exclusive: 57f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 211usize,
                wrapper: WrapperType::Interpolated {
                    cell_size_xz: 4i32,
                    cell_size_y: 8i32,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 212usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 209usize,
                argument2_index: 213usize,
                data: &BinaryData {
                    operation: BinaryOperation::Max,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 204usize,
                argument2_index: 214usize,
                data: &BinaryData {
                    operation: BinaryOperation::Sub,
                },
            },
            BaseNoiseFunctionComponent::RangeChoice {
                input_index: 203usize,
                when_in_range_index: 172usize,
                when_out_range_index: 215usize,
                data: &RangeChoiceData {
                    min_inclusive: -0.4f32,
                    max_exclusive: 0.4f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 216usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Constant { value: -0.3f32 },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::ORE_GAP,
                    xz_scale: 1f64,
                    y_scale: 1f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 218usize,
                argument2_index: 219usize,
                data: &BinaryData {
                    operation: BinaryOperation::Sub,
                },
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 19usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
        ],
        barrier_noise: 196usize,
        fluid_level_floodedness_noise: 197usize,
        fluid_level_spread_noise: 198usize,
        lava_noise: 199usize,
        erosion: 221usize,
        depth: 34usize,
        final_density: 195usize,
        vein_toggle: 203usize,
        vein_ridged: 217usize,
        vein_gap: 220usize,
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
            BaseNoiseFunctionComponent::Slice {
                input_index: 7usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 8usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Constant { value: 0f32 },
            BaseNoiseFunctionComponent::ShiftB {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 11usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 12usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 9usize,
                shift_y_index: 10usize,
                shift_z_index: 13usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::CONTINENTALNESS,
                },
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 14usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 15usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 9usize,
                shift_y_index: 10usize,
                shift_z_index: 13usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::EROSION,
                },
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 17usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 18usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::ShiftedNoise {
                shift_x_index: 9usize,
                shift_y_index: 10usize,
                shift_z_index: 13usize,
                data: &ShiftedNoiseData {
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                    noise_id: DoublePerlinNoiseParameters::RIDGE,
                },
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 20usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 21usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 22usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 23usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.6666667f32,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 24usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 25usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.33333334f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 26usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -3f32,
                },
            },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 16usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 27usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 27usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                                            location_function_index: 27usize,
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
                input_index: 28usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.50375f32,
                },
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 5usize,
                first_index: 6usize,
                second_index: 29usize,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 30usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 31usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 32usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 4usize,
                argument2_index: 33usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Constant { value: 10f32 },
            BaseNoiseFunctionComponent::Spline {
                spline: &SplineRepr::Standard {
                    location_function_index: 16usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.9f32,
                                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.69f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.45f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: 0.45f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.7f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                                            location_function_index: 27usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -0.7f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 22usize,
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
                first_index: 35usize,
                second_index: 36usize,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 37usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 38usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 39usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 34usize,
                argument2_index: 40usize,
                data: &BinaryData {
                    operation: BinaryOperation::Mul,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 41usize,
                data: &UnaryData {
                    operation: UnaryOperation::QuarterNegative,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 42usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 4f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 43usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.703125f32,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 44usize,
                data: &ClampData {
                    min_value: -64f32,
                    max_value: 64f32,
                },
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 2usize,
                first_index: 3usize,
                second_index: 45usize,
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 0usize,
                first_index: 1usize,
                second_index: 46usize,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 47usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.390625f32,
                },
            },
            BaseNoiseFunctionComponent::Constant {
                value: 0.2734375f32,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 49usize,
                argument2_index: 39usize,
                data: &BinaryData {
                    operation: BinaryOperation::Div,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 50usize,
                argument2_index: 32usize,
                data: &BinaryData {
                    operation: BinaryOperation::Sub,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 51usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -128f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 52usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: 128f32,
                },
            },
            BaseNoiseFunctionComponent::Clamp {
                input_index: 53usize,
                data: &ClampData {
                    min_value: -40f32,
                    max_value: 320f32,
                },
            },
            BaseNoiseFunctionComponent::FindTopSurface {
                density_index: 48usize,
                upper_bound_index: 54usize,
                data: &FindTopSurfaceData {
                    lower_bound: -64i32,
                    cell_height: 8i32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 55usize,
                wrapper: WrapperType::Interpolated {
                    cell_size_xz: 16i32,
                    cell_size_y: 1i32,
                },
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 56usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
        ],
    },
    multi_noise: BaseMultiNoiseRouter {
        full_component_stack: &[
            BaseNoiseFunctionComponent::ShiftA {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 0usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 1usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Constant { value: 0f32 },
            BaseNoiseFunctionComponent::ShiftB {
                noise_id: DoublePerlinNoiseParameters::OFFSET,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 4usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 5usize,
                wrapper: WrapperType::Cache,
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
            BaseNoiseFunctionComponent::Slice {
                input_index: 7usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 8usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 9usize,
                axis: Axis::Y,
                coordinate: 0i32,
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
            BaseNoiseFunctionComponent::Slice {
                input_index: 11usize,
                axis: Axis::Y,
                coordinate: 0i32,
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
            BaseNoiseFunctionComponent::Slice {
                input_index: 13usize,
                axis: Axis::Y,
                coordinate: 0i32,
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
            BaseNoiseFunctionComponent::Slice {
                input_index: 15usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 16usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 17usize,
                axis: Axis::Y,
                coordinate: 0i32,
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
            BaseNoiseFunctionComponent::Slice {
                input_index: 19usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 20usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 21usize,
                axis: Axis::Y,
                coordinate: 0i32,
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
                input_index: 9usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 26usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.6666667f32,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 27usize,
                data: &UnaryData {
                    operation: UnaryOperation::Abs,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 28usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.33333334f32,
                },
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 29usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: -3f32,
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
                                location_function_index: 21usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                location_function_index: 21usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                location_function_index: 21usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                location_function_index: 21usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 30usize,
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
                                            location_function_index: 30usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                location_function_index: 21usize,
                                points: &[
                                    SplinePoint {
                                        location: -0.85f32,
                                        value: &SplineRepr::Standard {
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 30usize,
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
                                            location_function_index: 30usize,
                                            points: &[
                                                SplinePoint {
                                                    location: -1f32,
                                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                                    derivative: 0f32,
                                                },
                                                SplinePoint {
                                                    location: -0.4f32,
                                                    value: &SplineRepr::Standard {
                                                        location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                                            location_function_index: 30usize,
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
                input_index: 31usize,
                data: &LinearData {
                    operation: LinearOperation::Add,
                    argument: -0.50375f32,
                },
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 24usize,
                first_index: 25usize,
                second_index: 32usize,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 33usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 34usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 35usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 23usize,
                argument2_index: 36usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
        ],
        temperature: 12usize,
        vegetation: 14usize,
        continents: 18usize,
        erosion: 22usize,
        depth: 37usize,
        ridges: 10usize,
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
                    xz_scale: 0.25f64,
                    y_scale: 0.375f64,
                    xz_factor: 80f64,
                    y_factor: 60f64,
                    smear_scale_multiplier: 8f64,
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
                wrapper: WrapperType::Interpolated {
                    cell_size_xz: 4i32,
                    cell_size_y: 8i32,
                },
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
            BaseNoiseFunctionComponent::Constant { value: 0f32 },
        ],
        barrier_noise: 13usize,
        fluid_level_floodedness_noise: 13usize,
        fluid_level_spread_noise: 13usize,
        lava_noise: 13usize,
        erosion: 13usize,
        depth: 13usize,
        final_density: 12usize,
        vein_toggle: 13usize,
        vein_ridged: 13usize,
        vein_gap: 13usize,
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
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                },
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 1usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Noise {
                data: &NoiseData {
                    noise_id: DoublePerlinNoiseParameters::NETHER_VEGETATION,
                    xz_scale: 0.25f64,
                    y_scale: 0f64,
                },
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 3usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
        ],
        temperature: 2usize,
        vegetation: 4usize,
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
            BaseNoiseFunctionComponent::Slice {
                input_index: 13usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 14usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 15usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::InterpolatedNoiseSampler {
                data: &InterpolatedNoiseSamplerData {
                    xz_scale: 0.25f64,
                    y_scale: 0.25f64,
                    xz_factor: 80f64,
                    y_factor: 160f64,
                    smear_scale_multiplier: 4f64,
                },
            },
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 16usize,
                argument2_index: 17usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 2usize,
                first_index: 3usize,
                second_index: 18usize,
            },
            BaseNoiseFunctionComponent::Lerp {
                alpha_index: 0usize,
                first_index: 1usize,
                second_index: 19usize,
            },
            BaseNoiseFunctionComponent::BlendDensity {
                input_index: 20usize,
            },
            BaseNoiseFunctionComponent::Linear {
                input_index: 21usize,
                data: &LinearData {
                    operation: LinearOperation::Mul,
                    argument: 0.64f32,
                },
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 22usize,
                wrapper: WrapperType::Interpolated {
                    cell_size_xz: 8i32,
                    cell_size_y: 4i32,
                },
            },
            BaseNoiseFunctionComponent::Unary {
                input_index: 23usize,
                data: &UnaryData {
                    operation: UnaryOperation::Squeeze,
                },
            },
            BaseNoiseFunctionComponent::Beardifier,
            BaseNoiseFunctionComponent::Binary {
                argument1_index: 24usize,
                argument2_index: 25usize,
                data: &BinaryData {
                    operation: BinaryOperation::Add,
                },
            },
            BaseNoiseFunctionComponent::Constant { value: 0f32 },
        ],
        barrier_noise: 27usize,
        fluid_level_floodedness_noise: 27usize,
        fluid_level_spread_noise: 27usize,
        lava_noise: 27usize,
        erosion: 16usize,
        depth: 27usize,
        final_density: 26usize,
        vein_toggle: 27usize,
        vein_ridged: 27usize,
        vein_gap: 27usize,
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
            BaseNoiseFunctionComponent::Slice {
                input_index: 10usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
            BaseNoiseFunctionComponent::Wrapper {
                input_index: 11usize,
                wrapper: WrapperType::Cache,
            },
            BaseNoiseFunctionComponent::Slice {
                input_index: 12usize,
                axis: Axis::Y,
                coordinate: 0i32,
            },
        ],
        temperature: 0usize,
        vegetation: 0usize,
        continents: 0usize,
        erosion: 13usize,
        depth: 0usize,
        ridges: 0usize,
    },
};
