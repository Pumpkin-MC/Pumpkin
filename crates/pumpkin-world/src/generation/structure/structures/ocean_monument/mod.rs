mod building;
mod graph;
mod rooms;

use std::sync::{Arc, Mutex};

use pumpkin_data::{Block, BlockState};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::{
    BlockDirection,
    math::{block_box::BlockBox, position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};

use crate::{
    ProtoChunk,
    biome::BiomeSupplier,
    generation::{
        biome_coords,
        noise::router::multi_noise_sampler::MultiNoiseSampler,
        positions::chunk_pos::{get_center_x, get_center_z, start_block_x, start_block_z},
        structure::{
            piece::StructurePieceType,
            structures::{
                StructureGenerator, StructureGeneratorContext, StructurePiece, StructurePieceBase,
                StructurePiecesCollector, StructurePosition, WorldPortalExt,
            },
        },
    },
};

use self::{
    graph::RoomGraph,
    rooms::{RoomKind, RoomPiece},
};

const BASE_GRAY: &BlockState = Block::PRISMARINE.default_state;
const BASE_LIGHT: &BlockState = Block::PRISMARINE_BRICKS.default_state;
const BASE_BLACK: &BlockState = Block::DARK_PRISMARINE.default_state;
const LAMP: &BlockState = Block::SEA_LANTERN.default_state;
const WATER: &BlockState = Block::WATER.default_state;

pub struct OceanMonumentGenerator;

pub(crate) fn has_valid_biomes(
    biome_supplier: &dyn BiomeSupplier,
    sampler: &mut MultiNoiseSampler,
    chunk_x: i32,
    chunk_z: i32,
    sea_level: i32,
    start_y: i32,
) -> bool {
    let start_x = get_center_x(chunk_x);
    let start_z = get_center_z(chunk_z);
    let start_biomes = pumpkin_data::tag::WorldgenBiome::MINECRAFT_HAS_STRUCTURE_OCEAN_MONUMENT.1;
    if !start_biomes.contains(
        &(biome_supplier
            .biome(
                biome_coords::from_block(start_x),
                biome_coords::from_block(start_y),
                biome_coords::from_block(start_z),
                sampler,
            )
            .id as u16),
    ) {
        return false;
    }

    let center_x = start_block_x(chunk_x) + 9;
    let center_z = start_block_z(chunk_z) + 9;
    let min_x = biome_coords::from_block(center_x - 29);
    let max_x = biome_coords::from_block(center_x + 29);
    let min_y = biome_coords::from_block(sea_level - 29);
    let max_y = biome_coords::from_block(sea_level + 29);
    let min_z = biome_coords::from_block(center_z - 29);
    let max_z = biome_coords::from_block(center_z + 29);
    let allowed = pumpkin_data::tag::WorldgenBiome::MINECRAFT_REQUIRED_OCEAN_MONUMENT_SURROUNDING.1;

    (min_x..=max_x).all(|x| {
        (min_z..=max_z).all(|z| {
            (min_y..=max_y)
                .all(|y| allowed.contains(&(biome_supplier.biome(x, y, z, sampler).id as u16)))
        })
    })
}

impl StructureGenerator for OceanMonumentGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        let center_x = get_center_x(context.chunk_x);
        let center_z = get_center_z(context.chunk_z);
        let start_y = context
            .height_sampler
            .as_deref_mut()
            .map_or(context.sea_level, |sampler| {
                sampler.estimate_ocean_floor_height(center_x, center_z)
            });
        let direction = BlockDirection::get_random_horizontal_direction(&mut context.random);
        let building = MonumentBuilding::new(
            &mut context.random,
            start_block_x(context.chunk_x) - 29,
            start_block_z(context.chunk_z) - 29,
            direction,
            context.sea_level,
        );
        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(building));

        Some(StructurePosition {
            start_pos: BlockPos::new(center_x, start_y, center_z),
            collector: Arc::new(Mutex::new(collector)),
        })
    }
}

struct MonumentBuilding {
    base: MonumentPiece,
    graph: RoomGraph,
    children: Vec<RoomPiece>,
    sea_level: i32,
}

impl MonumentBuilding {
    fn new(
        random: &mut RandomGenerator,
        west: i32,
        north: i32,
        direction: BlockDirection,
        sea_level: i32,
    ) -> Self {
        let mut piece = StructurePiece::new(
            StructurePieceType::OceanMonumentBase,
            BlockBox::new(west, 39, north, west + 57, 61, north + 57),
            0,
        );
        piece.set_facing(Some(direction));
        let base = MonumentPiece { piece };
        let (mut graph, room_order) = RoomGraph::generate(random);
        graph.rooms[graph.source].claimed = true;

        let mut children = vec![
            RoomPiece::for_room(
                RoomKind::Entry,
                direction,
                graph.source,
                graph.rooms[graph.source].index,
            ),
            RoomPiece::for_room(
                RoomKind::Core,
                direction,
                graph.core,
                graph.rooms[graph.core].index,
            ),
        ];
        for room in room_order {
            if graph.rooms[room].claimed || graph.rooms[room].is_special() {
                continue;
            }
            children.push(RoomPiece::fit(random, direction, room, &mut graph));
        }

        let room_offset = base.piece.offset_pos(9, 0, 22);
        for child in &mut children {
            child
                .piece
                .piece
                .bounding_box
                .move_pos(room_offset.x, room_offset.y, room_offset.z);
        }

        let left_wing = box_from_corners(
            base.piece.offset_pos(1, 1, 1),
            base.piece.offset_pos(23, 8, 21),
        );
        let right_wing = box_from_corners(
            base.piece.offset_pos(34, 1, 1),
            base.piece.offset_pos(56, 8, 21),
        );
        let penthouse = box_from_corners(
            base.piece.offset_pos(22, 13, 22),
            base.piece.offset_pos(35, 17, 35),
        );
        let wing_design = random.next_i32();
        children.push(RoomPiece::special(
            RoomKind::Wing(wing_design),
            direction,
            left_wing,
        ));
        children.push(RoomPiece::special(
            RoomKind::Wing(wing_design.wrapping_add(1)),
            direction,
            right_wing,
        ));
        children.push(RoomPiece::special(
            RoomKind::Penthouse,
            direction,
            penthouse,
        ));

        Self {
            base,
            graph,
            children,
            sea_level,
        }
    }
}

impl StructurePieceBase for MonumentBuilding {
    fn get_structure_piece(&self) -> &StructurePiece {
        &self.base.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.base.piece
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        self.place_building(chunk, random, chunk_box);
    }
}

struct MonumentPiece {
    piece: StructurePiece,
}

impl MonumentPiece {
    const fn for_room(
        kind: RoomKind,
        direction: BlockDirection,
        room_index: i32,
        width: i32,
        height: i32,
        depth: i32,
    ) -> Self {
        let room_x = room_index % 5;
        let room_z = room_index / 5 % 5;
        let room_y = room_index / 25;
        let mut bounding_box = match direction {
            BlockDirection::North | BlockDirection::South => {
                BlockBox::new(0, 0, 0, width * 8 - 1, height * 4 - 1, depth * 8 - 1)
            }
            _ => BlockBox::new(0, 0, 0, depth * 8 - 1, height * 4 - 1, width * 8 - 1),
        };
        match direction {
            BlockDirection::North => {
                bounding_box.move_pos(room_x * 8, room_y * 4, -(room_z + depth) * 8 + 1);
            }
            BlockDirection::South => {
                bounding_box.move_pos(room_x * 8, room_y * 4, room_z * 8);
            }
            BlockDirection::West => {
                bounding_box.move_pos(-(room_z + depth) * 8 + 1, room_y * 4, room_x * 8);
            }
            _ => {
                bounding_box.move_pos(room_z * 8, room_y * 4, room_x * 8);
            }
        }
        let mut piece = StructurePiece::new(kind.piece_type(), bounding_box, 1);
        piece.set_facing(Some(direction));
        Self { piece }
    }

    const fn special(kind: RoomKind, direction: BlockDirection, bounding_box: BlockBox) -> Self {
        let mut piece = StructurePiece::new(kind.piece_type(), bounding_box, 1);
        piece.set_facing(Some(direction));
        Self { piece }
    }

    #[expect(clippy::too_many_arguments)]
    fn fill(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        x0: i32,
        y0: i32,
        z0: i32,
        x1: i32,
        y1: i32,
        z1: i32,
        block: &BlockState,
    ) {
        self.piece
            .fill(chunk, chunk_box, x0, y0, z0, x1, y1, z1, block);
    }

    fn block(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        x: i32,
        y: i32,
        z: i32,
        block: &BlockState,
    ) {
        self.piece.add_block(chunk, block, x, y, z, chunk_box);
    }

    #[expect(clippy::too_many_arguments)]
    fn water(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        sea_level: i32,
        x0: i32,
        y0: i32,
        z0: i32,
        x1: i32,
        y1: i32,
        z1: i32,
    ) {
        for y in y0..=y1 {
            for x in x0..=x1 {
                for z in z0..=z1 {
                    let state = self.piece.get_block_at(chunk, x, y, z, chunk_box);
                    let block_id = state.id.to_block_id();
                    if [
                        Block::ICE.id,
                        Block::PACKED_ICE.id,
                        Block::BLUE_ICE.id,
                        Block::WATER.id,
                    ]
                    .contains(&block_id)
                    {
                        continue;
                    }
                    let state = if self.piece.offset_pos(x, y, z).y >= sea_level {
                        Block::AIR.default_state
                    } else {
                        WATER
                    };
                    self.block(chunk, chunk_box, x, y, z, state);
                }
            }
        }
    }

    fn default_floor(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        x: i32,
        z: i32,
        down_opening: bool,
    ) {
        if !down_opening {
            self.fill(chunk, chunk_box, x, 0, z, x + 7, 0, z + 7, BASE_GRAY);
            return;
        }
        self.fill(chunk, chunk_box, x, 0, z, x + 2, 0, z + 7, BASE_GRAY);
        self.fill(chunk, chunk_box, x + 5, 0, z, x + 7, 0, z + 7, BASE_GRAY);
        self.fill(chunk, chunk_box, x + 3, 0, z, x + 4, 0, z + 2, BASE_GRAY);
        self.fill(
            chunk,
            chunk_box,
            x + 3,
            0,
            z + 5,
            x + 4,
            0,
            z + 7,
            BASE_GRAY,
        );
        self.fill(
            chunk,
            chunk_box,
            x + 3,
            0,
            z + 2,
            x + 4,
            0,
            z + 2,
            BASE_LIGHT,
        );
        self.fill(
            chunk,
            chunk_box,
            x + 3,
            0,
            z + 5,
            x + 4,
            0,
            z + 5,
            BASE_LIGHT,
        );
        self.fill(
            chunk,
            chunk_box,
            x + 2,
            0,
            z + 3,
            x + 2,
            0,
            z + 4,
            BASE_LIGHT,
        );
        self.fill(
            chunk,
            chunk_box,
            x + 5,
            0,
            z + 3,
            x + 5,
            0,
            z + 4,
            BASE_LIGHT,
        );
    }

    #[expect(clippy::too_many_arguments)]
    fn fill_on_water(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        x0: i32,
        y0: i32,
        z0: i32,
        x1: i32,
        y1: i32,
        z1: i32,
        block: &BlockState,
    ) {
        for y in y0..=y1 {
            for x in x0..=x1 {
                for z in z0..=z1 {
                    if self
                        .piece
                        .get_block_at(chunk, x, y, z, chunk_box)
                        .id
                        .to_block_id()
                        == Block::WATER.id
                    {
                        self.block(chunk, chunk_box, x, y, z, block);
                    }
                }
            }
        }
    }

    fn chunk_intersects(&self, chunk_box: &BlockBox, x0: i32, z0: i32, x1: i32, z1: i32) -> bool {
        let first = self.piece.offset_pos(x0, 0, z0);
        let second = self.piece.offset_pos(x1, 0, z1);
        chunk_box.intersects_raw_xz(
            first.x.min(second.x),
            first.z.min(second.z),
            first.x.max(second.x),
            first.z.max(second.z),
        )
    }

    fn spawn_elder(&self, chunk: &mut ProtoChunk, chunk_box: &BlockBox, x: i32, y: i32, z: i32) {
        let pos = self.piece.offset_pos(x, y, z);
        if !chunk_box.contains_pos(&pos) {
            return;
        }
        chunk.add_structure_entity(elder_guardian_nbt(pos));
    }
}

fn elder_guardian_nbt(pos: Vector3<i32>) -> NbtCompound {
    let mut nbt = NbtCompound::new();
    nbt.put_string("id", "minecraft:elder_guardian".to_string());
    nbt.put(
        "Pos",
        NbtTag::List(vec![
            (f64::from(pos.x) + 0.5).into(),
            f64::from(pos.y).into(),
            (f64::from(pos.z) + 0.5).into(),
        ]),
    );
    nbt.put(
        "Motion",
        NbtTag::List(vec![0.0f64.into(), 0.0f64.into(), 0.0f64.into()]),
    );
    nbt.put("Rotation", NbtTag::List(vec![0.0f32.into(), 0.0f32.into()]));
    nbt.put_float("Health", 80.0);
    nbt.put_bool("PersistenceRequired", true);
    nbt
}

fn box_from_corners(first: Vector3<i32>, second: Vector3<i32>) -> BlockBox {
    BlockBox::new(
        first.x.min(second.x),
        first.y.min(second.y),
        first.z.min(second.z),
        first.x.max(second.x),
        first.y.max(second.y),
        first.z.max(second.z),
    )
}

#[cfg(test)]
mod tests {
    use pumpkin_data::{Block, dimension::Dimension};
    use pumpkin_util::{
        BlockDirection,
        math::block_box::BlockBox,
        random::{RandomGenerator, legacy_rand::LegacyRand},
        world_seed::Seed,
    };

    use crate::{
        ProtoChunk,
        generation::{
            get_world_gen,
            positions::chunk_pos::{start_block_x, start_block_z},
            structure::structures::{HeightSampler, StructureGenerator, StructureGeneratorContext},
        },
    };

    use super::{MonumentBuilding, OceanMonumentGenerator, elder_guardian_nbt};

    struct OceanFloorSampler;

    impl HeightSampler for OceanFloorSampler {
        fn estimate_height(&mut self, _block_x: i32, _block_z: i32) -> i32 {
            panic!("monuments must not use the world-surface heightmap");
        }

        fn estimate_ocean_floor_height(&mut self, _block_x: i32, _block_z: i32) -> i32 {
            37
        }
    }

    #[test]
    fn structure_start_uses_the_ocean_floor_heightmap() {
        let mut height_sampler = OceanFloorSampler;
        let context = StructureGeneratorContext {
            seed: 0,
            chunk_x: 2,
            chunk_z: -3,
            random: RandomGenerator::Legacy(LegacyRand::from_seed(0)),
            sea_level: 63,
            min_y: -64,
            height_sampler: Some(&mut height_sampler),
            structure_key: None,
        };

        let position = OceanMonumentGenerator
            .get_structure_position(context)
            .unwrap();

        assert_eq!(position.start_pos.0.y, 37);
    }

    #[test]
    fn orientation_does_not_move_the_monument_origin() {
        for direction in [
            BlockDirection::North,
            BlockDirection::South,
            BlockDirection::West,
            BlockDirection::East,
        ] {
            let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(0));
            let building = MonumentBuilding::new(&mut random, 100, 200, direction, 63);
            let bounds = building.base.piece.bounding_box;

            assert_eq!((bounds.min.x, bounds.min.y, bounds.min.z), (100, 39, 200));
            assert_eq!((bounds.max.x, bounds.max.y, bounds.max.z), (157, 61, 257));
            assert!(building.children.iter().all(|child| {
                let child = child.piece.piece.bounding_box;
                child.min.x >= bounds.min.x
                    && child.min.y >= bounds.min.y
                    && child.min.z >= bounds.min.z
                    && child.max.x <= bounds.max.x
                    && child.max.y <= bounds.max.y
                    && child.max.z <= bounds.max.z
            }));
        }
    }

    #[test]
    fn elder_guardian_nbt_is_accepted_by_the_entity_loader() {
        let nbt = elder_guardian_nbt(pumpkin_util::math::vector3::Vector3::new(1, 2, 3));

        assert_eq!(nbt.get_string("id"), Some("minecraft:elder_guardian"));
        assert_eq!(nbt.get_list("Pos").unwrap().len(), 3);
        assert_eq!(nbt.get_list("Motion").unwrap().len(), 3);
        assert_eq!(nbt.get_list("Rotation").unwrap().len(), 2);
        assert_eq!(nbt.get_float("Health"), Some(80.0));
    }

    #[test]
    fn every_orientation_places_the_complete_treasure_core() {
        let world_gen = get_world_gen(
            Seed(0),
            Dimension::OVERWORLD,
            false,
            Vec::new(),
            String::new(),
        );

        for direction in [
            BlockDirection::North,
            BlockDirection::South,
            BlockDirection::West,
            BlockDirection::East,
        ] {
            let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(0));
            let building = MonumentBuilding::new(&mut random, 0, 0, direction, 63);
            let bounds = building.base.piece.bounding_box;
            let mut gold_blocks = 0;

            for chunk_x in bounds.min.x.div_euclid(16)..=bounds.max.x.div_euclid(16) {
                for chunk_z in bounds.min.z.div_euclid(16)..=bounds.max.z.div_euclid(16) {
                    let mut chunk = ProtoChunk::new(chunk_x, chunk_z, &world_gen);
                    let chunk_box = BlockBox::new(
                        start_block_x(chunk_x),
                        chunk.bottom_y() as i32,
                        start_block_z(chunk_z),
                        start_block_x(chunk_x) + 15,
                        chunk.bottom_y() as i32 + chunk.height() as i32 - 1,
                        start_block_z(chunk_z) + 15,
                    );
                    let mut placement_random = RandomGenerator::Legacy(LegacyRand::from_seed(1));
                    building.place_building(&mut chunk, &mut placement_random, &chunk_box);

                    for x in 0..16 {
                        for y in 0..chunk.height() as i32 {
                            for z in 0..16 {
                                if chunk
                                    .get_block_state_raw(x, y, z)
                                    .to_state()
                                    .id
                                    .to_block_id()
                                    == Block::GOLD_BLOCK.id
                                {
                                    gold_blocks += 1;
                                }
                            }
                        }
                    }
                }
            }

            assert_eq!(gold_blocks, 8);
        }
    }
}
