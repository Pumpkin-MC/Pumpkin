use pumpkin_util::random::{RandomGenerator, RandomImpl};

use super::Direction;

pub(super) const CORRIDOR: i32 = 1;
pub(super) const ROOM: i32 = 2;
pub(super) const START_ROOM: i32 = 3;
const TEST_ROOM: i32 = 4;
const BLOCKED: i32 = 5;

pub(super) const ROOM_1X1: i32 = 0x1_0000;
pub(super) const ROOM_1X2: i32 = 0x2_0000;
pub(super) const ROOM_2X2: i32 = 0x4_0000;
pub(super) const ROOM_ORIGIN_FLAG: i32 = 0x10_0000;
pub(super) const ROOM_DOOR_FLAG: i32 = 0x20_0000;
pub(super) const ROOM_STAIRS_FLAG: i32 = 0x40_0000;
pub(super) const ROOM_CORRIDOR_FLAG: i32 = 0x80_0000;
pub(super) const ROOM_TYPE_MASK: i32 = 0xF_0000;
pub(super) const ROOM_ID_MASK: i32 = 0xFFFF;

const GRID_SIZE: usize = 11;
const GRID_AREA: usize = GRID_SIZE * GRID_SIZE;

#[derive(Clone)]
pub(super) struct SimpleGrid {
    cells: [i32; GRID_AREA],
    outside: i32,
}

impl SimpleGrid {
    const fn new(outside: i32) -> Self {
        Self {
            cells: [0; GRID_AREA],
            outside,
        }
    }

    #[expect(
        clippy::unused_self,
        reason = "keeps grid traversal call sites explicit"
    )]
    pub(super) const fn width(&self) -> i32 {
        GRID_SIZE as i32
    }

    #[expect(
        clippy::unused_self,
        reason = "keeps grid traversal call sites explicit"
    )]
    pub(super) const fn height(&self) -> i32 {
        GRID_SIZE as i32
    }

    pub(super) const fn set(&mut self, x: i32, y: i32, value: i32) {
        if let Some(index) = Self::index(x, y) {
            self.cells[index] = value;
        }
    }

    fn set_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, value: i32) {
        for y in y0..=y1 {
            for x in x0..=x1 {
                self.set(x, y, value);
            }
        }
    }

    pub(super) fn get(&self, x: i32, y: i32) -> i32 {
        Self::index(x, y).map_or(self.outside, |index| self.cells[index])
    }

    fn set_if(&mut self, x: i32, y: i32, expected: i32, value: i32) {
        if self.get(x, y) == expected {
            self.set(x, y, value);
        }
    }

    fn edges_to(&self, x: i32, y: i32, value: i32) -> bool {
        self.get(x - 1, y) == value
            || self.get(x + 1, y) == value
            || self.get(x, y - 1) == value
            || self.get(x, y + 1) == value
    }

    const fn index(x: i32, y: i32) -> Option<usize> {
        if x >= 0 && x < GRID_SIZE as i32 && y >= 0 && y < GRID_SIZE as i32 {
            Some(x as usize * GRID_SIZE + y as usize)
        } else {
            None
        }
    }
}

pub(super) struct MansionGrid {
    pub(super) base: SimpleGrid,
    pub(super) third_floor: SimpleGrid,
    pub(super) floor_rooms: [SimpleGrid; 3],
    pub(super) entrance_x: i32,
    pub(super) entrance_y: i32,
}

impl MansionGrid {
    #[expect(
        clippy::too_many_lines,
        reason = "ports Vanilla's seeded base-floor layout"
    )]
    pub(super) fn new(random: &mut RandomGenerator) -> Self {
        let entrance_x = 7;
        let entrance_y = 4;
        let mut base = SimpleGrid::new(BLOCKED);
        base.set_rect(
            entrance_x,
            entrance_y,
            entrance_x + 1,
            entrance_y + 1,
            START_ROOM,
        );
        base.set_rect(
            entrance_x - 1,
            entrance_y,
            entrance_x - 1,
            entrance_y + 1,
            ROOM,
        );
        base.set_rect(
            entrance_x + 2,
            entrance_y - 2,
            entrance_x + 3,
            entrance_y + 3,
            BLOCKED,
        );
        base.set_rect(
            entrance_x + 1,
            entrance_y - 2,
            entrance_x + 1,
            entrance_y - 1,
            CORRIDOR,
        );
        base.set_rect(
            entrance_x + 1,
            entrance_y + 2,
            entrance_x + 1,
            entrance_y + 3,
            CORRIDOR,
        );
        base.set(entrance_x - 1, entrance_y - 1, CORRIDOR);
        base.set(entrance_x - 1, entrance_y + 2, CORRIDOR);
        base.set_rect(0, 0, 11, 1, BLOCKED);
        base.set_rect(0, 9, 11, 11, BLOCKED);

        Self::recursive_corridor(
            random,
            &mut base,
            entrance_x,
            entrance_y - 2,
            Direction::West,
            6,
        );
        Self::recursive_corridor(
            random,
            &mut base,
            entrance_x,
            entrance_y + 3,
            Direction::West,
            6,
        );
        Self::recursive_corridor(
            random,
            &mut base,
            entrance_x - 2,
            entrance_y - 1,
            Direction::West,
            3,
        );
        Self::recursive_corridor(
            random,
            &mut base,
            entrance_x - 2,
            entrance_y + 2,
            Direction::West,
            3,
        );
        while Self::clean_edges(&mut base) {}

        let mut floor_rooms = std::array::from_fn(|_| SimpleGrid::new(BLOCKED));
        Self::identify_rooms(random, &base, &mut floor_rooms[0]);
        Self::identify_rooms(random, &base, &mut floor_rooms[1]);
        floor_rooms[0].set_rect(
            entrance_x + 1,
            entrance_y,
            entrance_x + 1,
            entrance_y + 1,
            ROOM_CORRIDOR_FLAG,
        );
        floor_rooms[1].set_rect(
            entrance_x + 1,
            entrance_y,
            entrance_x + 1,
            entrance_y + 1,
            ROOM_CORRIDOR_FLAG,
        );

        let mut mansion = Self {
            base,
            third_floor: SimpleGrid::new(BLOCKED),
            floor_rooms,
            entrance_x,
            entrance_y,
        };
        mansion.setup_third_floor(random);
        Self::identify_rooms(random, &mansion.third_floor, &mut mansion.floor_rooms[2]);
        mansion
    }

    pub(super) fn is_house(grid: &SimpleGrid, x: i32, y: i32) -> bool {
        matches!(grid.get(x, y), CORRIDOR | ROOM | START_ROOM | TEST_ROOM)
    }

    pub(super) fn is_room_id(&self, x: i32, y: i32, floor: usize, room_id: i32) -> bool {
        self.floor_rooms[floor].get(x, y) & ROOM_ID_MASK == room_id
    }

    pub(super) fn room_direction(
        &self,
        x: i32,
        y: i32,
        floor: usize,
        room_id: i32,
    ) -> Option<Direction> {
        Direction::HORIZONTAL.into_iter().find(|direction| {
            self.is_room_id(
                x + direction.step_x(),
                y + direction.step_z(),
                floor,
                room_id,
            )
        })
    }

    fn recursive_corridor(
        random: &mut RandomGenerator,
        grid: &mut SimpleGrid,
        x: i32,
        y: i32,
        heading: Direction,
        depth: i32,
    ) {
        if depth <= 0 {
            return;
        }
        grid.set(x, y, CORRIDOR);
        grid.set_if(x + heading.step_x(), y + heading.step_z(), 0, CORRIDOR);
        for _ in 0..8 {
            let next = Direction::from_2d_index(random.next_bounded_i32(4));
            if next == heading.opposite() || (next == Direction::East && random.next_bool()) {
                continue;
            }
            let next_x = x + heading.step_x();
            let next_y = y + heading.step_z();
            if grid.get(next_x + next.step_x(), next_y + next.step_z()) != 0
                || grid.get(next_x + next.step_x() * 2, next_y + next.step_z() * 2) != 0
            {
                continue;
            }
            Self::recursive_corridor(
                random,
                grid,
                next_x + next.step_x(),
                next_y + next.step_z(),
                next,
                depth - 1,
            );
            break;
        }

        let clockwise = heading.clockwise();
        let counterclockwise = heading.counterclockwise();
        for (offset_x, offset_y) in [
            (clockwise.step_x(), clockwise.step_z()),
            (counterclockwise.step_x(), counterclockwise.step_z()),
            (
                heading.step_x() + clockwise.step_x(),
                heading.step_z() + clockwise.step_z(),
            ),
            (
                heading.step_x() + counterclockwise.step_x(),
                heading.step_z() + counterclockwise.step_z(),
            ),
            (heading.step_x() * 2, heading.step_z() * 2),
            (clockwise.step_x() * 2, clockwise.step_z() * 2),
            (counterclockwise.step_x() * 2, counterclockwise.step_z() * 2),
        ] {
            grid.set_if(x + offset_x, y + offset_y, 0, ROOM);
        }
    }

    fn clean_edges(grid: &mut SimpleGrid) -> bool {
        let mut touched = false;
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                if grid.get(x, y) != 0 {
                    continue;
                }
                let direct_neighbors = [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
                    .into_iter()
                    .filter(|&(x, y)| Self::is_house(grid, x, y))
                    .count();
                if direct_neighbors >= 3 {
                    grid.set(x, y, ROOM);
                    touched = true;
                } else if direct_neighbors == 2 {
                    let diagonal_neighbors = [
                        (x + 1, y + 1),
                        (x - 1, y + 1),
                        (x + 1, y - 1),
                        (x - 1, y - 1),
                    ]
                    .into_iter()
                    .filter(|&(x, y)| Self::is_house(grid, x, y))
                    .count();
                    if diagonal_neighbors <= 1 {
                        grid.set(x, y, ROOM);
                        touched = true;
                    }
                }
            }
        }
        touched
    }

    fn setup_third_floor(&mut self, random: &mut RandomGenerator) {
        let mut potential_rooms = Vec::new();
        for y in 0..self.third_floor.height() {
            for x in 0..self.third_floor.width() {
                let room_data = self.floor_rooms[1].get(x, y);
                if room_data & ROOM_TYPE_MASK == ROOM_1X2
                    && room_data & ROOM_DOOR_FLAG == ROOM_DOOR_FLAG
                {
                    potential_rooms.push((x, y));
                }
            }
        }
        if potential_rooms.is_empty() {
            self.third_floor.set_rect(0, 0, 11, 11, BLOCKED);
            return;
        }

        let (room_x, room_y) =
            potential_rooms[random.next_bounded_i32(potential_rooms.len() as i32) as usize];
        let room_data = self.floor_rooms[1].get(room_x, room_y);
        self.floor_rooms[1].set(room_x, room_y, room_data | ROOM_STAIRS_FLAG);
        let Some(room_direction) = self.room_direction(room_x, room_y, 1, room_data & ROOM_ID_MASK)
        else {
            self.third_floor.set_rect(0, 0, 11, 11, BLOCKED);
            self.floor_rooms[1].set(room_x, room_y, room_data);
            return;
        };
        let room_end_x = room_x + room_direction.step_x();
        let room_end_y = room_y + room_direction.step_z();

        for y in 0..self.third_floor.height() {
            for x in 0..self.third_floor.width() {
                if !Self::is_house(&self.base, x, y) {
                    self.third_floor.set(x, y, BLOCKED);
                } else if x == room_x && y == room_y {
                    self.third_floor.set(x, y, START_ROOM);
                } else if x == room_end_x && y == room_end_y {
                    self.third_floor.set(x, y, START_ROOM);
                    self.floor_rooms[2].set(x, y, ROOM_CORRIDOR_FLAG);
                }
            }
        }

        let potential_corridors = Direction::HORIZONTAL
            .into_iter()
            .filter(|direction| {
                self.third_floor.get(
                    room_end_x + direction.step_x(),
                    room_end_y + direction.step_z(),
                ) == 0
            })
            .collect::<Vec<_>>();
        if potential_corridors.is_empty() {
            self.third_floor.set_rect(0, 0, 11, 11, BLOCKED);
            self.floor_rooms[1].set(room_x, room_y, room_data);
            return;
        }
        let direction =
            potential_corridors[random.next_bounded_i32(potential_corridors.len() as i32) as usize];
        Self::recursive_corridor(
            random,
            &mut self.third_floor,
            room_end_x + direction.step_x(),
            room_end_y + direction.step_z(),
            direction,
            4,
        );
        while Self::clean_edges(&mut self.third_floor) {}
    }

    fn identify_rooms(
        random: &mut RandomGenerator,
        from_grid: &SimpleGrid,
        room_grid: &mut SimpleGrid,
    ) {
        let mut room_positions = Vec::new();
        for y in 0..from_grid.height() {
            for x in 0..from_grid.width() {
                if from_grid.get(x, y) == ROOM {
                    room_positions.push((x, y));
                }
            }
        }
        for index in (1..room_positions.len()).rev() {
            let swap = random.next_bounded_i32((index + 1) as i32) as usize;
            room_positions.swap(index, swap);
        }

        let mut room_id = 10;
        for (x, y) in room_positions {
            if room_grid.get(x, y) != 0 {
                continue;
            }
            let (mut x0, mut x1, mut y0, mut y1) = (x, x, y, y);
            let mut room_type = ROOM_1X1;
            if room_grid.get(x + 1, y) == 0
                && room_grid.get(x, y + 1) == 0
                && room_grid.get(x + 1, y + 1) == 0
                && from_grid.get(x + 1, y) == ROOM
                && from_grid.get(x, y + 1) == ROOM
                && from_grid.get(x + 1, y + 1) == ROOM
            {
                x1 += 1;
                y1 += 1;
                room_type = ROOM_2X2;
            } else if room_grid.get(x - 1, y) == 0
                && room_grid.get(x, y + 1) == 0
                && room_grid.get(x - 1, y + 1) == 0
                && from_grid.get(x - 1, y) == ROOM
                && from_grid.get(x, y + 1) == ROOM
                && from_grid.get(x - 1, y + 1) == ROOM
            {
                x0 -= 1;
                y1 += 1;
                room_type = ROOM_2X2;
            } else if room_grid.get(x - 1, y) == 0
                && room_grid.get(x, y - 1) == 0
                && room_grid.get(x - 1, y - 1) == 0
                && from_grid.get(x - 1, y) == ROOM
                && from_grid.get(x, y - 1) == ROOM
                && from_grid.get(x - 1, y - 1) == ROOM
            {
                x0 -= 1;
                y0 -= 1;
                room_type = ROOM_2X2;
            } else if room_grid.get(x + 1, y) == 0 && from_grid.get(x + 1, y) == ROOM {
                x1 += 1;
                room_type = ROOM_1X2;
            } else if room_grid.get(x, y + 1) == 0 && from_grid.get(x, y + 1) == ROOM {
                y1 += 1;
                room_type = ROOM_1X2;
            } else if room_grid.get(x - 1, y) == 0 && from_grid.get(x - 1, y) == ROOM {
                x0 -= 1;
                room_type = ROOM_1X2;
            } else if room_grid.get(x, y - 1) == 0 && from_grid.get(x, y - 1) == ROOM {
                y0 -= 1;
                room_type = ROOM_1X2;
            }

            let mut door_x = if random.next_bool() { x0 } else { x1 };
            let mut door_y = if random.next_bool() { y0 } else { y1 };
            let mut door_flag = ROOM_DOOR_FLAG;
            if !from_grid.edges_to(door_x, door_y, CORRIDOR) {
                door_x = if door_x == x0 { x1 } else { x0 };
                door_y = if door_y == y0 { y1 } else { y0 };
                if !from_grid.edges_to(door_x, door_y, CORRIDOR) {
                    door_y = if door_y == y0 { y1 } else { y0 };
                    if !from_grid.edges_to(door_x, door_y, CORRIDOR) {
                        door_x = if door_x == x0 { x1 } else { x0 };
                        door_y = if door_y == y0 { y1 } else { y0 };
                        if !from_grid.edges_to(door_x, door_y, CORRIDOR) {
                            door_flag = 0;
                            door_x = x0;
                            door_y = y0;
                        }
                    }
                }
            }

            for room_y in y0..=y1 {
                for room_x in x0..=x1 {
                    let flags = if room_x == door_x && room_y == door_y {
                        ROOM_ORIGIN_FLAG | door_flag
                    } else {
                        0
                    };
                    room_grid.set(room_x, room_y, flags | room_type | room_id);
                }
            }
            room_id += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_util::random::legacy_rand::LegacyRand;

    use super::*;

    #[test]
    fn generated_rooms_are_partitioned_and_have_a_third_floor() {
        let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(1));
        let mansion = MansionGrid::new(&mut random);

        for floor in 0..2 {
            for y in 0..mansion.base.height() {
                for x in 0..mansion.base.width() {
                    if mansion.base.get(x, y) == ROOM {
                        assert_ne!(mansion.floor_rooms[floor].get(x, y) & ROOM_ID_MASK, 0);
                    }
                }
            }
        }
        assert!((0..mansion.third_floor.height()).any(|y| {
            (0..mansion.third_floor.width())
                .any(|x| MansionGrid::is_house(&mansion.third_floor, x, y))
        }));
    }
}
