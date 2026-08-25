use pumpkin_util::{math::block_box::BlockBox, random::RandomGenerator};

use crate::ProtoChunk;

use super::{BASE_BLACK, BASE_GRAY, BASE_LIGHT, LAMP, MonumentBuilding};

impl MonumentBuilding {
    pub(super) fn place_building(
        &self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        chunk_box: &BlockBox,
    ) {
        let sea_level = self.sea_level;
        let water_height = sea_level.max(64) - self.base.piece.bounding_box.min.y;
        self.base
            .water(chunk, chunk_box, sea_level, 0, 0, 0, 58, water_height, 58);
        self.generate_wing(chunk, chunk_box, false, 0);
        self.generate_wing(chunk, chunk_box, true, 33);
        self.generate_entrance_arches(chunk, chunk_box);
        self.generate_entrance_wall(chunk, chunk_box);
        self.generate_roof(chunk, chunk_box);
        self.generate_lower_wall(chunk, chunk_box);
        self.generate_middle_wall(chunk, chunk_box);
        self.generate_upper_wall(chunk, chunk_box);

        for pillar_x in 0..7 {
            let mut pillar_z = 0;
            while pillar_z < 7 {
                if pillar_z == 0 && pillar_x == 3 {
                    pillar_z = 6;
                }
                let x = pillar_x * 9;
                let z = pillar_z * 9;
                for dx in 0..4 {
                    for dz in 0..4 {
                        self.base
                            .block(chunk, chunk_box, x + dx, 0, z + dz, BASE_LIGHT);
                        self.base.piece.fill_downwards(
                            chunk,
                            BASE_LIGHT,
                            x + dx,
                            -1,
                            z + dz,
                            chunk_box,
                        );
                    }
                }
                pillar_z += if pillar_x == 0 || pillar_x == 6 { 1 } else { 6 };
            }
        }

        for offset in 0..5 {
            self.base.water(
                chunk,
                chunk_box,
                sea_level,
                -1 - offset,
                offset * 2,
                -1 - offset,
                -1 - offset,
                23,
                58 + offset,
            );
            self.base.water(
                chunk,
                chunk_box,
                sea_level,
                58 + offset,
                offset * 2,
                -1 - offset,
                58 + offset,
                23,
                58 + offset,
            );
            self.base.water(
                chunk,
                chunk_box,
                sea_level,
                -offset,
                offset * 2,
                -1 - offset,
                57 + offset,
                23,
                -1 - offset,
            );
            self.base.water(
                chunk,
                chunk_box,
                sea_level,
                -offset,
                offset * 2,
                58 + offset,
                57 + offset,
                23,
                58 + offset,
            );
        }

        for child in &self.children {
            if child.piece.piece.bounding_box.intersects(chunk_box) {
                child.place(chunk, random, chunk_box, sea_level, &self.graph);
            }
        }
    }

    fn generate_wing(&self, chunk: &mut ProtoChunk, chunk_box: &BlockBox, flipped: bool, x: i32) {
        let p = &self.base;
        if !p.chunk_intersects(chunk_box, x, 0, x + 23, 20) {
            return;
        }
        p.fill(chunk, chunk_box, x, 0, 0, x + 24, 0, 20, BASE_GRAY);
        p.water(chunk, chunk_box, self.sea_level, x, 1, 0, x + 24, 10, 20);
        for i in 0..4 {
            p.fill(
                chunk,
                chunk_box,
                x + i,
                i + 1,
                i,
                x + i,
                i + 1,
                20,
                BASE_LIGHT,
            );
            p.fill(
                chunk,
                chunk_box,
                x + i + 7,
                i + 5,
                i + 7,
                x + i + 7,
                i + 5,
                20,
                BASE_LIGHT,
            );
            p.fill(
                chunk,
                chunk_box,
                x + 17 - i,
                i + 5,
                i + 7,
                x + 17 - i,
                i + 5,
                20,
                BASE_LIGHT,
            );
            p.fill(
                chunk,
                chunk_box,
                x + 24 - i,
                i + 1,
                i,
                x + 24 - i,
                i + 1,
                20,
                BASE_LIGHT,
            );
            p.fill(
                chunk,
                chunk_box,
                x + i + 1,
                i + 1,
                i,
                x + 23 - i,
                i + 1,
                i,
                BASE_LIGHT,
            );
            p.fill(
                chunk,
                chunk_box,
                x + i + 8,
                i + 5,
                i + 7,
                x + 16 - i,
                i + 5,
                i + 7,
                BASE_LIGHT,
            );
        }
        p.fill(chunk, chunk_box, x + 4, 4, 4, x + 6, 4, 20, BASE_GRAY);
        p.fill(chunk, chunk_box, x + 7, 4, 4, x + 17, 4, 6, BASE_GRAY);
        p.fill(chunk, chunk_box, x + 18, 4, 4, x + 20, 4, 20, BASE_GRAY);
        p.fill(chunk, chunk_box, x + 11, 8, 11, x + 13, 8, 20, BASE_GRAY);
        for z in [12, 15, 18] {
            p.block(chunk, chunk_box, x + 12, 9, z, BASE_LIGHT);
        }
        let left = x + if flipped { 19 } else { 5 };
        let right = x + if flipped { 5 } else { 19 };
        for z in (5..=20).rev().step_by(3) {
            p.block(chunk, chunk_box, left, 5, z, BASE_LIGHT);
        }
        for z in (7..=19).rev().step_by(3) {
            p.block(chunk, chunk_box, right, 5, z, BASE_LIGHT);
        }
        for i in 0..4 {
            let pos = if flipped {
                x + 24 - (17 - i * 3)
            } else {
                x + 17 - i * 3
            };
            p.block(chunk, chunk_box, pos, 5, 5, BASE_LIGHT);
        }
        p.block(chunk, chunk_box, right, 5, 5, BASE_LIGHT);
        p.fill(chunk, chunk_box, x + 11, 1, 12, x + 13, 7, 12, BASE_GRAY);
        p.fill(chunk, chunk_box, x + 12, 1, 11, x + 12, 7, 13, BASE_GRAY);
    }

    fn generate_entrance_arches(&self, chunk: &mut ProtoChunk, chunk_box: &BlockBox) {
        let p = &self.base;
        if !p.chunk_intersects(chunk_box, 22, 5, 35, 17) {
            return;
        }
        p.water(chunk, chunk_box, self.sea_level, 25, 0, 0, 32, 8, 20);
        for i in 0..4 {
            let z = 5 + i * 4;
            p.fill(chunk, chunk_box, 24, 2, z, 24, 4, z, BASE_LIGHT);
            p.fill(chunk, chunk_box, 22, 4, z, 23, 4, z, BASE_LIGHT);
            p.block(chunk, chunk_box, 25, 5, z, BASE_LIGHT);
            p.block(chunk, chunk_box, 26, 6, z, BASE_LIGHT);
            p.block(chunk, chunk_box, 26, 5, z, LAMP);
            p.fill(chunk, chunk_box, 33, 2, z, 33, 4, z, BASE_LIGHT);
            p.fill(chunk, chunk_box, 34, 4, z, 35, 4, z, BASE_LIGHT);
            p.block(chunk, chunk_box, 32, 5, z, BASE_LIGHT);
            p.block(chunk, chunk_box, 31, 6, z, BASE_LIGHT);
            p.block(chunk, chunk_box, 31, 5, z, LAMP);
            p.fill(chunk, chunk_box, 27, 6, z, 30, 6, z, BASE_GRAY);
        }
    }

    fn generate_entrance_wall(&self, chunk: &mut ProtoChunk, chunk_box: &BlockBox) {
        let p = &self.base;
        if !p.chunk_intersects(chunk_box, 15, 20, 42, 21) {
            return;
        }
        p.fill(chunk, chunk_box, 15, 0, 21, 42, 0, 21, BASE_GRAY);
        p.water(chunk, chunk_box, self.sea_level, 26, 1, 21, 31, 3, 21);
        for &(x0, y, x1) in &[
            (21, 12, 36),
            (17, 11, 40),
            (16, 10, 41),
            (15, 9, 42),
            (15, 8, 42),
            (15, 7, 42),
            (16, 6, 41),
            (17, 5, 40),
            (21, 4, 36),
            (22, 3, 26),
            (31, 3, 35),
            (23, 2, 25),
            (32, 2, 34),
        ] {
            p.fill(chunk, chunk_box, x0, y, 21, x1, y, 21, BASE_GRAY);
        }
        p.fill(chunk, chunk_box, 28, 4, 20, 29, 4, 21, BASE_LIGHT);
        for &(x, y) in &[(27, 3), (30, 3), (26, 2), (31, 2), (25, 1), (32, 1)] {
            p.block(chunk, chunk_box, x, y, 21, BASE_LIGHT);
        }
        for i in 0..7 {
            p.block(chunk, chunk_box, 28 - i, 6 + i, 21, BASE_BLACK);
            p.block(chunk, chunk_box, 29 + i, 6 + i, 21, BASE_BLACK);
        }
        for i in 0..4 {
            p.block(chunk, chunk_box, 28 - i, 9 + i, 21, BASE_BLACK);
            p.block(chunk, chunk_box, 29 + i, 9 + i, 21, BASE_BLACK);
        }
        p.block(chunk, chunk_box, 28, 12, 21, BASE_BLACK);
        p.block(chunk, chunk_box, 29, 12, 21, BASE_BLACK);
        for i in 0..3 {
            for y in 8..=9 {
                p.block(chunk, chunk_box, 22 - i * 2, y, 21, BASE_BLACK);
                p.block(chunk, chunk_box, 35 + i * 2, y, 21, BASE_BLACK);
            }
        }
        for &(x0, y0, x1, y1) in &[
            (15, 13, 42, 15),
            (15, 1, 15, 6),
            (16, 1, 16, 5),
            (17, 1, 20, 4),
            (21, 1, 21, 3),
            (22, 1, 22, 2),
            (23, 1, 24, 1),
            (42, 1, 42, 6),
            (41, 1, 41, 5),
            (37, 1, 40, 4),
            (36, 1, 36, 3),
            (33, 1, 34, 1),
            (35, 1, 35, 2),
        ] {
            p.water(chunk, chunk_box, self.sea_level, x0, y0, 21, x1, y1, 21);
        }
    }

    fn generate_roof(&self, chunk: &mut ProtoChunk, chunk_box: &BlockBox) {
        let p = &self.base;
        if !p.chunk_intersects(chunk_box, 21, 21, 36, 36) {
            return;
        }
        p.fill(chunk, chunk_box, 21, 0, 22, 36, 0, 36, BASE_GRAY);
        p.water(chunk, chunk_box, self.sea_level, 21, 1, 22, 36, 23, 36);
        for i in 0..4 {
            p.fill(
                chunk,
                chunk_box,
                21 + i,
                13 + i,
                21 + i,
                36 - i,
                13 + i,
                21 + i,
                BASE_LIGHT,
            );
            p.fill(
                chunk,
                chunk_box,
                21 + i,
                13 + i,
                36 - i,
                36 - i,
                13 + i,
                36 - i,
                BASE_LIGHT,
            );
            p.fill(
                chunk,
                chunk_box,
                21 + i,
                13 + i,
                22 + i,
                21 + i,
                13 + i,
                35 - i,
                BASE_LIGHT,
            );
            p.fill(
                chunk,
                chunk_box,
                36 - i,
                13 + i,
                22 + i,
                36 - i,
                13 + i,
                35 - i,
                BASE_LIGHT,
            );
        }
        p.fill(chunk, chunk_box, 25, 16, 25, 32, 16, 32, BASE_GRAY);
        for &(x, z) in &[(25, 25), (32, 25), (25, 32), (32, 32)] {
            p.fill(chunk, chunk_box, x, 17, z, x, 19, z, BASE_LIGHT);
        }
        for &(x, y, z, block) in &[
            (26, 20, 26, BASE_LIGHT),
            (27, 21, 27, BASE_LIGHT),
            (27, 20, 27, LAMP),
            (26, 20, 31, BASE_LIGHT),
            (27, 21, 30, BASE_LIGHT),
            (27, 20, 30, LAMP),
            (31, 20, 31, BASE_LIGHT),
            (30, 21, 30, BASE_LIGHT),
            (30, 20, 30, LAMP),
            (31, 20, 26, BASE_LIGHT),
            (30, 21, 27, BASE_LIGHT),
            (30, 20, 27, LAMP),
        ] {
            p.block(chunk, chunk_box, x, y, z, block);
        }
        p.fill(chunk, chunk_box, 28, 21, 27, 29, 21, 27, BASE_GRAY);
        p.fill(chunk, chunk_box, 27, 21, 28, 27, 21, 29, BASE_GRAY);
        p.fill(chunk, chunk_box, 28, 21, 30, 29, 21, 30, BASE_GRAY);
        p.fill(chunk, chunk_box, 30, 21, 28, 30, 21, 29, BASE_GRAY);
    }

    fn generate_lower_wall(&self, chunk: &mut ProtoChunk, chunk_box: &BlockBox) {
        let p = &self.base;
        if p.chunk_intersects(chunk_box, 0, 21, 6, 58) {
            p.fill(chunk, chunk_box, 0, 0, 21, 6, 0, 57, BASE_GRAY);
            p.water(chunk, chunk_box, self.sea_level, 0, 1, 21, 6, 7, 57);
            p.fill(chunk, chunk_box, 4, 4, 21, 6, 4, 53, BASE_GRAY);
            for i in 0..4 {
                p.fill(chunk, chunk_box, i, i + 1, 21, i, i + 1, 57 - i, BASE_LIGHT);
            }
            for z in (23..53).step_by(3) {
                p.block(chunk, chunk_box, 5, 5, z, BASE_LIGHT);
            }
            p.block(chunk, chunk_box, 5, 5, 52, BASE_LIGHT);
            p.fill(chunk, chunk_box, 4, 1, 52, 6, 3, 52, BASE_GRAY);
            p.fill(chunk, chunk_box, 5, 1, 51, 5, 3, 53, BASE_GRAY);
        }
        if p.chunk_intersects(chunk_box, 51, 21, 58, 58) {
            p.fill(chunk, chunk_box, 51, 0, 21, 57, 0, 57, BASE_GRAY);
            p.water(chunk, chunk_box, self.sea_level, 51, 1, 21, 57, 7, 57);
            p.fill(chunk, chunk_box, 51, 4, 21, 53, 4, 53, BASE_GRAY);
            for i in 0..4 {
                p.fill(
                    chunk,
                    chunk_box,
                    57 - i,
                    i + 1,
                    21,
                    57 - i,
                    i + 1,
                    57 - i,
                    BASE_LIGHT,
                );
            }
            for z in (23..53).step_by(3) {
                p.block(chunk, chunk_box, 52, 5, z, BASE_LIGHT);
            }
            p.block(chunk, chunk_box, 52, 5, 52, BASE_LIGHT);
            p.fill(chunk, chunk_box, 51, 1, 52, 53, 3, 52, BASE_GRAY);
            p.fill(chunk, chunk_box, 52, 1, 51, 52, 3, 53, BASE_GRAY);
        }
        if p.chunk_intersects(chunk_box, 0, 51, 57, 57) {
            p.fill(chunk, chunk_box, 7, 0, 51, 50, 0, 57, BASE_GRAY);
            p.water(chunk, chunk_box, self.sea_level, 7, 1, 51, 50, 10, 57);
            for i in 0..4 {
                p.fill(
                    chunk,
                    chunk_box,
                    i + 1,
                    i + 1,
                    57 - i,
                    56 - i,
                    i + 1,
                    57 - i,
                    BASE_LIGHT,
                );
            }
        }
    }

    fn generate_middle_wall(&self, chunk: &mut ProtoChunk, chunk_box: &BlockBox) {
        let p = &self.base;
        if p.chunk_intersects(chunk_box, 7, 21, 13, 50) {
            p.fill(chunk, chunk_box, 7, 0, 21, 13, 0, 50, BASE_GRAY);
            p.water(chunk, chunk_box, self.sea_level, 7, 1, 21, 13, 10, 50);
            p.fill(chunk, chunk_box, 11, 8, 21, 13, 8, 53, BASE_GRAY);
            for i in 0..4 {
                p.fill(
                    chunk,
                    chunk_box,
                    i + 7,
                    i + 5,
                    21,
                    i + 7,
                    i + 5,
                    54,
                    BASE_LIGHT,
                );
            }
            for z in (21..=45).step_by(3) {
                p.block(chunk, chunk_box, 12, 9, z, BASE_LIGHT);
            }
        }
        if p.chunk_intersects(chunk_box, 44, 21, 50, 54) {
            p.fill(chunk, chunk_box, 44, 0, 21, 50, 0, 50, BASE_GRAY);
            p.water(chunk, chunk_box, self.sea_level, 44, 1, 21, 50, 10, 50);
            p.fill(chunk, chunk_box, 44, 8, 21, 46, 8, 53, BASE_GRAY);
            for i in 0..4 {
                p.fill(
                    chunk,
                    chunk_box,
                    50 - i,
                    i + 5,
                    21,
                    50 - i,
                    i + 5,
                    54,
                    BASE_LIGHT,
                );
            }
            for z in (21..=45).step_by(3) {
                p.block(chunk, chunk_box, 45, 9, z, BASE_LIGHT);
            }
        }
        if p.chunk_intersects(chunk_box, 8, 44, 49, 54) {
            p.fill(chunk, chunk_box, 14, 0, 44, 43, 0, 50, BASE_GRAY);
            p.water(chunk, chunk_box, self.sea_level, 14, 1, 44, 43, 10, 50);
            for x in (12..=45).step_by(3) {
                p.block(chunk, chunk_box, x, 9, 45, BASE_LIGHT);
                p.block(chunk, chunk_box, x, 9, 52, BASE_LIGHT);
                if [12, 18, 24, 33, 39, 45].contains(&x) {
                    for &(y, z) in &[
                        (9, 47),
                        (9, 50),
                        (10, 45),
                        (10, 46),
                        (10, 51),
                        (10, 52),
                        (11, 47),
                        (11, 50),
                        (12, 48),
                        (12, 49),
                    ] {
                        p.block(chunk, chunk_box, x, y, z, BASE_LIGHT);
                    }
                }
            }
            for i in 0..3 {
                p.fill(
                    chunk,
                    chunk_box,
                    8 + i,
                    5 + i,
                    54,
                    49 - i,
                    5 + i,
                    54,
                    BASE_GRAY,
                );
            }
            p.fill(chunk, chunk_box, 11, 8, 54, 46, 8, 54, BASE_LIGHT);
            p.fill(chunk, chunk_box, 14, 8, 44, 43, 8, 53, BASE_GRAY);
        }
    }

    fn generate_upper_wall(&self, chunk: &mut ProtoChunk, chunk_box: &BlockBox) {
        let p = &self.base;
        if p.chunk_intersects(chunk_box, 14, 21, 20, 43) {
            p.fill(chunk, chunk_box, 14, 0, 21, 20, 0, 43, BASE_GRAY);
            p.water(chunk, chunk_box, self.sea_level, 14, 1, 22, 20, 14, 43);
            p.fill(chunk, chunk_box, 18, 12, 22, 20, 12, 39, BASE_GRAY);
            p.fill(chunk, chunk_box, 18, 12, 21, 20, 12, 21, BASE_LIGHT);
            for i in 0..4 {
                p.fill(
                    chunk,
                    chunk_box,
                    i + 14,
                    i + 9,
                    21,
                    i + 14,
                    i + 9,
                    43 - i,
                    BASE_LIGHT,
                );
            }
            for z in (23..=39).step_by(3) {
                p.block(chunk, chunk_box, 19, 13, z, BASE_LIGHT);
            }
        }
        if p.chunk_intersects(chunk_box, 37, 21, 43, 43) {
            p.fill(chunk, chunk_box, 37, 0, 21, 43, 0, 43, BASE_GRAY);
            p.water(chunk, chunk_box, self.sea_level, 37, 1, 22, 43, 14, 43);
            p.fill(chunk, chunk_box, 37, 12, 22, 39, 12, 39, BASE_GRAY);
            p.fill(chunk, chunk_box, 37, 12, 21, 39, 12, 21, BASE_LIGHT);
            for i in 0..4 {
                p.fill(
                    chunk,
                    chunk_box,
                    43 - i,
                    i + 9,
                    21,
                    43 - i,
                    i + 9,
                    43 - i,
                    BASE_LIGHT,
                );
            }
            for z in (23..=39).step_by(3) {
                p.block(chunk, chunk_box, 38, 13, z, BASE_LIGHT);
            }
        }
        if p.chunk_intersects(chunk_box, 15, 37, 42, 43) {
            p.fill(chunk, chunk_box, 21, 0, 37, 36, 0, 43, BASE_GRAY);
            p.water(chunk, chunk_box, self.sea_level, 21, 1, 37, 36, 14, 43);
            p.fill(chunk, chunk_box, 21, 12, 37, 36, 12, 39, BASE_GRAY);
            for i in 0..4 {
                p.fill(
                    chunk,
                    chunk_box,
                    15 + i,
                    i + 9,
                    43 - i,
                    42 - i,
                    i + 9,
                    43 - i,
                    BASE_LIGHT,
                );
            }
            for x in (21..=36).step_by(3) {
                p.block(chunk, chunk_box, x, 13, 38, BASE_LIGHT);
            }
        }
    }
}
