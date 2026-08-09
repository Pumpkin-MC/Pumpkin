use pumpkin_data::Rotation;
use pumpkin_util::{
    math::vector3::Vector3,
    random::{RandomGenerator, RandomImpl},
};

use super::{
    Direction, RotateDirection, TemplateMirror,
    grid::{
        CORRIDOR, MansionGrid, ROOM, ROOM_1X1, ROOM_1X2, ROOM_2X2, ROOM_CORRIDOR_FLAG,
        ROOM_DOOR_FLAG, ROOM_ID_MASK, ROOM_ORIGIN_FLAG, ROOM_STAIRS_FLAG, ROOM_TYPE_MASK,
        START_ROOM, SimpleGrid,
    },
    offset, relative,
};

pub(super) struct PieceDescriptor {
    pub(super) template: String,
    pub(super) position: Vector3<i32>,
    pub(super) rotation: Rotation,
    pub(super) mirror: TemplateMirror,
}

#[derive(Clone)]
struct PlacementData {
    position: Vector3<i32>,
    rotation: Rotation,
    wall: &'static str,
}

#[derive(Clone, Copy)]
enum FloorRooms {
    First,
    Upper,
}

impl FloorRooms {
    fn one_by_one(self, random: &mut RandomGenerator) -> String {
        let prefix = match self {
            Self::First => "1x1_a",
            Self::Upper => "1x1_b",
        };
        numbered(prefix, random, 5)
    }

    fn one_by_one_secret(random: &mut RandomGenerator) -> String {
        numbered("1x1_as", random, 4)
    }

    fn one_by_two_side(self, random: &mut RandomGenerator, stairs: bool) -> String {
        match self {
            Self::First => numbered("1x2_a", random, 9),
            Self::Upper if stairs => "1x2_c_stairs".to_string(),
            Self::Upper => numbered("1x2_c", random, 4),
        }
    }

    fn one_by_two_front(self, random: &mut RandomGenerator, stairs: bool) -> String {
        match self {
            Self::First => numbered("1x2_b", random, 5),
            Self::Upper if stairs => "1x2_d_stairs".to_string(),
            Self::Upper => numbered("1x2_d", random, 5),
        }
    }

    fn one_by_two_secret(self, random: &mut RandomGenerator) -> String {
        match self {
            Self::First => numbered("1x2_s", random, 2),
            Self::Upper => numbered("1x2_se", random, 1),
        }
    }

    fn two_by_two(self, random: &mut RandomGenerator) -> String {
        match self {
            Self::First => numbered("2x2_a", random, 4),
            Self::Upper => numbered("2x2_b", random, 5),
        }
    }
}

fn numbered(prefix: &str, random: &mut RandomGenerator, count: i32) -> String {
    format!("{prefix}{}", random.next_bounded_i32(count) + 1)
}

pub(super) struct MansionPiecePlacer<'a> {
    random: &'a mut RandomGenerator,
    pieces: Vec<PieceDescriptor>,
    start_x: i32,
    start_y: i32,
}

impl<'a> MansionPiecePlacer<'a> {
    pub(super) fn create(
        random: &'a mut RandomGenerator,
        origin: Vector3<i32>,
        rotation: Rotation,
        mansion: &MansionGrid,
    ) -> Vec<PieceDescriptor> {
        let mut placer = Self {
            random,
            pieces: Vec::new(),
            start_x: mansion.entrance_x + 1,
            start_y: mansion.entrance_y + 1,
        };
        placer.create_mansion(origin, rotation, mansion);
        placer.pieces
    }

    fn add(&mut self, template: impl Into<String>, position: Vector3<i32>, rotation: Rotation) {
        self.add_mirrored(template, position, rotation, TemplateMirror::None);
    }

    fn add_mirrored(
        &mut self,
        template: impl Into<String>,
        position: Vector3<i32>,
        rotation: Rotation,
        mirror: TemplateMirror,
    ) {
        self.pieces.push(PieceDescriptor {
            template: template.into(),
            position,
            rotation,
            mirror,
        });
    }

    fn add_one_by_two_side(
        &mut self,
        rooms: FloorRooms,
        stairs: bool,
        position: Vector3<i32>,
        rotation: Rotation,
        mirror: TemplateMirror,
    ) {
        let template = rooms.one_by_two_side(self.random, stairs);
        self.add_mirrored(template, position, rotation, mirror);
    }

    fn add_one_by_two_front(
        &mut self,
        rooms: FloorRooms,
        stairs: bool,
        position: Vector3<i32>,
        rotation: Rotation,
    ) {
        let template = rooms.one_by_two_front(self.random, stairs);
        self.add(template, position, rotation);
    }

    fn add_one_by_two_secret(
        &mut self,
        rooms: FloorRooms,
        position: Vector3<i32>,
        rotation: Rotation,
    ) {
        let template = rooms.one_by_two_secret(self.random);
        self.add(template, position, rotation);
    }

    fn create_mansion(&mut self, origin: Vector3<i32>, rotation: Rotation, mansion: &MansionGrid) {
        let mut ground = PlacementData {
            position: origin,
            rotation,
            wall: "wall_flat",
        };
        self.entrance(&mut ground);
        let mut second = PlacementData {
            position: offset(ground.position, 0, 8, 0),
            rotation: ground.rotation,
            wall: "wall_window",
        };
        let end_x = mansion.entrance_x + 1;
        let end_y = mansion.entrance_y;
        self.traverse_outer_walls(
            &mut ground,
            &mansion.base,
            Direction::South,
            self.start_x,
            self.start_y,
            end_x,
            end_y,
        );
        self.traverse_outer_walls(
            &mut second,
            &mansion.base,
            Direction::South,
            self.start_x,
            self.start_y,
            end_x,
            end_y,
        );

        let mut third = PlacementData {
            position: offset(ground.position, 0, 19, 0),
            rotation: ground.rotation,
            wall: "wall_window",
        };
        'find_start: for y in 0..mansion.third_floor.height() {
            for x in (0..mansion.third_floor.width()).rev() {
                if !MansionGrid::is_house(&mansion.third_floor, x, y) {
                    continue;
                }
                third.position = relative(
                    third.position,
                    rotation.rotate(Direction::South),
                    8 + (y - self.start_y) * 8,
                );
                third.position = relative(
                    third.position,
                    rotation.rotate(Direction::East),
                    (x - self.start_x) * 8,
                );
                self.traverse_wall_piece(&mut third);
                self.traverse_outer_walls(
                    &mut third,
                    &mansion.third_floor,
                    Direction::South,
                    x,
                    y,
                    x,
                    y,
                );
                break 'find_start;
            }
        }

        self.create_roof(
            offset(origin, 0, 16, 0),
            rotation,
            &mansion.base,
            Some(&mansion.third_floor),
        );
        self.create_roof(
            offset(origin, 0, 27, 0),
            rotation,
            &mansion.third_floor,
            None,
        );

        for floor in 0..3 {
            let floor_origin = offset(origin, 0, 8 * floor as i32 + i32::from(floor == 2) * 3, 0);
            let rooms = &mansion.floor_rooms[floor];
            let grid = if floor == 2 {
                &mansion.third_floor
            } else {
                &mansion.base
            };
            let room_collection = if floor == 0 {
                FloorRooms::First
            } else {
                FloorRooms::Upper
            };
            self.add_corridors(floor, floor_origin, rotation, rooms, grid);
            self.add_rooms(
                floor,
                floor_origin,
                rotation,
                mansion,
                rooms,
                grid,
                room_collection,
            );
        }
    }

    fn add_corridors(
        &mut self,
        floor: usize,
        floor_origin: Vector3<i32>,
        rotation: Rotation,
        rooms: &SimpleGrid,
        grid: &SimpleGrid,
    ) {
        let south_carpet = if floor == 0 {
            "carpet_south_1"
        } else {
            "carpet_south_2"
        };
        let west_carpet = if floor == 0 {
            "carpet_west_1"
        } else {
            "carpet_west_2"
        };
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                if grid.get(x, y) != CORRIDOR {
                    continue;
                }
                let mut position = relative(
                    floor_origin,
                    rotation.rotate(Direction::South),
                    8 + (y - self.start_y) * 8,
                );
                position = relative(
                    position,
                    rotation.rotate(Direction::East),
                    (x - self.start_x) * 8,
                );
                self.add("corridor_floor", position, rotation);
                if grid.get(x, y - 1) == CORRIDOR
                    || rooms.get(x, y - 1) & ROOM_CORRIDOR_FLAG == ROOM_CORRIDOR_FLAG
                {
                    let position = offset(
                        relative(position, rotation.rotate(Direction::East), 1),
                        0,
                        1,
                        0,
                    );
                    self.add("carpet_north", position, rotation);
                }
                if grid.get(x + 1, y) == CORRIDOR
                    || rooms.get(x + 1, y) & ROOM_CORRIDOR_FLAG == ROOM_CORRIDOR_FLAG
                {
                    let position = relative(
                        relative(position, rotation.rotate(Direction::South), 1),
                        rotation.rotate(Direction::East),
                        5,
                    );
                    self.add("carpet_east", offset(position, 0, 1, 0), rotation);
                }
                if grid.get(x, y + 1) == CORRIDOR
                    || rooms.get(x, y + 1) & ROOM_CORRIDOR_FLAG == ROOM_CORRIDOR_FLAG
                {
                    let position = relative(
                        relative(position, rotation.rotate(Direction::South), 5),
                        rotation.rotate(Direction::West),
                        1,
                    );
                    self.add(south_carpet, position, rotation);
                }
                if grid.get(x - 1, y) == CORRIDOR
                    || rooms.get(x - 1, y) & ROOM_CORRIDOR_FLAG == ROOM_CORRIDOR_FLAG
                {
                    let position = relative(
                        relative(position, rotation.rotate(Direction::West), 1),
                        rotation.rotate(Direction::North),
                        1,
                    );
                    self.add(west_carpet, position, rotation);
                }
            }
        }
    }

    #[expect(clippy::too_many_arguments, clippy::too_many_lines)]
    fn add_rooms(
        &mut self,
        floor: usize,
        floor_origin: Vector3<i32>,
        rotation: Rotation,
        mansion: &MansionGrid,
        rooms: &SimpleGrid,
        grid: &SimpleGrid,
        room_collection: FloorRooms,
    ) {
        let wall = if floor == 0 {
            "indoors_wall_1"
        } else {
            "indoors_wall_2"
        };
        let door = if floor == 0 {
            "indoors_door_1"
        } else {
            "indoors_door_2"
        };
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let third_floor_start = floor == 2 && grid.get(x, y) == START_ROOM;
                if grid.get(x, y) != ROOM && !third_floor_start {
                    continue;
                }
                let room_data = rooms.get(x, y);
                let room_type = room_data & ROOM_TYPE_MASK;
                let room_id = room_data & ROOM_ID_MASK;
                let corridor_start =
                    third_floor_start && room_data & ROOM_CORRIDOR_FLAG == ROOM_CORRIDOR_FLAG;
                let door_directions = if room_data & ROOM_DOOR_FLAG == ROOM_DOOR_FLAG {
                    Direction::HORIZONTAL
                        .into_iter()
                        .filter(|direction| {
                            grid.get(x + direction.step_x(), y + direction.step_z()) == CORRIDOR
                        })
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                };
                let door_direction = if door_directions.is_empty() {
                    (room_data & ROOM_ORIGIN_FLAG == ROOM_ORIGIN_FLAG).then_some(Direction::Up)
                } else {
                    Some(
                        door_directions
                            [self.random.next_bounded_i32(door_directions.len() as i32) as usize],
                    )
                };

                let mut room_position = relative(
                    floor_origin,
                    rotation.rotate(Direction::South),
                    8 + (y - self.start_y) * 8,
                );
                room_position = relative(
                    room_position,
                    rotation.rotate(Direction::East),
                    -1 + (x - self.start_x) * 8,
                );

                if MansionGrid::is_house(grid, x - 1, y)
                    && !mansion.is_room_id(x - 1, y, floor, room_id)
                {
                    self.add(
                        if door_direction == Some(Direction::West) {
                            door
                        } else {
                            wall
                        },
                        room_position,
                        rotation,
                    );
                }
                if grid.get(x + 1, y) == CORRIDOR && !corridor_start {
                    self.add(
                        if door_direction == Some(Direction::East) {
                            door
                        } else {
                            wall
                        },
                        relative(room_position, rotation.rotate(Direction::East), 8),
                        rotation,
                    );
                }
                if MansionGrid::is_house(grid, x, y + 1)
                    && !mansion.is_room_id(x, y + 1, floor, room_id)
                {
                    let position = relative(
                        relative(room_position, rotation.rotate(Direction::South), 7),
                        rotation.rotate(Direction::East),
                        7,
                    );
                    self.add(
                        if door_direction == Some(Direction::South) {
                            door
                        } else {
                            wall
                        },
                        position,
                        rotation.then(Rotation::Clockwise90),
                    );
                }
                if grid.get(x, y - 1) == CORRIDOR && !corridor_start {
                    let position = relative(
                        relative(room_position, rotation.rotate(Direction::North), 1),
                        rotation.rotate(Direction::East),
                        7,
                    );
                    self.add(
                        if door_direction == Some(Direction::North) {
                            door
                        } else {
                            wall
                        },
                        position,
                        rotation.then(Rotation::Clockwise90),
                    );
                }

                match (room_type, door_direction) {
                    (ROOM_1X1, direction) => {
                        self.add_room_1x1(room_position, rotation, direction, room_collection);
                    }
                    (ROOM_1X2, Some(door_direction)) => {
                        if let Some(room_direction) = mansion.room_direction(x, y, floor, room_id) {
                            self.add_room_1x2(
                                room_position,
                                rotation,
                                room_direction,
                                door_direction,
                                room_collection,
                                room_data & ROOM_STAIRS_FLAG == ROOM_STAIRS_FLAG,
                            );
                        }
                    }
                    (ROOM_2X2, Some(Direction::Up)) => {
                        self.add_room_2x2_secret(room_position, rotation);
                    }
                    (ROOM_2X2, Some(door_direction)) => {
                        let mut room_direction = door_direction.clockwise();
                        if !mansion.is_room_id(
                            x + room_direction.step_x(),
                            y + room_direction.step_z(),
                            floor,
                            room_id,
                        ) {
                            room_direction = room_direction.opposite();
                        }
                        self.add_room_2x2(
                            room_position,
                            rotation,
                            room_direction,
                            door_direction,
                            room_collection,
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    #[expect(clippy::too_many_arguments)]
    fn traverse_outer_walls(
        &mut self,
        data: &mut PlacementData,
        grid: &SimpleGrid,
        mut direction: Direction,
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
    ) {
        let (mut grid_x, mut grid_y) = (start_x, start_y);
        let start_direction = direction;
        loop {
            if !MansionGrid::is_house(
                grid,
                grid_x + direction.step_x(),
                grid_y + direction.step_z(),
            ) {
                self.traverse_turn(data);
                direction = direction.clockwise();
                if grid_x != end_x || grid_y != end_y || start_direction != direction {
                    self.traverse_wall_piece(data);
                }
            } else if MansionGrid::is_house(
                grid,
                grid_x + direction.step_x() + direction.counterclockwise().step_x(),
                grid_y + direction.step_z() + direction.counterclockwise().step_z(),
            ) {
                Self::traverse_inner_turn(data);
                grid_x += direction.step_x();
                grid_y += direction.step_z();
                direction = direction.counterclockwise();
            } else {
                grid_x += direction.step_x();
                grid_y += direction.step_z();
                if grid_x != end_x || grid_y != end_y || start_direction != direction {
                    self.traverse_wall_piece(data);
                }
            }
            if grid_x == end_x && grid_y == end_y && start_direction == direction {
                break;
            }
        }
    }

    fn entrance(&mut self, data: &mut PlacementData) {
        let position = relative(data.position, data.rotation.rotate(Direction::West), 9);
        self.add("entrance", position, data.rotation);
        data.position = relative(data.position, data.rotation.rotate(Direction::South), 16);
    }

    fn traverse_wall_piece(&mut self, data: &mut PlacementData) {
        let position = relative(data.position, data.rotation.rotate(Direction::East), 7);
        self.add(data.wall, position, data.rotation);
        data.position = relative(data.position, data.rotation.rotate(Direction::South), 8);
    }

    fn traverse_turn(&mut self, data: &mut PlacementData) {
        data.position = relative(data.position, data.rotation.rotate(Direction::South), -1);
        self.add("wall_corner", data.position, data.rotation);
        data.position = relative(data.position, data.rotation.rotate(Direction::South), -7);
        data.position = relative(data.position, data.rotation.rotate(Direction::West), -6);
        data.rotation = data.rotation.then(Rotation::Clockwise90);
    }

    fn traverse_inner_turn(data: &mut PlacementData) {
        data.position = relative(data.position, data.rotation.rotate(Direction::South), 6);
        data.position = relative(data.position, data.rotation.rotate(Direction::East), 8);
        data.rotation = data.rotation.then(Rotation::CounterClockwise90);
    }
}

impl MansionPiecePlacer<'_> {
    fn roof_position(
        &self,
        origin: Vector3<i32>,
        rotation: Rotation,
        x: i32,
        y: i32,
    ) -> Vector3<i32> {
        let position = relative(
            origin,
            rotation.rotate(Direction::South),
            8 + (y - self.start_y) * 8,
        );
        relative(
            position,
            rotation.rotate(Direction::East),
            (x - self.start_x) * 8,
        )
    }

    #[expect(clippy::too_many_lines)]
    fn create_roof(
        &mut self,
        origin: Vector3<i32>,
        rotation: Rotation,
        grid: &SimpleGrid,
        above: Option<&SimpleGrid>,
    ) {
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let position = self.roof_position(origin, rotation, x, y);
                let covered = above.is_some_and(|above| MansionGrid::is_house(above, x, y));
                if !MansionGrid::is_house(grid, x, y) || covered {
                    continue;
                }
                self.add("roof", offset(position, 0, 3, 0), rotation);
                if !MansionGrid::is_house(grid, x + 1, y) {
                    self.add(
                        "roof_front",
                        relative(position, rotation.rotate(Direction::East), 6),
                        rotation,
                    );
                }
                if !MansionGrid::is_house(grid, x - 1, y) {
                    let position = relative(
                        relative(position, rotation.rotate(Direction::East), 0),
                        rotation.rotate(Direction::South),
                        7,
                    );
                    self.add("roof_front", position, rotation.then(Rotation::Rotate180));
                }
                if !MansionGrid::is_house(grid, x, y - 1) {
                    self.add(
                        "roof_front",
                        relative(position, rotation.rotate(Direction::West), 1),
                        rotation.then(Rotation::CounterClockwise90),
                    );
                }
                if !MansionGrid::is_house(grid, x, y + 1) {
                    let position = relative(
                        relative(position, rotation.rotate(Direction::East), 6),
                        rotation.rotate(Direction::South),
                        6,
                    );
                    self.add("roof_front", position, rotation.then(Rotation::Clockwise90));
                }
            }
        }

        if let Some(above) = above {
            for y in 0..grid.height() {
                for x in 0..grid.width() {
                    let position = self.roof_position(origin, rotation, x, y);
                    if !MansionGrid::is_house(grid, x, y) || !MansionGrid::is_house(above, x, y) {
                        continue;
                    }
                    if !MansionGrid::is_house(grid, x + 1, y) {
                        self.add(
                            "small_wall",
                            relative(position, rotation.rotate(Direction::East), 7),
                            rotation,
                        );
                    }
                    if !MansionGrid::is_house(grid, x - 1, y) {
                        let wall = relative(
                            relative(position, rotation.rotate(Direction::West), 1),
                            rotation.rotate(Direction::South),
                            6,
                        );
                        self.add("small_wall", wall, rotation.then(Rotation::Rotate180));
                    }
                    if !MansionGrid::is_house(grid, x, y - 1) {
                        let wall = relative(
                            relative(position, rotation.rotate(Direction::West), 0),
                            rotation.rotate(Direction::North),
                            1,
                        );
                        self.add(
                            "small_wall",
                            wall,
                            rotation.then(Rotation::CounterClockwise90),
                        );
                    }
                    if !MansionGrid::is_house(grid, x, y + 1) {
                        let wall = relative(
                            relative(position, rotation.rotate(Direction::East), 6),
                            rotation.rotate(Direction::South),
                            7,
                        );
                        self.add("small_wall", wall, rotation.then(Rotation::Clockwise90));
                    }
                    if !MansionGrid::is_house(grid, x + 1, y) {
                        if !MansionGrid::is_house(grid, x, y - 1) {
                            let corner = relative(
                                relative(position, rotation.rotate(Direction::East), 7),
                                rotation.rotate(Direction::North),
                                2,
                            );
                            self.add("small_wall_corner", corner, rotation);
                        }
                        if !MansionGrid::is_house(grid, x, y + 1) {
                            let corner = relative(
                                relative(position, rotation.rotate(Direction::East), 8),
                                rotation.rotate(Direction::South),
                                7,
                            );
                            self.add(
                                "small_wall_corner",
                                corner,
                                rotation.then(Rotation::Clockwise90),
                            );
                        }
                    }
                    if !MansionGrid::is_house(grid, x - 1, y) {
                        if !MansionGrid::is_house(grid, x, y - 1) {
                            let corner = relative(
                                relative(position, rotation.rotate(Direction::West), 2),
                                rotation.rotate(Direction::North),
                                1,
                            );
                            self.add(
                                "small_wall_corner",
                                corner,
                                rotation.then(Rotation::CounterClockwise90),
                            );
                        }
                        if !MansionGrid::is_house(grid, x, y + 1) {
                            let corner = relative(
                                relative(position, rotation.rotate(Direction::West), 1),
                                rotation.rotate(Direction::South),
                                8,
                            );
                            self.add(
                                "small_wall_corner",
                                corner,
                                rotation.then(Rotation::Rotate180),
                            );
                        }
                    }
                }
            }
        }

        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let position = self.roof_position(origin, rotation, x, y);
                let covered = above.is_some_and(|above| MansionGrid::is_house(above, x, y));
                if !MansionGrid::is_house(grid, x, y) || covered {
                    continue;
                }
                if !MansionGrid::is_house(grid, x + 1, y) {
                    let east = relative(position, rotation.rotate(Direction::East), 6);
                    if !MansionGrid::is_house(grid, x, y + 1) {
                        self.add(
                            "roof_corner",
                            relative(east, rotation.rotate(Direction::South), 6),
                            rotation,
                        );
                    } else if MansionGrid::is_house(grid, x + 1, y + 1) {
                        self.add(
                            "roof_inner_corner",
                            relative(east, rotation.rotate(Direction::South), 5),
                            rotation,
                        );
                    }
                    if !MansionGrid::is_house(grid, x, y - 1) {
                        self.add(
                            "roof_corner",
                            east,
                            rotation.then(Rotation::CounterClockwise90),
                        );
                    } else if MansionGrid::is_house(grid, x + 1, y - 1) {
                        let corner = relative(
                            relative(position, rotation.rotate(Direction::East), 9),
                            rotation.rotate(Direction::North),
                            2,
                        );
                        self.add(
                            "roof_inner_corner",
                            corner,
                            rotation.then(Rotation::Clockwise90),
                        );
                    }
                }
                if !MansionGrid::is_house(grid, x - 1, y) {
                    if !MansionGrid::is_house(grid, x, y + 1) {
                        self.add(
                            "roof_corner",
                            relative(position, rotation.rotate(Direction::South), 6),
                            rotation.then(Rotation::Clockwise90),
                        );
                    } else if MansionGrid::is_house(grid, x - 1, y + 1) {
                        let corner = relative(
                            relative(position, rotation.rotate(Direction::South), 8),
                            rotation.rotate(Direction::West),
                            3,
                        );
                        self.add(
                            "roof_inner_corner",
                            corner,
                            rotation.then(Rotation::CounterClockwise90),
                        );
                    }
                    if !MansionGrid::is_house(grid, x, y - 1) {
                        self.add("roof_corner", position, rotation.then(Rotation::Rotate180));
                    } else if MansionGrid::is_house(grid, x - 1, y - 1) {
                        self.add(
                            "roof_inner_corner",
                            relative(position, rotation.rotate(Direction::South), 1),
                            rotation.then(Rotation::Rotate180),
                        );
                    }
                }
            }
        }
    }

    fn add_room_1x1(
        &mut self,
        room_position: Vector3<i32>,
        rotation: Rotation,
        door_direction: Option<Direction>,
        rooms: FloorRooms,
    ) {
        let mut piece_rotation = Rotation::None;
        let mut room = rooms.one_by_one(self.random);
        match door_direction {
            Some(Direction::East) => {}
            Some(Direction::North) => piece_rotation = Rotation::CounterClockwise90,
            Some(Direction::West) => piece_rotation = Rotation::Rotate180,
            Some(Direction::South) => piece_rotation = Rotation::Clockwise90,
            _ => room = FloorRooms::one_by_one_secret(self.random),
        }
        let orientation = match piece_rotation {
            Rotation::None => (1, 0),
            Rotation::Clockwise90 => (7, 0),
            Rotation::Rotate180 => (7, 6),
            Rotation::CounterClockwise90 => (1, 6),
        };
        let orientation = rotation.rotate_offset(orientation.0, orientation.1);
        self.add(
            room,
            offset(room_position, orientation.0, 0, orientation.1),
            piece_rotation.then(rotation),
        );
    }

    #[expect(clippy::too_many_lines)]
    fn add_room_1x2(
        &mut self,
        room_position: Vector3<i32>,
        rotation: Rotation,
        room_direction: Direction,
        door_direction: Direction,
        rooms: FloorRooms,
        stairs: bool,
    ) {
        let east = rotation.rotate(Direction::East);
        let south = rotation.rotate(Direction::South);
        match (door_direction, room_direction) {
            (Direction::East, Direction::South) => self.add_one_by_two_side(
                rooms,
                stairs,
                relative(room_position, east, 1),
                rotation,
                TemplateMirror::None,
            ),
            (Direction::East, Direction::North) => {
                let position = relative(relative(room_position, east, 1), south, 6);
                self.add_one_by_two_side(
                    rooms,
                    stairs,
                    position,
                    rotation,
                    TemplateMirror::LeftRight,
                );
            }
            (Direction::West, Direction::North) => {
                let position = relative(relative(room_position, east, 7), south, 6);
                self.add_one_by_two_side(
                    rooms,
                    stairs,
                    position,
                    rotation.then(Rotation::Rotate180),
                    TemplateMirror::None,
                );
            }
            (Direction::West, Direction::South) => self.add_one_by_two_side(
                rooms,
                stairs,
                relative(room_position, east, 7),
                rotation,
                TemplateMirror::FrontBack,
            ),
            (Direction::South, Direction::East) => self.add_one_by_two_side(
                rooms,
                stairs,
                relative(room_position, east, 1),
                rotation.then(Rotation::Clockwise90),
                TemplateMirror::LeftRight,
            ),
            (Direction::South, Direction::West) => self.add_one_by_two_side(
                rooms,
                stairs,
                relative(room_position, east, 7),
                rotation.then(Rotation::Clockwise90),
                TemplateMirror::None,
            ),
            (Direction::North, Direction::West) => {
                let position = relative(relative(room_position, east, 7), south, 6);
                self.add_one_by_two_side(
                    rooms,
                    stairs,
                    position,
                    rotation.then(Rotation::Clockwise90),
                    TemplateMirror::FrontBack,
                );
            }
            (Direction::North, Direction::East) => {
                let position = relative(relative(room_position, east, 1), south, 6);
                self.add_one_by_two_side(
                    rooms,
                    stairs,
                    position,
                    rotation.then(Rotation::CounterClockwise90),
                    TemplateMirror::None,
                );
            }
            (Direction::South, Direction::North) => {
                let position = relative(
                    relative(room_position, east, 1),
                    rotation.rotate(Direction::North),
                    8,
                );
                self.add_one_by_two_front(rooms, stairs, position, rotation);
            }
            (Direction::North, Direction::South) => {
                let position = relative(relative(room_position, east, 7), south, 14);
                self.add_one_by_two_front(
                    rooms,
                    stairs,
                    position,
                    rotation.then(Rotation::Rotate180),
                );
            }
            (Direction::West, Direction::East) => self.add_one_by_two_front(
                rooms,
                stairs,
                relative(room_position, east, 15),
                rotation.then(Rotation::Clockwise90),
            ),
            (Direction::East, Direction::West) => {
                let position = relative(
                    relative(room_position, rotation.rotate(Direction::West), 7),
                    south,
                    6,
                );
                self.add_one_by_two_front(
                    rooms,
                    stairs,
                    position,
                    rotation.then(Rotation::CounterClockwise90),
                );
            }
            (Direction::Up, Direction::East) => self.add_one_by_two_secret(
                rooms,
                relative(room_position, east, 15),
                rotation.then(Rotation::Clockwise90),
            ),
            (Direction::Up, Direction::South) => {
                self.add_one_by_two_secret(rooms, relative(room_position, east, 1), rotation);
            }
            _ => {}
        }
    }

    fn add_room_2x2(
        &mut self,
        room_position: Vector3<i32>,
        rotation: Rotation,
        room_direction: Direction,
        door_direction: Direction,
        rooms: FloorRooms,
    ) {
        let (east, south, piece_rotation, mirror) = match (door_direction, room_direction) {
            (Direction::East, Direction::South) => (-7, 0, rotation, TemplateMirror::None),
            (Direction::East, Direction::North) => (-7, 6, rotation, TemplateMirror::LeftRight),
            (Direction::North, Direction::East) => (
                1,
                14,
                rotation.then(Rotation::CounterClockwise90),
                TemplateMirror::None,
            ),
            (Direction::North, Direction::West) => (
                7,
                14,
                rotation.then(Rotation::CounterClockwise90),
                TemplateMirror::LeftRight,
            ),
            (Direction::South, Direction::West) => (
                7,
                -8,
                rotation.then(Rotation::Clockwise90),
                TemplateMirror::None,
            ),
            (Direction::South, Direction::East) => (
                1,
                -8,
                rotation.then(Rotation::Clockwise90),
                TemplateMirror::LeftRight,
            ),
            (Direction::West, Direction::North) => (
                15,
                6,
                rotation.then(Rotation::Rotate180),
                TemplateMirror::None,
            ),
            (Direction::West, Direction::South) => (15, 0, rotation, TemplateMirror::FrontBack),
            _ => (0, 0, rotation, TemplateMirror::None),
        };
        let position = relative(
            relative(room_position, rotation.rotate(Direction::East), east),
            rotation.rotate(Direction::South),
            south,
        );
        let template = rooms.two_by_two(self.random);
        self.add_mirrored(template, position, piece_rotation, mirror);
    }

    fn add_room_2x2_secret(&mut self, room_position: Vector3<i32>, rotation: Rotation) {
        self.add(
            "2x2_s1",
            relative(room_position, rotation.rotate(Direction::East), 1),
            rotation,
        );
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_util::random::legacy_rand::LegacyRand;

    use super::*;

    fn layout(seed: u64) -> (usize, u64) {
        let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(seed));
        let rotation = Rotation::from_index(random.next_bounded_i32(4) as u8);
        let grid = MansionGrid::new(&mut random);
        let pieces =
            MansionPiecePlacer::create(&mut random, Vector3::new(0, 64, 0), rotation, &grid);
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        for piece in &pieces {
            let rotation = match piece.rotation {
                Rotation::None => 0,
                Rotation::Clockwise90 => 1,
                Rotation::Rotate180 => 2,
                Rotation::CounterClockwise90 => 3,
            };
            let mirror = match piece.mirror {
                TemplateMirror::None => 0,
                TemplateMirror::LeftRight => 1,
                TemplateMirror::FrontBack => 2,
            };
            let line = format!(
                "{};{},{},{};{};{}\n",
                piece.template,
                piece.position.x,
                piece.position.y,
                piece.position.z,
                rotation,
                mirror
            );
            for byte in line.bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x100_0000_01b3);
            }
        }
        (pieces.len(), hash)
    }

    #[test]
    fn layouts_match_java_26_2() {
        assert_eq!(layout(0), (590, 0xfc1e_ca03_a5d7_b852));
        assert_eq!(layout(1), (579, 0x1b41_bf3a_4fd9_020c));
        assert_eq!(layout(42), (519, 0xe367_67ff_2580_643e));
        assert_eq!(layout(8_675_309), (557, 0x8120_b948_7d0b_eb80));
    }
}
