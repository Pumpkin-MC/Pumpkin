use std::f64::consts;

use pumpkin_data::{Block, BlockState, block_properties::is_air};
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};

use crate::{block::BlockStateCodec, generation::proto_chunk::GenerationCache};

const SEA_LEVEL: i32 = 63; // TODO: use getSeaLevel() instead of hardcoding

pub struct IcebergFeature {
    pub main_block: BlockStateCodec,
}

impl IcebergFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: &str,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let origin = Vector3::new(pos.0.x, SEA_LEVEL, pos.0.z);
        let main_block = self.main_block.get_state();

        let snow_on_top = random.next_f64() > 0.7;
        let shape_angle = random.next_f64() * 2.0 * consts::PI;
        let shape_ellipse_a = 11 - random.next_bounded_i32(5);
        let shape_ellipse_c = 3 + random.next_bounded_i32(3);
        let is_ellipse = random.next_f64() > 0.7;

        let mut over_water_height = if is_ellipse {
            random.next_bounded_i32(6) + 6
        } else {
            random.next_bounded_i32(15) + 3
        };
        if !is_ellipse && random.next_f64() > 0.9 {
            over_water_height += random.next_bounded_i32(19) + 7;
        }

        let under_water_height = (over_water_height + random.next_bounded_i32(11)).min(18);
        let width =
            (over_water_height + random.next_bounded_i32(7) - random.next_bounded_i32(5)).min(11);
        let a = if is_ellipse { shape_ellipse_a } else { 11 };

        // Above-water pass
        for xo in -a..a {
            for zo in -a..a {
                for y_off in 0..over_water_height {
                    let radius = if is_ellipse {
                        self.height_dependent_radius_ellipse(y_off, over_water_height, width)
                    } else {
                        self.height_dependent_radius_round(random, y_off, over_water_height, width)
                    };
                    if is_ellipse || xo < radius {
                        self.generate_iceberg_block(
                            chunk,
                            random,
                            &origin,
                            over_water_height,
                            xo,
                            y_off,
                            zo,
                            radius,
                            a,
                            is_ellipse,
                            shape_ellipse_c,
                            shape_angle,
                            snow_on_top,
                            main_block,
                        );
                    }
                }
            }
        }

        self.smooth(
            chunk,
            &origin,
            width,
            over_water_height,
            is_ellipse,
            shape_ellipse_a,
        );

        // Below-water pass
        for xo in -a..a {
            for zo in -a..a {
                for y_neg in 1..under_water_height {
                    let y_off = -y_neg;
                    let new_a = if is_ellipse {
                        let y_off_sq = (y_off as f64).powi(2) as f32;
                        let denom = under_water_height as f32 * 8.0f32;
                        (a as f32 * (1.0f32 - y_off_sq / denom)).ceil() as i32
                    } else {
                        a
                    };
                    let radius = self.height_dependent_radius_steep(
                        random,
                        -y_off,
                        under_water_height,
                        width,
                    );
                    if xo < radius {
                        self.generate_iceberg_block(
                            chunk,
                            random,
                            &origin,
                            under_water_height,
                            xo,
                            y_off,
                            zo,
                            radius,
                            new_a,
                            is_ellipse,
                            shape_ellipse_c,
                            shape_angle,
                            snow_on_top,
                            main_block,
                        );
                    }
                }
            }
        }

        let do_cut_out = if is_ellipse {
            random.next_f64() > 0.1
        } else {
            random.next_f64() > 0.7
        };
        if do_cut_out {
            self.generate_cut_out(
                chunk,
                random,
                width,
                over_water_height,
                &origin,
                is_ellipse,
                shape_ellipse_a,
                shape_angle,
                shape_ellipse_c,
            );
        }

        true
    }

    #[expect(clippy::too_many_arguments)]
    fn generate_cut_out<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        width: i32,
        height: i32,
        global_origin: &Vector3<i32>,
        is_ellipse: bool,
        shape_ellipse_a: i32,
        shape_angle: f64,
        shape_ellipse_c: i32,
    ) {
        let random_sign_x: i32 = if random.next_bool() { -1 } else { 1 };
        let random_sign_z: i32 = if random.next_bool() { -1 } else { 1 };

        let mut x_off = random.next_bounded_i32((width / 2 - 2).max(1));
        if random.next_bool() {
            x_off = width / 2 + 1 - random.next_bounded_i32((width - width / 2 - 1).max(1));
        }

        let mut z_off = random.next_bounded_i32((width / 2 - 2).max(1));
        if random.next_bool() {
            z_off = width / 2 + 1 - random.next_bounded_i32((width - width / 2 - 1).max(1));
        }

        if is_ellipse {
            let v = random.next_bounded_i32((shape_ellipse_a - 5).max(1));
            x_off = v;
            z_off = v;
        }

        let local_origin = Vector3::new(random_sign_x * x_off, 0, random_sign_z * z_off);
        let angle = if is_ellipse {
            shape_angle + consts::FRAC_PI_2
        } else {
            random.next_f64() * 2.0 * consts::PI
        };

        // Above-water carve
        for y_off in 0..(height - 3) {
            let radius = self.height_dependent_radius_round(random, y_off, height, width);
            self.carve(
                chunk,
                radius,
                y_off,
                global_origin,
                false,
                angle,
                &local_origin,
                shape_ellipse_a,
                shape_ellipse_c,
            );
        }

        // Below-water carve
        let mut y_off = -1i32;
        while y_off > -height + random.next_bounded_i32(5) {
            let radius = self.height_dependent_radius_steep(random, -y_off, height, width);
            self.carve(
                chunk,
                radius,
                y_off,
                global_origin,
                true,
                angle,
                &local_origin,
                shape_ellipse_a,
                shape_ellipse_c,
            );
            y_off -= 1;
        }
    }

    #[expect(clippy::too_many_arguments)]
    fn carve<T: GenerationCache>(
        &self,
        chunk: &mut T,
        radius: i32,
        y_off: i32,
        global_origin: &Vector3<i32>,
        under_water: bool,
        angle: f64,
        local_origin: &Vector3<i32>,
        shape_ellipse_a: i32,
        shape_ellipse_c: i32,
    ) {
        let a = radius + 1 + shape_ellipse_a / 3;
        let c = (radius - 3).min(3) + shape_ellipse_c / 2 - 1;

        for xo in -a..a {
            for zo in -a..a {
                let signed_dist = self.signed_distance_ellipse(
                    xo,
                    zo,
                    local_origin.x,
                    local_origin.z,
                    a,
                    c,
                    angle,
                );
                if signed_dist < 0.0 {
                    let pos = global_origin.add(&Vector3::new(xo, y_off, zo));
                    let raw = GenerationCache::get_block_state(chunk, &pos);
                    let bid = raw.to_block_id();
                    if Self::is_iceberg_state(bid) || bid == Block::SNOW_BLOCK.id {
                        if under_water {
                            chunk.set_block_state(&pos, Block::WATER.default_state);
                        } else {
                            chunk.set_block_state(&pos, Block::AIR.default_state);
                            self.remove_floating_snow_layer(chunk, &pos);
                        }
                    }
                }
            }
        }
    }

    fn remove_floating_snow_layer<T: GenerationCache>(&self, chunk: &mut T, pos: &Vector3<i32>) {
        let above = pos.add(&Vector3::new(0, 1, 0));
        let raw = GenerationCache::get_block_state(chunk, &above);
        if raw.to_block_id() == Block::SNOW.id {
            chunk.set_block_state(&above, Block::AIR.default_state);
        }
    }

    #[expect(clippy::too_many_arguments)]
    fn generate_iceberg_block<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        origin: &Vector3<i32>,
        height: i32,
        xo: i32,
        y_off: i32,
        zo: i32,
        radius: i32,
        a: i32,
        is_ellipse: bool,
        shape_ellipse_c: i32,
        shape_angle: f64,
        snow_on_top: bool,
        main_block: &'static BlockState,
    ) {
        let signed_dist = if is_ellipse {
            self.signed_distance_ellipse(
                xo,
                zo,
                0,
                0,
                a,
                self.get_ellipse_c(y_off, height, shape_ellipse_c),
                shape_angle,
            )
        } else {
            self.signed_distance_circle(xo, zo, 0, 0, radius, random)
        };

        if signed_dist < 0.0 {
            let pos = origin.add(&Vector3::new(xo, y_off, zo));
            let compare_val: f64 = if is_ellipse {
                -0.5
            } else {
                -6.0 - random.next_bounded_i32(3) as f64
            };
            if signed_dist > compare_val && random.next_f64() > 0.9 {
                return;
            }
            self.set_iceberg_block(
                chunk,
                random,
                &pos,
                height - y_off,
                height,
                is_ellipse,
                snow_on_top,
                main_block,
            );
        }
    }

    #[expect(clippy::too_many_arguments)]
    fn set_iceberg_block<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: &Vector3<i32>,
        h_diff: i32,
        height: i32,
        is_ellipse: bool,
        snow_on_top: bool,
        main_block: &'static BlockState,
    ) {
        let raw = GenerationCache::get_block_state(chunk, pos);
        let bid = raw.to_block_id();
        if is_air(raw.0)
            || bid == Block::SNOW_BLOCK.id
            || bid == Block::ICE.id
            || bid == Block::WATER.id
        {
            let randomness = !is_ellipse || random.next_f64() > 0.05;
            let divisor = if is_ellipse { 3 } else { 2 };
            let threshold =
                random.next_bounded_i32((height / divisor).max(1)) as f64 + height as f64 * 0.6;
            if snow_on_top && bid != Block::WATER.id && h_diff as f64 <= threshold && randomness {
                chunk.set_block_state(pos, Block::SNOW_BLOCK.default_state);
            } else {
                chunk.set_block_state(pos, main_block);
            }
        }
    }

    fn smooth<T: GenerationCache>(
        &self,
        chunk: &mut T,
        origin: &Vector3<i32>,
        width: i32,
        height: i32,
        is_ellipse: bool,
        shape_ellipse_a: i32,
    ) {
        let a = if is_ellipse {
            shape_ellipse_a
        } else {
            width / 2
        };
        for x in -a..=a {
            for z in -a..=a {
                for y_off in 0..=height {
                    let pos = origin.add(&Vector3::new(x, y_off, z));
                    let raw = GenerationCache::get_block_state(chunk, &pos);
                    let bid = raw.to_block_id();
                    if Self::is_iceberg_state(bid) || bid == Block::SNOW.id {
                        let below = pos.add(&Vector3::new(0, -1, 0));
                        let below_raw = GenerationCache::get_block_state(chunk, &below);
                        if is_air(below_raw.0) {
                            chunk.set_block_state(&pos, Block::AIR.default_state);
                            let above = pos.add(&Vector3::new(0, 1, 0));
                            chunk.set_block_state(&above, Block::AIR.default_state);
                        } else if Self::is_iceberg_state(bid) {
                            let sides = [
                                pos.add(&Vector3::new(-1, 0, 0)), // west
                                pos.add(&Vector3::new(1, 0, 0)),  // east
                                pos.add(&Vector3::new(0, 0, -1)), // north
                                pos.add(&Vector3::new(0, 0, 1)),  // south
                            ];
                            let non_iceberg_count = sides
                                .iter()
                                .filter(|side| {
                                    !Self::is_iceberg_state(
                                        GenerationCache::get_block_state(chunk, side).to_block_id(),
                                    )
                                })
                                .count();
                            if non_iceberg_count >= 3 {
                                chunk.set_block_state(&pos, Block::AIR.default_state);
                            }
                        }
                    }
                }
            }
        }
    }

    const fn get_ellipse_c(&self, y_off: i32, height: i32, shape_ellipse_c: i32) -> i32 {
        let mut c = shape_ellipse_c;
        if y_off > 0 && height - y_off <= 3 {
            c = shape_ellipse_c - (4 - (height - y_off));
        }
        c
    }

    fn signed_distance_circle(
        &self,
        xo: i32,
        zo: i32,
        ox: i32,
        oz: i32,
        radius: i32,
        random: &mut RandomGenerator,
    ) -> f64 {
        let off = 10.0f32 * random.next_f32().clamp(0.2, 0.8) / radius as f32;
        off as f64 + ((xo - ox) as f64).powi(2) + ((zo - oz) as f64).powi(2)
            - (radius as f64).powi(2)
    }

    #[expect(clippy::too_many_arguments)]
    fn signed_distance_ellipse(
        &self,
        xo: i32,
        zo: i32,
        ox: i32,
        oz: i32,
        a: i32,
        c: i32,
        angle: f64,
    ) -> f64 {
        let dx = (xo - ox) as f64;
        let dz = (zo - oz) as f64;
        let cos = angle.cos();
        let sin = angle.sin();
        ((dx * cos - dz * sin) / a as f64).powi(2) + ((dx * sin + dz * cos) / c as f64).powi(2)
            - 1.0
    }

    fn height_dependent_radius_round(
        &self,
        random: &mut RandomGenerator,
        y_off: i32,
        height: i32,
        width: i32,
    ) -> i32 {
        let k = 3.5f32 - random.next_f32();
        let y_off_sq = (y_off as f64).powi(2) as f32;
        let mut scale = (1.0f32 - y_off_sq / (height as f32 * k)) * width as f32;
        if height > 15 + random.next_bounded_i32(5) {
            let temp_y_off = if y_off < 3 + random.next_bounded_i32(6) {
                y_off / 2
            } else {
                y_off
            };
            scale = (1.0f32 - temp_y_off as f32 / (height as f32 * k * 0.4f32)) * width as f32;
        }
        (scale / 2.0f32).ceil() as i32
    }

    fn height_dependent_radius_ellipse(&self, y_off: i32, height: i32, width: i32) -> i32 {
        let y_off_sq = (y_off as f64).powi(2) as f32;
        let scale = (1.0f32 - y_off_sq / height as f32) * width as f32;
        (scale / 2.0f32).ceil() as i32
    }

    fn height_dependent_radius_steep(
        &self,
        random: &mut RandomGenerator,
        y_off: i32,
        height: i32,
        width: i32,
    ) -> i32 {
        let k = 1.0f32 + random.next_f32() / 2.0f32;
        let scale = (1.0f32 - y_off as f32 / (height as f32 * k)) * width as f32;
        (scale / 2.0f32).ceil() as i32
    }

    fn is_iceberg_state(block_id: u16) -> bool {
        block_id == Block::PACKED_ICE.id
            || block_id == Block::SNOW_BLOCK.id
            || block_id == Block::BLUE_ICE.id
    }
}
