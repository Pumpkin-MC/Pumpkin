use pumpkin_util::{
    BlockDirection,
    random::{RandomGenerator, RandomImpl},
};

const GRID_SIZE: usize = 75;
const SOURCE_INDEX: i32 = room_index(2, 0, 0);
const TOP_CONNECT_INDEX: i32 = room_index(2, 2, 0);
const LEFT_CONNECT_INDEX: i32 = room_index(0, 1, 0);
const RIGHT_CONNECT_INDEX: i32 = room_index(4, 1, 0);

#[derive(Clone)]
pub(super) struct RoomDefinition {
    pub index: i32,
    pub connections: [Option<usize>; 6],
    pub openings: [bool; 6],
    pub claimed: bool,
    source: bool,
}

impl RoomDefinition {
    const fn new(index: i32) -> Self {
        Self {
            index,
            connections: [None; 6],
            openings: [false; 6],
            claimed: false,
            source: false,
        }
    }

    pub const fn is_special(&self) -> bool {
        self.index >= 75
    }

    pub fn count_openings(&self) -> usize {
        self.openings.iter().filter(|opening| **opening).count()
    }
}

pub(super) struct RoomGraph {
    pub rooms: Vec<RoomDefinition>,
    pub source: usize,
    pub core: usize,
}

impl RoomGraph {
    #[expect(clippy::too_many_lines)]
    pub fn generate(random: &mut RandomGenerator) -> (Self, Vec<usize>) {
        let mut graph = Self {
            rooms: Vec::with_capacity(49),
            source: 0,
            core: 0,
        };
        let mut grid = [None; GRID_SIZE];
        for y in 0..=1 {
            for z in 0..4 {
                for x in 0..5 {
                    let index = room_index(x, y, z);
                    grid[index as usize] = Some(graph.push(index));
                }
            }
        }
        for z in 0..2 {
            for x in 1..4 {
                let index = room_index(x, 2, z);
                grid[index as usize] = Some(graph.push(index));
            }
        }
        if let Some(src) = grid[SOURCE_INDEX as usize] {
            graph.source = src;
        }

        for x in 0..5 {
            for z in 0..5 {
                for y in 0..3 {
                    let index = room_index(x, y, z);
                    let Some(room) = grid[index as usize] else {
                        continue;
                    };
                    for direction in DIRECTIONS {
                        let step = direction.to_vector();
                        let nx = x + step.x;
                        let ny = y + step.y;
                        let nz = z + step.z;
                        if !(0..5).contains(&nx) || !(0..3).contains(&ny) || !(0..5).contains(&nz) {
                            continue;
                        }
                        let Some(neighbor) = grid[room_index(nx, ny, nz) as usize] else {
                            continue;
                        };
                        let connection_direction = if nz == z {
                            direction
                        } else {
                            direction.opposite()
                        };
                        graph.connect(room, connection_direction, neighbor);
                    }
                }
            }
        }

        let roof = graph.push(1003);
        let left_wing = graph.push(1001);
        let right_wing = graph.push(1002);
        if let Some(top) = grid[TOP_CONNECT_INDEX as usize] {
            graph.connect(top, BlockDirection::Up, roof);
        }
        if let Some(left) = grid[LEFT_CONNECT_INDEX as usize] {
            graph.connect(left, BlockDirection::South, left_wing);
        }
        if let Some(right) = grid[RIGHT_CONNECT_INDEX as usize] {
            graph.connect(right, BlockDirection::South, right_wing);
        }
        graph.rooms[roof].claimed = true;
        graph.rooms[left_wing].claimed = true;
        graph.rooms[right_wing].claimed = true;
        graph.rooms[graph.source].source = true;

        if let Some(core_room) = grid[room_index(random.next_bounded_i32(4), 0, 2) as usize] {
            graph.core = core_room;
        }
        graph.claim_core();
        for room in &mut graph.rooms {
            // Vanilla updates every grid room and the roof room here. The two wing
            // sentinels deliberately keep their side closed so that edge pruning
            // cannot disconnect their grid attachment.
            if !room.is_special() || room.index == 1003 {
                for direction in DIRECTIONS {
                    room.openings[direction as usize] =
                        room.connections[direction as usize].is_some();
                }
            }
        }

        let mut order = grid.into_iter().flatten().collect::<Vec<_>>();
        shuffle(&mut order, random);
        for &room in &order {
            let mut closed = 0;
            for _ in 0..5 {
                if closed == 2 {
                    break;
                }
                let direction = random.next_bounded_i32(6) as usize;
                if !graph.rooms[room].openings[direction] {
                    continue;
                }
                let Some(neighbor) = graph.rooms[room].connections[direction] else {
                    continue;
                };
                let opposite = opposite_index(direction);
                graph.rooms[room].openings[direction] = false;
                graph.rooms[neighbor].openings[opposite] = false;
                if graph.find_source(room) && graph.find_source(neighbor) {
                    closed += 1;
                } else {
                    graph.rooms[room].openings[direction] = true;
                    graph.rooms[neighbor].openings[opposite] = true;
                }
            }
        }
        order.extend([roof, left_wing, right_wing]);
        (graph, order)
    }

    fn push(&mut self, index: i32) -> usize {
        let room = self.rooms.len();
        self.rooms.push(RoomDefinition::new(index));
        room
    }

    fn connect(&mut self, room: usize, direction: BlockDirection, neighbor: usize) {
        self.rooms[room].connections[direction as usize] = Some(neighbor);
        self.rooms[neighbor].connections[direction.opposite() as usize] = Some(room);
    }

    fn claim_core(&mut self) {
        let east = self.connection(self.core, BlockDirection::East);
        let north = self.connection(self.core, BlockDirection::North);
        let north_east = self.connection(east, BlockDirection::North);
        let up = self.connection(self.core, BlockDirection::Up);
        let east_up = self.connection(east, BlockDirection::Up);
        let north_up = self.connection(north, BlockDirection::Up);
        let north_east_up = self.connection(north_east, BlockDirection::Up);
        for room in [
            self.core,
            east,
            north,
            north_east,
            up,
            east_up,
            north_up,
            north_east_up,
        ] {
            self.rooms[room].claimed = true;
        }
    }

    fn find_source(&self, start: usize) -> bool {
        let mut visited = vec![false; self.rooms.len()];
        let mut pending = vec![start];
        while let Some(room) = pending.pop() {
            if std::mem::replace(&mut visited[room], true) {
                continue;
            }
            if self.rooms[room].source {
                return true;
            }
            for direction in 0..6 {
                if self.rooms[room].openings[direction]
                    && let Some(neighbor) = self.rooms[room].connections[direction]
                {
                    pending.push(neighbor);
                }
            }
        }
        false
    }

    pub fn connection(&self, room: usize, direction: BlockDirection) -> usize {
        self.rooms[room].connections[direction as usize].unwrap_or(0)
    }
}

const DIRECTIONS: [BlockDirection; 6] = [
    BlockDirection::Down,
    BlockDirection::Up,
    BlockDirection::North,
    BlockDirection::South,
    BlockDirection::West,
    BlockDirection::East,
];

const fn room_index(x: i32, y: i32, z: i32) -> i32 {
    y * 25 + z * 5 + x
}

const fn opposite_index(direction: usize) -> usize {
    [1, 0, 3, 2, 5, 4][direction]
}

fn shuffle(values: &mut [usize], random: &mut RandomGenerator) {
    for index in (1..values.len()).rev() {
        values.swap(index, random.next_bounded_i32(index as i32 + 1) as usize);
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_util::random::{RandomGenerator, legacy_rand::LegacyRand};

    use super::RoomGraph;

    #[test]
    fn generated_graph_keeps_every_room_connected() {
        for seed in 0..128 {
            let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(seed));
            let (graph, order) = RoomGraph::generate(&mut random);

            assert_eq!(graph.rooms.len(), 49);
            assert_eq!(order.len(), 49);
            assert_eq!(
                graph.rooms.iter().filter(|room| room.is_special()).count(),
                3
            );
            assert!(
                graph
                    .rooms
                    .iter()
                    .enumerate()
                    .filter(|(_, definition)| !definition.is_special())
                    .all(|(room, _)| graph.find_source(room))
            );
        }
    }
}
